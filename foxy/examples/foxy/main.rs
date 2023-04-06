use foxy::prelude::*;

#[derive(Default)]
struct App {}

impl Runnable for App {
  fn start(&mut self, _: &mut Graphics) {
    tracing::info!("hi, friends!");
  }

  fn update(&mut self, gfx: &mut Graphics, _: &Time) {
    gfx.submit(Mesh::Triangle(
      Vertex::new(0., 0., 0.),
      Vertex::new(0., 0., 0.),
      Vertex::new(0., 0., 0.),
      None
    ));
  }

  fn tick(&mut self, _: &mut Graphics, _: &Time) {
    tracing::debug!("tick");
  }

  fn stop(&mut self, _: &mut Graphics, _: &Time) {
    tracing::info!("otsu kon deshita!");
  }
}

fn main() {
  Foxy::builder()
    .with_logging(Level::Trace)
    .with_wgpu_logging(Level::Error)
    .with_title("Foxy App")
    .with_tick_rate(1.)
    .build_and_run::<App>();
}