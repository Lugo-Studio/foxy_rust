use std::fmt::{Display, Formatter};
use std::time::{Duration, Instant};

pub struct Time {
  tick_rate: f64,
  tick_time: Duration,
  lag_time: Duration,
  step_count: u32,
  bail_threshold: u32,
  start_time: Instant,
  previous_frame: Instant,
  current_frame: Instant,
  delta_time: Duration,
  tick_previous_frame: Instant,
  tick_current_frame: Instant,
  tick_delta_time: Duration,
}

impl Time {
  pub fn new(tick_rate: f64, bail_threshold: u32) -> Self {
    Self {
      tick_rate,
      tick_time: Duration::from_secs_f64(1. / tick_rate),
      lag_time: Default::default(),
      step_count: 0,
      bail_threshold,
      start_time: Instant::now(),
      previous_frame: Instant::now(),
      current_frame: Instant::now(),
      delta_time: Default::default(),
      tick_previous_frame: Instant::now(),
      tick_current_frame: Instant::now(),
      tick_delta_time: Default::default(),
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

  pub fn delta_secs(&self) -> f64 {
    self.delta_time.as_secs_f64()
  }

  pub fn delta_tick(&self) -> &Duration {
    &self.tick_delta_time
  }

  pub fn delta_tick_secs(&self) -> f64 {
    self.tick_delta_time.as_secs_f64()
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
    self.tick_current_frame = Instant::now();
    self.tick_delta_time = self.tick_current_frame - self.tick_previous_frame;
    self.tick_previous_frame = self.tick_current_frame;
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

impl Display for Time {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "[tick_rate: {:?}, delta_tick: {:?}, delta: {:?}, time_since_start: {:?}, now: {:?}]",
      self.tick_rate,
      self.tick_delta_time,
      self.delta_time,
      self.now().duration_since(self.start_time),
      self.now(),
    )
  }
}