@dynamicfx 1
@graph
pass DistanceX: host_alpha -> distance_x
pass Distance: distance_x -> distance
pass FrostX: input -> blur_x
pass Frost: blur_x -> blur
pass Glass: input, distance, blur, host_alpha -> output
@end
@pass DistanceX
#version 450
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_in;
layout(set=0,binding=1) uniform sampler u_s;
// @param host_alpha hint:coverage
// @param padding label:"Sampling Margin (px)" hint:canvas min:0 max:128 default:80
layout(set=0,binding=2) uniform FxUniforms {vec2 u_resolution;float u_time;float u_frame;float padding;};
vec4 packFloat(float v){uint b=floatBitsToUint(v);return vec4(float(b&255u),float((b>>8u)&255u),float((b>>16u)&255u),float((b>>24u)&255u))/255.0;}
float unpackFloat(vec4 v){uvec4 b=uvec4(round(clamp(v,0.0,1.0)*255.0));return uintBitsToFloat(b.x|(b.y<<8u)|(b.z<<16u)|(b.w<<24u));}

// Exact bounded squared distance to transparent raster samples. Alpha remains
// untouched for compositing; positive fractional opacity still belongs to the shape.
void main(){ivec2 s=textureSize(sampler2D(u_in,u_s),0),p=ivec2(v_uv*vec2(s));float pixel=u_resolution.x/float(s.x);float best=4096.0;
for(int i=-64;i<=64;i++){float d=float(i)*pixel;if(abs(d)>64.0)continue;ivec2 q=p+ivec2(i,0);float a=0.0;if(all(greaterThanEqual(q,ivec2(0)))&&all(lessThan(q,s)))a=texelFetch(sampler2D(u_in,u_s),q,0).r;if(a<=0.0)best=min(best,d*d);}outColor=packFloat(best);}
@endpass
@pass Distance
#version 450
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_in;
layout(set=0,binding=1) uniform sampler u_s;
layout(set=0,binding=2) uniform FxUniforms {vec2 u_resolution;float u_time;float u_frame;};
vec4 packFloat(float v){uint b=floatBitsToUint(v);return vec4(float(b&255u),float((b>>8u)&255u),float((b>>16u)&255u),float((b>>24u)&255u))/255.0;}
float unpackFloat(vec4 v){uvec4 b=uvec4(round(clamp(v,0.0,1.0)*255.0));return uintBitsToFloat(b.x|(b.y<<8u)|(b.z<<16u)|(b.w<<24u));}

void main(){ivec2 s=textureSize(sampler2D(u_in,u_s),0),p=ivec2(v_uv*vec2(s));float pixel=u_resolution.y/float(s.y);float best=4096.0;
for(int i=-64;i<=64;i++){float d=float(i)*pixel;if(abs(d)>64.0)continue;ivec2 q=p+ivec2(0,i);float dx=0.0;if(all(greaterThanEqual(q,ivec2(0)))&&all(lessThan(q,s)))dx=unpackFloat(texelFetch(sampler2D(u_in,u_s),q,0));best=min(best,dx+d*d);}outColor=packFloat(sqrt(best));}
@endpass
@pass FrostX
#version 450
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_in;
layout(set=0,binding=1) uniform sampler u_s;
// @param frost label:"Frost Radius (px)" min:0 max:12 default:1
layout(set=0,binding=2) uniform FxUniforms {vec2 u_resolution;float u_time;float u_frame;float frost;};

void main(){ivec2 s=textureSize(sampler2D(u_in,u_s),0),p=ivec2(v_uv*vec2(s));float pixel=u_resolution.x/float(s.x);float sigma=clamp(frost,0.0,12.0);if(sigma<0.01){outColor=texelFetch(sampler2D(u_in,u_s),p,0);return;}vec4 sum=vec4(0.0);float weights=0.0;
for(int i=-36;i<=36;i++){float d=float(i)*pixel;if(abs(d)>max(3.0*sigma,pixel))continue;float w=exp(-0.5*d*d/(sigma*sigma));ivec2 q=p+ivec2(i,0);vec4 c=vec4(0.0);if(all(greaterThanEqual(q,ivec2(0)))&&all(lessThan(q,s)))c=texelFetch(sampler2D(u_in,u_s),q,0);sum+=c*w;weights+=w;}outColor=sum/weights;}
@endpass
@pass Frost
#version 450
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_in;
layout(set=0,binding=1) uniform sampler u_s;
layout(set=0,binding=2) uniform FxUniforms {vec2 u_resolution;float u_time;float u_frame;float frost;};

