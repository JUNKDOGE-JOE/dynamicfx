struct FxUniforms {
    u_resolution: vec2<f32>,
    u_time: f32,
    u_frame: f32,
};
@group(0) @binding(0) var u_input: texture_2d<f32>;
@group(0) @binding(1) var u_sampler: sampler;
@group(0) @binding(2) var<uniform> fx: FxUniforms;
@group(0) @binding(3) var original: texture_2d<f32>;
@fragment
fn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {
    let delta=vec2<f32>(1.0/fx.u_resolution.x,0.0);
    let a=textureSample(u_input,u_sampler,v_uv-delta);
    let b=textureSample(u_input,u_sampler,v_uv);
    let c=textureSample(u_input,u_sampler,v_uv+delta);
    let base=textureSample(original,u_sampler,v_uv);
    return (a+2.0*b+c)*0.25+base*0.025;
}
