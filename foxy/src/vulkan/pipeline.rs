use std::sync::Arc;
use tracing::{trace};
use tracing_unwrap::{ResultExt};
use vulkano::{
  image::SampleCount,
  pipeline::{
    graphics::{
      input_assembly::{InputAssemblyState},
      color_blend::{ColorBlendAttachmentState, ColorBlendState, ColorComponents},
      depth_stencil::{CompareOp, DepthBoundsState, DepthState, DepthStencilState},
      multisample::MultisampleState,
      rasterization::{CullMode, FrontFace, PolygonMode, RasterizationState},
      vertex_input::{BuffersDefinition},
      viewport::ViewportState
    },
    GraphicsPipeline,
    PipelineLayout,
    StateMode,
  },
  render_pass::{Subpass},
  command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
};
use vulkano::pipeline::graphics::viewport::{Scissor, Viewport};
use winit::dpi::PhysicalSize;
use crate::vulkan::device::EngineDevice;
use crate::vulkan::shader::{Shader, ShaderStage};

pub struct EnginePipeline {
  pipeline: Arc<GraphicsPipeline>,
  // shader: Arc<Shader>,
  // device: Arc<EngineDevice>
}

impl EnginePipeline {
  pub fn new(
    device: Arc<EngineDevice>,
    shader: Arc<Shader>,
    subpass: Subpass,
    layout: Option<Arc<PipelineLayout>>,
    vertex_definition: BuffersDefinition,
    create_info: PipelineCreateInfo
  ) -> Self {
    trace!("Initializing graphics pipeline...");

    let pipeline = Self::create_pipeline(device, shader, subpass, layout, vertex_definition, create_info);

    trace!("Initialized graphics pipeline.");
    Self {
      pipeline,
      // shader,
      // device,
    }
  }

  pub fn pipeline(&self) -> Arc<GraphicsPipeline> {
    self.pipeline.clone()
  }

  pub fn bind<'a>(
    &self, command_buffer:
    &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>
  ) -> &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> {
    command_buffer.bind_pipeline_graphics(self.pipeline.clone())
  }

  fn create_pipeline(
    device: Arc<EngineDevice>,
    shader: Arc<Shader>,
    subpass: Subpass,
    layout: Option<Arc<PipelineLayout>>,
    vertex_definition: BuffersDefinition,
    create_info: PipelineCreateInfo
  ) -> Arc<GraphicsPipeline> {
    let mut builder = GraphicsPipeline::start()
      .vertex_input_state(vertex_definition)
      .input_assembly_state(create_info.input_assembly_state)
      .viewport_state(create_info.viewport_state)
      .rasterization_state(create_info.rasterization_state)
      .multisample_state(create_info.multisample_state)
      .depth_stencil_state(create_info.depth_stencil_state)
      .color_blend_state(create_info.color_blend_state)
      .render_pass(subpass);

    for stage in shader.shader_modules().keys() {
      if let Some(entry_point) = shader.entry_point(*stage) {
        builder = match stage {
          ShaderStage::Vertex   => builder.vertex_shader(entry_point, ()),
          ShaderStage::Fragment => builder.fragment_shader(entry_point, ()),
          ShaderStage::Geometry => builder.geometry_shader(entry_point, ()),
          ShaderStage::Compute => builder, // no compute support in GraphicsPipeline
        };
      }
    }

    let result = match layout {
      Some(value) => {
        builder.with_pipeline_layout(device.device(), value)
      }
      None => {
        let x = builder.build(device.device());
        x
      }
    };

    result.unwrap_or_log()
  }
}

#[derive(Default, Clone)]
pub struct ShaderMissing;
#[derive(Clone)]
pub struct ShaderSet(Arc<Shader>);
#[derive(Default, Clone)]
pub struct SubpassMissing;
#[derive(Clone)]
pub struct SubpassSet(Subpass);
#[derive(Default, Clone)]
pub struct LayoutMissing;
#[derive(Clone)]
pub struct LayoutSet(Arc<PipelineLayout>);
#[derive(Default, Clone)]
pub struct VertexDefinitionMissing;
#[derive(Default, Clone)]
pub struct VertexDefinitionSet(BuffersDefinition);

pub struct EnginePipelineBuilder<Sh, Sp, L, V> {
  shader: Sh,
  subpass: Sp,
  layout: L,
  vertex: V,
  create_info: PipelineCreateInfo
}

impl EnginePipelineBuilder<ShaderMissing, SubpassMissing, LayoutMissing, VertexDefinitionMissing> {
  pub fn new() -> Self {
    Self {
      shader: ShaderMissing,
      subpass: SubpassMissing,
      layout: LayoutMissing,
      vertex: VertexDefinitionMissing,
      create_info: PipelineCreateInfo::default()
    }
  }

  // pub fn from_size(size: PhysicalSize<u32>) -> Self {
  //   Self {
  //     shader: ShaderMissing,
  //     subpass: SubpassMissing,
  //     layout: LayoutMissing,
  //     vertex: VertexDefinitionMissing,
  //     create_info: PipelineCreateInfo::from_size(size)
  //   }
  // }
}

impl<Sh, Sp, L, V> EnginePipelineBuilder<Sh, Sp, L, V> {
  pub fn shader(self, s: Arc<Shader>) -> EnginePipelineBuilder<ShaderSet, Sp, L, V> {
    EnginePipelineBuilder {
      shader: ShaderSet(s),
      subpass: self.subpass,
      layout: self.layout,
      vertex: self.vertex,
      create_info: self.create_info,
    }
  }

