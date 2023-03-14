mod state;

use tracing::{Level, trace};
use foxy::canvas::Canvas;
use crate::app::state::State;

pub struct App {
  canvas: Canvas,
  state: State,
}

impl App {
  pub fn new(enable_logging: bool) -> Self {
    if enable_logging {
      tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_max_level(Level::TRACE)
        .init();
    }
    trace!("Initializing framework...");

    let (state, canvas) = State::new();

    Self {
      canvas,
      state
    }
  }

  pub fn run(self) {
    self.canvas.run(self.state);
  }
}

impl Default for App {
  fn default() -> Self {
    Self::new(false)
  }
}