use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct TimeUniform {
  pub time: f32,
  pub delta: f32,
}

impl TimeUniform {
  pub fn new() -> Self {
    Self {
      time: 0.0,
      delta: 0.0,
    }
  }

  pub fn update(&mut self, time: f32, delta: f32) {
    self.time = time;
    self.delta = delta;
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct CircleUniform {
  pub thickness: f32,
  pub fade:      f32,
}