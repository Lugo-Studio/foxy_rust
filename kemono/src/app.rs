use std::sync::Arc;
use legion::Resources;
use legion::systems::ParallelRunnable;
use tracing::info;
use tracing_unwrap::ResultExt;

use winit::{
  event_loop::EventLoop,
  window::WindowBuilder
};
use winit::dpi::LogicalSize;
use crate::app_loop::{AppLoop, Time};
use crate::app_state::AppState;
use crate::lifecycle::{Event, Lifecycle, LifecycleBuilder};

pub struct App {
  app_state: AppState,
  app_loop: AppLoop,
}

impl App {
  pub fn new(
    title: String,
    size: LogicalSize<i32>,
    resources: Resources,
    time: Time,
    lifecycle: Lifecycle,
  ) -> Self {
    tracing_subscriber::fmt::init();
    info!("Starting framework...");

    let event_loop = EventLoop::new();
    let window = Arc::new(
      WindowBuilder::new()
        .with_title(title)
        .with_inner_size(size)
        .build(&event_loop)
        .expect_or_log("Failed to create new Window")
    );

    Self {
      app_state: AppState::new(window, resources, time, lifecycle),
      app_loop: AppLoop::new(event_loop),
    }
  }

  pub fn run(self) {
    self.app_loop.run(self.app_state);
  }
}

pub struct AppBuilder {
  title: String,
  tick_rate: f64,
  size: (i32, i32),
  resources: Resources,
  lifecycle: LifecycleBuilder,
}

impl AppBuilder {
  pub fn new() -> Self {
    Self {
      title: "Kemono App".into(),
      tick_rate: 128.,
      size: (800, 500),
      resources: Resources::default(),
      lifecycle: LifecycleBuilder::new(),
    }
  }

  pub fn title(mut self, title: &str) -> Self {
    self.title = title.into();
    self
  }

  pub fn tick_rate(mut self, tick_rate: f64) -> Self {
    self.tick_rate = tick_rate;
    self
  }

  pub fn size(mut self, width: i32, height: i32) -> Self {
    self.size = (width, height);
    self
  }

  pub fn insert_resource<T: Send + Sync + Sized + 'static>(mut self, resource: T) -> Self {
    self.resources.insert(resource);
    self
  }

  pub fn insert_system(mut self, event: Event, system: impl ParallelRunnable + 'static) -> Self {
    self.lifecycle.builders.get_mut(&event).unwrap().add_system(system);
    self
  }

  pub fn build(self) -> App {
    App::new(
      self.title,
      LogicalSize::new(self.size.0, self.size.1),
      self.resources,
      Time::new(self.tick_rate, 1024),
      self.lifecycle.build()
    )
  }

  pub fn build_and_run(self) {
    App::new(
      self.title,
      LogicalSize::new(self.size.0, self.size.1),
      self.resources,
      Time::new(self.tick_rate, 1024),
      self.lifecycle.build()
    ).run()
  }
}

impl Default for AppBuilder {
  fn default() -> Self {
    Self::new()
  }
}