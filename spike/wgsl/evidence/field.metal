// language: metal2.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct FxUniforms {
    metal::float2 u_resolution;
    float u_time;
    float u_frame;
    float gain;
    char _pad4[12];
    metal::packed_float3 tint;
    int enabled;
};

float smooth_noise(
    metal::float2 p
) {
    metal::float2 q = metal::floor(p);
    metal::float2 f = metal::fract(p);
    metal::float2 u = ((f * f) * f) * ((f * ((f * 6.0) - metal::float2(15.0))) + metal::float2(10.0));
    float a = metal::fract(metal::sin(metal::dot(q, metal::float2(12.9898, 78.233))) * 43758.547);
    float b = metal::fract(metal::sin(metal::dot(q + metal::float2(1.0, 0.0), metal::float2(12.9898, 78.233))) * 43758.547);
    float c = metal::fract(metal::sin(metal::dot(q + metal::float2(0.0, 1.0), metal::float2(12.9898, 78.233))) * 43758.547);
    float d = metal::fract(metal::sin(metal::dot(q + metal::float2(1.0, 1.0), metal::float2(12.9898, 78.233))) * 43758.547);
    return metal::mix(metal::mix(a, b, u.x), metal::mix(c, d, u.x), u.y);
}

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
) {
    const auto v_uv = varyings.v_uv;
    float _e7 = fx.u_resolution.x;
    float _e11 = fx.u_resolution.y;
    metal::float2 p_1 = (v_uv - metal::float2(0.5)) * metal::float2(_e7 / _e11, 1.0);
    float _e20 = fx.u_time;
    float _e25 = smooth_noise((p_1 * 5.0) + metal::float2(_e20 * 0.1));
    float d_1 = (metal::length(p_1) - 0.31) - (0.025 * _e25);
    float _e32 = metal::fwidth(d_1);
    float aa = metal::max(_e32, 0.0001);
    float edge = 1.0 - metal::smoothstep(-(aa), aa, d_1);
    metal::float4 source = u_input.sample(u_sampler, v_uv);
    metal::float3 _e44 = fx.tint;
    float _e47 = fx.gain;
    metal::float3 color = (_e44 * ((edge * _e47) * 1.8)) + (source.xyz * 0.15);
    int _e60 = fx.enabled;
    return main_Output { (_e60 != 0) ? metal::float4(color, 1.0) : source };
}
