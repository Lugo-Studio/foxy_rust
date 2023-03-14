use std::path::PathBuf;
use thiserror::Error;
use vulkano::format::Format;
use vulkano::swapchain::{ColorSpace, PresentMode};
use crate::vulkan::shader::ShaderStage;

#[derive(Error, Debug)]
pub enum FoxyError {
  #[error("Runtime error [{0}]")]
  RuntimeError(&'static str),
  #[error("Failed to access file \"{0:?}\"")]
  FileIoError(PathBuf),
}

#[derive(Error, Debug)]
pub enum ShaderError {
  #[error("Failed to access file \"{0:?}\"")]
  FileIoError(PathBuf),
  #[error("Shader stage \"{0:?}\" not in cache")]
  StageNotCached(ShaderStage),
  #[error("Shader stage \"{0:?}\" cache not valid")]
  StageCacheInvalid(ShaderStage),
  #[error("Shader stage \"{0:?}\" failed to compile: [{1}]")]
  CompilationError(ShaderStage, String),
  #[error("Shader modules failed to build [{0}]")]
  ModuleBuildError(String),
}

#[derive(Error, Debug)]
pub enum SwapchainError {
  #[error("Present mode \"{0:?}\" unsupported on device")]
  InvalidPresentMode(PresentMode),
  #[error("Format \"{0:?}\" unsupported on device")]
  InvalidFormatMode((Format, ColorSpace)),
}