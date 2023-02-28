struct FragInput
{
  float4 position : SV_POSITION;
  [[vk::location(0)]] float4 color : COLOR0;
};

FragInput vertex_main(uint vertex_index : SV_VertexID)
{
  FragInput output;

  // Vulkan is (-,-) top-left and (+,+) bottom-right
  float4 positions[3] = {
    float4(-0.5,  0.5, 0.0, 1.0),
    float4( 0.5,  0.5, 0.0, 1.0),
    float4( 0.0, -0.5, 0.0, 1.0),
  };

  float4 colors[3] = {
    float4(1.00, 0.25, 0.25, 1.00),
    float4(0.25, 1.00, 0.25, 1.00),
    float4(0.25, 0.25, 1.00, 1.00),
  };

  output.position = positions[vertex_index];
  output.color = colors[vertex_index];

  return output;
}

float4 fragment_main(FragInput input) : SV_TARGET
{
  return input.color;
}