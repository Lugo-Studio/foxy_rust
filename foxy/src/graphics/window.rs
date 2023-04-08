use tracing_unwrap::ResultExt;
use winit::{event_loop::EventLoop, window::WindowBuilder, dpi::{LogicalSize, PhysicalSize}};
use winit::dpi::PhysicalPosition;

pub struct Window {
  window: winit::window::Window,
}

impl Window {
  pub fn new(title: &'static str, width: u32, height: u32) -> (Self, EventLoop<()>) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
      .with_title(title)
      .with_inner_size(LogicalSize::new(width, height))
      .with_visible(false)
      .build(&event_loop)
      .unwrap_or_log();
    
    let win = Self {
      window,
    };

    (win, event_loop)
  }

  pub fn winit(&self) -> &winit::window::Window {
    &self.window
  }

  pub fn title(&self) -> String {
    self.window.title()
  }

  pub fn size(&self) -> PhysicalSize<u32> {
    self.window.inner_size()
  }

  pub fn center_on_monitor(&mut self) {
    let monitor = self.window.current_monitor().unwrap();
    let monitor_center = PhysicalPosition::new(
      monitor.position().x + (monitor.size().width as f32 * 0.5).floor() as i32,
      monitor.position().y + (monitor.size().height as f32 * 0.5).floor() as i32
    );
    let window_offset = PhysicalPosition::new(
      monitor_center.x - (self.window.outer_size().width as f32 * 0.5).floor() as i32,
      monitor_center.y - (self.window.outer_size().height as f32 * 0.5).floor() as i32
    );
    self.window.set_outer_position(window_offset);
  }

  pub fn size_tuple(&self) -> (u32, u32) {
    let x = self.window.inner_size();
    (x.width, x.height)
  }

  pub fn request_redraw(&self) {
    self.window.request_redraw();
  }

  pub fn set_visible(&mut self, visible: bool) {
    self.window.set_visible(visible);
  }
}