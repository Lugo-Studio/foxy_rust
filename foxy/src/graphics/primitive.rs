use std::mem::size_of;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
  pub position: [f32; 4],
  pub normal:   [f32; 3],
  pub uv:       [f32; 2],
  pub color:    [f32; 4],
}

impl Vertex {
  const ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
    0 => Float32x4,
    1 => Float32x3,
    2 => Float32x2,
    3 => Float32x4,
  ];

  pub fn new(x: f32, y: f32, z: f32) -> Self {
    Vertex {
      position: [x, y, z, 1.],
      normal:   [0., 0., 0.],
      uv:       [0., 0.],
      color:    [1., 1., 1., 1.],
    }
  }

  pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
    self.color = [r, g, b, a];
    self
  }

  pub fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &Self::ATTRIBUTES,
    }
  }
}

#[derive(Debug, Clone)]
pub enum Mesh {
  Triangle(Vertex, Vertex, Vertex, Option<[f32; 4]>),
  Square(Vertex, Vertex, Option<[f32; 4]>), // https://math.stackexchange.com/a/506853/1167638
  Rectangle(Vertex, Vertex, Vertex, Vertex, Option<[f32; 4]>),
  Circle(Vertex, u32, Option<[f32; 4]>),
  Custom(Vec<Vertex>)
}