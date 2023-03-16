pub mod primitives;

use std::iter;
use std::sync::Arc;
use rgb::RGBA8;
use tracing::{error, trace};
use tracing_unwrap::{OptionExt, ResultExt};
use wgpu::SurfaceError;
use winit::window::Window;
use crate::canvas::{CanvasDescriptor, Visibility};
use crate::color::{FromHex, ToWGPU};
use crate::include_shader;
use crate::renderer::primitives::Vertex;

pub struct Renderer {
  window: Arc<Window>,
  surface: wgpu::Surface,
  surface_config: wgpu::SurfaceConfiguration,
  device: wgpu::Device,
  queue: wgpu::Queue,
  render_pipeline: wgpu::RenderPipeline,
  clear_color: RGBA8,
}

impl Renderer {
  pub fn new(window: Arc<Window>) -> Self {
    trace!("Initializing renderer...");
    // let rt = tokio::runtime::Runtime::new().unwrap_or_log();
    let rt = tokio::runtime::Builder::new_current_thread().build().unwrap_or_log();
    rt.block_on(rt.spawn(async move {
      let instance = wgpu::Instance::new(wgpu::InstanceDescriptor  {
        backends: wgpu::Backends::DX12 | wgpu::Backends::VULKAN | wgpu::Backends::METAL,
        dx12_shader_compiler: Default::default()
      });

      let surface = unsafe {
        instance.create_surface(window.as_ref())
      }.unwrap_or_log();

      let adapter = instance.request_adapter(
        &wgpu::RequestAdapterOptions {
          power_preference: wgpu::PowerPreference::HighPerformance,
          compatible_surface: Some(&surface),
          force_fallback_adapter: false,
        }
      ).await.unwrap_or_log();

      let surface_capabilities = surface.get_capabilities(&adapter);

      let surface_format = surface_capabilities.formats
                                               .iter()
                                               .copied()
                                               .find(|f| f.describe().srgb)
                                               .unwrap_or(surface_capabilities.formats[0]);

      let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: wgpu::PresentMode::AutoNoVsync,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![]
      };

      let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor {
          features: wgpu::Features::empty(),
          limits: wgpu::Limits::default(),
          label: None
        },
        None
      ).await.unwrap_or_log();

      surface.configure(&device, &surface_config);

      let shader = include_shader!["../res/shaders/simple.wgsl", device];

      let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
      });

      let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: shader.vertex_state(&[]).unwrap_or_log(),
        fragment: shader.fragment_state(&[
          Some(wgpu::ColorTargetState {
            format: surface_config.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL
          })
        ]),
        primitive: wgpu::PrimitiveState {
          topology: wgpu::PrimitiveTopology::TriangleList,
          strip_index_format: None,
          front_face: wgpu::FrontFace::Ccw,
          cull_mode: Some(wgpu::Face::Back),
          unclipped_depth: false,
          polygon_mode: wgpu::PolygonMode::Fill,
          conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
          count: 1,
          mask: !0,
          alpha_to_coverage_enabled: false
        },
        multiview: None,
      });

      let _vertices = [
        Vertex {
          position: [-0.5, 0.5, 0.0, 1.0],
          color: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex {
          position: [0.5, 0.5, 0.0, 1.0],
          color: [0.0, 1.0, 0.0, 1.0],
        },
        Vertex {
          position: [0.0, -0.5, 0.0, 1.0],
          color: [0.0, 0.0, 1.0, 1.0],
        },
      ];

      trace!("Initialized renderer.");

      Self {
        window,
        surface,
        surface_config,
        device,
        queue,
        render_pipeline,
        clear_color: RGBA8::from_hex("43bfefff"),
      }
    })).unwrap_or_log()
  }

  pub fn window(&self) -> Arc<Window> {
    self.window.clone()
  }

  pub fn from_canvas(window: Arc<Window>, descriptor: &CanvasDescriptor) -> Self {
    let renderer = Self::new(window.clone());

    if descriptor.visibility == Visibility::Wait {
      window.set_visible(true);
    }

    renderer
  }

  pub fn set_clear_color(&mut self, color: RGBA8) {
    self.clear_color = color;
  }

  pub fn render(&mut self) {
    match self.surface.get_current_texture() {
      Ok(output) => {
        if output.suboptimal {
          self.reconfigure();
        }

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
          label: Some("Rendering Encoder")
        });

        {
          let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
              view: &view,
              resolve_target: None,
              ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(
                  self.clear_color.to_wgpu()
                ),
                store: true
              }
            })],
            depth_stencil_attachment: None
          });

          render_pass.set_pipeline(&self.render_pipeline);
          render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();
      }
      Err(SurfaceError::Outdated) =>
        self.reconfigure(),
      Err(SurfaceError::Lost) =>
        self.reconfigure(),
      Err(err) => {
        error!("{err}");
      }
    };

  }

  pub fn reconfigure(&mut self) {
    let new_size = self.window.inner_size();
    if new_size.width > 0 && new_size.height > 0 {
      self.surface_config.width = new_size.width;
      self.surface_config.height = new_size.height;
      self.surface.configure(&self.device, &self.surface_config);
    }
  }
}