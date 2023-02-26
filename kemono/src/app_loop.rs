use std::time::{Duration, Instant};
use tracing::warn;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use crate::app_state::AppState;

pub struct AppLoop {
  event_loop: EventLoop<()>,
}

impl AppLoop {
  pub fn new(event_loop: EventLoop<()>) -> Self {
    Self {
      event_loop,
    }
  }

  pub fn run(self, mut app_state: AppState) {
    app_state.on_start();
    self.event_loop.run(move |event, _, control_flow| {
      if app_state.should_exit() {
        app_state.on_exit_request(control_flow);
      }

      match event {
        Event::WindowEvent {
          event: WindowEvent::CloseRequested,
          ..
        } => {
          app_state.on_exit_request(control_flow);
        }
        Event::RedrawEventsCleared => {
          app_state.get_resources().get_mut::<Time>().unwrap().update();
          while app_state.get_resources().get::<Time>().unwrap().should_do_tick() {
            app_state.get_resources().get_mut::<Time>().unwrap().tick();
            app_state.on_tick();
          }
          app_state.on_update();
        }
        _ => {}
      }
    });
  }
}

pub struct Time {
  tick_rate: f64,
  fixed_time_step: Duration,
  bail_count: u32,
  bail_step_count: u32,
  time_last: Instant,
  time_current: Instant,
  delta: Duration,
  lag: Duration,
}

impl Time {
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

impl Default for Time {
  fn default() -> Self {
    Self::new(128., 1024)
  }
}