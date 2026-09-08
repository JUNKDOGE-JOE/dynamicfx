@dynamicfx 1
@graph
pass field: input -> output
@end
@pass field
struct FxUniforms {u_resolution:vec2<f32>,u_time:f32,u_frame:f32,}
@@group(0) @binding(0) var u_input:texture_2d<f32>;
@@group(0) @binding(1) var u_sampler:sampler;
@@group(0) @binding(2) var<uniform> fx:FxUniforms;
@@fragment fn main(@location(0) v_uv:vec2<f32>)->@location(0) vec4<f32>{
let c=textureSample(u_input,u_sampler,v_uv);return vec4<f32>(c.r,v_uv.y,0.25,1.0);}
@endpass
