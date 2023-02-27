use std::sync::Arc;
use tracing::{Level, trace};
use tracing_unwrap::ResultExt;
use winit::{
  event_loop::EventLoop,
  window::WindowBuilder,
  dpi::LogicalSize,
  event::Event
};
use foxy::renderer::{Renderer, VsyncMode};
use crate::app_state::AppState;

pub struct App {
  event_loop: EventLoop<()>,
  app_state: AppState,
}

impl App {
  pub fn new() -> Self {
    tracing_subscriber::fmt()
      .with_thread_names(true)
      .with_max_level(Level::TRACE)
      .init();
    trace!("Initializing framework...");

    let event_loop = EventLoop::new();
    let window = Arc::new(
      WindowBuilder::new()
        .with_title("Kemono App")
        .with_inner_size(LogicalSize::new(800, 500))
        .with_visible(false) // spawn invisible until renderer is ready to avoid white flash
        .build(&event_loop)
        .expect_or_log("Failed to create new Window")
    );

    let app_state = AppState {
      renderer: Renderer::from_window(window.clone(), VsyncMode::Hybrid),
    };

    window.set_visible(true);

    Self {
      event_loop,
      app_state,
    }
  }

  pub fn run(self) {
    // this isn't strictly necessary to keep as a non-member fn, but enforces that
    // the event_loop cannot be owned by AppState or else it'll move the state alongside it,
    // preventing mutability.
    Self::run_internal(self.event_loop, self.app_state);
  }

  fn run_internal(event_loop: EventLoop<()>, mut app_state: AppState) {
    app_state.on_start();
    event_loop.run(move |event, _, control_flow| {
      app_state.renderer.end_previous_frame();

      if let Event::WindowEvent { event, window_id } = event {
        app_state.window_event_dispatch(event, window_id, control_flow);
      } else {
        app_state.app_event_dispatch(event, control_flow);
      }
    });
  }
}

impl Default for App {
  fn default() -> Self {
    Self::new()
  }
}