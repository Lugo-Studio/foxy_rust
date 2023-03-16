pub mod event_dispatcher;

use std::sync::Arc;
use tracing_unwrap::ResultExt;
use winit::{
  event::Event,
  event_loop::EventLoop,
  window::{Window, WindowBuilder}
};
use winit::dpi::PhysicalSize;
use crate::canvas::event_dispatcher::EventDispatcher;

/// Visibility of window on startup. Defaults to VISIBLE.
#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub enum Visibility {
  /// Start with window visible.
  #[default]
  Visible,
  /// Start with window hidden.
  Hidden,
  /// Wait until renderer is initialized to avoid white flash on window startup.
  Wait,
}

#[derive(Clone)]
pub struct CanvasDescriptor {
  pub title: &'static str,
  pub size: PhysicalSize<u32>,
  pub visibility: Visibility,
}

impl Default for CanvasDescriptor {
  fn default() -> Self {
    Self {
      title: "Foxy Window",
      size: PhysicalSize::new(800, 500),
      visibility: Default::default(),
    }
  }
}

pub struct Canvas {
  descriptor: CanvasDescriptor,
  event_loop: EventLoop<()>,
  window: Arc<Window>,
  dispatcher: Box<dyn EventDispatcher>,
}

impl Canvas {
  pub fn new<D: EventDispatcher + 'static>(descriptor: CanvasDescriptor) -> Self {
    let event_loop = EventLoop::new();

    let window = Arc::new(WindowBuilder::new()
      .with_title(descriptor.title)
      .with_inner_size(descriptor.size)
      .with_visible(match descriptor.visibility {
        Visibility::Visible => true,
        Visibility::Hidden => false,
        Visibility::Wait => false
      })
      .build(&event_loop)
      .expect_or_log("Failed to create new Window"));

    let dispatcher = Box::new(D::new(window.clone(), &descriptor));

    Self {
      descriptor,
      event_loop,
      window,
      dispatcher,
    }
  }

  pub fn descriptor(&self) -> &CanvasDescriptor {
    &self.descriptor
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

  pub fn run(mut self) {
    self.dispatcher.on_start();
    self.event_loop.run(move |event, _, control_flow| {
      if let Event::WindowEvent { event, window_id } = event {
        if self.window.id() == window_id {
          self.dispatcher.window_event_dispatch(event, self.window.clone(), window_id, control_flow);
        }
      } else {
        self.dispatcher.app_event_dispatch(event, self.window.clone(), control_flow);
      }
    });
  }
}