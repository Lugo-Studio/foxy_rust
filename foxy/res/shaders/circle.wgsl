struct VertexInput {
  @location(0)   pos: vec4<f32>,
  @location(1)  norm: vec3<f32>,
  @location(2)    uv: vec2<f32>,
  @location(3) color: vec4<f32>,
};

struct FragmentInput {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0)       vert_pos: vec4<f32>,
  @location(1)           norm: vec3<f32>,
  @location(2)             uv: vec2<f32>,
  @location(3)          color: vec4<f32>,
};

@vertex
fn vertex_main(
  in: VertexInput
) -> FragmentInput {
  var out: FragmentInput;
  out.clip_pos = in.pos;
  out.vert_pos = in.pos;
  out.norm     = in.norm;
  out.uv       = in.uv;
  out.color    = in.color;
  return out;
}

@fragment
fn fragment_main(
  in: FragmentInput
) -> @location(0) vec4<f32> {
  var thicc = 0.25;
  var fade = 0.05;

  var dist = 1.0 - length(in.uv * 2.0 - 1.0);
  var mask = vec4<f32>(smoothstep(0.0, fade, dist));
  mask *= vec4<f32>(smoothstep(thicc + fade, thicc, dist));

  return mask * in.color;
}
