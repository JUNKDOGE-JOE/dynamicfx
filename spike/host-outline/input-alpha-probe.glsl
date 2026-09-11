#version 450
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_in;
layout(set=0,binding=1) uniform sampler u_s;
layout(set=0,binding=2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
};
void main() {
    float outsideWithinBounds = texture(sampler2D(u_in,u_s), vec2(210,490)/u_resolution).a;
    float inside = texture(sampler2D(u_in,u_s), vec2(400,300)/u_resolution).a;
    float outsideBounds = texture(sampler2D(u_in,u_s), vec2(50,50)/u_resolution).a;
    outColor = vec4(outsideWithinBounds,inside,outsideBounds,1.0);
}
