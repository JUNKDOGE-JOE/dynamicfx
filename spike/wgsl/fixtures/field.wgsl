// @param gain label:"Gain" default:0.72 min:0 max:2
// @param tint hint:color default:0.25,0.7,1.0
// @param enabled hint:bool default:1
struct FxUniforms {
    u_resolution: vec2<f32>,
    u_time: f32,
    u_frame: f32,
    gain: f32,
    tint: vec3<f32>,
    enabled: i32,
};
@group(0) @binding(0) var u_input: texture_2d<f32>;
@group(0) @binding(1) var u_sampler: sampler;
@group(0) @binding(2) var<uniform> fx: FxUniforms;

fn smooth_noise(p: vec2<f32>) -> f32 {
    let q = floor(p);
    let f = fract(p);
    let u = f*f*f*(f*(f*6.0-15.0)+10.0);
    let a = fract(sin(dot(q, vec2<f32>(12.9898,78.233)))*43758.5453);
    let b = fract(sin(dot(q+vec2<f32>(1,0), vec2<f32>(12.9898,78.233)))*43758.5453);
    let c = fract(sin(dot(q+vec2<f32>(0,1), vec2<f32>(12.9898,78.233)))*43758.5453);
    let d = fract(sin(dot(q+vec2<f32>(1,1), vec2<f32>(12.9898,78.233)))*43758.5453);
    return mix(mix(a,b,u.x), mix(c,d,u.x), u.y);
}
@fragment
fn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {
    let p = (v_uv-0.5)*vec2<f32>(fx.u_resolution.x/fx.u_resolution.y,1.0);
    let n = smooth_noise(p*5.0+fx.u_time*0.1);
    let d = length(p)-0.31-0.025*n;
    let aa = max(fwidth(d), 0.0001);
    let edge = 1.0-smoothstep(-aa,aa,d);
    let source = textureSample(u_input,u_sampler,v_uv);
    let color = fx.tint*(edge*fx.gain*1.8)+source.rgb*0.15;
    return select(source,vec4<f32>(color,1.0),fx.enabled!=0);
}
