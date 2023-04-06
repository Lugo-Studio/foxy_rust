mod state;

use tracing::{Level, trace};

pub struct App {

}

impl App {
  pub fn new(enable_logging: bool) -> Self {
    if enable_logging {
      tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_max_level(Level::INFO)
        .init();
    }
    trace!("Initializing framework...");

    Self {

    }
  }

  pub fn run(self) {

  }
}

impl Default for App {
  fn default() -> Self {
    Self::new(false)
  }
}