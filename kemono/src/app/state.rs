use tracing::{debug, info};
use foxy::{
  renderer::{
    Renderer,
    vsync::VsyncMode
  },
  canvas::{Canvas},
  canvas::event_dispatcher::EventDispatcher,
  winit::{
    dpi::PhysicalPosition,
    event::{
      ElementState,
      ModifiersState,
      MouseButton,
      MouseScrollDelta,
      VirtualKeyCode,
      ScanCode
    }
  },
};
use foxy::winit::dpi::PhysicalSize;
use foxy::winit::event::WindowEvent;

pub struct State {
  //window: Arc<Window>,
  renderer: Renderer,
}

impl State {
  pub fn new() -> (Self, Canvas) {
    let canvas = Canvas::new(false);
    let renderer = Renderer::from_canvas(&canvas, VsyncMode::Hybrid);
    canvas.set_visible(true);
    let state = Self {
      //window: canvas.window(),
      renderer,
    };

    (state, canvas)
  }
}

impl EventDispatcher for State {
  fn egui_event_dispatch(&mut self, event: WindowEvent) {
    self.renderer.update_gui(event);
  }

  fn on_start(&mut self) {
    info!("Starting framework...");
  }

  fn on_update(&mut self) {
    self.renderer.reset();

    self.renderer.render();
  }

  fn on_stop(&mut self) {
    info!("Stopping framework...");
  }

  fn on_resize(&mut self, _new_inner_size: PhysicalSize<u32>) {
    self.renderer.recreate_swapchain();
    // info!("{_new_inner_size:?}");
  }

  fn on_rescale(&mut self, _scale_factor: f64, _new_inner_size: &mut PhysicalSize<u32>) {
    self.renderer.recreate_swapchain();
    // info!("{_scale_factor}, {_new_inner_size:?}");
  }

  fn on_keyboard_input(&mut self, state: ElementState, keycode: VirtualKeyCode, scancode: ScanCode) {
    debug!("Keyboard {{ state: {state:?}, scancode: {scancode:?} , keycode: {keycode:?} }}");
    // if state == ElementState::Pressed && keycode == VirtualKeyCode::V {
    //   match self.renderer.vsync_mode() {
    //     VsyncMode::Enabled => self.renderer.set_vsync_mode(VsyncMode::Disabled),
    //     VsyncMode::Disabled => self.renderer.set_vsync_mode(VsyncMode::Hybrid),
    //     VsyncMode::Hybrid => self.renderer.set_vsync_mode(VsyncMode::Enabled)
    //   };
    // }
  }

  fn on_modifiers_changed(&mut self, mods: ModifiersState) {
    debug!("Modifiers {{ {mods:?} }}");
  }

  fn on_mouse_button_input(&mut self, state: ElementState, button: MouseButton) {
    debug!("Mouse {{ state: {state:?}, button: {button:?} }}");
  }

  fn on_mouse_scroll_input(&mut self, delta: MouseScrollDelta) {
    debug!("{delta:?}");
  }

  fn on_mouse_cursor_input(&mut self, _position: PhysicalPosition<f64>) {

  }
}