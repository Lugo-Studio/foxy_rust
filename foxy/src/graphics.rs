pub mod window;
pub mod renderer;
pub mod shader;
pub mod primitive;
pub mod color;

use winit::event_loop::EventLoop;

use self::{window::Window, renderer::Renderer, primitive::Mesh};

pub struct Graphics {
  window: Window,
  renderer: Renderer,
}

impl Graphics {
  pub fn new(title: &'static str, width: u32, height: u32, centered_window: bool) -> (Self, EventLoop<()>) {
    tracing::trace!("Initializing Graphics...");

    let (mut window, event_loop) = Window::new(title, width, height);
    if centered_window {
      window.center_on_monitor();
    }
    let renderer = Renderer::new(&window);
    window.set_visible(true);

    tracing::trace!("Graphics ready!");

    let gfx = Self { 
      window,
      renderer,
    };

    (gfx, event_loop)
  }

  pub fn window(&self) -> &Window {
    &self.window
  }

  pub fn submit(&mut self, _primitive: Mesh) {

  }

  pub(crate) fn render(&mut self) {
    self.renderer.render_frame(&self.window);
  }

  pub(crate) fn post_render(&mut self) {
    self.renderer.end_frame();
  }
}