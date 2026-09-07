@dynamicfx 1
@graph
pass field: input -> light
pass glow: light, input -> output
@end
@pass field
// DynamicFX 0.1.0+: select Language = WGSL. This is an envelope, so literal
// line-leading WGSL attributes use @@. Apply to an opaque solid or footage.
// The intermediate stores coverage-weighted RGB plus coverage in alpha.
// The glow pass really reads neighboring intermediate texels and original input.
// @param speed label:"Flow Speed" min:0 max:4 default:0.65
// @param radius label:"Field Radius" min:0.05 max:0.45 default:0.27
// @param tint label:"Light Tint" hint:color default:#42BFFF
struct FxUniforms {
    u_resolution: vec2<f32>,
    u_time: f32,
    u_frame: f32,
    speed: f32,
    radius: f32,
    tint: vec4<f32>,
}
@@group(0) @binding(0) var u_input: texture_2d<f32>;
@@group(0) @binding(1) var u_sampler: sampler;
@@group(0) @binding(2) var<uniform> fx: FxUniforms;

@@fragment
fn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {
    let resolution = max(fx.u_resolution, vec2<f32>(1.0));
    let p = (v_uv - vec2<f32>(0.5)) * resolution / min(resolution.x, resolution.y);
    let time = fx.u_time * clamp(fx.speed, 0.0, 4.0);
    let distance = length(p) - clamp(fx.radius, 0.05, 0.45)
        - 0.016 * sin(p.x * 9.0 + time) * sin(p.y * 7.0 - time);
    let coverage = clamp(0.5 - distance / max(fwidth(distance), 0.00001), 0.0, 1.0);
    let wave = 0.5 + 0.5 * sin(p.y * 13.0 + sin(p.x * 8.0 + time) - time);
    let source = textureSample(u_input, u_sampler, v_uv);
    let color = mix(fx.tint.rgb, vec3<f32>(0.85, 0.3, 1.0), wave)
        * (0.25 + 0.75 * wave) + source.rgb * 0.12;
    let alpha = coverage * clamp(fx.tint.a, 0.0, 1.0);
    return vec4<f32>(color * alpha, alpha);
}
@endpass
@pass glow
// Only this pass declares these controls, so they belong to the glow group.
// Radius is in logical canvas pixels, not physical texels: preview scale
// changes the texel footprint without changing the authored radius.
// @param blur_px label:"Glow Radius (px)" min:0 max:12 default:4
// @param strength label:"Glow Strength" min:0 max:2 default:0.75
// @param source_mix label:"Source Amount" min:0 max:1 default:0.8
struct FxUniforms {
    u_resolution: vec2<f32>,
    u_time: f32,
    u_frame: f32,
    blur_px: f32,
    strength: f32,
    source_mix: f32,
}
@@group(0) @binding(0) var u_light: texture_2d<f32>;
@@group(0) @binding(1) var u_sampler: sampler;
@@group(0) @binding(2) var<uniform> fx: FxUniforms;
@@group(0) @binding(3) var u_original: texture_2d<f32>;

@@fragment
fn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {
    let delta = clamp(fx.blur_px, 0.0, 12.0) / max(fx.u_resolution, vec2<f32>(1.0));
    // Compact 3x3 binomial kernel, weights sum to 16. This is a small-radius
    // demonstration, not an accurate wide Gaussian blur or a mip pyramid.
    var light = textureSample(u_light, u_sampler, v_uv) * 4.0;
    light += textureSample(u_light, u_sampler, v_uv + vec2<f32>(delta.x, 0.0)) * 2.0;
    light += textureSample(u_light, u_sampler, v_uv - vec2<f32>(delta.x, 0.0)) * 2.0;
    light += textureSample(u_light, u_sampler, v_uv + vec2<f32>(0.0, delta.y)) * 2.0;
    light += textureSample(u_light, u_sampler, v_uv - vec2<f32>(0.0, delta.y)) * 2.0;
    light += textureSample(u_light, u_sampler, v_uv + delta);
    light += textureSample(u_light, u_sampler, v_uv - delta);
    light += textureSample(u_light, u_sampler, v_uv + vec2<f32>(delta.x, -delta.y));
    light += textureSample(u_light, u_sampler, v_uv + vec2<f32>(-delta.x, delta.y));
    light *= 0.0625;
    let source = textureSample(u_original, u_sampler, v_uv);
    let rgb = source.rgb * clamp(fx.source_mix, 0.0, 1.0)
        + light.rgb * max(fx.strength, 0.0);
    // Opaque demonstration: use footage or a comp-sized solid. The shader
    // owns this output policy; the runtime does not convert alpha or color.
    return vec4<f32>(rgb, 1.0);
}
@endpass
