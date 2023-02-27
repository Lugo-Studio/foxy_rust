use std::sync::Arc;
use tracing::{error, info, trace, warn};
use tracing_unwrap::{OptionExt, ResultExt};
use vulkano::{
  image::{
    SwapchainImage,
    ImageAccess,
    view::ImageView
  },
  device::{
    physical::PhysicalDeviceType,
    Device,
    DeviceCreateInfo,
    DeviceExtensions,
    Queue,
    QueueCreateInfo
  },
  VulkanLibrary,
  instance::{
    Instance,
    InstanceCreateInfo
  },
  Version,
  swapchain::{
    self,
    Surface,
    SwapchainCreateInfo,
    AcquireError,
    SwapchainCreationError,
    SwapchainPresentInfo,
    PresentMode
  },
  single_pass_renderpass,
  command_buffer::allocator::StandardCommandBufferAllocator,
  pipeline::graphics::viewport::Viewport,
  render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
  sync,
  command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents},
  format::ClearValue,
  sync::{FlushError, GpuFuture},
  swapchain::Swapchain
};
use vulkano_win::{create_surface_from_winit, required_extensions};
use winit::{
  event_loop::EventLoop,
  window::{Window, WindowBuilder}
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum VsyncMode {
  Enabled,
  Disabled,
  Hybrid,
}

#[allow(dead_code)]
pub struct Renderer {
  instance: Arc<Instance>,
  surface: Arc<Surface>,
  device: Arc<Device>,
  queue: Arc<Queue>,
  swapchain: Arc<Swapchain>,
  images: Vec<Arc<SwapchainImage>>,
  command_buffer_allocator: StandardCommandBufferAllocator,
  render_pass: Arc<RenderPass>,
  viewport: Viewport,
  framebuffers: Vec<Arc<Framebuffer>>,
  previous_frame_end: Option<Box<dyn GpuFuture>>,
  clear_colors: Vec<Option<ClearValue>>,
  vsync_mode: VsyncMode,
}

impl Renderer {
  pub fn from_window(window: Arc<Window>, vsync_mode: VsyncMode) -> Self {
    trace!("Initializing renderer...");

    let instance = Self::create_vulkan_instance();
    let surface = create_surface_from_winit(window, instance.clone()).expect_or_log("Failed to create new Vulkan surface");
    let (device, queue) = Self::pick_vulkan_device(instance.clone(), surface.clone());

    let mut vsync_mode = vsync_mode;
    let (swapchain, images) = Self::create_vulkan_swapchain(device.clone(), surface.clone(), &mut vsync_mode);

    let command_buffer_allocator = StandardCommandBufferAllocator::new(
      device.clone(),
      Default::default(),
    );

    let render_pass = single_pass_renderpass!(
      device.clone(),
      attachments: {
        color: {
          load: Clear,
          store: Store,
          format: swapchain.image_format(),
          samples: 1,
        }
      },
      pass: {
        color: [color],
        depth_stencil: {}
      }
    ).expect_or_log("Failed to create Vulkan render pass");

    let mut viewport = Viewport {
      origin: [0., 0.],
      dimensions: [0., 0.],
      depth_range: 0.0..1.0,
    };

    let framebuffers = Self::window_size_dependent_setup(&images, render_pass.clone(), &mut viewport);

    let previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);

    let clear_colors = vec![
      Some([0.0, 0.69, 1.0, 1.0].into())
    ];

    trace!("Initialized renderer.");
    Self {
      instance,
      surface,
      device,
      queue,
      swapchain,
      images,
      command_buffer_allocator,
      render_pass,
      viewport,
      framebuffers,
      previous_frame_end,
      clear_colors,
      vsync_mode,
    }
  }

  pub fn new(vsync_mode: VsyncMode) -> (Self, EventLoop<()>) {
    let event_loop = EventLoop::new();
    let window = Arc::new(
      WindowBuilder::new()
        .build(&event_loop)
        .expect_or_log("Failed to build window")
    );

    (Self::from_window(window, vsync_mode), event_loop)
  }

  pub fn get_vsync_mode(&self) -> VsyncMode {
    self.vsync_mode
  }

  pub fn set_vsync_mode(&mut self, vsync_mode: VsyncMode) {
    self.vsync_mode = vsync_mode;
    self.recreate_swapchain();
    info!("Vsync: {:?}", self.vsync_mode);
  }

  pub fn render(&mut self) {
    let (image_index, suboptimal, acquire_future) =
      match swapchain::acquire_next_image(self.swapchain.clone(), None) {
        Ok(result) => result,
        Err(AcquireError::OutOfDate) => {
          self.recreate_swapchain();
          return;
        }
        Err(e) => {
          error!("Failed to acquire next swapchain image: {:?}", e);
          panic!();
        }
      };

    if suboptimal {
      self.recreate_swapchain();
    }

    let command_buffer = {
      let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
        &self.command_buffer_allocator,
        self.queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit
      ).unwrap_or_log();

      command_buffer_builder
        .begin_render_pass(
          RenderPassBeginInfo {
            clear_values: self.clear_colors.clone(),
            ..RenderPassBeginInfo::framebuffer(
              self.framebuffers[image_index as usize].clone()
            )
          },
          SubpassContents::Inline
        )
        .unwrap_or_log()
        .end_render_pass()
        .unwrap_or_log();

      command_buffer_builder.build().unwrap_or_log()
    };

    let future = self.previous_frame_end
      .take()
      .unwrap_or_log()
      .join(acquire_future)
      .then_execute(self.queue.clone(), command_buffer)
      .unwrap_or_log()
      .then_swapchain_present(
        self.queue.clone(),
        SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index)
      )
      .then_signal_fence_and_flush();

    match future {
      Ok(future) => {
        self.previous_frame_end = Some(Box::new(future) as Box<_>);
      },
      Err(FlushError::OutOfDate) => {
        self.recreate_swapchain();
        self.previous_frame_end = Some(Box::new(sync::now(self.device.clone())) as Box<_>);
      },
      Err(e) => {
        error!("Failed to flush future: {:?}", e);
        self.previous_frame_end = Some(Box::new(sync::now(self.device.clone())) as Box<_>);
      }
    }
  }

  pub fn end_previous_frame(&mut self) {
    self.previous_frame_end
      .as_mut()
      .take()
      .unwrap_or_log()
      .cleanup_finished();
  }

  pub fn recreate_swapchain(&mut self) {
    let window = self.surface.object().unwrap_or_log().downcast_ref::<Window>().unwrap_or_log();
    let image_extent: [u32; 2] = window.inner_size().into();

    let (new_swapchain, new_images) = loop {
      let present_mode = match self.vsync_mode {
        VsyncMode::Enabled => PresentMode::Fifo,
        VsyncMode::Disabled => PresentMode::Immediate,
        VsyncMode::Hybrid => PresentMode::Mailbox,
      };

      match self.swapchain.recreate(
        SwapchainCreateInfo {
          image_extent,
          present_mode,
          ..self.swapchain.create_info()
        }
      ) {
        Ok(result) => break result,
        Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
        Err(SwapchainCreationError::ImageExtentZeroLengthDimensions { .. }) => {
          // warn!("Swapchain image extent has zero dimension.");
          return;
        },
        Err(SwapchainCreationError::PresentModeNotSupported) => {
          warn!("Present mode \"{:?}\" unsupported on device. Defaulting to FIFO", self.vsync_mode);
          self.vsync_mode = VsyncMode::Enabled;
        }
        Err(e) => {
          error!("Failed to recreate swapchain: {:?}", e);
          panic!();
        }
      };
    };

    self.swapchain = new_swapchain;
    self.framebuffers = Self::window_size_dependent_setup(
      &new_images,
      self.render_pass.clone(),
      &mut self.viewport
    );
  }

  fn create_vulkan_instance() -> Arc<Instance> {
    let library = VulkanLibrary::new().expect_or_log("Failed to load Vulkan library");
    let required_extensions = required_extensions(&library);

    Instance::new(
      library,
      InstanceCreateInfo {
        enabled_extensions: required_extensions,
        enumerate_portability: false,
        max_api_version: Some(Version::V1_2),
        ..Default::default()
      }
    )
    .expect_or_log("Failed to create new Vulkan instance")
  }

  fn pick_vulkan_device(instance: Arc<Instance>, surface: Arc<Surface>) -> (Arc<Device>, Arc<Queue>) {
    let device_extensions = DeviceExtensions {
      khr_swapchain: true,
      ..DeviceExtensions::empty()
    };

    let (physical_device, queue_family_index) = instance
      .enumerate_physical_devices()
      .expect_or_log("Failed to enumerate physical devices")
      .filter(|p| p.supported_extensions().contains(&device_extensions))
      .filter_map(|p|
        p.queue_family_properties()
         .iter()
         .enumerate()
         .position(|(i, q)| {
           q.queue_flags.graphics && p.surface_support(i as u32, &surface).unwrap_or(false)
         })
         .map(|i| (p, i as u32))
      )
      .min_by_key(|(p, _)|
        match p.properties().device_type {
          PhysicalDeviceType::DiscreteGpu => 0,
          PhysicalDeviceType::IntegratedGpu => 1,
          PhysicalDeviceType::VirtualGpu => 2,
          PhysicalDeviceType::Cpu => 3,
          PhysicalDeviceType::Other => 4,
          _ => 5
        }
      )
      .expect_or_log("No suitable Vulkan hardware could be found");

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
          enabled_extensions: device_extensions,
          queue_create_infos: vec![
            QueueCreateInfo{
              queue_family_index,
              ..Default::default()
            }
          ],
          ..Default::default()
        }
      )
      .expect_or_log("Failed to create new Vulkan device");
    let queue = queues.next().unwrap();

    (device, queue)
  }

  fn create_vulkan_swapchain(device: Arc<Device>, surface: Arc<Surface>, vsync_mode: &mut VsyncMode) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>) {
    let capabilities = device
      .physical_device()
      .surface_capabilities(&surface, Default::default())
      .expect_or_log("Failed to access surface capabilities");

    let (image_usage, composite_alpha) = {
      (capabilities.supported_usage_flags, capabilities.supported_composite_alpha.iter().next().unwrap_or_log())
    };

    let (image_format, image_color_space) = device
      .physical_device()
      .surface_formats(&surface, Default::default())
      .unwrap_or_log()[0];

    let window = surface.object().unwrap_or_log().downcast_ref::<Window>().unwrap_or_log();
    let image_extent: [u32; 2] = window.inner_size().into();

    loop {
      let present_mode = match vsync_mode {
        VsyncMode::Enabled => PresentMode::Fifo,
        VsyncMode::Disabled => PresentMode::Immediate,
        VsyncMode::Hybrid => PresentMode::Mailbox,
      };

      let swapchain = Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
          min_image_count: capabilities.min_image_count,
          image_format: Some(image_format),
          image_extent,
          image_usage,
          composite_alpha,
          image_color_space,
          present_mode,
          ..Default::default()
        }
      );

      match swapchain {
        Ok(result) => {
          info!("Vsync: {vsync_mode:?}");
          break result
        },
        Err(SwapchainCreationError::PresentModeNotSupported) => {
          warn!("Present mode \"{:?}\" unsupported on device. Defaulting to FIFO", present_mode);
          *vsync_mode = VsyncMode::Enabled;
        }
        Err(e) => {
          warn!("Error creating swapchain: {:?}", e);
          panic!();
        }
      }
    }
  }

  fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
  ) -> Vec<Arc<Framebuffer>> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

    images
      .iter()
      .map(|image| {
        let view = ImageView::new_default(image.clone()).unwrap_or_log();
        Framebuffer::new(
          render_pass.clone(),
          FramebufferCreateInfo {
            attachments: vec![view],
            ..Default::default()
          },
        ).unwrap_or_log()
      })
      .collect::<Vec<_>>()
  }
}