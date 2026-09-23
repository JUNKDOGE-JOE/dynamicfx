@dynamicfx 1
@graph
pass main: input, coverage -> output
@end
@pass main
#version 450
// @param coverage hint:coverage
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_in;
layout(set=0,binding=1) uniform sampler u_s;
layout(set=0,binding=2) uniform FxUniforms { vec2 u_resolution; float u_time; float u_frame; };
layout(set=0,binding=3) uniform texture2D u_cov;
void main(){ float a=texelFetch(sampler2D(u_cov,u_s),ivec2(v_uv*vec2(textureSize(sampler2D(u_cov,u_s),0))),0).r; outColor=vec4(0,0,0,a); }
@endpass