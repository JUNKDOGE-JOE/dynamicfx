@dynamicfx 1
@graph
pass field: input -> output
@end
@pass field
#version 450
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_input;
layout(set=0,binding=1) uniform sampler u_sampler;
layout(set=0,binding=2) uniform FxUniforms {vec2 u_resolution;float u_time;float u_frame;};
void main(){vec4 c=texture(sampler2D(u_input,u_sampler),v_uv);outColor=vec4(c.r,v_uv.y,0.25,1.0);}
@endpass
