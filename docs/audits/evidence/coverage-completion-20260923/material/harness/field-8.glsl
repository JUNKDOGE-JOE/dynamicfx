@dynamicfx 1
@graph
pass SurfaceX: host_alpha -> surface_x
pass SurfaceY: surface_x -> surface
pass Decode: surface -> output
@end
@pass SurfaceX
#version 450
// @param host_alpha hint:coverage
// @param edge_width label:"Edge Width (px)" min:2 max:64 default:22
// @param padding label:"Sampling Margin (px)" hint:canvas min:0 max:128 default:64
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_in;
layout(set=0,binding=1) uniform sampler u_s;
layout(set=0,binding=2) uniform FxUniforms {
    vec2 u_resolution; float u_time; float u_frame;
    float edge_width; float padding;
};

// Float bits survive even an 8-bpc intermediate. Consumers decode texels
// before filtering; interpolating the encoded channels would corrupt the field.
vec4 encode_field(float value) {
    uint bits=floatBitsToUint(value);
    return vec4(float(bits&255u),float((bits>>8u)&255u),
                float((bits>>16u)&255u),float((bits>>24u)&255u))/255.0;
}
void main() {
    ivec2 size=textureSize(sampler2D(u_in,u_s),0);
    ivec2 p=ivec2(v_uv*vec2(size));
    float pixel=u_resolution.x/float(size.x);
    float radius=clamp(edge_width,2.0,64.0);
    float sigma=max(radius/3.0,pixel*0.65);
    int reach=int(ceil(radius/pixel));
    float sum=0.0,weights=0.0;
    for(int i=-64;i<=64;i++) {
        if(abs(i)>reach)continue;
        float d=float(i)*pixel;
        float w=exp(-0.5*d*d/(sigma*sigma));
        ivec2 q=p+ivec2(i,0);
        float a=0.0;
        if(all(greaterThanEqual(q,ivec2(0)))&&all(lessThan(q,size)))
            a=texelFetch(sampler2D(u_in,u_s),q,0).r;
        sum+=a*w; weights+=w;
    }
    outColor=encode_field(sum/weights);
}
@endpass
@pass SurfaceY
#version 450
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_in;
layout(set=0,binding=1) uniform sampler u_s;
layout(set=0,binding=2) uniform FxUniforms {
    vec2 u_resolution; float u_time; float u_frame;
    float edge_width;
};
float decode_field(vec4 encoded) {
    uvec4 b=uvec4(round(clamp(encoded,0.0,1.0)*255.0));
    return uintBitsToFloat(b.x|(b.y<<8u)|(b.z<<16u)|(b.w<<24u));
}
vec4 encode_field(float value) {
    uint bits=floatBitsToUint(value);
    return vec4(float(bits&255u),float((bits>>8u)&255u),
                float((bits>>16u)&255u),float((bits>>24u)&255u))/255.0;
}
void main() {
    ivec2 size=textureSize(sampler2D(u_in,u_s),0);
    ivec2 p=ivec2(v_uv*vec2(size));
    float pixel=u_resolution.y/float(size.y);
    float radius=clamp(edge_width,2.0,64.0);
    float sigma=max(radius/3.0,pixel*0.65);
    int reach=int(ceil(radius/pixel));
    float sum=0.0,weights=0.0;
    for(int i=-64;i<=64;i++) {
        if(abs(i)>reach)continue;
        float d=float(i)*pixel;
        float w=exp(-0.5*d*d/(sigma*sigma));
        ivec2 q=p+ivec2(0,i);
        float a=0.0;
        if(all(greaterThanEqual(q,ivec2(0)))&&all(lessThan(q,size)))
            a=decode_field(texelFetch(sampler2D(u_in,u_s),q,0));
        sum+=a*w; weights+=w;
    }
    outColor=encode_field(sum/weights);
}
@endpass
@pass Decode
#version 450
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_in;
layout(set=0,binding=1) uniform sampler u_s;
layout(set=0,binding=2) uniform FxUniforms {vec2 u_resolution;float u_time;float u_frame;};
void main(){outColor=texelFetch(sampler2D(u_in,u_s),ivec2(v_uv*vec2(textureSize(sampler2D(u_in,u_s),0))),0);}
@endpass
