use std::path::Path;
use std::sync::Arc;
use vulkano::device::Device;
use vulkano::shader::ShaderModule;

pub struct FoxyShaderCreateInfo {
  pub path: Path,
  vertex:
}

pub struct FoxyShader {
  shader_module: Arc<ShaderModule>
}

impl FoxyShader {
  pub fn new(device: Arc<Device>) -> Self {
    Self {}
  }
}