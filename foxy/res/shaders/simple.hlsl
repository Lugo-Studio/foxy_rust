// struct VertexInput
// {
//   [[vk::location(0)]] float4 position : POSITION0;
//   [[vk::location(1)]] float4 color    : COLOR0;
// };

struct FragmentInput
{
  float4 position : SV_POSITION;
  [[vk::location(0)]] float4 color    : COLOR0;
};

FragmentInput vertex_main(
  [[vk::location(0)]] float4 position : POSITION0,
  [[vk::location(1)]] float4 color : COLOR0
) {
  FragmentInput output;

  output.position = position;
  output.color = color;

  return output;
}

float4 fragment_main(FragmentInput input) : SV_TARGET
{
  return input.color;
}