use std::{
  marker::PhantomData,
  sync::Arc,
  collections::HashSet
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tracing::info;

use wgpu::{ColorTargetState, ShaderModule, VertexBufferLayout};


#[derive(EnumIter, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ShaderStage {
  Vertex,
  Fragment,
  Compute,
  Geometry,
}

impl ShaderStage {
  pub fn entry_point(&self) -> &'static str {
    match self {
      ShaderStage::Vertex => "vertex_main",
      ShaderStage::Fragment => "fragment_main",
      ShaderStage::Compute => "compute_main",
      ShaderStage::Geometry => "geometry_main",
    }
  }
}

#[derive(Default, Clone)]
pub struct ShaderDescriptor {
  pub name: &'static str,
  pub source: &'static str,
  pub stages: HashSet<ShaderStage>,
}

#[allow(unused)]
pub struct Shader {
  module: ShaderModule,
  stages: HashSet<ShaderStage>,
}

impl Shader {
  #[allow(unused)]
  pub fn new(
    device: &wgpu::Device,
    descriptor: ShaderDescriptor
  ) -> Self {
    // wgsl means no shader cache anymore :(

    // load modules
    info!("[{:?}] Building shader...", &descriptor.name);
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some(descriptor.name),
      source: wgpu::ShaderSource::Wgsl(descriptor.source.into()),
    });
    info!("[{:?}] Built shader.", &descriptor.name);

    Self {
      module,
      stages: descriptor.stages
    }
  }

  #[allow(unused)]
  pub fn module(&self) -> &ShaderModule {
    &self.module
  }

  #[allow(unused)]
  pub fn stages(&self) -> &HashSet<ShaderStage> {
    &self.stages
  }

  #[allow(unused)]
  pub fn vertex_state<'a>(&'a self, buffers: &'a [VertexBufferLayout]) -> Option<wgpu::VertexState> {
    if self.stages.contains(&ShaderStage::Vertex) {
      Some(wgpu::VertexState {
        module: &self.module,
        entry_point: ShaderStage::Vertex.entry_point(),
        buffers,
      })
    } else {
      None
    }
  }

  #[allow(unused)]
  pub fn fragment_state<'a>(&'a self, targets: &'a [Option<ColorTargetState>]) -> Option<wgpu::FragmentState> {
    if self.stages.contains(&ShaderStage::Fragment) {
      Some(wgpu::FragmentState {
        module: &self.module,
        entry_point: ShaderStage::Fragment.entry_point(),
        targets,
      })
    } else {
      None
    }
  }
}

#[derive(Default, Clone)]
pub struct ShaderStagesMissing;
#[derive(Default, Clone)]
pub struct ShaderStagesSpecified;

pub struct ShaderBuilder<S> {
  _specified: PhantomData<S>,
  descriptor: ShaderDescriptor,
}

impl ShaderBuilder<ShaderStagesMissing> {
  pub fn new(name: &'static str, source: &'static str) -> Self {
    Self {
      _specified: PhantomData,
      descriptor: ShaderDescriptor {
        name,
        source,
        stages: Default::default()
      }
    }
  }

  pub fn detect_stages(self) -> ShaderBuilder<ShaderStagesSpecified> {
    let mut stages = HashSet::new();
    for stage in ShaderStage::iter() {
      if self.descriptor.source.contains(stage.entry_point()) {
        stages.insert(stage);
      }
    }

    ShaderBuilder {
      _specified: PhantomData,
      descriptor: ShaderDescriptor {
        name: self.descriptor.name,
        source: self.descriptor.source,
        stages
      }
    }
  }
}

#[allow(unused)]
impl<S> ShaderBuilder<S> {
  pub fn with_stage(self, stage: ShaderStage) -> ShaderBuilder<ShaderStagesSpecified> {
    let mut stages = self.descriptor.stages.clone();
    stages.insert(stage);

    ShaderBuilder {
      _specified: PhantomData,
      descriptor: ShaderDescriptor {
        name: self.descriptor.name,
        source: self.descriptor.source,
        stages
      }
    }
  }
}

impl ShaderBuilder<ShaderStagesSpecified> {
  pub fn build(self, device: &wgpu::Device) -> Arc<Shader> {
    Arc::new(Shader::new(device, self.descriptor))
  }
}

#[macro_export]
macro_rules! shader_builder {
  ($file_name:literal) => {{
    let source = include_str![$file_name];
    $crate::shader::ShaderBuilder::new($file_name, source.into())
  }};
}

#[macro_export]
macro_rules! include_shader {
  ($file_name:literal, $device:expr) => {{
    $crate::shader_builder![$file_name]
      .detect_stages()
      .build(&$device)
  }};
}