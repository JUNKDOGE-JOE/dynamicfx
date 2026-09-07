@dynamicfx 1
@graph
pass field: input -> output
@end
@pass field
// DynamicFX 0.1.0+: select Language = WGSL, then wrap this whole file in
// backticks and append ;0. Envelope attributes start with @@ deliberately.
// A continuous analytic field with derivative coverage; no random cell grid.
// Apply to a comp-sized solid or footage. RGB changes; input alpha is carried.
// @param speed label:"Flow Speed" min:0 max:4 default:0.7
// @param radius label:"Radius (short edge)" min:0.05 max:0.45 default:0.29
// @param rim_px label:"Rim Width (px)" min:0.1 max:8 default:1.5
// @param amount label:"Field Amount" min:0 max:1 default:0.92
// @param cyan label:"Cyan" hint:color default:#20D9FF
// @param violet label:"Violet" hint:color default:#A04CFF
// @param enabled label:"Enable Field" hint:bool default:1
struct FxUniforms {
    u_resolution: vec2<f32>,
    u_time: f32,
    u_frame: f32,
    speed: f32,
    radius: f32,
    rim_px: f32,
    amount: f32,
    cyan: vec4<f32>,
    violet: vec4<f32>,
    enabled: i32,
}
@@group(0) @binding(0) var u_input: texture_2d<f32>;
@@group(0) @binding(1) var u_sampler: sampler;
@@group(0) @binding(2) var<uniform> fx: FxUniforms;

@@fragment
fn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {
    let resolution = max(fx.u_resolution, vec2<f32>(1.0));
    let short_edge = min(resolution.x, resolution.y);
    let p = (v_uv - vec2<f32>(0.5)) * resolution / short_edge;
    let time = fx.u_time * clamp(fx.speed, 0.0, 4.0);
    let wobble = 0.018 * sin(p.x * 8.0 + time)
        * sin(p.y * 6.0 - time * 0.7);
    let distance = length(p) - clamp(fx.radius, 0.05, 0.45) - wobble;
    // Logical geometry stays fixed at reduced previews; fwidth follows the
    // physical fragment footprint. Derivatives run before any selection.
    let footprint = max(fwidth(distance), 0.00001);
    let coverage = clamp(0.5 - distance / footprint, 0.0, 1.0);
    let half_rim = max(fx.rim_px, 0.1) * 0.5 / short_edge;
    let rim = clamp(0.5 - (distance - half_rim) / footprint, 0.0, 1.0)
        - clamp(0.5 - (distance + half_rim) / footprint, 0.0, 1.0);
    let phase = p.y * 11.0 + sin(p.x * 9.0 + time * 0.5) * 1.7 - time;
    let weight = 0.5 + 0.5 * sin(phase);
    let tint = mix(fx.cyan, fx.violet, weight);
    let radiance = tint.rgb * (0.35 + 0.65 * weight)
        + vec3<f32>(rim * 0.5);
    let source = textureSample(u_input, u_sampler, v_uv);
    let opacity = coverage * clamp(fx.amount * tint.a, 0.0, 1.0);
    let result = vec4<f32>(mix(source.rgb, radiance, opacity), source.a);
    return select(source, result, fx.enabled != 0);
}
@endpass
