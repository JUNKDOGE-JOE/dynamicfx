@dynamicfx 1
@graph
pass seed: input -> sd
pass core: input -> cr
pass b1a: sd -> t1
pass b1b: t1 -> s1
pass b2a: s1 -> t2
pass b2b: t2 -> s2
pass b3a: s2 -> t3
pass b3b: t3 -> s3
pass finish: cr, s1, s2, s3 -> output
@end
@pass seed
#version 450
// Ink Bleed -- analog chromatic-bleed titling for DynamicFX.
//
// The source's ink is diffused into a three-octave blur pyramid whose radius
// differs per RGB channel (Chromatic Spread), is elongated along a smear
// axis, and is tinted by turbulence-driven color patches BEFORE the blur so
// the colors spread organically. The same turbulence field melts the source
// into the cloud (optionally through a 45-degree halftone screen), erodes it
// away (Dissolve) and warps it (Distortion). On top: a thresholded
// multi-layer glow with an inner/outer tint gradient, film halation, an
// anamorphic flare, ink-drip streaks, echo ghosts, paper fiber, edge-ink
// rings and midtone film grain.
//
// Drive it on the layer itself, or comp-wide on a BLACK comp-sized solid
// with adjustmentLayer enabled above the content and an opaque backdrop at
// the bottom of the stack -- an adjustment layer's render region is bounded
// by the composite below it, and this shader replaces the frame with
// alpha 1, so the carrier solid's own color must be black. Ink masks come
// from value*alpha, so they work over that opaque backdrop.
//
// Angle dials: Shift/Smear/Fiber/Flare read 0 as horizontal, Streak as
// down, Echo as right. Evolution is an angle -- one revolution advances the
// turbulence a full phase; keyframe it, or leave it and use Evolution Speed.
// Bleed Blend selects the cloud composite: 0 behind, 1 screen, 2 add.
//
// @param bleed_amount label:"Bleed Amount (px)" min:0 max:250 default:70
// @param bleed_aspect label:"Bleed Aspect" min:-1 max:1 default:0
// @param chroma_spread label:"Chromatic Spread" min:0 max:1 default:0.4
// @param bleed_intensity label:"Bleed Intensity" min:0 max:3 default:1.2
// @param bleed_blend label:"Bleed Blend (0/1/2)" min:0 max:2 default:1
// @param color_shift label:"Color Shift (px)" min:0 max:40 default:6
// @param shift_radial label:"Radial Shift" hint:bool default:0
// @param smear_strength label:"Smear Strength (%)" min:0 max:300 default:75
// @param src_opacity label:"Source Opacity (%)" min:0 max:100 default:100
// @param src_softness label:"Source Softness (px)" min:0 max:25 default:3
// @param soft_var label:"Softness Variation (%)" min:0 max:100 default:50
// @param streak_len label:"Streak Length (px)" min:0 max:200 default:30
// @param streak_detail label:"Streak Detail (px)" min:2 max:30 default:10
// @param echo_copies label:"Echo Copies" min:0 max:3 default:0
// @param echo_dist label:"Echo Distance (px)" min:0 max:250 default:75
// @param echo_decay label:"Echo Decay (%)" min:0 max:100 default:45
// @param melt_amount label:"Melt Amount (%)" min:0 max:100 default:35
// @param melt_halftone label:"Halftone Melt" hint:bool default:0
// @param halftone_size label:"Halftone Size (px)" min:2 max:24 default:8
// @param dissolve label:"Dissolve (%)" min:0 max:100 default:0
// @param color_var label:"Color Variation (%)" min:0 max:100 default:55
// @param color_boost label:"Color Boost (%)" min:0 max:300 default:100
// @param duotone label:"Duotone Tint" hint:bool default:0
// @param turb_scale label:"Turbulence Scale (px)" min:20 max:800 default:250
// @param noise_contrast label:"Noise Contrast (%)" min:0 max:300 default:100
// @param noise_bright label:"Noise Brightness (%)" min:-100 max:100 default:0
// @param rseed label:"Random Seed" min:0 max:100 default:0
// @param distortion label:"Distortion (px)" min:0 max:50 default:8
// @param evo_speed label:"Evolution Speed" min:0 max:5 default:0.3
// @param flicker label:"Flicker (%)" min:0 max:100 default:8
// @param flicker_speed label:"Flicker Speed (Hz)" min:0 max:20 default:8
// @param grain label:"Grain (%)" min:0 max:100 default:30
// @param grain_anim label:"Animate Grain" hint:bool default:1
// @param fiber label:"Paper Fiber (%)" min:0 max:100 default:0
// @param fiber_scale label:"Fiber Scale (%)" min:10 max:2000 default:300
// @param edge_ink label:"Edge Ink (%)" min:0 max:100 default:12
// @param edge_ink_w label:"Edge Ink Width (%)" min:10 max:300 default:110
// @param glow_intensity label:"Glow Intensity" min:0 max:4 default:1.4
// @param glow_radius label:"Glow Radius (px)" min:0 max:300 default:90
// @param glow_layers label:"Glow Layers" min:1 max:3 default:3
// @param glow_thresh label:"Glow Threshold (%)" min:0 max:100 default:42
// @param tint_gradient label:"Glow Tint Gradient (%)" min:0 max:100 default:60
// @param halation label:"Halation (%)" min:0 max:100 default:25
// @param flare label:"Flare (%)" min:0 max:100 default:15
// @param flare_len label:"Flare Length (px)" min:0 max:500 default:150
// @param exposure label:"Exposure" min:0.2 max:3 default:1
// @param linear_light label:"Linear Light" hint:bool default:1
// @param shift_angle label:"Shift Angle" hint:angle default:0
// @param smear_angle label:"Smear Direction" hint:angle default:0
// @param streak_angle label:"Streak Direction" hint:angle default:0
// @param echo_angle label:"Echo Direction" hint:angle default:0
// @param fiber_angle label:"Fiber Angle" hint:angle default:0
// @param flare_angle label:"Flare Angle" hint:angle default:0
// @param evolution label:"Evolution" hint:angle default:0
// @param shift_center label:"Shift Center"
// @param tint_a label:"Tint A" hint:color default:1,0.373,0.635
// @param tint_b label:"Tint B" hint:color default:0.31,0.847,1
// @param glow_tint label:"Glow Tint" hint:color default:1,1,1
// @param glow_tint_outer label:"Glow Tint Outer" hint:color default:1,0.698,0.369
// @param flare_tint label:"Flare Tint" hint:color default:0.498,0.706,1
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float bleed_amount;
    float bleed_aspect;
    float chroma_spread;
    float bleed_intensity;
    int bleed_blend;
    float color_shift;
    int shift_radial;
    float smear_strength;
    float src_opacity;
    float src_softness;
    float soft_var;
    float streak_len;
    float streak_detail;
    int echo_copies;
    float echo_dist;
    float echo_decay;
    float melt_amount;
    int melt_halftone;
    float halftone_size;
    float dissolve;
    float color_var;
    float color_boost;
    int duotone;
    float turb_scale;
    float noise_contrast;
    float noise_bright;
    float rseed;
    float distortion;
    float evo_speed;
    float flicker;
    float flicker_speed;
    float grain;
    int grain_anim;
    float fiber;
    float fiber_scale;
    float edge_ink;
    float edge_ink_w;
    float glow_intensity;
    float glow_radius;
    int glow_layers;
    float glow_thresh;
    float tint_gradient;
    float halation;
    float flare;
    float flare_len;
    float exposure;
    int linear_light;
    float shift_angle;
    float smear_angle;
    float streak_angle;
    float echo_angle;
    float fiber_angle;
    float flare_angle;
    float evolution;
    vec2 shift_center;
    vec3 tint_a;
    vec3 tint_b;
    vec3 glow_tint;
    vec3 glow_tint_outer;
    vec3 flare_tint;
};

