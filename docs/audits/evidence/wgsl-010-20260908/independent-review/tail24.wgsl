@dynamicfx 1
@graph
pass test: input -> output
@end
@pass test
// @param gain default:0.25
struct Uniforms { u_resolution: vec2f, u_time: f32, u_frame: f32, gain: f32, }
@@group(0) @binding(2) var<uniform> fx: Uniforms;
@@fragment fn main() -> @location(0) vec4f { return vec4f(fx.gain, fx.u_resolution.x / 64.0, fx.u_time, 1.0); }
@endpass
