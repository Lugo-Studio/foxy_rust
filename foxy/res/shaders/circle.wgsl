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
  var thicc = 1.0;
  var fade = 0.005;
  var color = vec4<f32>(0.8, 0.1, 0.2, 1.0);

  var uv = in.uv * 2.0 - 1.0; 
  var dist = 1.0 - length(uv);
  var mask = vec4<f32>(smoothstep(0.0, fade, dist));
  mask *= vec4<f32>(smoothstep(thicc + fade, thicc, dist));

  return mask * color;
}
