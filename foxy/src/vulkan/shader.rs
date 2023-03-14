use std::{
  env,
  fs,
  marker::PhantomData,
  path::{PathBuf},
  sync::Arc,
  fs::File,
  io::{Read, Write},
  path::Path
};
use fxhash::{FxHashMap, FxHashSet};
use shaderc::{OptimizationLevel, SourceLanguage};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use tracing::{error, info, trace};
use tracing_unwrap::{OptionExt, ResultExt};
use vulkano::shader::{EntryPoint, ShaderModule};
use crate::error::{ShaderError};
use crate::vulkan::device::EngineDevice;

#[derive(EnumIter, Display, Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

  pub fn entry_point(&self) -> String {
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
  pub stages: FxHashSet<ShaderStage>,
}

impl ShaderCreateInfo {
  pub fn has_stage(&self, stage: ShaderStage) -> bool {
    self.stages.contains(&stage)
  }
}

pub struct Shader {
  shader_modules: FxHashMap<ShaderStage, Arc<ShaderModule>>
}

impl Shader {
  pub fn new(
    device: Arc<EngineDevice>,
    info: ShaderCreateInfo,
  ) -> Self {
    let runtime_dir = env::current_exe().unwrap_or_log().parent().unwrap_or_log().join("tmp/res/shaders");
    let shader_cache_dir = runtime_dir.join(info.path.file_stem().unwrap_or_log());

    // create cache
    match fs::create_dir_all(&shader_cache_dir) {
      Ok(()) => {}
      Err(result) => {
        error!("{shader_cache_dir:?} | {result}");
      }
    }

    // load modules
    info!("[{:?}] Loading shader...", &info.path);
    let shader_modules = match Self::fetch_shader_bytecode(&info, &shader_cache_dir) {
      Ok(result) => {
        trace!("[{:?}] Building modules...", &info.path);
        Self::build_shader_modules(device, &info, result)
      }
      Err(err) => {
        error!("{shader_cache_dir:?} | {err}");
        Default::default()
      },
    };

    Self { shader_modules }
  }

  pub fn shader_modules(&self) -> &FxHashMap<ShaderStage, Arc<ShaderModule>> {
    &self.shader_modules
  }

  pub fn entry_point(&self, stage: ShaderStage) -> Option<EntryPoint> {
    match self.shader_modules.get(&stage) {
      Some(module) => module.entry_point(stage.entry_point().as_str()),
      None => None
    }
  }

  fn fetch_shader_bytecode(info: &ShaderCreateInfo, shader_cache_dir: &Path) -> Result<FxHashMap<ShaderStage, Vec<u8>>, ShaderError> {
    let mut stage_bytecodes: FxHashMap<ShaderStage, Vec<u8>> = Default::default();

    for stage in ShaderStage::iter().filter(|s| info.has_stage(*s)) {
      match Self::fetch_stage_bytecode(info, stage, shader_cache_dir) {
        Ok(bytecode) => {
          stage_bytecodes.insert(stage, bytecode);
        }
        Err(error) => error!("{error}")
      }
    }

    Ok(stage_bytecodes)
  }

  fn fetch_stage_bytecode(info: &ShaderCreateInfo, stage: ShaderStage, shader_cache_dir: &Path) -> Result<Vec<u8>, ShaderError> {
    let cached_stage_file_path = shader_cache_dir.join(stage.to_string()).with_extension("spv");

    if cached_stage_file_path.exists() && Self::cached_file_younger_than_exe(&cached_stage_file_path) {
      trace!("[{stage:?}] Reading cached stage...");
      match File::open(&cached_stage_file_path) {
        Ok(file) => Ok(file.bytes().map(|b| b.unwrap()).collect::<Vec<u8>>()),
        Err(..) => Err(ShaderError::FileIoError(cached_stage_file_path))
      }
    } else {
      trace!("[{stage:?}] Recompiling stage...");
      Self::compile_shader_type(&info.source, stage, &cached_stage_file_path)
    }
  }

  fn cached_file_younger_than_exe(cached_file: &Path) -> bool {
    let file_age = cached_file.metadata().unwrap_or_log().modified().unwrap_or_log();
    let exe_age = env::current_exe().unwrap_or_log().metadata().unwrap_or_log().modified().unwrap_or_log();
    // info!("File age: [{file_age:?}], Exe age: [{exe_age:?}]");
    file_age >= exe_age
  }

