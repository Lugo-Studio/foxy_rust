use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FoxyError {
  #[error("Runtime error [{0}]")]
  RuntimeError(&'static str),
  #[error("Failed to access file \"{0:?}\"")]
  FileIoError(PathBuf),
}