pub mod vsync;
pub mod primitives;

use std::sync::Arc;
use egui_winit_vulkano::{Gui, GuiConfig};
use tracing::{trace, warn};
use tracing_unwrap::{OptionExt, ResultExt};
use vulkano::{
  command_buffer::{
    allocator::StandardCommandBufferAllocator,
    AutoCommandBufferBuilder,
    CommandBufferUsage,
    RenderPassBeginInfo,
    SubpassContents
  },
  format::ClearValue,
  render_pass::Subpass,
  Version
};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use winit::event::WindowEvent;
use winit::window::Window;
use crate::{
  vulkan::{
    device::EngineDevice,
    pipeline::{EnginePipeline, EnginePipelineBuilder},
    swapchain::{EngineSwapchain}
  },
  include_shader,
  renderer::vsync::VsyncMode
};
use crate::canvas::Canvas;
use crate::renderer::primitives::Vertex;

#[allow(dead_code)]
pub struct Renderer {
  device: Arc<EngineDevice>,
  swapchain: EngineSwapchain,
  pipeline: Arc<EnginePipeline>,

  vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,

  command_buffer_allocator: StandardCommandBufferAllocator,
  memory_allocator: Arc<StandardMemoryAllocator>,

  clear_colors: Vec<Option<ClearValue>>,

  gui: Gui,
}

impl Renderer {
  pub fn from_canvas(canvas: &Canvas, vsync_mode: VsyncMode) -> Self {
    trace!("Initializing renderer...");

    let device = Arc::new(EngineDevice::new(
      canvas.window(),
      "Foxy App".into(),
      Version::major_minor(0, 1),
      "Foxy Renderer".into(),
      Version::major_minor(0, 1),
    ));

    let command_buffer_allocator = StandardCommandBufferAllocator::new(
      device.device(),
      Default::default(),
    );

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.device()));

    let swapchain = EngineSwapchain::new(device.clone(), vsync_mode.to_present_mode());

    let shader = include_shader!["../res/shaders/simple.hlsl", device.clone()];

    let pipeline = EnginePipelineBuilder::new()
      .shader(shader)
      .subpass(Subpass::from(swapchain.render_pass(), 0).unwrap_or_log())
      .vertex(BuffersDefinition::new().vertex::<Vertex>())
      .build_auto_layout(device.clone());

    let clear_colors = vec![
      Some([0.0, 0.69, 1.0, 1.0].into())
    ];

    let vertices = [
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

    let vertex_buffer = CpuAccessibleBuffer::from_iter(
      &memory_allocator,
      BufferUsage {
        vertex_buffer: true,
        ..BufferUsage::empty()
      },
      false,
      vertices,
    ).unwrap_or_log();

    let gui = Gui::new(canvas.event_loop(), device.surface(), device.queues().graphics, GuiConfig::default());

    trace!("Initialized renderer.");

    Self {
      device,
      swapchain,
      pipeline,
      vertex_buffer,
      command_buffer_allocator,
      memory_allocator,
      clear_colors,
      gui,
    }
  }

  pub fn vsync_mode(&self) -> VsyncMode {
    self.swapchain.present_mode().into()
  }

  pub fn set_vsync_mode(&mut self, vsync_mode: VsyncMode) {
    match self.swapchain.set_present_mode(vsync_mode.to_present_mode()) {
      Ok(_) => {}
      Err(err) => {
        warn!("{err}");
      }
    };
  }

  pub fn update_gui(&mut self, event: WindowEvent) {
    self.gui.update(&event);
  }

  pub fn reset(&mut self) {
    self.swapchain.end_previous_frame();
  }

  pub fn render(&mut self) {
    if let Some((image_index, acquire_future)) = self.swapchain.acquire_next_image() {
      self.gui.immediate_ui(|gui| {
        let ctx = gui.context();


      });

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
                self.swapchain.framebuffer(image_index)
              )
            },
            SubpassContents::Inline
          )
          .unwrap_or_log()
          .set_viewport(0, [self.device.viewport()])
          .bind_pipeline_graphics(self.pipeline.pipeline())
          .bind_vertex_buffers(0, self.vertex_buffer.clone())
          .draw(self.vertex_buffer.len() as u32, 1, 0, 0)
          .unwrap_or_log()
          .end_render_pass()
          .unwrap_or_log();

        command_buffer_builder.build().unwrap_or_log()
      };

      self.swapchain.present(
        acquire_future,
        command_buffer,
        &mut self.gui,
        image_index
      );
    }
  }

  pub fn recreate_swapchain(&mut self) {
    self.swapchain.recreate();
  }
}