float maxc(vec3 c) { return max(max(c.r, c.g), c.b); }
float h21(vec2 p) {
    vec3 p3 = fract(vec3(p.x, p.y, p.x) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}
float vnoise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    float a = h21(i);
    float b = h21(i + vec2(1.0, 0.0));
    float c = h21(i + vec2(0.0, 1.0));
    float d = h21(i + vec2(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}
float fbm4(vec2 p) {
    return (0.5 * vnoise(p) + 0.25 * vnoise(p * 2.03 + 11.7)
          + 0.125 * vnoise(p * 4.11 + 23.3) + 0.0625 * vnoise(p * 8.07 + 41.9)) / 0.9375;
}

void main() {
    vec2 pxv = 1.0 / u_resolution;
    vec2 uvpx = v_uv * u_resolution;

    // Shared turbulence field. Seed shoves the domain, evolution drifts it;
    // contrast/brightness shape the same map every consumer reads (melt,
    // softness variation, dissolve), which is what makes the docs' "Noise
    // Brightness raises melting/blur/erosion together" fall out for free.
    float tpx = max(turb_scale, 1.0);
    vec2 q = uvpx / tpx;
    vec2 so = vec2(rseed * 1.37, rseed * 0.91);
    float ev = evolution / 360.0 * 3.0 + u_time * evo_speed;
    vec2 ed = vec2(ev * 0.31, -ev * 0.23);
    float N = fbm4(q + so + ed);
    float Nc = clamp((N - 0.5) * (noise_contrast / 100.0) + 0.5
                     + noise_bright / 100.0, 0.0, 1.0);

    // Distortion warp ("through melted glass").
    float wpx = distortion;
    vec2 w = (vec2(vnoise(q * 1.7 + so + ed + 5.2),
                   vnoise(q * 1.7 + so + ed + 9.7)) - 0.5) * 2.0 * wpx * pxv;
    vec2 suv = v_uv + w;

    vec4 src = texture(sampler2D(u_in, u_s), suv);
    float ink = maxc(src.rgb) * clamp(src.a, 0.0, 1.0);

    // Dissolve: a sweep threshold over the field. Dissolved areas seed
    // nothing, so the glow dies there too, matching the docs.
    float dth = dissolve / 100.0 * 1.15 - 0.075;
    float alive = smoothstep(dth, dth + 0.10, Nc);

    // Melt: noise-selected areas hand their material to the cloud, denser
    // than plain ink so melted spots read as thicker bleed.
    float mm = melt_amount / 100.0 * smoothstep(0.25, 0.85, Nc);
    float dens = ink * (0.55 + mm * 2.6) * alive;

    // Color patches, applied BEFORE the blur so the pyramid diffuses actual
    // colored ink instead of tinting a gray cloud afterwards. Boost widens
    // the gate and deepens saturation at once, per the docs' usage note.
    float boost = color_boost / 100.0;
    float Np = fbm4(q * 0.55 + so * 1.7 + ed * 0.8 + 31.4);
    float gate = smoothstep(0.62 - 0.30 * boost, 0.92 - 0.30 * boost, Np);
    vec3 pal = 0.5 + 0.5 * cos(6.2831853 * (fract(Np * 2.3 + rseed * 0.7)
                                            + vec3(0.0, 0.33, 0.67)));
    pal = mix(vec3(1.0), pal, clamp(0.35 + boost * 0.55, 0.0, 1.6));
    vec3 duo = mix(tint_a.rgb, tint_b.rgb, smoothstep(0.35, 0.65, Np));
    vec3 patch = mix(pal, duo, float(duotone));
    float pv = clamp(color_var / 100.0 * gate, 0.0, 1.0);
    vec3 col = mix(src.rgb, patch * max(maxc(src.rgb), 0.02) * 1.15, pv);

    // Linear Light: linearize BEFORE the blur so highlights dominate the
    // diffusion the way real light does -- this is what makes small lights
    // bloom while midtones stay put.
    col = mix(col, col * col, float(linear_light));
    outColor = vec4(col * dens, dens);
}
@endpass
@pass core
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float bleed_amount;
    float bleed_aspect;
    float chroma_spread;
    float bleed_intensity;
    int bleed_blend;
    float color_shift;
    int shift_radial;
    float smear_strength;
    float src_opacity;
    float src_softness;
    float soft_var;
    float streak_len;
    float streak_detail;
    int echo_copies;
    float echo_dist;
    float echo_decay;
    float melt_amount;
    int melt_halftone;
    float halftone_size;
    float dissolve;
    float color_var;
    float color_boost;
    int duotone;
    float turb_scale;
    float noise_contrast;
    float noise_bright;
    float rseed;
    float distortion;
    float evo_speed;
    float flicker;
    float flicker_speed;
    float grain;
    int grain_anim;
    float fiber;
    float fiber_scale;
    float edge_ink;
    float edge_ink_w;
    float glow_intensity;
    float glow_radius;
    int glow_layers;
    float glow_thresh;
    float tint_gradient;
    float halation;
    float flare;
    float flare_len;
    float exposure;
    int linear_light;
    float shift_angle;
    float smear_angle;
    float streak_angle;
    float echo_angle;
    float fiber_angle;
    float flare_angle;
    float evolution;
    vec2 shift_center;
    vec3 tint_a;
    vec3 tint_b;
    vec3 glow_tint;
    vec3 glow_tint_outer;
    vec3 flare_tint;
};

float maxc(vec3 c) { return max(max(c.r, c.g), c.b); }
float h21(vec2 p) {
    vec3 p3 = fract(vec3(p.x, p.y, p.x) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}
float vnoise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    float a = h21(i);
    float b = h21(i + vec2(1.0, 0.0));
    float c = h21(i + vec2(0.0, 1.0));
    float d = h21(i + vec2(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}
float fbm4(vec2 p) {
    return (0.5 * vnoise(p) + 0.25 * vnoise(p * 2.03 + 11.7)
          + 0.125 * vnoise(p * 4.11 + 23.3) + 0.0625 * vnoise(p * 8.07 + 41.9)) / 0.9375;
}

void main() {
    vec2 pxv = 1.0 / u_resolution;
    vec2 uvpx = v_uv * u_resolution;

    // Same field math as the seed pass -- the two must agree or melt would
    // eat the core in one place and thicken the cloud in another.
    float tpx = max(turb_scale, 1.0);
    vec2 q = uvpx / tpx;
    vec2 so = vec2(rseed * 1.37, rseed * 0.91);
    float ev = evolution / 360.0 * 3.0 + u_time * evo_speed;
    vec2 ed = vec2(ev * 0.31, -ev * 0.23);
    float N = fbm4(q + so + ed);
    float Nc = clamp((N - 0.5) * (noise_contrast / 100.0) + 0.5
                     + noise_bright / 100.0, 0.0, 1.0);
    float wpx = distortion;
    vec2 w = (vec2(vnoise(q * 1.7 + so + ed + 5.2),
                   vnoise(q * 1.7 + so + ed + 9.7)) - 0.5) * 2.0 * wpx * pxv;
    vec2 suv = v_uv + w;

    float dth = dissolve / 100.0 * 1.15 - 0.075;
    float alive = smoothstep(dth, dth + 0.10, Nc);
    float mm = melt_amount / 100.0 * smoothstep(0.25, 0.85, Nc);

    // Halftone melt: the eroded material leaves through a 45-degree dot
    // screen, dots growing with the melt mask until they coalesce. Smooth
    // style just fades. The cloud keeps the smooth mask either way -- the
    // pattern lives on the core, which is where it reads in the reference.
    float mcore = mm;
    if (melt_halftone == 1) {
        float cell = max(halftone_size, 1.0);
        vec2 hp = mat2(0.7071, -0.7071, 0.7071, 0.7071) * uvpx / cell;
        float d = length(fract(hp) - 0.5) / 0.7071;
        mcore = smoothstep(d - 0.12, d + 0.12, sqrt(clamp(mm, 0.0, 1.0)) * 1.12);
    }
    float keep = alive * (1.0 - mcore * 0.92);

    // Source Softness with turbulence-driven variation (0 = uniform blur,
    // 1 = the noise decides where the core stays sharp).
    float rad = src_softness * mix(1.0, Nc, soft_var / 100.0);
    vec3 rgb;
    float a;
    if (rad < 0.3) {
        vec4 s = texture(sampler2D(u_in, u_s), suv);
        rgb = s.rgb;
        a = clamp(s.a, 0.0, 1.0);
    } else {
        vec2 P[12] = vec2[](
            vec2(-0.326, -0.406), vec2(-0.840, -0.074), vec2(-0.696, 0.457),
            vec2(-0.203, 0.621), vec2(0.962, -0.195), vec2(0.473, -0.480),
            vec2(0.519, 0.767), vec2(0.185, -0.893), vec2(0.507, 0.064),
            vec2(0.896, 0.412), vec2(-0.322, -0.933), vec2(-0.792, -0.598));
        vec4 acc = texture(sampler2D(u_in, u_s), suv);
        for (int i = 0; i < 12; i++)
            acc += texture(sampler2D(u_in, u_s), suv + P[i] * rad * pxv);
        acc /= 13.0;
        rgb = acc.rgb;
        a = clamp(acc.a, 0.0, 1.0);
    }

    // rgb keeps its real value (a photo must pass through 1:1); alpha is an
    // ink mask for Behind-blend / streaks, from value*alpha per the backdrop
    // rule in ae-adjustment-layer-render-region.
    float ink = maxc(rgb) * a;
    outColor = vec4(rgb * keep, ink * keep);
}
@endpass
@pass b1a
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float bleed_amount;
    float bleed_aspect;
    float chroma_spread;
    float bleed_intensity;
    int bleed_blend;
    float color_shift;
    int shift_radial;
    float smear_strength;
    float src_opacity;
    float src_softness;
    float soft_var;
    float streak_len;
    float streak_detail;
    int echo_copies;
    float echo_dist;
    float echo_decay;
    float melt_amount;
    int melt_halftone;
    float halftone_size;
    float dissolve;
    float color_var;
    float color_boost;
    int duotone;
    float turb_scale;
    float noise_contrast;
    float noise_bright;
    float rseed;
    float distortion;
    float evo_speed;
    float flicker;
    float flicker_speed;
    float grain;
    int grain_anim;
    float fiber;
    float fiber_scale;
    float edge_ink;
    float edge_ink_w;
    float glow_intensity;
    float glow_radius;
    int glow_layers;
    float glow_thresh;
    float tint_gradient;
    float halation;
    float flare;
    float flare_len;
    float exposure;
    int linear_light;
    float shift_angle;
    float smear_angle;
    float streak_angle;
    float echo_angle;
    float fiber_angle;
    float flare_angle;
    float evolution;
    vec2 shift_center;
    vec3 tint_a;
    vec3 tint_b;
    vec3 glow_tint;
    vec3 glow_tint_outer;
    vec3 flare_tint;
};
vec2 dirFromAngle(float deg) {
    float a = radians(deg);
    return vec2(sin(a), -cos(a));
}
// Octave 1, major axis (along the smear direction). Per-channel sigma is the
// Chromatic Spread: red tightens, blue widens, so the cloud fringes into
// spectra as it diffuses. Glow Radius floors the pyramid so the glow
// survives Bleed Amount = 0.
void main() {
    float base = max(bleed_amount, glow_radius * 0.33) * 1.0;
    float asp = bleed_aspect;
    vec2 sca = vec2(1.0 + max(-asp, 0.0) * 2.2, 1.0 + max(asp, 0.0) * 2.2);
    vec2 ax = dirFromAngle(smear_angle + 90.0);
    float sig = base * length(ax * sca);
    sig *= max(1.0, smear_strength / 50.0);
    sig = max(sig, 0.6);
    float kR = 1.0 - chroma_spread * 0.45;
    float kB = 1.0 + chroma_spread * 0.85;
    vec3 sigc = vec3(sig * kR, sig, sig * kB);
    vec2 pxv = 1.0 / u_resolution;
    vec3 acc = vec3(0.0);
    vec3 wsum = vec3(0.0);
    float aacc = 0.0;
    float awsum = 0.0;
    vec3 jp3 = fract(vec3(v_uv.x, v_uv.y, v_uv.x) * vec3(443.897, 441.423, 437.195));
    jp3 += dot(jp3, jp3.yzx + 19.19);
    float jit = fract((jp3.x + jp3.y) * jp3.z) - 0.5;
    for (int i = -15; i <= 15; i++) {
        float x = (float(i) + jit) / 15.0 * (2.5 * sigc.b);
        vec3 wc = exp(-0.5 * x * x / (sigc * sigc));
        float wa = exp(-0.5 * x * x / (sig * sig));
        vec4 s = texture(sampler2D(u_in, u_s), v_uv + ax * x * pxv);
        acc += s.rgb * wc;
        wsum += wc;
        aacc += s.a * wa;
        awsum += wa;
    }
    outColor = vec4(acc / max(wsum, vec3(1e-5)), aacc / max(awsum, 1e-5));
}
@endpass
@pass b1b
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float bleed_amount;
    float bleed_aspect;
    float chroma_spread;
    float bleed_intensity;
    int bleed_blend;
    float color_shift;
    int shift_radial;
    float smear_strength;
    float src_opacity;
    float src_softness;
    float soft_var;
    float streak_len;
    float streak_detail;
    int echo_copies;
    float echo_dist;
    float echo_decay;
    float melt_amount;
    int melt_halftone;
    float halftone_size;
    float dissolve;
    float color_var;
    float color_boost;
    int duotone;
    float turb_scale;
    float noise_contrast;
    float noise_bright;
    float rseed;
    float distortion;
    float evo_speed;
    float flicker;
    float flicker_speed;
    float grain;
    int grain_anim;
    float fiber;
    float fiber_scale;
    float edge_ink;
    float edge_ink_w;
    float glow_intensity;
    float glow_radius;
    int glow_layers;
    float glow_thresh;
    float tint_gradient;
    float halation;
    float flare;
    float flare_len;
    float exposure;
    int linear_light;
    float shift_angle;
    float smear_angle;
    float streak_angle;
    float echo_angle;
    float fiber_angle;
    float flare_angle;
    float evolution;
    vec2 shift_center;
    vec3 tint_a;
    vec3 tint_b;
    vec3 glow_tint;
    vec3 glow_tint_outer;
    vec3 flare_tint;
};
vec2 dirFromAngle(float deg) {
    float a = radians(deg);
    return vec2(sin(a), -cos(a));
}
// Octave 1, minor axis (perpendicular to the smear -- no smear elongation).
void main() {
    float base = max(bleed_amount, glow_radius * 0.33) * 1.0;
    float asp = bleed_aspect;
    vec2 sca = vec2(1.0 + max(-asp, 0.0) * 2.2, 1.0 + max(asp, 0.0) * 2.2);
    vec2 a1 = dirFromAngle(smear_angle + 90.0);
    vec2 ax = vec2(-a1.y, a1.x);
    float sig = base * length(ax * sca);
    sig = max(sig, 0.6);
    float kR = 1.0 - chroma_spread * 0.45;
    float kB = 1.0 + chroma_spread * 0.85;
    vec3 sigc = vec3(sig * kR, sig, sig * kB);
    vec2 pxv = 1.0 / u_resolution;
    vec3 acc = vec3(0.0);
    vec3 wsum = vec3(0.0);
    float aacc = 0.0;
    float awsum = 0.0;
    vec3 jp3 = fract(vec3(v_uv.x, v_uv.y, v_uv.x) * vec3(443.897, 441.423, 437.195));
    jp3 += dot(jp3, jp3.yzx + 19.19);
    float jit = fract((jp3.x + jp3.y) * jp3.z) - 0.5;
    for (int i = -15; i <= 15; i++) {
        float x = (float(i) + jit) / 15.0 * (2.5 * sigc.b);
        vec3 wc = exp(-0.5 * x * x / (sigc * sigc));
        float wa = exp(-0.5 * x * x / (sig * sig));
        vec4 s = texture(sampler2D(u_in, u_s), v_uv + ax * x * pxv);
        acc += s.rgb * wc;
        wsum += wc;
        aacc += s.a * wa;
        awsum += wa;
    }
    outColor = vec4(acc / max(wsum, vec3(1e-5)), aacc / max(awsum, 1e-5));
}
@endpass
@pass b2a
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float bleed_amount;
    float bleed_aspect;
    float chroma_spread;
    float bleed_intensity;
    int bleed_blend;
    float color_shift;
    int shift_radial;
    float smear_strength;
    float src_opacity;
    float src_softness;
    float soft_var;
    float streak_len;
    float streak_detail;
    int echo_copies;
    float echo_dist;
    float echo_decay;
    float melt_amount;
    int melt_halftone;
    float halftone_size;
    float dissolve;
    float color_var;
    float color_boost;
    int duotone;
    float turb_scale;
    float noise_contrast;
    float noise_bright;
    float rseed;
    float distortion;
    float evo_speed;
    float flicker;
    float flicker_speed;
    float grain;
    int grain_anim;
    float fiber;
    float fiber_scale;
    float edge_ink;
    float edge_ink_w;
    float glow_intensity;
    float glow_radius;
    int glow_layers;
    float glow_thresh;
    float tint_gradient;
    float halation;
    float flare;
    float flare_len;
    float exposure;
    int linear_light;
    float shift_angle;
    float smear_angle;
    float streak_angle;
    float echo_angle;
    float fiber_angle;
    float flare_angle;
    float evolution;
    vec2 shift_center;
    vec3 tint_a;
    vec3 tint_b;
    vec3 glow_tint;
    vec3 glow_tint_outer;
    vec3 flare_tint;
};
vec2 dirFromAngle(float deg) {
    float a = radians(deg);
    return vec2(sin(a), -cos(a));
}
// Octave 2 major: re-blurs octave 1, giving ~2x the effective radius.
void main() {
    float base = max(bleed_amount, glow_radius * 0.33) * 1.8;
    float asp = bleed_aspect;
    vec2 sca = vec2(1.0 + max(-asp, 0.0) * 2.2, 1.0 + max(asp, 0.0) * 2.2);
    vec2 ax = dirFromAngle(smear_angle + 90.0);
    float sig = base * length(ax * sca);
    sig *= max(1.0, smear_strength / 50.0);
    sig = max(sig, 0.6);
    float kR = 1.0 - chroma_spread * 0.45;
    float kB = 1.0 + chroma_spread * 0.85;
    vec3 sigc = vec3(sig * kR, sig, sig * kB);
    vec2 pxv = 1.0 / u_resolution;
    vec3 acc = vec3(0.0);
    vec3 wsum = vec3(0.0);
    float aacc = 0.0;
    float awsum = 0.0;
    vec3 jp3 = fract(vec3(v_uv.x, v_uv.y, v_uv.x) * vec3(443.897, 441.423, 437.195));
    jp3 += dot(jp3, jp3.yzx + 19.19);
    float jit = fract((jp3.x + jp3.y) * jp3.z) - 0.5;
    for (int i = -15; i <= 15; i++) {
        float x = (float(i) + jit) / 15.0 * (2.5 * sigc.b);
        vec3 wc = exp(-0.5 * x * x / (sigc * sigc));
        float wa = exp(-0.5 * x * x / (sig * sig));
        vec4 s = texture(sampler2D(u_in, u_s), v_uv + ax * x * pxv);
        acc += s.rgb * wc;
        wsum += wc;
        aacc += s.a * wa;
        awsum += wa;
    }
    outColor = vec4(acc / max(wsum, vec3(1e-5)), aacc / max(awsum, 1e-5));
}
@endpass
@pass b2b
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float bleed_amount;
    float bleed_aspect;
    float chroma_spread;
    float bleed_intensity;
    int bleed_blend;
    float color_shift;
    int shift_radial;
    float smear_strength;
    float src_opacity;
    float src_softness;
    float soft_var;
    float streak_len;
    float streak_detail;
    int echo_copies;
    float echo_dist;
    float echo_decay;
    float melt_amount;
    int melt_halftone;
    float halftone_size;
    float dissolve;
    float color_var;
    float color_boost;
    int duotone;
    float turb_scale;
    float noise_contrast;
    float noise_bright;
    float rseed;
    float distortion;
    float evo_speed;
    float flicker;
    float flicker_speed;
    float grain;
    int grain_anim;
    float fiber;
    float fiber_scale;
    float edge_ink;
    float edge_ink_w;
    float glow_intensity;
    float glow_radius;
    int glow_layers;
    float glow_thresh;
    float tint_gradient;
    float halation;
    float flare;
    float flare_len;
    float exposure;
    int linear_light;
    float shift_angle;
    float smear_angle;
    float streak_angle;
    float echo_angle;
    float fiber_angle;
    float flare_angle;
    float evolution;
    vec2 shift_center;
    vec3 tint_a;
    vec3 tint_b;
    vec3 glow_tint;
    vec3 glow_tint_outer;
    vec3 flare_tint;
};
vec2 dirFromAngle(float deg) {
    float a = radians(deg);
    return vec2(sin(a), -cos(a));
}
// Octave 2 minor.
void main() {
    float base = max(bleed_amount, glow_radius * 0.33) * 1.8;
    float asp = bleed_aspect;
    vec2 sca = vec2(1.0 + max(-asp, 0.0) * 2.2, 1.0 + max(asp, 0.0) * 2.2);
    vec2 a1 = dirFromAngle(smear_angle + 90.0);
    vec2 ax = vec2(-a1.y, a1.x);
    float sig = base * length(ax * sca);
    sig = max(sig, 0.6);
    float kR = 1.0 - chroma_spread * 0.45;
    float kB = 1.0 + chroma_spread * 0.85;
    vec3 sigc = vec3(sig * kR, sig, sig * kB);
    vec2 pxv = 1.0 / u_resolution;
    vec3 acc = vec3(0.0);
    vec3 wsum = vec3(0.0);
    float aacc = 0.0;
    float awsum = 0.0;
    vec3 jp3 = fract(vec3(v_uv.x, v_uv.y, v_uv.x) * vec3(443.897, 441.423, 437.195));
    jp3 += dot(jp3, jp3.yzx + 19.19);
    float jit = fract((jp3.x + jp3.y) * jp3.z) - 0.5;
    for (int i = -15; i <= 15; i++) {
        float x = (float(i) + jit) / 15.0 * (2.5 * sigc.b);
        vec3 wc = exp(-0.5 * x * x / (sigc * sigc));
        float wa = exp(-0.5 * x * x / (sig * sig));
        vec4 s = texture(sampler2D(u_in, u_s), v_uv + ax * x * pxv);
        acc += s.rgb * wc;
        wsum += wc;
        aacc += s.a * wa;
        awsum += wa;
    }
    outColor = vec4(acc / max(wsum, vec3(1e-5)), aacc / max(awsum, 1e-5));
}
@endpass
@pass b3a
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float bleed_amount;
    float bleed_aspect;
    float chroma_spread;
    float bleed_intensity;
    int bleed_blend;
    float color_shift;
    int shift_radial;
    float smear_strength;
    float src_opacity;
    float src_softness;
    float soft_var;
    float streak_len;
    float streak_detail;
    int echo_copies;
    float echo_dist;
    float echo_decay;
    float melt_amount;
    int melt_halftone;
    float halftone_size;
    float dissolve;
    float color_var;
    float color_boost;
    int duotone;
    float turb_scale;
    float noise_contrast;
    float noise_bright;
    float rseed;
    float distortion;
    float evo_speed;
    float flicker;
    float flicker_speed;
    float grain;
    int grain_anim;
    float fiber;
    float fiber_scale;
    float edge_ink;
    float edge_ink_w;
    float glow_intensity;
    float glow_radius;
    int glow_layers;
    float glow_thresh;
    float tint_gradient;
    float halation;
    float flare;
    float flare_len;
    float exposure;
    int linear_light;
    float shift_angle;
    float smear_angle;
    float streak_angle;
    float echo_angle;
    float fiber_angle;
    float flare_angle;
    float evolution;
    vec2 shift_center;
    vec3 tint_a;
    vec3 tint_b;
    vec3 glow_tint;
    vec3 glow_tint_outer;
    vec3 flare_tint;
};
vec2 dirFromAngle(float deg) {
    float a = radians(deg);
    return vec2(sin(a), -cos(a));
}
// Octave 3 major: the big halo, ~4x effective radius.
void main() {
    float base = max(bleed_amount, glow_radius * 0.33) * 3.5;
    float asp = bleed_aspect;
    vec2 sca = vec2(1.0 + max(-asp, 0.0) * 2.2, 1.0 + max(asp, 0.0) * 2.2);
    vec2 ax = dirFromAngle(smear_angle + 90.0);
    float sig = base * length(ax * sca);
    sig *= max(1.0, smear_strength / 50.0);
    sig = max(sig, 0.6);
    float kR = 1.0 - chroma_spread * 0.45;
    float kB = 1.0 + chroma_spread * 0.85;
    vec3 sigc = vec3(sig * kR, sig, sig * kB);
    vec2 pxv = 1.0 / u_resolution;
    vec3 acc = vec3(0.0);
    vec3 wsum = vec3(0.0);
    float aacc = 0.0;
    float awsum = 0.0;
    vec3 jp3 = fract(vec3(v_uv.x, v_uv.y, v_uv.x) * vec3(443.897, 441.423, 437.195));
    jp3 += dot(jp3, jp3.yzx + 19.19);
    float jit = fract((jp3.x + jp3.y) * jp3.z) - 0.5;
    for (int i = -15; i <= 15; i++) {
        float x = (float(i) + jit) / 15.0 * (2.5 * sigc.b);
        vec3 wc = exp(-0.5 * x * x / (sigc * sigc));
        float wa = exp(-0.5 * x * x / (sig * sig));
        vec4 s = texture(sampler2D(u_in, u_s), v_uv + ax * x * pxv);
        acc += s.rgb * wc;
        wsum += wc;
        aacc += s.a * wa;
        awsum += wa;
    }
    outColor = vec4(acc / max(wsum, vec3(1e-5)), aacc / max(awsum, 1e-5));
}
@endpass
@pass b3b
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float bleed_amount;
    float bleed_aspect;
    float chroma_spread;
    float bleed_intensity;
    int bleed_blend;
    float color_shift;
    int shift_radial;
    float smear_strength;
    float src_opacity;
    float src_softness;
    float soft_var;
    float streak_len;
    float streak_detail;
    int echo_copies;
    float echo_dist;
    float echo_decay;
    float melt_amount;
    int melt_halftone;
    float halftone_size;
    float dissolve;
    float color_var;
    float color_boost;
    int duotone;
    float turb_scale;
    float noise_contrast;
    float noise_bright;
    float rseed;
    float distortion;
    float evo_speed;
    float flicker;
    float flicker_speed;
    float grain;
    int grain_anim;
    float fiber;
    float fiber_scale;
    float edge_ink;
    float edge_ink_w;
    float glow_intensity;
    float glow_radius;
    int glow_layers;
    float glow_thresh;
    float tint_gradient;
    float halation;
    float flare;
    float flare_len;
    float exposure;
    int linear_light;
    float shift_angle;
    float smear_angle;
    float streak_angle;
    float echo_angle;
    float fiber_angle;
    float flare_angle;
    float evolution;
    vec2 shift_center;
    vec3 tint_a;
    vec3 tint_b;
    vec3 glow_tint;
    vec3 glow_tint_outer;
    vec3 flare_tint;
};
vec2 dirFromAngle(float deg) {
    float a = radians(deg);
    return vec2(sin(a), -cos(a));
}
// Octave 3 minor.
void main() {
    float base = max(bleed_amount, glow_radius * 0.33) * 3.5;
    float asp = bleed_aspect;
    vec2 sca = vec2(1.0 + max(-asp, 0.0) * 2.2, 1.0 + max(asp, 0.0) * 2.2);
    vec2 a1 = dirFromAngle(smear_angle + 90.0);
    vec2 ax = vec2(-a1.y, a1.x);
    float sig = base * length(ax * sca);
    sig = max(sig, 0.6);
    float kR = 1.0 - chroma_spread * 0.45;
    float kB = 1.0 + chroma_spread * 0.85;
    vec3 sigc = vec3(sig * kR, sig, sig * kB);
    vec2 pxv = 1.0 / u_resolution;
    vec3 acc = vec3(0.0);
    vec3 wsum = vec3(0.0);
    float aacc = 0.0;
    float awsum = 0.0;
    vec3 jp3 = fract(vec3(v_uv.x, v_uv.y, v_uv.x) * vec3(443.897, 441.423, 437.195));
    jp3 += dot(jp3, jp3.yzx + 19.19);
    float jit = fract((jp3.x + jp3.y) * jp3.z) - 0.5;
    for (int i = -15; i <= 15; i++) {
        float x = (float(i) + jit) / 15.0 * (2.5 * sigc.b);
        vec3 wc = exp(-0.5 * x * x / (sigc * sigc));
        float wa = exp(-0.5 * x * x / (sig * sig));
        vec4 s = texture(sampler2D(u_in, u_s), v_uv + ax * x * pxv);
        acc += s.rgb * wc;
        wsum += wc;
        aacc += s.a * wa;
        awsum += wa;
    }
    outColor = vec4(acc / max(wsum, vec3(1e-5)), aacc / max(awsum, 1e-5));
}
@endpass
@pass finish
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_core;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float bleed_amount;
    float bleed_aspect;
    float chroma_spread;
    float bleed_intensity;
    int bleed_blend;
    float color_shift;
    int shift_radial;
    float smear_strength;
    float src_opacity;
    float src_softness;
    float soft_var;
    float streak_len;
    float streak_detail;
    int echo_copies;
    float echo_dist;
    float echo_decay;
    float melt_amount;
    int melt_halftone;
    float halftone_size;
    float dissolve;
    float color_var;
    float color_boost;
    int duotone;
    float turb_scale;
    float noise_contrast;
    float noise_bright;
    float rseed;
    float distortion;
    float evo_speed;
    float flicker;
    float flicker_speed;
    float grain;
    int grain_anim;
    float fiber;
    float fiber_scale;
    float edge_ink;
    float edge_ink_w;
    float glow_intensity;
    float glow_radius;
    int glow_layers;
    float glow_thresh;
    float tint_gradient;
    float halation;
    float flare;
    float flare_len;
    float exposure;
    int linear_light;
    float shift_angle;
    float smear_angle;
    float streak_angle;
    float echo_angle;
    float fiber_angle;
    float flare_angle;
    float evolution;
    vec2 shift_center;
    vec3 tint_a;
    vec3 tint_b;
    vec3 glow_tint;
    vec3 glow_tint_outer;
    vec3 flare_tint;
};
layout(set = 0, binding = 3) uniform texture2D u_s1;
layout(set = 0, binding = 4) uniform texture2D u_s2;
layout(set = 0, binding = 5) uniform texture2D u_s3;

