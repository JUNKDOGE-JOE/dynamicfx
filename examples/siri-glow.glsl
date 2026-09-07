@dynamicfx 1
@graph
pass aurora: input -> output
@end
@pass aurora
#version 450
// Siri-inspired aurora rim, not a version-specific Apple UI reproduction.
// Apply to a comp-sized black solid, or footage. All geometry is logical
// pixels; derivatives measure the actual preview footprint. One pass keeps
// the smooth field in float until final output (16/32-bpc recommended).
// The frame is deliberately drawn INSIDE the canvas: no padding is needed.
// @param speed label:"Flow Speed" min:0 max:3 default:0.55
// @param activity label:"Activity" min:0 max:2 default:0.7
// @param inset label:"Inset (px)" min:4 max:150 default:26
// @param corner label:"Corner Radius (px)" min:0 max:300 default:84
// @param line_width label:"Rim Width (px)" min:0.5 max:16 default:3
// @param glow_width label:"Glow Width (px)" min:2 max:180 default:32
// @param detail_scale label:"Flow Detail (px)" min:20 max:800 default:210
// @param intensity label:"Intensity" min:0 max:4 default:1.4
// @param grain label:"Dither (1/255)" min:0 max:2 default:0.65
// @param cyan label:"Ice" hint:color default:#3ACBFF
// @param violet label:"Violet" hint:color default:#833EFF
// @param coral label:"Coral" hint:color default:#FF5C8A
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float speed;
    float activity;
    float inset;
    float corner;
    float line_width;
    float glow_width;
    float detail_scale;
    float intensity;
    float grain;
    vec4 cyan;
    vec4 violet;
    vec4 coral;
};

// Integer mixing: no large sin argument whose rounding can form seams.
uint hashBits(uvec2 p) {
    uint h = p.x * 1664525u + p.y * 1013904223u + 2246822519u;
    h ^= h >> 16u;
    h *= 2246822519u;
    h ^= h >> 13u;
    h *= 3266489917u;
    h ^= h >> 16u;
    return h;
}
float lattice(ivec2 p) {
    return float(hashBits(uvec2(p)) >> 8u) * (1.0 / 16777216.0);
}
float smoothNoise(vec2 p) {
    ivec2 i = ivec2(floor(p));
    vec2 f = fract(p);
    // Quintic interpolation has zero first AND second derivatives at cells.
    vec2 w = f*f*f*(f*(f*6.0-15.0)+10.0);
    return mix(mix(lattice(i), lattice(i+ivec2(1,0)), w.x),
               mix(lattice(i+ivec2(0,1)), lattice(i+ivec2(1,1)), w.x), w.y);
}
float filteredNoise(vec2 p, vec2 dx, vec2 dy) {
    float sum = 0.0;
    float amplitude = 0.55;
    mat2 rotateOctave = mat2(0.8, 0.6, -0.6, 0.8);
    for (int i=0; i<5; i++) {
        // Smoothly retire octaves that cannot fit the pixel footprint.
        // Keep a zero-mean signal; do not renormalize disappearing octaves.
        float footprint = max(length(dx), length(dy));
        float keep = 1.0-smoothstep(0.75, 2.0, footprint);
        sum += amplitude * keep * (smoothNoise(p)-0.5);
        p = rotateOctave*p*2.03 + vec2(13.7, 9.2);
        dx = rotateOctave*dx*2.03;
        dy = rotateOctave*dy*2.03;
        amplitude *= 0.48;
    }
    return sum;
}
float roundRect(vec2 p, vec2 halfSize, float radius) {
    vec2 q = abs(p)-halfSize+radius;
    return length(max(q,0.0))+min(max(q.x,q.y),0.0)-radius;
}
void main() {
    vec2 resolution = max(u_resolution, vec2(1.0));
    vec2 p = (v_uv-0.5)*resolution;
    float t = u_time*clamp(speed,0.0,20.0);
    vec2 q = p/max(detail_scale,8.0) + vec2(t*0.17,-t*0.13);
    // Evaluate derivatives in uniform control flow, before any branches.
    float noise = filteredNoise(q,dFdx(q),dFdy(q));
    float act = clamp(activity,0.0,3.0);
    vec2 halfSize = max(resolution*0.5-max(inset,2.0), vec2(2.0));
    float radius = clamp(corner,0.0,min(halfSize.x,halfSize.y));
    float d = roundRect(p,halfSize,radius) + noise*act*3.0;
    float footprint = max(fwidth(d),0.0001);
    float halfLine = max(line_width,0.1)*0.5;
    // Coverage of two boundaries: preserves energy of subpixel thin rims.
    float rim = clamp(0.5-(d-halfLine)/footprint,0.0,1.0)
              - clamp(0.5-(d+halfLine)/footprint,0.0,1.0);
    float width = max(glow_width,0.5)*(0.84+act*(noise+0.15));
    float sigma = sqrt(width*width+footprint*footprint/12.0);
    float halo = exp(-0.5*(d/sigma)*(d/sigma));
    float inner = exp(-0.5*(d/(sigma*0.23))*(d/(sigma*0.23)));
    // Continuous coordinates avoid angular wrap seams at -pi/pi.
    float phase = p.x/resolution.x*4.8+p.y/resolution.y*3.7+t*0.7+noise*3.5;
    vec3 weights = exp(vec3(2.3)*sin(vec3(phase,phase+2.0944,phase+4.1888)));
    vec3 color = (cyan.rgb*weights.x+violet.rgb*weights.y+coral.rgb*weights.z)
               / (weights.x+weights.y+weights.z);
    float pulse = 0.85+0.15*sin(t*1.3+noise*4.0);
    float energy = max(intensity,0.0)*pulse*(rim*1.5+inner*0.27+halo*0.22);
    float opacity = 1.0-exp(-energy);
    vec3 light = mix(color,vec3(1.0),rim*0.4);
    vec4 base = texture(sampler2D(u_in,u_s),v_uv);
    float alpha = opacity+base.a*(1.0-opacity);
    vec3 rgb = (light*opacity+base.rgb*base.a*(1.0-opacity))/max(alpha,0.00001);
    // Sub-LSB, zero-mean triangular dither. Pixel hash is deliberately only
    // for final grain; it is never the coordinate field for the fluid glow.
    ivec2 pixel = ivec2(gl_FragCoord.xy);
    float dither = lattice(pixel)+lattice(pixel+ivec2(719,193))-1.0;
    rgb += dither*(clamp(grain,0.0,4.0)/255.0)*opacity;
    outColor = vec4(max(rgb,vec3(0.0)),alpha);
}
@endpass
