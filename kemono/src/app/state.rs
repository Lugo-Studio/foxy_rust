use std::sync::Arc;
use tracing::{info};
use foxy::{
  winit::{
    dpi::{PhysicalSize, PhysicalPosition},
    event::{ElementState, ModifiersState, MouseButton, MouseScrollDelta, VirtualKeyCode, ScanCode},
    window::Window
  },
  renderer::Renderer,
  canvas::{
    event_dispatcher::EventDispatcher,
    CanvasDescriptor
  },
};

pub struct State {
  renderer: Renderer,
}

impl EventDispatcher for State {
  fn new(window: Arc<Window>, descriptor: &CanvasDescriptor) -> Self {
    let renderer = Renderer::from_canvas(window, descriptor);

    Self {
      renderer,
    }
  }

  fn on_start(&mut self) {
    info!("Starting framework...");
  }

  fn on_update(&mut self) {

  }

  fn on_render(&mut self) {
    self.renderer.render();
  }

  fn on_stop(&mut self) {
    info!("Stopping framework...");
  }

  fn on_resize(&mut self, _new_inner_size: PhysicalSize<u32>) {
    self.renderer.reconfigure();
  }

  fn on_rescale(&mut self, _scale_factor: f64, _new_inner_size: &mut PhysicalSize<u32>) {
    self.renderer.reconfigure();
  }

  fn on_keyboard_input(&mut self, state: ElementState, keycode: VirtualKeyCode, scancode: ScanCode) {
    info!("Keyboard {{ state: {state:?}, scancode: {scancode:?} , keycode: {keycode:?} }}");
  }

  fn on_modifiers_changed(&mut self, mods: ModifiersState) {
    info!("Modifiers {{ {mods:?} }}");
  }

  fn on_mouse_button_input(&mut self, state: ElementState, button: MouseButton) {
    info!("Mouse {{ state: {state:?}, button: {button:?} }}");
  }

  fn on_mouse_scroll_input(&mut self, delta: MouseScrollDelta) {
    info!("{delta:?}");
  }

  fn on_mouse_cursor_input(&mut self, _position: PhysicalPosition<f64>) {

  }
}