float maxc(vec3 c) { return max(max(c.r, c.g), c.b); }
float h21(vec2 p) {
    vec3 p3 = fract(vec3(p.x, p.y, p.x) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}
float vnoise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    float a = h21(i);
    float b = h21(i + vec2(1.0, 0.0));
    float c = h21(i + vec2(0.0, 1.0));
    float d = h21(i + vec2(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}
vec2 dirFromAngle(float deg) {
    float a = radians(deg);
    return vec2(sin(a), -cos(a));
}
float lum(vec3 c) { return dot(c, vec3(0.299, 0.587, 0.114)); }

void main() {
    vec2 pxv = 1.0 / u_resolution;
    vec2 uvpx = v_uv * u_resolution;
    float LL = float(linear_light);

    // --- color shift (directional or radial-from-center) --------------------
    float spx = color_shift;
    vec2 offD = dirFromAngle(shift_angle + 90.0) * spx * pxv;
    vec2 offR = (v_uv - shift_center) * 2.0 * spx * pxv;
    vec2 off = mix(offD, offR, float(shift_radial));
    vec2 offW = off * 1.4;

    // R sampled at +off, B at -off -- the docs' "R and B shift opposite ways".
    vec4 coreG = texture(sampler2D(u_core, u_s), v_uv);
    vec4 core = vec4(texture(sampler2D(u_core, u_s), v_uv + off).r,
                     coreG.g,
                     texture(sampler2D(u_core, u_s), v_uv - off).b,
                     coreG.a);
    vec4 s1G = texture(sampler2D(u_s1, u_s), v_uv);
    vec4 s1 = vec4(texture(sampler2D(u_s1, u_s), v_uv + offW).r,
                   s1G.g,
                   texture(sampler2D(u_s1, u_s), v_uv - offW).b,
                   s1G.a);
    vec4 s2G = texture(sampler2D(u_s2, u_s), v_uv);
    vec4 s2 = vec4(texture(sampler2D(u_s2, u_s), v_uv + offW).r,
                   s2G.g,
                   texture(sampler2D(u_s2, u_s), v_uv - offW).b,
                   s2G.a);
    vec4 s3G = texture(sampler2D(u_s3, u_s), v_uv);
    vec4 s3 = vec4(texture(sampler2D(u_s3, u_s), v_uv + offW).r,
                   s3G.g,
                   texture(sampler2D(u_s3, u_s), v_uv - offW).b,
                   s3G.a);

    // --- echo ghosts ---------------------------------------------------------
    vec3 echoCore = vec3(0.0);
    vec3 echoCloud = vec3(0.0);
    float echoInk = 0.0;
    if (echo_copies >= 1) {
        vec2 edir = dirFromAngle(echo_angle + 90.0);
        float edist = echo_dist;
        for (int e = 1; e <= 3; e++) {
            if (e <= echo_copies) {
                vec2 eo = edir * edist * float(e) * pxv;
                vec4 ec = texture(sampler2D(u_core, u_s), v_uv - eo);
                vec4 e1 = texture(sampler2D(u_s1, u_s), v_uv - eo);
                float amp = pow(clamp(1.0 - echo_decay / 100.0, 0.0, 1.0) + 1e-5, float(e));
                echoCore += ec.rgb * amp;
                echoCloud += e1.rgb * 0.6 * amp;
                echoInk = max(echoInk, ec.a * amp);
            }
        }
    }

    // --- ink-drip streaks ----------------------------------------------------
    // March against the drip direction collecting core ink; each perpendicular
    // band gets its own reach and a thin line profile so it reads as drips,
    // not as another directional blur.
    vec3 streakC = vec3(0.0);
    float slpx = streak_len;
    if (slpx > 1.0) {
        vec2 sdir = dirFromAngle(streak_angle + 180.0);
        vec2 sperp = vec2(-sdir.y, sdir.x);
        float band = max(streak_detail, 1.0);
        float pc = dot(uvpx, sperp);
        float hh = h21(vec2(floor(pc / band), rseed * 0.17 + 3.7));
        float reach = slpx * (0.25 + 0.75 * hh);
        float lineP = 1.0 - smoothstep(0.12, 0.5, abs(fract(pc / band) - 0.5));
        vec3 acc = vec3(0.0);
        for (int i = 1; i <= 16; i++) {
            float d = float(i) / 16.0 * slpx;
            if (d <= reach) {
                vec4 cs = texture(sampler2D(u_core, u_s), v_uv - sdir * d * pxv);
                float mass = smoothstep(0.45, 0.85, cs.a);
                // pow() base must stay strictly positive: on this DX12/naga
                // stack a base of exactly 0 can yield NaN, and 0*NaN poisons
                // the whole accumulator column (seen as full-height black
                // seams before this clamp).
                float fall = max(1.0 - d / max(reach, 1.0), 0.0) + 1e-5;
                acc += mix(cs.rgb, cs.rgb * cs.rgb, LL) * mass * pow(fall, 1.6);
            }
        }
        streakC = acc / 16.0 * 1.6 * lineP;
    }

    // --- bleed cloud ---------------------------------------------------------
    vec3 cloud = (s1.rgb * 0.50 + s2.rgb * 0.32 + s3.rgb * 0.18)
               * bleed_intensity;
    cloud *= smoothstep(0.0, 2.0, bleed_amount);
    cloud += streakC + echoCloud;

    // paper fiber: noise stretched hard along the fiber axis, multiplying the
    // cloud like ink soaking along paper grain.
    if (fiber > 0.1) {
        vec2 fd = dirFromAngle(fiber_angle + 90.0);
        float fsc = fiber_scale / 100.0;
        vec2 fp = vec2(dot(uvpx, fd) / (24.0 * fsc),
                       dot(uvpx, vec2(-fd.y, fd.x)) / (2.2 * fsc));
        float fn = vnoise(fp + rseed * 1.37);
        cloud *= 1.0 + fiber / 100.0 * (fn - 0.5) * 1.7;
    }

    // edge ink: a coffee-ring of pigment where the cloud density falls off.
    if (edge_ink > 0.1) {
        float dens = clamp(s1.a * 0.6 + s2.a * 0.4, 0.0, 1.0);
        float srng = 0.02 + edge_ink_w / 100.0 * 0.077;
        float ring = exp(-(dens - 0.30) * (dens - 0.30) / (2.0 * srng * srng));
        cloud *= 1.0 - edge_ink / 100.0 * ring * 0.65;
    }

    // --- flicker (bleed + glow only, like a dying neon tube) -----------------
    float flick = 1.0 + (flicker / 100.0 * 0.9)
        * (vnoise(vec2(u_time * flicker_speed, rseed * 0.37)) * 2.0 - 1.0);

    // --- composite core + cloud ---------------------------------------------
    // Linear Light works in gamma-2 space: cheap, and enough to make Screen /
    // Add stop clipping like paint.
    float srcW = src_opacity / 100.0;
    vec3 coreC = mix(core.rgb, core.rgb * core.rgb, LL) * srcW
               + mix(echoCore, echoCore * echoCore, LL) * srcW;
    float ink = clamp(max(core.a * srcW, echoInk * 0.7), 0.0, 1.0);
    // s1..s3 already carry linearized ink (seed pass linearizes before the
    // blur), so the cloud is used as-is; only the core needed lifting.
    vec3 cloudC = cloud * flick;

    float wB = float(bleed_blend == 0);
    float wS = float(bleed_blend == 1);
    float wA = float(bleed_blend == 2);
    vec3 col = wB * (coreC + cloudC * (1.0 - ink))
             + wS * (coreC + cloudC * (1.0 - clamp(coreC, 0.0, 1.0)))
             + wA * (coreC + cloudC);

    // --- thresholded multi-layer glow ---------------------------------------
    // The pyramid doubles as the bloom stack: Radius crossfades which octave
    // dominates, Layers gates how many join, tint runs inner->outer.
    if (glow_intensity > 0.005) {
        float r01 = clamp(glow_radius / 300.0, 0.0, 1.0);
        vec3 rw = vec3((1.0 - r01) * (1.0 - r01),
                       2.0 * r01 * (1.0 - r01),
                       r01 * r01);
        vec3 lay = vec3(1.0, glow_layers >= 2 ? 1.0 : 0.0,
                        glow_layers >= 3 ? 1.0 : 0.0);
        vec3 w = rw * lay;
        w /= max(w.x + w.y + w.z, 1e-4);
        vec3 g = vec3(0.0);
        float gt = glow_thresh / 100.0;
        float thrL = mix(gt, gt * gt, LL);
        float knee = mix(0.22, 0.10, LL);
        float t1 = smoothstep(thrL, thrL + knee, lum(s1.rgb));
        float t2 = smoothstep(thrL, thrL + knee, lum(s2.rgb));
        float t3 = smoothstep(thrL, thrL + knee, lum(s3.rgb));
        g += s1.rgb * t1 * mix(glow_tint.rgb, glow_tint_outer.rgb, 0.0) * w.x;
        g += s2.rgb * t2 * mix(glow_tint.rgb, glow_tint_outer.rgb, 0.5 * tint_gradient / 100.0) * w.y;
        g += s3.rgb * t3 * mix(glow_tint.rgb, glow_tint_outer.rgb, tint_gradient / 100.0) * w.z;
        g *= glow_intensity;
        col += g * flick;
    }

    // --- film halation (independent of glow, per docs) -----------------------
    if (halation > 0.1) {
        vec3 h = max(s2.rgb * 0.4 + s3.rgb * 0.6 - mix(0.12, 0.05, LL), 0.0)
               * vec3(1.0, 0.42, 0.22);
        col += h * (halation / 100.0) * 2.0 * flick;
    }

    // --- anamorphic flare ----------------------------------------------------
    if (flare > 0.1) {
        vec2 fdir = dirFromAngle(flare_angle + 90.0);
        float flpx = flare_len;
        vec3 fa = vec3(0.0);
        float fw = 0.0;
        for (int i = -10; i <= 10; i++) {
            float t = float(i) / 10.0;
            vec3 sv = texture(sampler2D(u_s1, u_s), v_uv + fdir * t * flpx * pxv).rgb;
            float gtF = glow_thresh / 100.0;
            float thrF = mix(gtF, gtF * gtF, LL) * 0.6;
            float b = smoothstep(thrF, thrF + 0.25, lum(sv));
            float wt = pow(max(1.0 - abs(t), 0.0) + 1e-5, 2.2);
            fa += sv * b * wt;
            fw += wt;
        }
        vec3 fc = fa / max(fw, 1e-3) * flare_tint.rgb * (flare / 100.0 * 2.5);
        col += fc * flick;
    }

    // --- output: exposure, de-linearize, rolloff, grain ----------------------
    col *= exposure;
    col = mix(col, sqrt(max(col, 0.0)), LL);
    // Shoulder-only rolloff: identity below 0.75 so the photo keeps its
    // levels, highlights ease into 1 instead of clipping flat.
    vec3 hi = smoothstep(vec3(0.75), vec3(0.9), col);
    vec3 shoulder = 0.75 + 0.25 * (1.0 - exp(-max(col - 0.75, vec3(0.0)) / 0.25));
    col = mix(col, shoulder, hi);

    // film grain living in the midtones, quiet in blacks and whites.
    float gj = grain_anim == 1 ? u_frame : 0.0;
    float gn = h21(uvpx + vec2(gj * 7.3, gj * 3.1));
    float l = clamp(maxc(col), 0.0, 1.0);
    col *= 1.0 + (gn - 0.5) * (grain / 100.0) * 0.4 * (l * (1.0 - l) * 4.0);
    col += (gn - 0.5) * (1.5 / 255.0);

    // Full-frame replace: alpha 1 always. Where the effect output has alpha
    // below 1, AE shows the carrier solid's own color through -- with a=1 the
    // solid color can never leak. Dissolve therefore erodes to the backdrop
    // (black in the standard rig), matching the demo's on-black look.
    outColor = vec4(max(col, vec3(0.0)), 1.0);
}
@endpass
