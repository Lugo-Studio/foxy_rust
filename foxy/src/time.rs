use std::time::{Duration, Instant};

pub struct Time {
  tick_rate: f64,
  tick_time: Duration,
  lag_time: Duration,
  step_count: u32,
  bail_threshold: u32,
  previous_frame: Instant,
  current_frame: Instant,
  delta_time: Duration,
}

impl Time {
  pub fn new(tick_rate: f64, bail_threshold: u32) -> Self {
    Self {
      tick_rate,
      tick_time: Duration::from_secs_f64(1. / tick_rate),
      lag_time: Default::default(),
      step_count: 0,
      bail_threshold,
      previous_frame: Instant::now(),
      current_frame: Instant::now(),
      delta_time: Default::default(),
    }
  }

  pub fn tick_rate(&self) -> f64 {
    self.tick_rate
  }

  pub fn tick_time(&self) -> &Duration {
    &self.tick_time
  }

  pub fn delta(&self) -> &Duration {
    &self.delta_time
  }

  pub fn now(&self) -> Instant {
    Instant::now()
  }

  pub(crate) fn update(&mut self) {
    self.current_frame = Instant::now();
    self.delta_time = self.current_frame - self.previous_frame;
    self.previous_frame = self.current_frame;
    self.lag_time += self.delta_time;
    self.step_count = 0;
  }

  pub(crate) fn tick(&mut self) {
    self.lag_time -= self.tick_time;
    self.step_count += 1;
  }

  pub(crate) fn should_do_tick(&self) -> bool {
    if self.step_count >= self.bail_threshold {
      tracing::warn!("Struggling to catch up with tick rate.");
    }
    self.lag_time >= self.tick_time && self.step_count < self.bail_threshold
  }
}

impl Default for Time {
  fn default() -> Self {
    Self::new(128., 1024)
  }
}