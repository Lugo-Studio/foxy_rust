use foxy::prelude::*;

#[derive(Default)]
struct App {}

impl Runnable for App {
  fn start(&mut self, _: &mut Graphics) {
    tracing::info!("hi, friends!");
  }

  fn update(&mut self, _: &mut Graphics, _: &Time) {
    // tracing::info!("update {}", t);
  }

  fn tick(&mut self, _: &mut Graphics, _: &Time) {
    // tracing::debug!("tick {}", t);
  }

  fn stop(&mut self, _: &mut Graphics, _: &Time) {
    tracing::info!("otsu kon deshita!");
  }

  fn input(&mut self, _: &mut Graphics, event: InputEvent, _: &Time) {
    match event {
      InputEvent::Cursor => {}
      InputEvent::Scroll => {}
      _ => {
        tracing::info!("input")
      }
    }
  }
}

fn main() {
  Foxy::builder()
    .with_logging(Some(Level::Trace))
    .with_wgpu_logging(Some(Level::Error))
    .with_title("Foxy App")
    .with_centered(true)
    .with_tick_rate(128.)
    .build_and_run::<App>();
}