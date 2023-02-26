use std::sync::Arc;
use tracing::info;
use tracing_unwrap::{OptionExt, ResultExt};
use vulkano::{image::SwapchainImage, device::physical::{PhysicalDeviceType}, device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo}, VulkanLibrary, instance::{
  Instance,
  InstanceCreateInfo
}, Version, swapchain::{Surface, Swapchain, SwapchainCreateInfo}, single_pass_renderpass};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::image::ImageAccess;
use vulkano::image::view::ImageView;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano_win::{create_surface_from_winit, required_extensions};
use winit::{
  event_loop::EventLoop,
  window::{Window, WindowBuilder}
};
use kemono_transform::transform::Transform;
use crate::components::{Material, Mesh};

#[allow(dead_code)]
pub struct Renderer {
  instance: Arc<Instance>,
  surface: Arc<Surface>,
  device: Arc<Device>,
  queue: Arc<Queue>,
  swapchain: Arc<Swapchain>,
  images: Vec<Arc<SwapchainImage>>,
}

impl Renderer {
  pub fn from_window(window: Arc<Window>) -> Self {
    info!("Initializing renderer...");

    let instance = Self::create_vulkan_instance();
    let surface = create_surface_from_winit(window, instance.clone()).expect_or_log("Failed to create new Vulkan surface");
    let (device, queue) = Self::pick_vulkan_device(instance.clone(), surface.clone());

    let (mut swapchain, images) = Self::create_vulkan_swapchain(device.clone(), surface.clone());

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

    let mut framebuffers = Self::window_size_dependent_setup(&images, render_pass, &mut viewport);

    let mut recreate_swapchain = false;

    info!("Initialized renderer.");
    Self {
      instance,
      surface,
      device,
      queue,
      swapchain,
      images,
    }
  }

  pub fn new() -> (Self, EventLoop<()>) {
    let event_loop = EventLoop::new();
    let window = Arc::new(
      WindowBuilder::new()
        .build(&event_loop)
        .expect_or_log("Failed to build window")
    );

    (Self::from_window(window), event_loop)
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

  fn create_vulkan_swapchain(device: Arc<Device>, surface: Arc<Surface>) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>) {
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

    Swapchain::new(
      device,
      surface.clone(),
      SwapchainCreateInfo {
        min_image_count: capabilities.min_image_count,
        image_format: Some(image_format),
        image_extent,
        image_usage,
        composite_alpha,
        image_color_space,
        ..Default::default()
      }
    )
    .expect_or_log("Failed to create Vulkan swapchain")
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

  pub fn draw(&self, mesh: &Mesh, material: &Material, transform: &Transform) {

  }
}