  pub fn subpass(self, s: Subpass) -> EnginePipelineBuilder<Sh, SubpassSet, L, V> {
    EnginePipelineBuilder {
      shader: self.shader,
      subpass: SubpassSet(s),
      layout: self.layout,
      vertex: self.vertex,
      create_info: self.create_info,
    }
  }

  pub fn layout(self, l: Arc<PipelineLayout>) -> EnginePipelineBuilder<Sh, Sp, LayoutSet, V> {
    EnginePipelineBuilder {
      shader: self.shader,
      subpass: self.subpass,
      layout: LayoutSet(l),
      vertex: self.vertex,
      create_info: self.create_info,
    }
  }

  pub fn vertex(self, v: BuffersDefinition) -> EnginePipelineBuilder<Sh, Sp, L, VertexDefinitionSet> {
    EnginePipelineBuilder {
      shader: self.shader,
      subpass: self.subpass,
      layout: self.layout,
      vertex: VertexDefinitionSet(v),
      create_info: self.create_info,
    }
  }

  #[allow(unused)]
  pub fn create_info(self, c: PipelineCreateInfo) -> EnginePipelineBuilder<Sh, Sp, L, V> {
    EnginePipelineBuilder {
      shader: self.shader,
      subpass: self.subpass,
      layout: self.layout,
      vertex: self.vertex,
      create_info: c,
    }
  }
}

impl EnginePipelineBuilder<ShaderSet, SubpassSet, LayoutSet, VertexDefinitionSet> {
  pub fn build(self, device: Arc<EngineDevice>) -> Arc<EnginePipeline> {
    Arc::new(EnginePipeline::new(
      device,
      self.shader.0,
      self.subpass.0,
      Some(self.layout.0),
      self.vertex.0,
      self.create_info
    ))
  }
}

impl EnginePipelineBuilder<ShaderSet, SubpassSet, LayoutMissing, VertexDefinitionSet> {
  pub fn build_auto_layout(self, device: Arc<EngineDevice>) -> Arc<EnginePipeline> {
    Arc::new(EnginePipeline::new(
      device,
      self.shader.0,
      self.subpass.0,
      None,
      self.vertex.0,
      self.create_info
    ))
  }
}

pub struct PipelineCreateInfo {
  pub input_assembly_state: InputAssemblyState,
  pub viewport_state: ViewportState,
  pub rasterization_state: RasterizationState,
  pub multisample_state: MultisampleState,
  pub depth_stencil_state: DepthStencilState,
  pub color_blend_state: ColorBlendState,
}

impl PipelineCreateInfo {
  pub fn from_size(size: PhysicalSize<u32>) -> Self {
    let viewport = Viewport {
      origin:      [0., 0.],
      dimensions:  [size.width as f32, size.height as f32],
      depth_range: 0.0..1.0,
    };
    let scissor = Scissor {
      origin:     [0, 0],
      dimensions: [size.width, size.height],
    };

    let viewport_state = ViewportState::viewport_fixed_scissor_fixed(vec![(viewport, scissor)]);

    Self {
      viewport_state,
      ..Default::default()
    }
  }

  pub fn new() -> Self {
    let input_assembly_state = InputAssemblyState::new();
    let viewport_state = ViewportState::viewport_dynamic_scissor_irrelevant();

    let rasterization_state = RasterizationState {
      depth_clamp_enable: false,
      rasterizer_discard_enable: StateMode::Fixed(false),
      polygon_mode: PolygonMode::Fill,
      cull_mode: StateMode::Fixed(CullMode::Back),
      front_face: StateMode::Fixed(FrontFace::CounterClockwise),
      depth_bias: None,
      line_width: StateMode::Fixed(1.0),
      line_rasterization_mode: Default::default(),
      line_stipple: None,
    };

    let multisample_state = MultisampleState {
      rasterization_samples: SampleCount::Sample1,
      sample_shading: None,
      sample_mask: [0xFFFFFFFF; 2],
      alpha_to_coverage_enable: false,
      alpha_to_one_enable: false,
    };

    let color_blend_state = ColorBlendState {
      logic_op: None, //Some(StateMode::Fixed(LogicOp::Copy)),
      attachments: vec![
        ColorBlendAttachmentState {
          blend: None,
          color_write_mask: ColorComponents::all(),
          color_write_enable: StateMode::Fixed(true),
        }
      ],
      blend_constants: StateMode::Fixed([0.0, 0.0, 0.0, 0.0]),
    };

    let depth_stencil_state = DepthStencilState {
      depth: Some(DepthState {
        enable_dynamic: false,
        write_enable: StateMode::Fixed(true),
        compare_op: StateMode::Fixed(CompareOp::Less),
      }),
      depth_bounds: Some(DepthBoundsState {
        enable_dynamic: false,
        bounds: StateMode::Fixed(0.0..=1.0)
      }),
      stencil: None,
    };

    Self {
      input_assembly_state,
      viewport_state,
      rasterization_state,
      multisample_state,
      depth_stencil_state,
      color_blend_state,
    }
  }
}

impl Default for PipelineCreateInfo {
  fn default() -> Self {
    let input_assembly_state = InputAssemblyState::new();
    let viewport_state = ViewportState::viewport_dynamic_scissor_irrelevant();
    let rasterization_state = RasterizationState::new();
    let multisample_state = MultisampleState::new();
    let color_blend_state = ColorBlendState::new(1);
    let depth_stencil_state = DepthStencilState::disabled();

    Self {
      input_assembly_state,
      viewport_state,
      rasterization_state,
      multisample_state,
      depth_stencil_state,
      color_blend_state,
    }
  }
}