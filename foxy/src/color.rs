use rgb::{RGBA8};

pub type Color = RGBA8;

pub trait FromHex {
  fn hex(value: &'static str) -> Self;
}

impl FromHex for RGBA8 {
  fn hex(value: &'static str) -> Self {
    assert_eq!(value.len(), 8, "Invalid hex code. Requires 8 digits.");

    match hex::decode(value).map(|b| RGBA8::new(b[0], b[1], b[2], b[3])) {
      Ok(result) => result,
      Err(err) => {
        tracing::error!("{err}");
        RGBA8::default()
      }
    }
  }
}

pub trait ToWGPU {
  fn as_wgpu(&self) -> wgpu::Color;
}

impl ToWGPU for RGBA8 {
  fn as_wgpu(&self) -> wgpu::Color {
    wgpu::Color {
      r: self.r as f64 / 255.0,
      g: self.g as f64 / 255.0,
      b: self.b as f64 / 255.0,
      a: self.a as f64 / 255.0,
    }
  }
}

pub trait FromF32 {
  fn from_f32(r: f32, g: f32, b: f32, a: f32) -> Self;
}

impl FromF32 for RGBA8 {
  fn from_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
    Self::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, (a * 255.0) as u8)
  }
}

pub trait ToF32 {
  fn as_f32(&self) -> [f32; 4];
}

impl ToF32 for RGBA8 {
  fn as_f32(&self) -> [f32; 4] {
    [
      self.r as f32 / 255.0,
      self.g as f32 / 255.0,
      self.b as f32 / 255.0,
      self.a as f32 / 255.0,
    ]
  }
}