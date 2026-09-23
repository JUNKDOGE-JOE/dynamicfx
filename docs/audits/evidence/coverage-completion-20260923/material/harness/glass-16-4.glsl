@dynamicfx 1
@graph
pass SurfaceX: host_alpha -> surface_x
pass SurfaceY: surface_x -> surface
pass Glass: input, surface, host_alpha -> output
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
@pass Glass
#version 450
// @param refraction label:"Refraction (px)" min:0 max:80 default:26
// @param dispersion label:"Dispersion" min:0 max:0.15 default:0.035
// @param frost label:"Frost (px)" min:0 max:3 default:0.6
// @param highlight label:"Edge Light" min:0 max:2 default:0.65
// @param light_angle label:"Light Direction" hint:angle default:135
// @param tint label:"Glass Tint" hint:color default:#D8EEFF
// @param tint_amount label:"Tint Amount" min:0 max:1 default:0.16
// @param amount label:"Amount" min:0 max:1 default:1
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_in;
layout(set=0,binding=1) uniform sampler u_s;
layout(set=0,binding=2) uniform FxUniforms {
    vec2 u_resolution; float u_time; float u_frame;
    float edge_width; float refraction; float dispersion; float frost;
    float highlight; float light_angle; vec4 tint; float tint_amount; float amount;
};
layout(set=0,binding=3) uniform texture2D u_surface;
layout(set=0,binding=4) uniform texture2D u_coverage;

float field(ivec2 p,ivec2 size) {
    if(any(lessThan(p,ivec2(0)))||any(greaterThanEqual(p,size)))return 0.0;
    uvec4 b=uvec4(round(clamp(texelFetch(sampler2D(u_surface,u_s),p,0),0.0,1.0)*255.0));
    return uintBitsToFloat(b.x|(b.y<<8u)|(b.z<<16u)|(b.w<<24u));
}
vec4 background(vec2 uv) {
    vec2 reach=vec2(clamp(frost,0.0,3.0))/u_resolution;
    vec4 sum=vec4(0.0);
    for(int y=-1;y<=1;y++)for(int x=-1;x<=1;x++) {
        float w=(x==0?2.0:1.0)*(y==0?2.0:1.0);
        sum+=textureLod(sampler2D(u_in,u_s),uv+vec2(x,y)*reach,0.0)*w;
    }
    return sum/16.0;
}
void main() {
    vec4 base=textureLod(sampler2D(u_in,u_s),v_uv,0.0);
    float coverage=textureLod(sampler2D(u_coverage,u_s),v_uv,0.0).r;
    if(coverage<=0.0||amount<=0.0){outColor=base;return;}
    ivec2 size=textureSize(sampler2D(u_surface,u_s),0);
    ivec2 p=ivec2(v_uv*vec2(size));
    vec2 pixel=u_resolution/vec2(size);
    float height=field(p,size);
    vec2 slope=vec2(field(p+ivec2(1,0),size)-field(p-ivec2(1,0),size),
                    field(p+ivec2(0,1),size)-field(p-ivec2(0,1),size))/(2.0*pixel);
    slope=-slope*clamp(edge_width,2.0,64.0)/max(max(coverage,height),0.00001);
    float bend=length(slope);
    vec2 direction=slope/max(bend,0.00001);
    float rim=clamp(bend,0.0,1.0);
    vec2 offset=direction*rim*clamp(refraction,0.0,80.0)/u_resolution;
    float split=clamp(dispersion,0.0,0.15);
    vec4 middle=background(v_uv-offset);
    vec3 color=vec3(background(v_uv-offset*(1.0+split)).r,middle.g,
                    background(v_uv-offset*(1.0-split)).b);
    color*=mix(vec3(1.0),tint.rgb,clamp(tint_amount,0.0,1.0));
    vec2 light=vec2(cos(radians(light_angle)),-sin(radians(light_angle)));
    float glint=pow(max(dot(direction,light),0.0),4.0)
                +0.22*pow(max(dot(direction,-light),0.0),7.0);
    color+=vec3(pow(rim,1.3)*glint*clamp(highlight,0.0,2.0))*base.a;
    // AE applies the adjustment layer's native coverage exactly once.
    outColor=mix(base,vec4(color,base.a),clamp(amount,0.0,1.0));
}
@endpass
