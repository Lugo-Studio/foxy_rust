mod state;

use tracing::{Level, trace};
use foxy::canvas::{Canvas, CanvasDescriptor, Visibility};
use foxy::winit::dpi::PhysicalSize;
use crate::app::state::State;

pub struct App {
  canvas: Canvas,
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

    let canvas = Canvas::new::<State>(CanvasDescriptor {
      title: "Kemono App",
      size: PhysicalSize::new(800, 500),
      visibility: Visibility::Wait,
    });

    Self {
      canvas
    }
  }

  pub fn run(self) {
    self.canvas.run();
  }
}

impl Default for App {
  fn default() -> Self {
    Self::new(false)
  }
}