  fn compile_shader_type(source: &str, stage: ShaderStage, cached_stage_file_path: &Path) -> Result<Vec<u8>, ShaderError> {
    let compiler = shaderc::Compiler::new().unwrap_or_log();
    let mut options = shaderc::CompileOptions::new().unwrap_or_log();
    options.set_source_language(SourceLanguage::HLSL);

    /* TODO:
        So this kills the release mode apparently LMAO. definitely something to look into...
        I should allow reading file directory as list of multiple individual shader files.
        This will allow for optimizations only if the shader is a multi-file shader.
    */
    //
    // if cfg!(not(debug_assertions)) {
    //   options.set_optimization_level(OptimizationLevel::Performance);
    // }

    match compiler.compile_into_spirv(
      source,
      stage.as_shaderc(),
      cached_stage_file_path.file_name().unwrap_or_log().to_str().unwrap_or_log(),
      stage.entry_point().as_str(),
      Some(&options)
    ) {
      Ok(result) => {
        trace!("[{stage:?}] Compiled stage.");

        match File::create(cached_stage_file_path) {
          Ok(mut file) => {
            match file.write_all(result.as_binary_u8()) {
              Ok(_) => trace!("[{stage:?}] Cached stage."),
              Err(_) => error!("[{stage:?}] Failed to write stage to shader cache.")
            }
          }
          Err(_) => error!("[{stage:?}] Failed to write stage to shader cache.")
        }

        Ok(result.as_binary_u8().into())
      }
      Err(err) => Err(ShaderError::CompilationError(stage, err.to_string()))
    }
  }

  fn build_shader_modules(
    device: Arc<EngineDevice>,
    info: &ShaderCreateInfo,
    mut stage_bytecodes: FxHashMap<ShaderStage, Vec<u8>>
  ) -> FxHashMap<ShaderStage, Arc<ShaderModule>> {
    let mut shader_modules: FxHashMap<ShaderStage, Arc<ShaderModule>> = Default::default();

    'stage_loop: for stage in ShaderStage::iter().filter(|s| info.has_stage(*s)) {
      if let Some(bytecode) = stage_bytecodes.get_mut(&stage) {
        let shader_module = unsafe {
          let mut attempt = 0;

          loop {
            match ShaderModule::from_bytes(device.device(), bytecode) {
              Ok(module) => break module,
              Err(err) => {
                if attempt >= 2 {
                  error!("Could not recover from shader module creation failure ({err})");
                  continue 'stage_loop;
                }

                error!("Shader module creation failure, attempting to recompile ({err})");
                let runtime_dir = env::current_exe().unwrap_or_log().parent().unwrap_or_log().join("tmp/res/shaders");
                let shader_cache_dir = runtime_dir.join(info.path.file_stem().unwrap_or_log());
                let cached_stage_file_path = shader_cache_dir.with_file_name(stage.to_string()).with_extension("spv");
                if let Ok(code) = Self::compile_shader_type(&info.source, stage, &cached_stage_file_path) {
                  bytecode.clear();
                  bytecode.extend(code);
                };

                attempt += 1;
              }
            }
          }
        };

        shader_modules.insert(stage, shader_module);
      }
    }

    info!("[{:?}] Loaded shader.", &info.path);
    shader_modules
  }
}


#[derive(Default, Clone)]
pub struct ShaderStagesMissing;
#[derive(Default, Clone)]
pub struct ShaderStagesSpecified;

pub struct ShaderBuilder<S> {
  specified: PhantomData<S>,
  create_info: ShaderCreateInfo,
}

impl ShaderBuilder<ShaderStagesMissing> {
  pub fn new(path: PathBuf, source: String) -> Self {
    Self {
      specified: PhantomData,
      create_info: ShaderCreateInfo {
        path,
        source,
        stages: Default::default()
      }
    }
  }

  pub fn detect_stages(self) -> ShaderBuilder<ShaderStagesSpecified> {
    let mut create_info = ShaderCreateInfo {
      path: self.create_info.path,
      source: self.create_info.source,
      stages: self.create_info.stages,
    };

    for stage in ShaderStage::iter() {
      if create_info.source.contains(stage.entry_point().as_str()) {
        create_info.stages.insert(stage);
      }
    }

    ShaderBuilder {
      specified: PhantomData,
      create_info
    }
  }
}

#[allow(unused)]
impl<S> ShaderBuilder<S> {
  pub fn with_stage(self, stage: ShaderStage) -> ShaderBuilder<ShaderStagesSpecified> {
    let mut create_info = ShaderCreateInfo {
      path: self.create_info.path,
      source: self.create_info.source,
      stages: self.create_info.stages,
    };
    create_info.stages.insert(stage);

    ShaderBuilder {
      specified: PhantomData,
      create_info
    }
  }
}

impl ShaderBuilder<ShaderStagesSpecified> {
  pub fn build(self, device: Arc<EngineDevice>) -> Arc<Shader> {
    Arc::new(Shader::new(device, self.create_info))
  }
}

#[macro_export]
macro_rules! shader_builder {
  ($file_name:literal) => {{
    let source = include_str![$file_name];
    $crate::vulkan::shader::ShaderBuilder::new(std::path::PathBuf::from($file_name), source.into())
  }};
}

#[macro_export]
macro_rules! include_shader {
  ($file_name:literal, $device:expr) => {{
    $crate::shader_builder![$file_name]
      .detect_stages()
      .build($device)
  }};
}