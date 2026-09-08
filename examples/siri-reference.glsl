@dynamicfx 1
@graph
pass blurh: input -> bh
pass blurv: bh -> bgblur
pass emit: input -> em
pass bl1h: em -> t1
pass bl1v: t1 -> wb1
pass bl2h: wb1 -> t2
pass bl2v: t2, wb1 -> glow
pass glass: input, bgblur, em, glow -> output
@end
@pass blurh
#version 450
// @param bg_blur label:"BG Blur (px)" min:0 max:60 default:14
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float bg_blur;
};
void main() {
    float stp = bg_blur / 12.0;
    if (stp < 0.02) { outColor = texture(sampler2D(u_in, u_s), v_uv); return; }
    vec4 acc = vec4(0.0);
    float wsum = 0.0;
    for (int i = -12; i <= 12; i++) {
        float fi = float(i);
        float w = exp(-fi * fi / 60.0);
        vec2 off = vec2(fi * stp / u_resolution.x, 0.0);
        acc += texture(sampler2D(u_in, u_s), v_uv + off) * w;
        wsum += w;
    }
    outColor = acc / wsum;
}
@endpass
@pass blurv
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float bg_blur;
};
void main() {
    float stp = bg_blur / 12.0;
    if (stp < 0.02) { outColor = texture(sampler2D(u_in, u_s), v_uv); return; }
    vec4 acc = vec4(0.0);
    float wsum = 0.0;
    for (int i = -12; i <= 12; i++) {
        float fi = float(i);
        float w = exp(-fi * fi / 60.0);
        vec2 off = vec2(0.0, fi * stp / u_resolution.y);
        acc += texture(sampler2D(u_in, u_s), v_uv + off) * w;
        wsum += w;
    }
    outColor = acc / wsum;
}
@endpass
@pass emit
#version 450
// @param phase_lock label:"Phase Settling" min:0 max:3 default:0.65
// @param breath_amt label:"Glass Breath (px)" min:0 max:20 default:3.2
// @param breath_rate label:"Glass Breath Rate" min:0.1 max:3 default:0.5
// @param sheet_separation label:"Sheet Separation" min:0 max:2 default:1
// @param motion_offset label:"Motion Time Offset" min:-5 max:5 default:0
// @param capsule_reflect label:"Capsule Reflection" min:0 max:2 default:0.35
// @param center label:"Glass Center"
// @param activation label:"Activation" min:0 max:1 default:1
// @param overshoot label:"Pop Overshoot" min:0 max:3 default:0.35
// @param half_w label:"Glass Half W" min:0.05 max:0.45 default:0.1952
// @param half_h label:"Glass Half H" min:0.05 max:0.45 default:0.1398
// @param roundness label:"Roundness" min:0.2 max:1 default:0.86
// @param pill_center label:"Pill Center"
// @param pill_half_w label:"Pill Half W (px)" min:10 max:400 default:180
// @param pill_half_h label:"Pill Half H (px)" min:5 max:200 default:52
// @param band_y label:"Wave Y" min:-1 max:1 default:0.14
// @param rf_height label:"Ribbon Spread" min:0 max:0.8 default:.22
// @param rf_flow label:"Ribbon Flow" min:-12 max:12 default:4.18879
// @param rf_phase label:"Ribbon Phase" min:-10 max:10 default:1.466
// @param rf_twist label:"Ribbon Twist" min:0 max:4 default:1.5
// @param rf_sway label:"Ribbon Sway" min:0 max:0.5 default:0
// @param rf_width label:"Ribbon Tail" min:0.01 max:0.8 default:.14
// @param rf_edge label:"Ribbon Edge (px)" min:0.1 max:20 default:1.6
// @param rf_envelope label:"Ribbon Extent" min:0.1 max:1.5 default:0.95
// @param rf_gain label:"Ribbon Radiance" min:0 max:5 default:0.72
// @param rf_fold label:"Fold Light" min:0 max:3 default:0
// @param rf_transmission label:"Ribbon Transmission" min:0 max:1 default:0.7
// @param rf_sheen label:"Ribbon Sheen" min:0 max:2 default:0.55
// @param rf_blue label:"Ribbon Blue" hint:color default:0.025,0.14,1
// @param rf_cyan label:"Ribbon Cyan" hint:color default:0.01,0.93,1
// @param rf_yellow label:"Ribbon Yellow" hint:color default:1,0.95,0.03
// @param rf_red label:"Ribbon Red" hint:color default:1,0.035,0.018
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    vec2 center;
    float activation;
    float overshoot;
    float breath_amt;
    float breath_rate;
    float half_w;
    float half_h;
    float roundness;
    vec2 pill_center;
    float pill_half_w;
    float pill_half_h;
    float band_y;
    float zoom;
    float refract_px;
    float refract_band;
    float rim_peak;
    float aberration;
    float wave_refract;
    float wave_disp;
    float wave_amp;
    float phase_lock;
    float sheet_separation;
    float motion_offset;
    float rf_height;
    float rf_flow;
    float rf_phase;
    float rf_twist;
    float rf_sway;
    float rf_width;
    float rf_edge;
    float rf_envelope;
    float rf_gain;
    float rf_fold;
    float rf_transmission;
    float rf_sheen;
    vec3 rf_blue;
    vec3 rf_cyan;
    vec3 rf_yellow;
    vec3 rf_red;
};
float easeOutBack(float t, float ovr) {
    float c1 = 1.70158 * ovr;
    float c3 = c1 + 1.0;
    float u = t - 1.0;
    return 1.0 + c3 * u * u * u + c1 * u * u;
}
vec3 sdSuperG(vec2 c, vec2 hs, float n) {
    vec2 q = c / hs;
    vec2 a = max(abs(q), vec2(1e-6));
    float s = pow(a.x, n) + pow(a.y, n);
    float f = pow(s, 1.0 / n) - 1.0;
    float k = pow(s, 1.0 / n - 1.0);
    vec2 g = k * sign(q) * vec2(pow(a.x, n - 1.0), pow(a.y, n - 1.0)) / hs;
    float gl = max(length(g), 1e-6);
    float mn = min(hs.x, hs.y);
    return vec3(clamp(f / gl, -mn, mn * 4.0), g / gl);
}
vec3 sdRoundRectG(vec2 c, vec2 hs, float r) {
    vec2 cc = abs(c) - (hs - vec2(r));
    float outside = length(max(cc, vec2(0.0))) - r;
    float inside = min(max(cc.x, cc.y), 0.0);
    vec2 g;
    if (cc.x >= 0.0 || cc.y >= 0.0) {
        g = sign(c) * normalize(max(cc, vec2(0.0)) + 1e-6);
    } else {
        float gx = step(cc.y, cc.x);
        g = sign(c) * vec2(gx, 1.0 - gx);
    }
    return vec3(outside + inside, g);
}
// glass outline: the Dynamic Island capsule at activation 0, the bulbous superellipse when open
vec3 glassSDF(vec2 c, vec2 hs, float n, float morph) {
    vec3 cap = sdRoundRectG(c, hs, min(hs.x, hs.y) * 0.98);
    vec3 sq = sdSuperG(c, hs, n);
    float d = mix(cap.x, sq.x, morph);
    vec2 g = normalize(mix(cap.yz, sq.yz, morph) + vec2(1e-5));
    return vec3(d, g);
}
float rimDisp(float d, float B, float P, float u0) {
    float q = clamp(1.0 - d / max(B, 1.0), 0.0, 1.0);
    return P * pow(q, max(1.0 / max(u0, 0.03) - 1.0, 1.0));
}
// Each sheet keeps its identity while projected width and phase evolve continuously.
float sheetProfile(float d, float direction, float tail, float edge) {
    float above = mix(edge, tail, smoothstep(-0.3, 0.3, -direction));
    float below = mix(edge, tail, smoothstep(-0.3, 0.3, direction));
    float sigma = mix(above, below, smoothstep(-edge, edge, d));
    return exp(-0.5 * d * d / max(sigma * sigma, 0.01));
}
vec3 sheetAt(vec2 fc, vec2 cen, float hw, float hh, float morph, float t, float footprint) {
    float x = (fc.x - cen.x) / max(hw * rf_envelope, 1.0);
    float envelope = pow(max(1.0 - x * x, 0.0), 1.25);
    float clock = (t + motion_offset) * rf_flow;
    float age = max(t + motion_offset, 0.0);
    float convergence = exp(-phase_lock * age);
    float phaseGap = mix(0.62, 0.72, convergence) + 0.18 * sin(clock * 0.23);
    float warmPhaseGap = mix(0.62, 0.72, convergence) + 0.18 * sin(clock * 0.19 + 0.9);
    float phase = rf_phase - clock + rf_twist * x
                + 0.16 * sin(0.37 * clock + 1.1 * x);
    float base = cen.y + hh * (band_y + rf_sway * sin(1.7 * x - 0.15 * clock) * envelope);
    float breathing = 0.86 + 0.10 * cos(0.48 * clock) + 0.04 * cos(0.91 * clock);
    float spread = hh * rf_height * envelope * sheet_separation
                 * (0.78 + 0.22 * convergence) * breathing;
    float edge = sqrt(rf_edge * rf_edge + footprint * footprint / 12.0);
    vec3 radiance = vec3(0.0);
    for (int i = 0; i < 4; i++) {
        float offset = i == 0 ? 0.0 : (i == 1 ? phaseGap : (i == 2 ? 3.141592654 + warmPhaseGap : 3.141592654));
        float angle = phase + offset;
        float orientation = cos(angle);
        float height = base + spread * orientation;
        float projected = 0.42 + 0.58 * (sqrt(orientation * orientation + 0.01) - 0.1) / 0.905;
        float colorThickness = i == 0 ? 1.08 : (i == 1 ? 0.62 : (i == 2 ? 0.62 : 1.15));
        float tail = max(edge, hh * rf_width * envelope * projected * breathing * colorThickness);
        float profile = sheetProfile(fc.y - height, -orientation, tail, edge);
        vec3 tint = i == 0 ? rf_blue : (i == 1 ? rf_cyan : (i == 2 ? rf_yellow : rf_red));
        float facing = 0.76 + 0.24 * sin(angle + 0.4);
        float distanceToEdge = fc.y - height;
        float edgeLight = exp(-0.5 * pow(distanceToEdge / max(edge * 1.5 + tail * 0.08, 1.0), 2.0));
        float surfaceRoll = exp(-0.5 * pow(distanceToEdge / max(tail * 0.38, edge), 2.0));
        float halo = exp(-0.5 * pow(distanceToEdge / max(tail * 1.35, edge), 2.0));
        float colorWeight = i == 0 ? 1.05 : (i == 1 ? 1.5 : (i == 2 ? 0.9 : 1.0));
        float body = profile * mix(1.0, 0.5 + 0.5 * surfaceRoll, rf_transmission);
        float neutralSheen = i == 1 || i == 2 ? 0.85 : 0.15;
        vec3 edgeTint = mix(tint, vec3(1.0), neutralSheen * rf_transmission);
        float specular = rf_sheen * edgeLight * (0.65 + 0.35 * pow(sin(angle), 2.0)) * exp(-1.5 * x * x);
        radiance += colorWeight * (tint * (body * facing + 0.12 * halo) + edgeTint * specular);
    }
    float fold = exp(-pow((fc.y - base) / max(edge * 1.4, 1.0), 2.0)) * exp(-3.0 * x * x);
    return (radiance + vec3(rf_fold * fold)) * envelope * rf_gain;
}

