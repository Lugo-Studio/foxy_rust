use std::sync::Arc;
use tracing::info;
use type_map::concurrent::TypeMap;
use winit::{
  event_loop::ControlFlow,
  window::Window
};
use foxy::renderer::Renderer;
use crate::app::AppTime;

pub struct AppState {
  renderer: Renderer,
  resources: TypeMap,
  should_exit: bool,
}

impl AppState {
  pub fn new(window: Arc<Window>) -> Self {
    let renderer = Renderer::from_window(window);
    let resources = TypeMap::new();

    Self {
      renderer,
      resources,
      should_exit: false,
    }
  }

  pub fn should_exit(&self) -> bool {
    self.should_exit
  }

  pub fn insert_resource<T: Send + Sync + 'static>(&mut self, resource: T) -> Option<T> {
    self.resources.insert(resource)
  }

  pub fn get_resource<T: Send + Sync + 'static>(&self) -> Option<&T> {
    self.resources.get::<T>()
  }

  pub fn get_mut_resource<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
    self.resources.get_mut::<T>()
  }

  pub fn on_start(&mut self, _time: &AppTime) {

  }

  pub fn on_tick(&mut self, _time: &AppTime) {
  }

  pub fn on_update(&mut self, _time: &AppTime) {
    info!("{:?}", 1. / _time.delta().as_secs_f64());
    self.renderer.draw();
  }

  pub fn on_stop(&mut self, _time: &AppTime) {

  }

  pub fn on_exit_request(&mut self, time: &AppTime, control_flow: &mut ControlFlow) {
    self.on_stop(time);

    *control_flow = ControlFlow::Exit;

    info!("Stopping framework...");
  }
}