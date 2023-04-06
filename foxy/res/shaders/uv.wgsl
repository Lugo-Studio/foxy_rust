struct VertexInput {
  @location(0) pos: vec4<f32>,
  @location(1)  uv: vec2<f32>,
};

struct FragmentInput {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0)       vert_pos: vec4<f32>,
  @location(1)             uv: vec2<f32>,
};

@vertex
fn vertex_main(
  in: VertexInput
) -> FragmentInput {
  var out: FragmentInput;
  out.clip_pos = in.pos;
  out.vert_pos = in.pos;
  out.uv       = in.uv;
  return out;
}

@fragment
fn fragment_main(
  in: FragmentInput
) -> @location(0) vec4<f32> {
  return vec4<f32>(in.uv, 0.0, 1.0);
}