void main() {
    vec2 R = u_resolution;
    vec2 fc = v_uv * R;
    if (activation < 0.003) { outColor = vec4(0.0, 0.0, 0.0, 1.0); return; }
    float ag = easeOutBack(clamp(activation, 0.0, 1.0), overshoot);
    ag -= 0.12 * sin(3.14159265359 * clamp(activation / 0.25, 0.0, 1.0));
    float hwFull = half_w * R.x;
    float hhFull = half_h * R.x;
    float hw = max(mix(pill_half_w, hwFull, pow(max(ag, 0.0), 1.2)), 1.0);
    float hh = max(mix(pill_half_h, hhFull, ag), 1.0);
    float pulseCos = cos(6.283185307 * breath_rate * (u_time - 0.15));
    float pulse = 0.5 + 0.5 * pulseCos;
    float glassBreath = breath_amt * pulse * smoothstep(0.25, 0.8, activation);
    hw += glassBreath * 0.18;
    hh += glassBreath;
    float s = hh / max(hhFull, 1.0);
    float morph = smoothstep(0.0, 0.7, clamp(ag, 0.0, 1.0));
    vec2 cen = mix(pill_center * R, vec2(center.x * R.x, center.y * R.y), ag);
    cen.y -= glassBreath + pill_half_h * 0.385 * ag * (1.0 - ag);
    vec2 c = fc - cen;
    vec3 un = glassSDF(c, vec2(hw, hh), mix(5.0, 2.0, roundness), morph);
    float sd = un.x;
    float outMask = 1.0 - smoothstep(-30.0, 40.0, sd);

    float t = u_time;
    // the lens of the glass pass: rim bump per channel, zoom about the centre, chromatic shift
    // at the rim; the mapping fades to identity just outside the outline
    float d = max(-sd, 0.0);
    float B = max(refract_band * s * (0.82 + 0.18 * abs(un.y)), 1.0);
    float dispG = rimDisp(d, B, refract_px * s * smoothstep(0.15, 0.8, activation), rim_peak);
    float dispR = dispG * (1.0 + aberration);
    float dispB = dispG * (1.0 - aberration);
    vec2 cuv = cen / R;
    vec2 gpx = un.yz / R;
    float wm = wave_refract;
    float zw = 1.0 + (zoom - 1.0) * wm;
    float edgeZone = 1.0 - smoothstep(0.0, B, d);
    float wd = wave_disp * s * edgeZone;
    float lens = 1.0 - smoothstep(0.0, 1.5, sd);
    vec2 fcR = mix(v_uv, cuv + ((v_uv - gpx * (dispR * wm + wd)) - cuv) / zw, lens) * R;
    vec2 fcG = mix(v_uv, cuv + ((v_uv - gpx * (dispG * wm)) - cuv) / zw, lens) * R;
    vec2 fcB = mix(v_uv, cuv + ((v_uv - gpx * (dispB * wm - wd)) - cuv) / zw, lens) * R;
    // Evaluate the lens Jacobian before pixel-dependent exits.
    vec2 dxR = dFdx(fcR), dyR = dFdy(fcR);
    vec2 dxG = dFdx(fcG), dyG = dFdy(fcG);
    vec2 dxB = dFdx(fcB), dyB = dFdy(fcB);
    float footprint = max(max(length(dxG), length(dyG)), 1e-4);
    if (outMask <= 0.0) { outColor = vec4(0.0, 0.0, 0.0, 1.0); return; }
    vec3 acc = vec3(0.0);
    for (int k = 0; k < 4; k++) {
        vec2 j = (vec2(float(k & 1), float(k >> 1)) - 0.5) * 0.5;
        vec3 sampleLight = sheetAt(fcG + dxG * j.x + dyG * j.y, cen, hw, hh, morph, t, footprint);
        if (abs(dispG * wm * aberration) + abs(wd) > 0.05) {
            sampleLight.r = sheetAt(fcR + dxR * j.x + dyR * j.y, cen, hw, hh, morph, t, footprint).r;
            sampleLight.b = sheetAt(fcB + dxB * j.x + dyB * j.y, cen, hw, hh, morph, t, footprint).b;
        }
        acc += sampleLight;
    }
    vec3 col = acc * 0.25 * outMask;
    outColor = vec4(max(col * 0.25, vec3(0.0)), 1.0);
}
@endpass
@pass bl1h
#version 450
// @param bloom1_px label:"Glow1 Radius" min:0 max:40 default:8
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float bloom1_px;
};
void main() {
    float stp = bloom1_px / 12.0;
    if (stp < 0.02) { outColor = texture(sampler2D(u_in, u_s), v_uv); return; }
    vec4 acc = vec4(0.0);
    float wsum = 0.0;
    for (int i = -12; i <= 12; i++) {
        float fi = float(i);
        float w = exp(-fi * fi / 60.0);
        vec2 off = vec2(fi * stp / u_resolution.x, 0.0);
        acc += texture(sampler2D(u_in, u_s), v_uv + off) * w;
        wsum += w;
    }
    outColor = acc / wsum;
}
@endpass
@pass bl1v
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float bloom1_px;
};
void main() {
    float stp = bloom1_px / 12.0;
    if (stp < 0.02) { outColor = texture(sampler2D(u_in, u_s), v_uv); return; }
    vec4 acc = vec4(0.0);
    float wsum = 0.0;
    for (int i = -12; i <= 12; i++) {
        float fi = float(i);
        float w = exp(-fi * fi / 60.0);
        vec2 off = vec2(0.0, fi * stp / u_resolution.y);
        acc += texture(sampler2D(u_in, u_s), v_uv + off) * w;
        wsum += w;
    }
    outColor = acc / wsum;
}
@endpass
@pass bl2h
#version 450
// @param bloom2_px label:"Glow2 Radius" min:0 max:80 default:64
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float bloom2_px;
};
void main() {
    float stp = bloom2_px / 12.0;
    if (stp < 0.02) { outColor = texture(sampler2D(u_in, u_s), v_uv); return; }
    vec4 acc = vec4(0.0);
    float wsum = 0.0;
    for (int i = -12; i <= 12; i++) {
        float fi = float(i);
        float w = exp(-fi * fi / 60.0);
        vec2 off = vec2(fi * stp / u_resolution.x, 0.0);
        acc += texture(sampler2D(u_in, u_s), v_uv + off) * w;
        wsum += w;
    }
    outColor = acc / wsum;
}
@endpass
@pass bl2v
#version 450
// Second blur octave, then folds the first octave in so the glass pass reads one glow texture.
// @param glow_mix1 label:"Glow Near" min:0 max:3 default:1.1
// @param glow_mix2 label:"Glow Far" min:0 max:3 default:0.7
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float bloom2_px;
    float glow_mix1;
    float glow_mix2;
};
layout(set = 0, binding = 3) uniform texture2D u_wb1;
void main() {
    float stp = bloom2_px / 12.0;
    vec4 far;
    if (stp < 0.02) { far = texture(sampler2D(u_in, u_s), v_uv); }
    else {
        vec4 acc = vec4(0.0);
        float wsum = 0.0;
        for (int i = -12; i <= 12; i++) {
            float fi = float(i);
            float w = exp(-fi * fi / 60.0);
            vec2 off = vec2(0.0, fi * stp / u_resolution.y);
            acc += texture(sampler2D(u_in, u_s), v_uv + off) * w;
            wsum += w;
        }
        far = acc / wsum;
    }
    vec4 near = texture(sampler2D(u_wb1, u_s), v_uv);
    outColor = vec4(clamp(near.rgb * glow_mix1 + far.rgb * glow_mix2, 0.0, 1.0), 1.0);
}
@endpass
@pass glass
#version 450
// A non-monotone rim displacement forms the mirrored meniscus.
// @param wave_amp label:"Wave Amp" min:0 max:3 default:1
// @param zoom label:"Lens Zoom" min:1 max:1.8 default:1.0
// @param refract_px label:"Rim Refract (px)" min:0 max:300 default:83.4
// @param refract_band label:"Rim Band (px)" min:10 max:400 default:65.3
// @param rim_peak label:"Rim Peak Pos" min:0.03 max:0.6 default:0.2857
// @param aberration label:"Rim Dispersion" min:0 max:0.3 default:0.015
// @param blur_mix label:"Frost Mix" min:0 max:1 default:0.14
// @param glass_tint label:"Glass Absorption" min:0 max:1 default:0.38
// @param island_shade label:"Island Shade" min:0 max:1 default:1
// @param island_bottom label:"Island Bottom" min:-1 max:1 default:0.12
// @param island_soft label:"Island Softness" min:0.02 max:1 default:0.38
// @param island_inset label:"Island Inset (px)" min:0 max:60 default:16
// @param wave_refract label:"Wave Refract Mix" min:0 max:1.5 default:0.35
// @param wave_disp label:"Wave Dispersion (px)" min:0 max:30 default:2
// @param sharp_amt label:"Wave Sharp Amt" min:0 max:3 default:1
// @param glow_amt label:"Wave Glow Amt" min:0 max:3 default:.12
// @param glow_in_black label:"Glow Over Island" min:0 max:1 default:0.45
// @param bottom_rim label:"Bottom Rim" min:0 max:2 default:0.65
// @param rim_w label:"Rim Sheen (px)" min:0.5 max:20 default:1.8
// @param top_sheen label:"Top Sheen" min:0 max:1 default:0.08
// @param rim_line label:"Rim Dark Line" min:0 max:1 default:0.1
// @param shadow_amt label:"Shadow" min:0 max:1 default:0.28
// @param shadow_soft label:"Shadow Soft (px)" min:2 max:200 default:40
// @param spill_amt label:"Under Spill" min:0 max:3 default:0.12
// @param spill_soft label:"Spill Soft (px)" min:10 max:400 default:200
// @param spill_spread label:"Spill Spread" min:1 max:5 default:2.6
// @param spill_disp label:"Spill Dispersion" min:0 max:0.6 default:0.25
// @param grain_amt label:"Grain" min:0 max:3 default:1
// @param hardware_keep label:"Preserve Hardware" min:0 max:1 default:1
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    vec2 center;
    float activation;
    float overshoot;
    float breath_amt;
    float breath_rate;
    float half_w;
    float half_h;
    float roundness;
    vec2 pill_center;
    float pill_half_w;
    float pill_half_h;
    float band_y;
    float wave_amp;
    float zoom;
    float refract_px;
    float refract_band;
    float rim_peak;
    float aberration;
    float blur_mix;
    float glass_tint;
    float island_shade;
    float island_bottom;
    float island_soft;
    float island_inset;
    float wave_refract;
    float wave_disp;
    float sharp_amt;
    float glow_amt;
    float glow_in_black;
    float bottom_rim;
    float rim_w;
    float top_sheen;
    float rim_line;
    float shadow_amt;
    float shadow_soft;
    float spill_amt;
    float spill_soft;
    float spill_spread;
    float spill_disp;
    float grain_amt;
    float hardware_keep;
    float capsule_reflect;
};
layout(set = 0, binding = 3) uniform texture2D u_bgblur;
layout(set = 0, binding = 4) uniform texture2D u_em;
layout(set = 0, binding = 5) uniform texture2D u_glow;
float easeOutBack(float t, float ovr) {
    float c1 = 1.70158 * ovr;
    float c3 = c1 + 1.0;
    float u = t - 1.0;
    return 1.0 + c3 * u * u * u + c1 * u * u;
}
vec3 sdRoundRectG(vec2 c, vec2 hs, float r) {
    vec2 cc = abs(c) - (hs - vec2(r));
    float outside = length(max(cc, vec2(0.0))) - r;
    float inside = min(max(cc.x, cc.y), 0.0);
    vec2 g;
    if (cc.x >= 0.0 || cc.y >= 0.0) {
        g = sign(c) * normalize(max(cc, vec2(0.0)) + 1e-6);
    } else {
        float gx = step(cc.y, cc.x);
        g = sign(c) * vec2(gx, 1.0 - gx);
    }
    return vec3(outside + inside, g);
}
vec3 sdfUnionG(vec3 a, vec3 b, float k) {
    float kEff = k * clamp(0.5 - 0.5 * dot(a.yz, b.yz), 0.0, 1.0) + 1e-4;
    float h = clamp(0.5 + 0.5 * (b.x - a.x) / kEff, 0.0, 1.0);
    float d = mix(b.x, a.x, h) - kEff * h * (1.0 - h);
    vec2 g = normalize(mix(b.yz, a.yz, h) + vec2(1e-5));
    return vec3(d, g);
}
vec3 sdSuperG(vec2 c, vec2 hs, float n) {
    vec2 q = c / hs;
    vec2 a = max(abs(q), vec2(1e-6));
    float s = pow(a.x, n) + pow(a.y, n);
    float f = pow(s, 1.0 / n) - 1.0;
    float k = pow(s, 1.0 / n - 1.0);
    vec2 g = k * sign(q) * vec2(pow(a.x, n - 1.0), pow(a.y, n - 1.0)) / hs;
    float gl = max(length(g), 1e-6);
    float mn = min(hs.x, hs.y);
    return vec3(clamp(f / gl, -mn, mn * 4.0), g / gl);
}
// glass outline: the Dynamic Island capsule at activation 0, the bulbous superellipse when open
vec3 glassSDF(vec2 c, vec2 hs, float n, float morph) {
    vec3 cap = sdRoundRectG(c, hs, min(hs.x, hs.y) * 0.98);
    vec3 sq = sdSuperG(c, hs, n);
    float d = mix(cap.x, sq.x, morph);
    vec2 g = normalize(mix(cap.yz, sq.yz, morph) + vec2(1e-5));
    return vec3(d, g);
}
float rimDisp(float d, float B, float P, float u0) {
    float q = clamp(1.0 - d / max(B, 1.0), 0.0, 1.0);
    return P * pow(q, max(1.0 / max(u0, 0.03) - 1.0, 1.0));
}
vec4 cubicW(float t) {
    float t2 = t * t;
    float t3 = t2 * t;
    return vec4(-0.5 * t3 + t2 - 0.5 * t, 1.5 * t3 - 2.5 * t2 + 1.0, -1.5 * t3 + 2.0 * t2 + 0.5 * t, 0.5 * t3 - 0.5 * t2);
}
float w4(vec4 w, int k) {
    if (k == 0) return w.x;
    if (k == 1) return w.y;
    if (k == 2) return w.z;
    return w.w;
}
// Catmull-Rom read of the wallpaper: the rim band magnifies it, and bilinear texels would show as blocks
vec3 inCubic(vec2 uv) {
    vec2 R = vec2(textureSize(sampler2D(u_in, u_s), 0));
    vec2 st = uv * R - 0.5;
    vec2 i = floor(st);
    vec2 f = st - i;
    vec4 wx = cubicW(f.x);
    vec4 wy = cubicW(f.y);
    vec3 acc = vec3(0.0);
    for (int y = 0; y < 4; y++) {
        for (int x = 0; x < 4; x++) {
            vec2 suv = (i + vec2(float(x) - 1.0, float(y) - 1.0) + 0.5) / R;
            acc += texture(sampler2D(u_in, u_s), suv).rgb * (w4(wx, x) * w4(wy, y));
        }
    }
    return max(acc, vec3(0.0));
}
vec3 lensBG(vec2 uv, vec2 dx, vec2 dy) {
    vec2 size = vec2(textureSize(sampler2D(u_in, u_s), 0));
    float sx = length(dx * size), sy = length(dy * size);
    if (max(sx, sy) <= 1.25) return mix(inCubic(uv), texture(sampler2D(u_bgblur, u_s), uv).rgb, blur_mix);
    int nx = int(clamp(ceil(sx * 1.5), 2.0, 8.0));
    int ny = int(clamp(ceil(sy * 1.5), 2.0, 8.0));
    vec3 sharp = vec3(0.0);
    for (int y = 0; y < ny; y++) {
        for (int x = 0; x < nx; x++) {
            vec2 q = vec2((float(x) + 0.5) / float(nx), (float(y) + 0.5) / float(ny)) - 0.5;
            sharp += texture(sampler2D(u_in, u_s), uv + dx * q.x + dy * q.y).rgb;
        }
    }
    sharp /= float(nx * ny);
    return mix(sharp, texture(sampler2D(u_bgblur, u_s), uv).rgb, blur_mix);
}
vec3 reflectedRibbon(float x, float y, float hh) {
    vec3 light = vec3(0.0);
    for (int i = -2; i <= 2; i++) {
        float w = exp(-0.25 * float(i * i));
        light += texture(sampler2D(u_glow, u_s), vec2(x, (y + float(i) * hh * 0.1) / u_resolution.y)).rgb * w;
    }
    return light * (4.0 / 3.2933604);
}
float ign(vec2 fc) {
    return fract(52.9829189 * fract(dot(fc, vec2(0.06711056, 0.00583715))));
}
void main() {
    vec2 R = u_resolution;
    vec2 fc = v_uv * R;
    float hardwareAA = max(length(vec2(dFdx(fc.x), dFdy(fc.y))) * 0.5, 0.75);
    vec3 bgSharp = texture(sampler2D(u_in, u_s), v_uv).rgb;
    if (activation < 0.003) { outColor = vec4(bgSharp, 1.0); return; }

    float ag = easeOutBack(clamp(activation, 0.0, 1.0), overshoot);
    ag -= 0.12 * sin(3.14159265359 * clamp(activation / 0.25, 0.0, 1.0));
    float actv = clamp(ag, 0.0, 1.0);
    float hwFull = half_w * R.x;
    float hhFull = half_h * R.x;
    float hw = max(mix(pill_half_w, hwFull, pow(max(ag, 0.0), 1.2)), 1.0);
    float hh = max(mix(pill_half_h, hhFull, ag), 1.0);
    float pulseCos = cos(6.283185307 * breath_rate * (u_time - 0.15));
    float pulse = 0.5 + 0.5 * pulseCos;
    float glassBreath = breath_amt * pulse * smoothstep(0.25, 0.8, activation);
    hw += glassBreath * 0.18;
    hh += glassBreath;
    float s = hh / max(hhFull, 1.0);
    // the island capsule becomes glass while it grows: outline, black body and rim lighting all follow morph
    float morph = smoothstep(0.0, 0.7, clamp(ag, 0.0, 1.0));
    // ribbons and their spill stay hidden while the island is still solid hardware black
    float waveIn = smoothstep(0.0, 0.12, clamp(activation, 0.0, 1.0));
    vec2 cen = mix(pill_center * R, vec2(center.x * R.x, center.y * R.y), ag);
    cen.y -= glassBreath + pill_half_h * 0.385 * ag * (1.0 - ag);
    vec2 c = fc - cen;
    float nExp = mix(5.0, 2.0, roundness);
    vec3 un = glassSDF(c, vec2(hw, hh), nExp, morph);
    float sd = un.x;
    float coverageWidth = max(0.75 * fwidth(sd), 0.75);
    vec2 g = un.yz;
    float ny0 = c.y / max(hh, 1.0);
    float nx0 = c.x / max(hw, 1.0);

    float belowBias = clamp(0.15 + 0.85 * clamp(ny0, 0.0, 1.0), 0.0, 1.0);
    float shadow = exp(-max(sd, 0.0) / max(shadow_soft, 1.0)) * shadow_amt * belowBias * actv;
    vec2 bandUV = (cen + vec2(0.0, band_y * hh)) / R;
    float downMask = smoothstep(0.45, 1.0, ny0);
    float distFall = exp(-max(sd, 0.0) / max(spill_soft, 1.0));
    float spY = max(spill_spread * 2.0, 1.0);
    vec2 dUV = v_uv - bandUV;
    vec2 sUVr = bandUV + dUV / vec2(spill_spread * (1.0 + spill_disp), spY * (1.0 + spill_disp * 0.5));
    vec2 sUVg = bandUV + dUV / vec2(spill_spread, spY);
    vec2 sUVb = bandUV + dUV / vec2(max(spill_spread * (1.0 - spill_disp), 1.0), max(spY * (1.0 - spill_disp * 0.5), 1.0));
    vec3 spill = vec3(texture(sampler2D(u_glow, u_s), sUVr).r, texture(sampler2D(u_glow, u_s), sUVg).g, texture(sampler2D(u_glow, u_s), sUVb).b) * 4.0;
    spill *= spill_amt * wave_amp * downMask * distFall * waveIn;
    vec3 causticLight = reflectedRibbon(v_uv.x, cen.y + hh * 0.18, hh);
    float caustic = exp(-pow((nx0 + 0.27) / 0.62, 2.0)) * exp(-pow((ny0 - 1.04) / 0.24, 2.0));
    vec3 outsideCol = bgSharp * (1.0 - shadow) + spill + causticLight * caustic * capsule_reflect * 1.25 * wave_amp * waveIn;

    float d = max(-sd, 0.0);
    float B = max(refract_band * s * (0.82 + 0.18 * abs(un.y)), 1.0);
    float dispG = rimDisp(d, B, refract_px * s * smoothstep(0.15, 0.8, activation), rim_peak);
    float dispR = dispG * (1.0 + aberration);
    float dispB = dispG * (1.0 - aberration);
    vec2 cuv = cen / R;
    vec2 gpx = g / R;
    vec2 uvR = cuv + ((v_uv - gpx * dispR) - cuv) / zoom;
    vec2 uvG = cuv + ((v_uv - gpx * dispG) - cuv) / zoom;
    vec2 uvB = cuv + ((v_uv - gpx * dispB) - cuv) / zoom;
    vec2 dxR = dFdx(uvR), dyR = dFdy(uvR);
    vec2 dxG = dFdx(uvG), dyG = dFdy(uvG);
    vec2 dxB = dFdx(uvB), dyB = dFdy(uvB);
    if (sd > coverageWidth) {
        outsideCol += (ign(fc) - 0.5) * (grain_amt * 2.0 / 255.0);
        outColor = vec4(outsideCol, 1.0);
        return;
    }
    vec3 col = vec3(lensBG(uvR, dxR, dyR).r, lensBG(uvG, dxG, dyG).g, lensBG(uvB, dxB, dyB).b);
    col *= exp(-glass_tint * vec3(1.22, 1.38, 1.44));

    vec2 islHS = max(vec2(hw, hh) - vec2(island_inset * s, island_inset * s * 0.35), vec2(1.0));
    vec3 isl = glassSDF(c, islHS, nExp, morph);
    float islMask = (1.0 - smoothstep(-6.0, 2.0, isl.x));
    float vert = 1.0 - smoothstep(island_bottom - island_soft, island_bottom + island_soft, ny0);
    float black = mix(1.0, vert * island_shade, smoothstep(0.08, 0.42, clamp(activation, 0.0, 1.0)));
    col = mix(col, vec3(0.002, 0.003, 0.005), black);

    float wm = wave_refract;
    float zw = 1.0 + (zoom - 1.0) * wm;
    float edgeZone = 1.0 - smoothstep(0.0, B, d);
    vec3 sharpW = texture(sampler2D(u_em, u_s), v_uv).rgb * 4.0;
    vec3 glowW = texture(sampler2D(u_glow, u_s), v_uv).rgb * 4.0;
    float bgLum = dot(col, vec3(0.299, 0.587, 0.114));
    float addGain = mix(1.0, 0.6, smoothstep(0.55, 1.0, bgLum));
    float glowVis = 1.0 - black * (1.0 - glow_in_black);
    float rimAtten = mix(1.0, 0.45, edgeZone * edgeZone);
    col += (sharpW * sharp_amt + glowW * glow_amt * glowVis * 0.33) * (wave_amp * addGain * rimAtten * waveIn);

    col *= 1.0 - rim_line * 0.55 * (1.0 - smoothstep(0.0, 2.5 * s, d));
    float bandMask = 1.0 - smoothstep(0.0, B * 0.6, d);
    float topness = (1.0 - smoothstep(-0.9, 0.2, ny0));
    col *= 1.0 - 0.18 * bandMask * topness;
    float bottomMask = smoothstep(-0.1, 0.75, ny0);
    float cornerBoost = 0.6 + 0.8 * smoothstep(0.2, 0.8, abs(nx0));
    float sheen = (1.0 - smoothstep(0.0, rim_w * s, d)) + 0.35 * (1.0 - smoothstep(0.0, rim_w * 3.0 * s, d));
    sheen *= bottomMask * cornerBoost;
    vec2 rimUV = cuv + ((v_uv - gpx * (B * 0.5)) - cuv) / zw;
    vec3 rimGlow = texture(sampler2D(u_glow, u_s), rimUV).rgb * 4.0;
    col += (vec3(0.92, 0.95, 1.0) * bottom_rim * 0.55 + rimGlow * wave_amp * bottom_rim * 1.2) * sheen * morph;
    float topMask = (1.0 - smoothstep(-0.7, 0.1, ny0));
    col += vec3(0.75, 0.80, 0.90) * top_sheen * (1.0 - smoothstep(0.0, 2.5 * s, abs(d - 1.5 * s))) * topMask * morph;

    float aa = 1.0 - smoothstep(-coverageWidth, coverageWidth, sd);
    col = mix(outsideCol, col, aa);
    col += (ign(fc) - 0.5) * (grain_amt * 2.0 / 255.0);
    vec3 hardware = sdRoundRectG(fc - pill_center * R, max(vec2(pill_half_w, pill_half_h) - 3.0, vec2(1.0)), max(min(pill_half_w, pill_half_h) - 3.0, 1.0) * 0.98);
    float hardwareMask = 1.0 - smoothstep(-hardwareAA, hardwareAA, hardware.x);
    vec2 reflectedUV = vec2(v_uv.x, (cen.y + band_y * hh - hh * 0.10) / R.y);
    vec3 reflectedLight = texture(sampler2D(u_glow, u_s), reflectedUV).rgb * 4.0;
    float capsuleEdge = exp(-pow((hardware.x + 6.0) / 19.0, 2.0));
    float underside = smoothstep(-0.45, 0.85, (fc.y - pill_center.y * R.y) / max(pill_half_h, 1.0));
    vec3 hardwareColor = bgSharp + reflectedLight * capsule_reflect * (capsuleEdge * underside * 0.07 + underside * 0.0225) * wave_amp * waveIn;
    col = mix(col, hardwareColor, hardwareMask * hardware_keep);
    float glassReflection = exp(-pow((d - B * 0.45) / max(B * 0.45, 1.0), 2.0));
    vec3 lowerLight = reflectedRibbon(v_uv.x, cen.y + hh * 0.18, hh);
    float bounce = exp(-pow((ny0 - 0.84) / 0.42, 2.0)) * exp(-pow((nx0 + 0.27) / 0.67, 2.0));
    col += lowerLight * capsule_reflect * (bounce * 1.25 + glassReflection * bottomMask * 0.10) * wave_amp * waveIn * aa * (1.0 - clamp(dot(col, vec3(0.299, 0.587, 0.114)), 0.0, 1.0) * 0.65);
    outColor = vec4(col, 1.0);
}
@endpass
