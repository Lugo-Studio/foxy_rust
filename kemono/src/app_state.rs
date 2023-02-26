use std::sync::Arc;
use legion::{component, IntoQuery, Resources, World};
use tracing::info;
use winit::{
  event_loop::ControlFlow,
  window::Window
};
use foxy::components::{Hidden, Material, Mesh};
use foxy::renderer::Renderer;
use kemono_transform::transform::Transform;
use crate::app_loop::Time;
use crate::lifecycle::{Event, Lifecycle};

pub struct AppState {
  should_exit: bool,

  renderer: Renderer,
  resources: Resources,
  lifecycle: Lifecycle,

  world: World,
}

impl AppState {
  pub fn new(window: Arc<Window>, mut resources: Resources, time: Time, lifecycle: Lifecycle) -> Self {
    let renderer = Renderer::from_window(window);
    let world = World::default();

    resources.insert(time);

    Self {
      should_exit: false,
      renderer,
      resources,
      lifecycle,
      world,
    }
  }

  pub fn should_exit(&self) -> bool {
    self.should_exit
  }

  pub fn get_resources(&self) -> &Resources {
    &self.resources
  }

  pub fn on_start(&mut self) {
    self.lifecycle.run_systems(Event::PreStart, &mut self.world, &mut self.resources);
    self.lifecycle.run_systems(Event::Start, &mut self.world, &mut self.resources);
  }

  pub fn on_tick(&mut self) {
    self.lifecycle.run_systems(Event::Tick, &mut self.world, &mut self.resources);
  }

  pub fn on_update(&mut self) {
    self.lifecycle.run_systems(Event::Update, &mut self.world, &mut self.resources);
    self.lifecycle.run_systems(Event::PostUpdate, &mut self.world, &mut self.resources);

    self.on_render();
  }

  fn on_render(&mut self) {
    let mut query = <(&Mesh, &Material, &Transform)>::query()
      .filter(!component::<Hidden>());
    query.par_for_each_mut(&mut self.world, |(mesh, material, transform)| {
      self.renderer.draw(mesh, material, transform);
    });
  }

  pub fn on_stop(&mut self) {
    self.lifecycle.run_systems(Event::Stop, &mut self.world, &mut self.resources);
  }

  pub fn on_exit_request(&mut self, control_flow: &mut ControlFlow) {
    self.on_stop();

    *control_flow = ControlFlow::Exit;

    info!("Stopping framework...");
  }
}