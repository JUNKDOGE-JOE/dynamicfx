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
layout(set=0,binding=2,std140) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float gain;
    vec3 tint;
    int enabled;
};
float smooth_noise(vec2 p) {
    vec2 q=floor(p), f=fract(p);
    vec2 u=f*f*f*(f*(f*6.0-15.0)+10.0);
    float a=fract(sin(dot(q,vec2(12.9898,78.233)))*43758.5453);
    float b=fract(sin(dot(q+vec2(1,0),vec2(12.9898,78.233)))*43758.5453);
    float c=fract(sin(dot(q+vec2(0,1),vec2(12.9898,78.233)))*43758.5453);
    float d=fract(sin(dot(q+vec2(1,1),vec2(12.9898,78.233)))*43758.5453);
    return mix(mix(a,b,u.x),mix(c,d,u.x),u.y);
}
void main() {
    vec2 p=(v_uv-0.5)*vec2(u_resolution.x/u_resolution.y,1.0);
    float n=smooth_noise(p*5.0+u_time*0.1);
    float d=length(p)-0.31-0.025*n;
    float aa=max(fwidth(d),0.0001);
    float edge=1.0-smoothstep(-aa,aa,d);
    vec4 source=texture(sampler2D(u_input,u_sampler),v_uv);
    vec3 color=tint*(edge*gain*1.8)+source.rgb*0.15;
    outColor=enabled!=0?vec4(color,1):source;
}
@endpass