void main(){ivec2 s=textureSize(sampler2D(u_in,u_s),0),p=ivec2(v_uv*vec2(s));float pixel=u_resolution.y/float(s.y);float sigma=clamp(frost,0.0,12.0);if(sigma<0.01){outColor=texelFetch(sampler2D(u_in,u_s),p,0);return;}vec4 sum=vec4(0.0);float weights=0.0;
for(int i=-36;i<=36;i++){float d=float(i)*pixel;if(abs(d)>max(3.0*sigma,pixel))continue;float w=exp(-0.5*d*d/(sigma*sigma));ivec2 q=p+ivec2(0,i);vec4 c=vec4(0.0);if(all(greaterThanEqual(q,ivec2(0)))&&all(lessThan(q,s)))c=texelFetch(sampler2D(u_in,u_s),q,0);sum+=c*w;weights+=w;}outColor=sum/weights;}
@endpass
@pass Glass
#version 450
/*
MIT License

Copyright (c) 2024 Charles Yin
Copyright (c) 2022 Adam Lastowka (color functions)

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_in;
layout(set=0,binding=1) uniform sampler u_s;
// Optical shading adapted from iyinchao/liquid-glass-studio, MIT, Charles Yin.
// Pinned source and modifications: examples/liquid-glass-upstream.md.
// @param edge_width label:"Glass Thickness (px)" min:2 max:60 default:20
// @param refraction label:"Refraction (px)" min:0 max:160 default:70.710678
// @param ior label:"Index of Refraction" min:1 max:4 default:1.4
// @param dispersion_strength label:"Dispersion" min:0 max:50 default:7
// @param fresnel_range label:"Fresnel Range" min:1 max:90 default:30
// @param fresnel_strength label:"Fresnel Strength" min:0 max:1 default:0.2
// @param fresnel_hardness label:"Fresnel Hardness" min:0 max:1 default:0.2
// @param glare_range label:"Glare Range" min:1 max:90 default:30
// @param glare_hardness label:"Glare Hardness" min:0 max:1 default:0.2
// @param glare_convergence label:"Glare Convergence" min:0 max:1 default:0.5
// @param glare_opposite label:"Opposite Glare" min:0 max:1 default:0.8
// @param highlight label:"Glare Strength" min:0 max:1.2 default:0.9
// @param light_angle label:"Glare Angle" hint:angle default:-45
// @param tint label:"Glass Tint" hint:color default:#FFFFFF
// @param tint_amount label:"Tint Amount" min:0 max:1 default:0
// @param amount label:"Amount" min:0 max:1 default:1
// @param linear_color label:"Linear RGB Input" hint:bool default:0
layout(set=0,binding=2) uniform FxUniforms {vec2 u_resolution;float u_time;float u_frame;float edge_width;float refraction;float ior;float dispersion_strength;float fresnel_range;float fresnel_strength;float fresnel_hardness;float glare_range;float glare_hardness;float glare_convergence;float glare_opposite;float highlight;float light_angle;vec4 tint;float tint_amount;float amount;int linear_color;};
layout(set=0,binding=3) uniform texture2D u_distance;
layout(set=0,binding=4) uniform texture2D u_blur;
layout(set=0,binding=5) uniform texture2D u_coverage;
vec4 packFloat(float v){uint b=floatBitsToUint(v);return vec4(float(b&255u),float((b>>8u)&255u),float((b>>16u)&255u),float((b>>24u)&255u))/255.0;}
float unpackFloat(vec4 v){uvec4 b=uvec4(round(clamp(v,0.0,1.0)*255.0));return uintBitsToFloat(b.x|(b.y<<8u)|(b.z<<16u)|(b.w<<24u));}
// LCH helpers: GLSL-Color-Functions, MIT, Adam Lastowka; see bundled license.
//                          0.3127/0.3290  1.0  (1.0-0.3127-0.3290)/0.329
const vec3 D65_WHITE = vec3(0.95045592705, 1.0, 1.08905775076);
//                          0.3457/0.3585  1.0  (1.0-0.3457-0.3585)/0.3585
const vec3 D50_WHITE = vec3(0.96429567643, 1.0, 0.82510460251);
vec3 WHITE = D65_WHITE;
const mat3 RGB_TO_XYZ_M = mat3(
  0.4124, 0.3576, 0.1805,
  0.2126, 0.7152, 0.0722,
  0.0193, 0.1192, 0.9505
);
const mat3 XYZ_TO_XYZ50_M = mat3(
   1.0479298208405488  ,  0.022946793341019088, -0.05019222954313557 ,
   0.029627815688159344,  0.990434484573249   , -0.01707382502938514 ,
  -0.009243058152591178,  0.015055144896577895,  0.7518742899580008
);
const mat3 XYZ_TO_RGB_M = mat3(
   3.2406255, -1.537208 , -0.4986286,
  -0.9689307,  1.8757561,  0.0415175,
   0.0557101, -0.2040211,  1.0569959
);
const mat3 XYZ50_TO_XYZ_M = mat3(
   0.9554734527042182  , -0.023098536874261423,  0.0632593086610217  ,
  -0.028369706963208136,  1.0099954580058226  ,  0.021041398966943008,
   0.012314001688319899, -0.020507696433477912,  1.3303659366080753
);
float UNCOMPAND_SRGB(float a) {
  return a > 0.04045
    ? pow((a + 0.055) / 1.055, 2.4)
    : a / 12.92;
}
float COMPAND_RGB(float a) {
  return a <= 0.0031308
    ? 12.92 * a
    : 1.055 * pow(a, 0.41666666666) - 0.055;
}
vec3 RGB_TO_XYZ(vec3 rgb) {
  return WHITE == D65_WHITE
    ? rgb * RGB_TO_XYZ_M
    : rgb * RGB_TO_XYZ_M * XYZ_TO_XYZ50_M;
}
vec3 SRGB_TO_RGB(vec3 srgb) {
  return vec3(UNCOMPAND_SRGB(srgb.x), UNCOMPAND_SRGB(srgb.y), UNCOMPAND_SRGB(srgb.z));
}
vec3 RGB_TO_SRGB(vec3 rgb) {
  return vec3(COMPAND_RGB(rgb.x), COMPAND_RGB(rgb.y), COMPAND_RGB(rgb.z));
}
vec3 SRGB_TO_XYZ(vec3 srgb) {
  return RGB_TO_XYZ(SRGB_TO_RGB(srgb));
}
float XYZ_TO_LAB_F(float x) {
  //          (24/116)^3                         1/(3*(6/29)^2)     4/29
  return x > 0.00885645167
    ? pow(x, 0.333333333)
    : 7.78703703704 * x + 0.13793103448;
}
vec3 XYZ_TO_LAB(vec3 xyz) {
  vec3 xyz_scaled = xyz / WHITE;
  xyz_scaled = vec3(
    XYZ_TO_LAB_F(xyz_scaled.x),
    XYZ_TO_LAB_F(xyz_scaled.y),
    XYZ_TO_LAB_F(xyz_scaled.z)
  );
  return vec3(
    116.0 * xyz_scaled.y - 16.0,
    500.0 * (xyz_scaled.x - xyz_scaled.y),
    200.0 * (xyz_scaled.y - xyz_scaled.z)
  );
}
vec3 SRGB_TO_LAB(vec3 srgb) {
  return XYZ_TO_LAB(SRGB_TO_XYZ(srgb));
}
vec3 LAB_TO_LCH(vec3 Lab) {
  return vec3(Lab.x, sqrt(dot(Lab.yz, Lab.yz)), atan(Lab.z, Lab.y) * 57.2957795131);
}
vec3 SRGB_TO_LCH(vec3 srgb) {
  return LAB_TO_LCH(SRGB_TO_LAB(srgb));
}
vec3 XYZ_TO_RGB(vec3 xyz) {
  return WHITE == D65_WHITE
    ? xyz * XYZ_TO_RGB_M
    : xyz * XYZ50_TO_XYZ_M * XYZ_TO_RGB_M;
}
vec3 XYZ_TO_SRGB(vec3 xyz) {
  return RGB_TO_SRGB(XYZ_TO_RGB(xyz));
}
float LAB_TO_XYZ_F(float x) {
  //                                     3*(6/29)^2         4/29
  return x > 0.206897
    ? x * x * x
    : 0.12841854934 * (x - 0.137931034);
}
vec3 LAB_TO_XYZ(vec3 Lab) {
  float w = (Lab.x + 16.0) / 116.0;
  return WHITE *
  vec3(LAB_TO_XYZ_F(w + Lab.y / 500.0), LAB_TO_XYZ_F(w), LAB_TO_XYZ_F(w - Lab.z / 200.0));
}
vec3 LAB_TO_SRGB(vec3 lab) {
  return XYZ_TO_SRGB(LAB_TO_XYZ(lab));
}
vec3 LCH_TO_LAB(vec3 LCh) {
  return vec3(LCh.x, LCh.y * cos(LCh.z * 0.01745329251), LCh.y * sin(LCh.z * 0.01745329251));
}
vec3 LCH_TO_SRGB(vec3 lch) {
  return LAB_TO_SRGB(LCH_TO_LAB(lch));
}

float rawDistance(ivec2 p,ivec2 s){if(any(lessThan(p,ivec2(0)))||any(greaterThanEqual(p,s)))return 0.0;return unpackFloat(texelFetch(sampler2D(u_distance,u_s),p,0));}
float smoothDistance(ivec2 p,ivec2 s){float v=0.0;for(int y=-2;y<=2;y++)for(int x=-2;x<=2;x++){float wx=x==0?6.0:(abs(x)==1?4.0:1.0);float wy=y==0?6.0:(abs(y)==1?4.0:1.0);v+=rawDistance(p+ivec2(x,y),s)*wx*wy;}return v/256.0;}
vec3 straight(vec4 c){return c.a>0.000001?c.rgb/c.a:vec3(0.0);}
vec3 toLCH(vec3 c){return linear_color!=0?LAB_TO_LCH(XYZ_TO_LAB(RGB_TO_XYZ(c))):SRGB_TO_LCH(c);}
vec3 fromLCH(vec3 c){return linear_color!=0?XYZ_TO_RGB(LAB_TO_XYZ(LCH_TO_LAB(c))):LCH_TO_SRGB(c);}
float edgeBand(float distance,float range,float hardness){float t=max(0.0,1.0-distance/1500.0*pow(500.0/max(range,1.0),2.0)+hardness);return clamp(pow(t,5.0),0.0,1.0);}
void main(){vec4 base=textureLod(sampler2D(u_in,u_s),v_uv,0.0);float a=textureLod(sampler2D(u_coverage,u_s),v_uv,0.0).r;if(a<=0.0||amount<=0.0){outColor=base;return;}
ivec2 s=textureSize(sampler2D(u_distance,u_s),0),p=ivec2(v_uv*vec2(s));vec2 pixel=u_resolution/vec2(s);
float distance=max(0.0,smoothDistance(p,s)-0.5*min(pixel.x,pixel.y));
vec2 gradient=vec2(smoothDistance(p+ivec2(1,0),s)-smoothDistance(p-ivec2(1,0),s),smoothDistance(p+ivec2(0,1),s)-smoothDistance(p-ivec2(0,1),s))/(2.0*pixel);
float lengthN=length(gradient);vec2 normal=-gradient/max(lengthN,0.00001);
float ratio=clamp(1.0-distance/clamp(edge_width,2.0,60.0),0.0,1.0);
// Evaluate the same Snell angle difference algebraically to avoid GPU inverse-trig error.
float sinI=ratio*ratio,sinT=sinI/clamp(ior,1.0,4.0);
float cosI=sqrt(max(0.0,1.0-sinI*sinI)),cosT=sqrt(max(0.0,1.0-sinT*sinT));
float edgeFactor=distance>=edge_width?0.0:max(0.0,(sinI*cosT-cosI*sinT)/max(cosI*cosT+sinI*sinT,0.000001));
// Upstream's normal scaling and viewport factors reduce to a pixel displacement.
vec2 offset=-normal*min(edgeFactor*clamp(refraction,0.0,160.0),512.0)/u_resolution;
float spread=clamp(dispersion_strength,0.0,50.0)*0.02;
vec3 refracted=vec3(straight(textureLod(sampler2D(u_blur,u_s),v_uv+offset*(1.0+spread),0.0)).r,straight(textureLod(sampler2D(u_blur,u_s),v_uv+offset,0.0)).g,straight(textureLod(sampler2D(u_blur,u_s),v_uv+offset*(1.0-spread),0.0)).b);
vec3 result=mix(refracted,tint.rgb,clamp(tint_amount,0.0,1.0)*0.8);
if(edgeFactor>0.0&&lengthN>0.00001){float fresnel=edgeBand(distance,fresnel_range,fresnel_hardness);
vec3 fTint=toLCH(mix(vec3(1.0),tint.rgb,tint_amount*0.5));fTint.x=clamp(fTint.x+20.0*fresnel*fresnel_strength,0.0,100.0);
result=mix(result,fromLCH(fTint),clamp(fresnel*fresnel_strength*0.7,0.0,1.0));
vec2 screenNormal=vec2(normal.x,-normal.y);float angle=atan(screenNormal.y,screenNormal.x);if(angle<0.0)angle+=6.28318530718;
float glareAngle=(angle-0.78539816339+radians(light_angle))*2.0;
bool farSide=(glareAngle>3.14159265359*1.5&&glareAngle<3.14159265359*3.5)||glareAngle< -3.14159265359*0.5;
float angular=(0.5+sin(glareAngle)*0.5)*(farSide?1.2*glare_opposite:1.2)*highlight;
angular=clamp(pow(max(angular,0.0),0.1+glare_convergence*2.0),0.0,1.0);
float strength=angular*edgeBand(distance,glare_range,glare_hardness);
vec3 gTint=toLCH(mix(refracted,tint.rgb,tint_amount*0.5));gTint.x=clamp(gTint.x+150.0*strength,0.0,120.0);gTint.y+=30.0*strength;
result=mix(result,fromLCH(gTint),clamp(strength,0.0,1.0));}
// Preserve associated RGB and let AE composite native coverage once.
outColor=mix(base,vec4(result*base.a,base.a),clamp(amount,0.0,1.0));}
@endpass
