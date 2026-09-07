@dynamicfx 1
@graph
pass test: input -> output
@end
@pass test
// @param gain default:0.25
// @param tint default:0.2,0.6,1
// @param tail default:-0.5
struct Uniforms { u_resolution: vec2f, u_time: f32, u_frame: f32, @align(32) gain: f32, @size(32) tint: vec3f, tail: f32, }
@@group(0) @binding(2) var<uniform> fx: Uniforms;
@@fragment fn main() -> @location(0) vec4f { return vec4f(fx.gain, fx.tint.y, fx.tail, 1.0); }
@endpass
