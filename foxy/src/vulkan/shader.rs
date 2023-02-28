use std::{env, fs};
use std::fs::File;
use std::io::Read;
use std::marker::PhantomData;
use std::path::{PathBuf};
use std::sync::Arc;
use fxhash::FxHashMap;
use shaderc::SourceLanguage;
use strum::{
  IntoEnumIterator,
  VariantNames
};
use strum_macros::{EnumIter, EnumVariantNames};
use tracing::error;
use tracing_unwrap::{OptionExt, ResultExt};
use vulkano::shader::ShaderModule;
use crate::vulkan::device::FoxyDevice;

#[derive(EnumIter, EnumVariantNames, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[strum(serialize_all = "snake_case")]
pub enum ShaderStage {
  Vertex,
  Fragment,
  Compute,
  Geometry,
}

impl ShaderStage {
  fn as_shaderc(&self) -> shaderc::ShaderKind {
    match self {
      ShaderStage::Vertex => shaderc::ShaderKind::Vertex,
      ShaderStage::Fragment => shaderc::ShaderKind::Fragment,
      ShaderStage::Compute => shaderc::ShaderKind::Compute,
      ShaderStage::Geometry => shaderc::ShaderKind::Geometry,
    }
  }

  fn entrypoint(&self) -> String {
    match self {
      ShaderStage::Vertex => "vertex_main".into(),
      ShaderStage::Fragment => "fragment_main".into(),
      ShaderStage::Compute => "compute_main".into(),
      ShaderStage::Geometry => "geometry_main".into(),
    }
  }
}

#[derive(Default)]
pub struct ShaderCreateInfo {
  pub path: PathBuf,
  pub source: String,
  pub vertex: bool,
  pub fragment: bool,
  pub compute: bool,
  pub geometry: bool,
}

impl ShaderCreateInfo {
  pub fn has_stage(&self, stage: ShaderStage) -> bool {
    match stage {
      ShaderStage::Vertex => self.vertex,
      ShaderStage::Fragment => self.fragment,
      ShaderStage::Compute => self.compute,
      ShaderStage::Geometry => self.geometry,
    }
  }
}

pub struct Shader {
  shader_modules: FxHashMap<ShaderStage, Arc<ShaderModule>>
}

impl Shader {
  pub fn new(
    device: &FoxyDevice,
    info: ShaderCreateInfo,
  ) -> Self {
    // let file_str = Self::read_file(&info.path);
    let runtime_dir = env::current_exe().unwrap_or_log().parent().unwrap_or_log().join("tmp/res/shaders");
    let shader_cache_dir = runtime_dir.join(info.path.file_stem().unwrap_or_log());
    match fs::create_dir_all(&shader_cache_dir) {
      Ok(()) => {}
      Err(result) => {
        error!("{shader_cache_dir:?} | {result:?}");
      }
    }

    let shader_stages = Self::compile_to_spirv_binary(&info);

    Self { shader_modules: Default::default() }
  }

  fn read_file(path: &PathBuf) -> String {
    let mut result = String::new();
    File::open(path).expect_or_log("Failed to open file.")
      .read_to_string(&mut result)
      .expect_or_log("Failed to read file.");
    result
  }

  fn compile_to_spirv_binary(info: &ShaderCreateInfo) -> FxHashMap<ShaderStage, Vec<u32>> {
    let compiler = shaderc::Compiler::new().unwrap_or_log();
    let mut options = shaderc::CompileOptions::new().unwrap_or_log();
    options.set_source_language(SourceLanguage::HLSL);

    let mut shader_stages: FxHashMap<ShaderStage, Vec<u32>> = Default::default();

    for stage in ShaderStage::iter() {
      if info.has_stage(stage) {
        let binary_result = compiler.compile_into_spirv(
          info.source.as_str(),
          stage.as_shaderc(),
          info.path.to_str().unwrap_or_log(),
          stage.entrypoint().as_str(),
          Some(&options)
        ).unwrap_or_log();

        shader_stages.insert(stage, binary_result.as_binary().into());
      }
    }

    shader_stages
  }
}


#[derive(Default, Clone)]
pub struct ShaderStagesSpecified;
#[derive(Default, Clone)]
pub struct ShaderStagesMissing;

pub struct ShaderBuilder<S> {
  specified: PhantomData<S>,
  _create_info: ShaderCreateInfo,
}

impl ShaderBuilder<ShaderStagesMissing> {
  pub fn new(path: PathBuf, source: String) -> Self {
    Self {
      specified: PhantomData,
      _create_info: ShaderCreateInfo {
        path,
        source,
        vertex: false,
        fragment: false,
        compute: false,
        geometry: false,
      }
    }
  }
}

impl<S> ShaderBuilder<S> {
  pub fn with_stage(self, stage: ShaderStage) -> ShaderBuilder<ShaderStagesSpecified> {
    ShaderBuilder {
      specified: PhantomData,
      _create_info: ShaderCreateInfo {
        path: self._create_info.path,
        source: self._create_info.source,
        vertex: stage == ShaderStage::Vertex || self._create_info.vertex,
        fragment: stage == ShaderStage::Fragment || self._create_info.fragment,
        compute: stage == ShaderStage::Compute || self._create_info.compute,
        geometry: stage == ShaderStage::Geometry || self._create_info.geometry,
      }
    }
  }
}

impl ShaderBuilder<ShaderStagesSpecified> {
  pub fn build(self, device: &FoxyDevice) -> Shader {
    Shader::new(device, self._create_info)
  }
}

#[macro_export]
macro_rules! shader_builder {
  ($file_name:literal) => {{
    let source = include_str![$file_name];
    $crate::vulkan::shader::ShaderBuilder::new(PathBuf::from($file_name), source.into())
  }};
}