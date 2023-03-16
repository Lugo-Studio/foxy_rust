pub use rgb::RGBA8;

pub trait FromHex {
  fn hex(value: &'static str) -> Self;
}

impl FromHex for RGBA8 {
  fn hex(value: &'static str) -> Self {
    assert_eq!(value.len(), 8);

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