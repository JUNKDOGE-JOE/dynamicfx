#version 450
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_input;
layout(set=0,binding=1) uniform sampler u_sampler;
layout(set=0,binding=2,std140) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
};
layout(set=0,binding=3) uniform texture2D original;
void main() {
    vec2 delta=vec2(1.0/u_resolution.x,0.0);
    vec4 a=texture(sampler2D(u_input,u_sampler),v_uv-delta);
    vec4 b=texture(sampler2D(u_input,u_sampler),v_uv);
    vec4 c=texture(sampler2D(u_input,u_sampler),v_uv+delta);
    vec4 base=texture(sampler2D(original,u_sampler),v_uv);
    outColor=(a+2.0*b+c)*0.25+base*0.025;
}
