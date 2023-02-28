use std::sync::Arc;
use tracing::{error, warn};
use tracing_unwrap::{OptionExt, ResultExt};
use vulkano::command_buffer::pool::{CommandPool, CommandPoolCreateInfo};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo};
use vulkano::format::Format;
use vulkano::instance::debug::{DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger, DebugUtilsMessengerCreateInfo};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::swapchain::{ColorSpace, PresentMode, Surface, SurfaceCapabilities};
use vulkano::{Version, VulkanLibrary};
use vulkano::device::physical::{PhysicalDeviceType};
use winit::window::Window;

pub struct FoxyDevice {
  _debug_messenger: Option<DebugUtilsMessenger>,

  instance: Arc<Instance>,
  surface: Arc<Surface>,
  window: Arc<Window>,
  device: Arc<Device>,

  command_pool: CommandPool,
  queues: QueueFamilies,
}

impl FoxyDevice { // Public
  pub fn new(
    window: Arc<Window>,
    app_name: String,
    app_version: Version,
    engine_name: String,
    engine_version: Version
  ) -> Self {
    let instance = Self::create_instance(app_name, app_version, engine_name, engine_version);
    let _debug_messenger = Self::create_debug_messenger(instance.clone());
    let surface = vulkano_win::create_surface_from_winit(window.clone(),instance.clone())
      .expect_or_log("Failed to create new Vulkan surface");
    let (device, queues) = Self::pick_vulkan_device(instance.clone(), surface.clone());
    let command_pool = Self::create_command_pool(device.clone(), queues.clone());

    Self {
      _debug_messenger,
      instance,
      surface,
      window,
      device,
      command_pool,
      queues,
    }
  }

  pub fn device(&self) -> Arc<Device> {
    self.device.clone()
  }

  pub fn surface(&self) -> Arc<Surface> {
    self.surface.clone()
  }

  pub fn window(&self) -> Arc<Window> {
    self.window.clone()
  }

  pub fn queues(&self) -> QueueFamilies {
    self.queues.clone()
  }

  pub fn command_pool(&self) -> &CommandPool {
    &self.command_pool
  }

  pub fn command_pool_mut(&mut self) -> &mut CommandPool {
    &mut self.command_pool
  }

  pub fn swapchain_support(&mut self) -> SwapchainSupportDetails {
    let capabilities = self.device
      .physical_device()
      .surface_capabilities(&self.surface, Default::default())
      .expect_or_log("Failed to access surface capabilities");

    let formats = self.device
      .physical_device()
      .surface_formats(&self.surface, Default::default())
      .unwrap_or_log();

    let present_modes = self.device
      .physical_device()
      .surface_present_modes(&self.surface)
      .unwrap_or_log()
      .collect();

    SwapchainSupportDetails {
      capabilities,
      formats,
      present_modes
    }
  }
}

impl FoxyDevice { // Private
  fn create_instance(
    application_name: String,
    application_version: Version,
    engine_name: String,
    engine_version: Version
  ) -> Arc<Instance> {
    let library = VulkanLibrary::new()
      .expect_or_log("Failed to load Vulkan library");

    let enabled_extensions = vulkano_win::required_extensions(&library);

    Instance::new(
      library,
      InstanceCreateInfo {
        enabled_extensions,
        enumerate_portability: false,
        max_api_version: Some(Version::V1_2),
        application_name: Some(application_name),
        application_version,
        engine_name: Some(engine_name),
        engine_version,
        ..Default::default()
      }
    ).expect_or_log("Failed to create new Vulkan instance")
  }

  fn create_debug_messenger(instance: Arc<Instance>) -> Option<DebugUtilsMessenger> {
    if !Self::validation_layer_supported(instance.library()) {
      warn!("Validation layers not available.");
      return None;
    }

    let mut debug_messenger_create_info = DebugUtilsMessengerCreateInfo::user_callback(
      Arc::new(|message| {
        match message.severity {
          DebugUtilsMessageSeverity { error: true, .. } => { error!("VULKAN: {}", message.description); }
          DebugUtilsMessageSeverity { warning: true, .. } => { warn!("VULKAN: {}", message.description); }
          _ => {}
        }
      })
    );

    debug_messenger_create_info.message_severity = DebugUtilsMessageSeverity {
      error: true,
      warning: true,
      information: false,
      verbose: false,
      ..Default::default()
    };

    debug_messenger_create_info.message_type = DebugUtilsMessageType {
      general: true,
      validation: true,
      performance: true,
      ..Default::default()
    };

    unsafe {
      if cfg!(debug_assertions) {
        Some(
          DebugUtilsMessenger::new(instance, debug_messenger_create_info)
            .unwrap_or_log()
        )
      } else {
        None
      }
    }
  }

  fn validation_layer_supported(library: &VulkanLibrary) -> bool {
    let validation_layers = vec![
      "VK_LAYER_KHRONOS_validation"
    ];

    for layer_name in validation_layers {
      let mut layer_found = false;

      for layer_properties in library.layer_properties().unwrap_or_log() {
        if layer_name == layer_properties.name() {
          layer_found = true;
          break;
        }
      }

      if !layer_found {
        return layer_found;
      }
    }

    true
  }

  fn pick_vulkan_device(instance: Arc<Instance>, surface: Arc<Surface>) -> (Arc<Device>, QueueFamilies) {
    let device_extensions = DeviceExtensions {
      khr_swapchain: true,
      ..DeviceExtensions::empty()
    };

    let (physical_device, queue_family_index) = instance
      .enumerate_physical_devices()
      .unwrap_or_log()
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
          QueueCreateInfo {
            queue_family_index,
            ..Default::default()
          },
          QueueCreateInfo {
            queue_family_index,
            ..Default::default()
          }
        ],
        ..Default::default()
      }
    ).unwrap_or_log();

    let graphics = queues.next().unwrap_or_log();
    let present = queues.next().unwrap_or_log();
    (device, QueueFamilies { graphics, present })
  }

  fn create_command_pool(device: Arc<Device>, queues: QueueFamilies) -> CommandPool {
    CommandPool::new(
      device,
      CommandPoolCreateInfo {
        queue_family_index: queues.graphics.queue_family_index(),
        transient: true,
        reset_command_buffer: true,
        ..Default::default()
      }
    ).unwrap_or_log()
  }
}

pub struct SwapchainSupportDetails {
  pub capabilities: SurfaceCapabilities,
  pub formats: Vec<(Format, ColorSpace)>,
  pub present_modes: Vec<PresentMode>,
}

#[derive(Clone)]
pub struct QueueFamilies {
  pub graphics: Arc<Queue>,
  pub present: Arc<Queue>,
}
