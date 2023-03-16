use std::sync::Arc;
use winit::{
  dpi::PhysicalPosition,
  event::{ElementState, Event, ModifiersState, MouseButton, MouseScrollDelta, WindowEvent},
  event_loop::ControlFlow,
  window::WindowId
};
use winit::dpi::PhysicalSize;
use winit::event::{ScanCode, VirtualKeyCode};
use winit::window::Window;
use crate::canvas::{CanvasDescriptor};

pub trait EventDispatcher {
  fn new(window: Arc<Window>, descriptor: &CanvasDescriptor) -> Self where Self: Sized;

  #[allow(unused)]
  fn window_event_dispatch(
    &mut self,
    event: WindowEvent,
    window: Arc<Window>,
    window_id: WindowId,
    control_flow: &mut ControlFlow,
  ) {
    match event {
      WindowEvent::Resized(new_inner_size) => {
        self.on_resize(new_inner_size);
      }
      WindowEvent::ScaleFactorChanged {
        scale_factor,
        new_inner_size,
      } => {
        self.on_rescale(scale_factor, new_inner_size);
      },
      WindowEvent::CloseRequested => {
        *control_flow = ControlFlow::Exit;
      }
      WindowEvent::KeyboardInput {
        input,
        ..
      }
      if input.virtual_keycode.is_some() => {
        self.on_keyboard_input(input.state, input.virtual_keycode.unwrap(), input.scancode);
      }
      WindowEvent::ModifiersChanged(state) => {
        self.on_modifiers_changed(state);
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
  #[allow(unused)]
  fn app_event_dispatch(
    &mut self,
    event: Event<()>,
    window: Arc<Window>,
    control_flow: &mut ControlFlow,
  ) {
    match event {
      Event::MainEventsCleared => {
        self.on_update();
        window.request_redraw();
      }
      Event::RedrawRequested(_) => {
        self.on_render();
      }
      Event::LoopDestroyed => {
        self.on_stop();
      }
      _ => {}
    }
  }

  #[allow(unused)]
  fn egui_event_dispatch(
    &mut self,
    event: WindowEvent
  ) {}

  #[allow(unused)]
  fn on_start(&mut self) {}
  #[allow(unused)]
  fn on_update(&mut self) {}
  #[allow(unused)]
  fn on_render(&mut self);
  #[allow(unused)]
  fn on_stop(&mut self) {}

  #[allow(unused)]
  fn on_resize(&mut self, new_inner_size: PhysicalSize<u32>) {}
  #[allow(unused)]
  fn on_rescale(&mut self, scale_factor: f64, new_inner_size: &mut PhysicalSize<u32>) {}
  #[allow(unused)]
  fn on_keyboard_input(&mut self, state: ElementState, keycode: VirtualKeyCode, scancode: ScanCode) {}
  #[allow(unused)]
  fn on_modifiers_changed(&mut self, mods: ModifiersState) {}
  #[allow(unused)]
  fn on_mouse_button_input(&mut self, state: ElementState, button: MouseButton) {}
  #[allow(unused)]
  fn on_mouse_scroll_input(&mut self, delta: MouseScrollDelta) {}
  #[allow(unused)]
  fn on_mouse_cursor_input(&mut self, position: PhysicalPosition<f64>) {}
}