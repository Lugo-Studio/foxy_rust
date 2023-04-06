use futures::executor::block_on;
use tracing_unwrap::{ResultExt, OptionExt};

use crate::graphics::window::Window;

#[allow(unused)]
pub struct Renderer {
  surface: wgpu::Surface,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
}

impl Renderer {
  pub fn new(window: &Window) -> Self {
    let size = window.size();
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::VULKAN | wgpu::Backends::DX12,
      dx12_shader_compiler: Default::default(),
    });
    let surface = unsafe { instance.create_surface(window.winit()) }.unwrap_or_log();
    let adapter = block_on(async {
      instance.request_adapter(
        &wgpu::RequestAdapterOptions {
          power_preference: wgpu::PowerPreference::HighPerformance,
          compatible_surface: Some(&surface),
          force_fallback_adapter: false,
        }
      ).await.unwrap_or_log()
    });

    let (device, queue) = block_on(async {
      adapter.request_device(
        &wgpu::DeviceDescriptor {
          features: wgpu::Features::empty(),
          limits: wgpu::Limits::default(),
          label: None,
        }, 
        None,
      ).await.unwrap_or_log()
    });
    let surface_capabilities = surface.get_capabilities(&adapter);
    let surface_format = surface_capabilities
      .formats
      .iter()
      .copied()
      .find(|f| f.describe().srgb)
      .unwrap_or(surface_capabilities.formats[0]);
    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width,
      height: size.height,
      present_mode: wgpu::PresentMode::AutoNoVsync,
      alpha_mode: wgpu::CompositeAlphaMode::Auto,
      view_formats: vec![]
    };
    surface.configure(&device, &config);
    
    Self {  
      surface,
      device,
      queue,
      config,
    }
  }

  pub fn render_frame(&mut self) {

  }

  pub fn end_frame(&mut self) {

  }
}