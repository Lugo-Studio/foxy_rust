pub mod event_dispatcher;

use std::sync::Arc;
use tracing_unwrap::ResultExt;
use winit::{
  event::Event,
  event_loop::{EventLoop},
  window::{Window, WindowBuilder}
};
use winit::dpi::PhysicalSize;
use crate::canvas::event_dispatcher::EventDispatcher;

pub struct Canvas {
  event_loop: EventLoop<()>,
  window: Arc<Window>,
}

impl Canvas {
  pub fn new(visible: bool) -> Self {
    let event_loop = EventLoop::new();
    let window = Arc::new(WindowBuilder::new()
      .with_title("Kemono App")
      .with_inner_size(PhysicalSize::new(800, 500))
      .with_visible(visible) // spawn invisible until renderer is ready to avoid white flash
      .build(&event_loop)
      .expect_or_log("Failed to create new Window"));

    Self {
      event_loop,
      window
    }
  }

  pub fn window(&self) -> Arc<Window> {
    self.window.clone()
  }

  pub fn event_loop(&self) -> &EventLoop<()> {
    &self.event_loop
  }

  pub fn set_visible(&self, visible: bool) {
    self.window.set_visible(visible);
  }

  pub fn run(self, mut event_dispatcher: impl EventDispatcher + 'static) {
    event_dispatcher.on_start();
    self.event_loop.run(move |event, _, control_flow| {
      if let Event::WindowEvent { event, window_id } = event {
        event_dispatcher.window_event_dispatch(event, window_id, control_flow);
      } else {
        event_dispatcher.app_event_dispatch(event, control_flow);
      }
    });
  }
}