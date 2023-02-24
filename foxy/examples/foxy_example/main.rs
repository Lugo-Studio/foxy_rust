use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;
use foxy::prelude::*;

#[allow(unused)]
fn main() {
  let (renderer, event_loop) = Renderer::new();
  event_loop.run(move |event, _, control_flow| {
    if let Event::WindowEvent {
      event: WindowEvent::CloseRequested,
      ..
    } = event {
      *control_flow = ControlFlow::Exit;
    }
  })
}