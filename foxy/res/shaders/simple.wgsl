struct FragmentInput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) position: vec4<f32>,
  @location(1) color: vec4<f32>,
};

@vertex
fn vertex_main(
  @builtin(vertex_index) in_vertex_index: u32,
) -> FragmentInput {
  var out: FragmentInput;
  let x = f32(1 - i32(in_vertex_index)) * 0.5;
  let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
  out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
  out.position      = vec4<f32>(x, y, 0.0, 1.0);
  out.color         = vec4<f32>(x + 0.5, y + 0.5, 0.0, 1.0);
  return out;
}

@fragment
fn fragment_main(
  in: FragmentInput
) -> @location(0) vec4<f32> {
  return in.color;
}
