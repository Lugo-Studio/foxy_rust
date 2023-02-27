use tracing::{debug, info};
use winit::{
  event::{
    Event,
    WindowEvent,
    ElementState,
    KeyboardInput,
    MouseButton,
    MouseScrollDelta
  },
  event_loop::ControlFlow,
  window::WindowId,
  dpi::PhysicalPosition,
};
use winit::event::VirtualKeyCode;
use foxy::renderer::{Renderer, VsyncMode};

pub struct AppState {
  pub renderer: Renderer,
}

impl AppState {
  pub fn window_event_dispatch(
    &mut self,
    event: WindowEvent,
    _window_id: WindowId,
    control_flow: &mut ControlFlow,
  ) {
    match event {
      WindowEvent::Resized(_) => {
        self.renderer.recreate_swapchain();
      }
      WindowEvent::CloseRequested => {
        *control_flow = ControlFlow::Exit;
      }
      WindowEvent::KeyboardInput {
        input,
        ..
      } => {
        self.on_keyboard_input(input);
      }
      WindowEvent::MouseInput {
        state, button, ..
      } => {
        self.on_mouse_button_input(state, button);
      },
      WindowEvent::MouseWheel {
        delta, ..
      } => {
        self.on_mouse_scroll_input(delta);
      },
      WindowEvent::CursorMoved {
        position,
        ..
      } => {
        self.on_mouse_cursor_input(position);
      }
      _ => {},
    }
  }

  // Window events fall under this match, but do NOT get processed here.
  // They are caught before this runs by the on_window_event fn.
  pub fn app_event_dispatch<T: 'static>(
    &mut self,
    event: Event<T>,
    _control_flow: &mut ControlFlow,
  ) {
    match event {
      Event::MainEventsCleared => {
        self.on_update();
      }
      Event::LoopDestroyed => {
        self.on_stop();
      }
      _ => {}
    }
  }

  pub fn on_start(&mut self) {
    info!("Starting framework...");
  }

  fn on_update(&mut self) {
    self.renderer.render();
  }

  fn on_stop(&mut self) {
    info!("Stopping framework...");
  }

  fn on_keyboard_input(&mut self, input: KeyboardInput) {
    debug!("{input:?}");

    if let KeyboardInput {
      virtual_keycode: Some(VirtualKeyCode::V),
      state: ElementState::Pressed, ..
    } = input {
      match self.renderer.get_vsync_mode() {
        VsyncMode::Enabled => self.renderer.set_vsync_mode(VsyncMode::Disabled),
        VsyncMode::Disabled => self.renderer.set_vsync_mode(VsyncMode::Hybrid),
        VsyncMode::Hybrid => self.renderer.set_vsync_mode(VsyncMode::Enabled)
      };
    }
  }

  fn on_mouse_button_input(&mut self, state: ElementState, button: MouseButton) {
    debug!("MouseInput {{ state: {state:?}, button: {button:?} }}");
  }

  fn on_mouse_scroll_input(&mut self, delta: MouseScrollDelta) {
    debug!("{delta:?}");
  }

  fn on_mouse_cursor_input(&mut self, _position: PhysicalPosition<f64>) {
    // debug!("{position:?}");
  }
}