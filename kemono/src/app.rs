use std::{
  sync::Arc,
  time::{Duration, Instant}
};
use tracing::{info, warn};
use tracing_unwrap::ResultExt;
use winit::{
  event::{Event, WindowEvent},
  event_loop::EventLoop,
  window::{Window, WindowBuilder}
};
use crate::app_state::AppState;

pub struct App {
  event_loop: EventLoop<()>,
  window: Arc<Window>,
  app_state: AppState,
  time: AppTime,
}

impl App {
  pub fn new() -> Self {
    tracing_subscriber::fmt::init();
    info!("Starting framework...");

    let event_loop = EventLoop::new();
    let window = Arc::new(
      WindowBuilder::new()
        .build(&event_loop)
        .expect_or_log("Failed to create new Window")
    );
    let app_state = AppState::new(window.clone());

    Self {
      event_loop,
      window,
      app_state,
      time: AppTime::default()
    }
  }

  pub fn insert_resource<T: Send + Sync + 'static>(&mut self, resource: T) -> Option<T> {
    self.app_state.insert_resource(resource)
  }

  pub fn get_resource<T: Send + Sync + 'static>(&self) -> Option<&T> {
    self.app_state.get_resource()
  }

  pub fn get_mut_resource<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
    self.app_state.get_mut_resource()
  }

  pub fn get_mut_state(&mut self) -> &mut AppState {
    &mut self.app_state
  }

  pub fn run(self) {
    Self::run_internal(self.event_loop, self.window, self.time, self.app_state);
  }

  fn run_internal(event_loop: EventLoop<()>, _window: Arc<Window>, mut time: AppTime, mut app_state: AppState) {
    app_state.on_start(&time);
    event_loop.run(move |event, _, control_flow| {
      if app_state.should_exit() {
        app_state.on_exit_request(&time, control_flow);
      }

      if let Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } = event {
        app_state.on_exit_request(&time, control_flow);
      } else {
        while time.should_do_tick() {
          app_state.on_tick(&time);
          time.tick();
        }
        app_state.on_update(&time);
        time.update();
      }
    });
  }
}

impl Default for App {
  fn default() -> Self {
    Self::new()
  }
}

pub struct AppTime {
  tick_rate: f64,
  fixed_time_step: Duration,
  bail_count: u32,
  bail_step_count: u32,
  time_last: Instant,
  time_current: Instant,
  delta: Duration,
  lag: Duration,
}

impl AppTime {
  pub fn new(tick_rate: f64, bail_count: u32) -> Self {
    Self {
      tick_rate,
      fixed_time_step: Duration::from_secs(1).div_f64(tick_rate),
      bail_count,
      bail_step_count: 0,
      time_last: Instant::now(),
      time_current: Instant::now(),
      delta: Duration::from_secs(0),
      lag: Duration::from_secs(0),
    }
  }

  #[allow(unused)]
  pub fn tick_rate(&self) -> f64 {
    self.tick_rate
  }

  #[allow(unused)]
  pub fn fixed_time_step(&self) -> Duration {
    self.fixed_time_step
  }

  #[allow(unused)]
  pub fn delta(&self) -> Duration {
    self.delta
  }

  #[allow(unused)]
  pub fn lag(&self) -> Duration {
    self.lag
  }

  fn should_do_tick(&self) -> bool {
    if self.bail_step_count >= self.bail_count {
      warn!("Struggling to keep up with tick rate!");
    }

    self.lag >= self.fixed_time_step && self.bail_step_count < self.bail_count
  }

  fn update(&mut self) {
    self.time_current = Instant::now();
    self.delta = self.time_current - self.time_last;
    self.time_last = self.time_current;
    self.lag += self.delta;
    self.bail_step_count = 0;
  }

  fn tick(&mut self) {
    self.lag -= self.fixed_time_step;
    self.bail_step_count += 1;
  }
}

impl Default for AppTime {
  fn default() -> Self {
    Self::new(128., 1024)
  }
}