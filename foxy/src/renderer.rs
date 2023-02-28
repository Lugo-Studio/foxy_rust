mod pipeline;

use std::path::PathBuf;
use std::sync::Arc;
use enumflags2::BitFlags;
use tracing::{error, info, trace, warn};
use tracing_unwrap::{OptionExt, ResultExt};
use vulkano::{
  image::{
    SwapchainImage,
    ImageAccess,
    view::ImageView
  },
  device::Device,
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
use winit::window::Window;
use crate::shader_builder;
use crate::vulkan::device::FoxyDevice;
use crate::vulkan::shader::{Shader, ShaderCreateInfo, ShaderStage};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum VsyncMode {
  Enabled,
  Disabled,
  Hybrid,
}

#[allow(dead_code)]
pub struct Renderer {
  device: FoxyDevice,

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

    let device = FoxyDevice::new(
      window.clone(),
      "Foxy App".into(),
      Version::major_minor(0, 1),
      "Foxy Renderer".into(),
      Version::major_minor(0, 1),
    );

    let mut vsync_mode = vsync_mode;
    let (swapchain, images) = Self::create_vulkan_swapchain(device.device().clone(), device.surface().clone(), &mut vsync_mode);

    let command_buffer_allocator = StandardCommandBufferAllocator::new(
      device.device().clone(),
      Default::default(),
    );

    let render_pass = single_pass_renderpass!(
      device.device().clone(),
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

    let previous_frame_end = Some(Box::new(sync::now(device.device().clone())) as Box<dyn GpuFuture>);

    let clear_colors = vec![
      Some([0.0, 0.69, 1.0, 1.0].into())
    ];

    // let shader = Shader::new(
    //   &device,
    //   ShaderCreateInfo {
    //     path: PathBuf::from(""),
    //     vertex: false,
    //     fragment: false,
    //     ..Default::default()
    //   }
    // );

    let shader = shader_builder!("../res/fixed_value.hlsl")
      .with_stage(ShaderStage::Vertex)
      .with_stage(ShaderStage::Fragment)
      .build(&device);

    trace!("Initialized renderer.");
    Self {
      device,
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
        self.device.queues().graphics.queue_family_index(),
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
      .then_execute(self.device.queues().graphics.clone(), command_buffer)
      .unwrap_or_log()
      .then_swapchain_present(
        self.device.queues().graphics.clone(),
        SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index)
      )
      .then_signal_fence_and_flush();

    match future {
      Ok(future) => {
        self.previous_frame_end = Some(Box::new(future) as Box<_>);
      },
      Err(FlushError::OutOfDate) => {
        self.recreate_swapchain();
        self.previous_frame_end = Some(Box::new(sync::now(self.device.device().clone())) as Box<_>);
      },
      Err(e) => {
        error!("Failed to flush future: {:?}", e);
        self.previous_frame_end = Some(Box::new(sync::now(self.device.device().clone())) as Box<_>);
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
    let image_extent: [u32; 2] = self.device.window().inner_size().into();

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