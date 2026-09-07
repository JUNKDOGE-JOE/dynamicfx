// language: metal2.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct FxUniforms {
    metal::float2 u_resolution;
    float u_time;
    float u_frame;
};

struct main_Input {
    metal::float2 v_uv [[user(loc0), center_perspective]];
};
struct main_Output {
    metal::float4 member [[color(0)]];
};
fragment main_Output main_(
  main_Input varyings [[stage_in]]
, metal::texture2d<float, metal::access::sample> u_input [[texture(0)]]
, metal::sampler u_sampler [[sampler(1)]]
, constant FxUniforms& fx [[buffer(2)]]
, metal::texture2d<float, metal::access::sample> original [[texture(3)]]
) {
    const auto v_uv = varyings.v_uv;
    float _e4 = fx.u_resolution.x;
    metal::float2 delta = metal::float2(1.0 / _e4, 0.0);
    metal::float4 a = u_input.sample(u_sampler, v_uv - delta);
    metal::float4 b = u_input.sample(u_sampler, v_uv);
    metal::float4 c = u_input.sample(u_sampler, v_uv + delta);
    metal::float4 base = original.sample(u_sampler, v_uv);
    return main_Output { (((a + (2.0 * b)) + c) * 0.25) + (base * 0.025) };
}
