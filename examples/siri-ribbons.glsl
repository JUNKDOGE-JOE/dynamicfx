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
// @param center label:"Glass Center"
// @param activation label:"Activation" min:0 max:1 default:1
// @param overshoot label:"Pop Overshoot" min:0 max:3 default:0.7
// @param half_w label:"Glass Half W" min:0.05 max:0.45 default:0.1907
// @param half_h label:"Glass Half H" min:0.05 max:0.45 default:0.145
// @param roundness label:"Roundness" min:0.2 max:1 default:0.86
// @param pill_center label:"Pill Center"
// @param pill_half_w label:"Pill Half W (px)" min:10 max:400 default:180
// @param pill_half_h label:"Pill Half H (px)" min:5 max:200 default:52
// @param band_y label:"Wave Y" min:-1 max:1 default:0.2
// @param rf_height label:"Ribbon Spread" min:0 max:0.8 default:0.19
// @param rf_flow label:"Ribbon Flow" min:-12 max:12 default:4.18879
// @param rf_phase label:"Ribbon Phase" min:-10 max:10 default:0.9
// @param rf_twist label:"Ribbon Twist" min:0 max:4 default:1.2
// @param rf_sway label:"Ribbon Sway" min:0 max:0.5 default:0.06
// @param rf_width label:"Ribbon Tail" min:0.01 max:0.8 default:0.085
// @param rf_edge label:"Ribbon Edge (px)" min:0.1 max:20 default:2.2
// @param rf_envelope label:"Ribbon Extent" min:0.1 max:1.5 default:0.95
// @param rf_gain label:"Ribbon Radiance" min:0 max:5 default:0.828
// @param rf_fold label:"Fold Light" min:0 max:3 default:0.25
// @param rf_blue label:"Ribbon Blue" hint:color default:0.08,0.28,1
// @param rf_cyan label:"Ribbon Cyan" hint:color default:0.08,0.95,1
// @param rf_yellow label:"Ribbon Yellow" hint:color default:1,0.88,0.12
// @param rf_red label:"Ribbon Red" hint:color default:1,0.09,0.04
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
    if (d <= 0.0 || d >= B) return 0.0;
    float q = (d / B) / max(u0, 0.01);
    return P * q * exp(1.0 - q);
}
// Keep distance unscaled when the tail changes sides; a zero orientation must not turn a sheet into a column.
float ribbonProfile(float distancePx, float direction, float tailPx, float edgePx) {
    float widthAbove = mix(edgePx, tailPx, smoothstep(-0.25, 0.25, -direction));
    float widthBelow = mix(edgePx, tailPx, smoothstep(-0.25, 0.25, direction));
    float sigma = mix(widthAbove, widthBelow, smoothstep(-edgePx, edgePx, distancePx));
    return exp(-0.5 * distancePx * distancePx / max(sigma * sigma, 0.01));
}
vec3 sheetAt(vec2 fc, vec2 cen, float hw, float hh, float morph, float t, float footprint) {
    vec2 c = fc - cen;
    float x = c.x / hw;
    float u = clamp(abs(x) / max(rf_envelope, 0.01), 0.0, 1.0);
    float envelope = pow(max(cos(1.570796327 * u), 0.0), 1.6);
    float lightEnvelope = envelope * exp(-1.2 * x * x);
    float phase = rf_phase - rf_flow * t + rf_twist * x;
    float sway = rf_sway * sin(1.7 * x - 0.65 * t) * envelope;
    float baseY = cen.y + hh * (band_y + sway);
    float breath = (0.85 + 0.15 * smoothstep(0.6, 1.2, wave_amp)) * (0.9 + 0.1 * sin(0.91 * t + 1.3));
    float spread = hh * rf_height * envelope * breath;
    float edge = sqrt(rf_edge * rf_edge + footprint * footprint / 12.0);
    float tail = max(hh * rf_width * envelope * breath, edge);
    vec3 radiance = vec3(0.0);
    for (int i = 0; i < 4; i++) {
        float offset = i == 0 ? 0.0 : (i == 1 ? 0.72 : (i == 2 ? 3.86 : 3.141592654));
        float angle = phase + offset;
        float location = cos(angle);
        float edgeY = baseY + spread * location;
        float direction = -location;
        float profile = ribbonProfile(fc.y - edgeY, direction, tail, edge);
        vec3 tint = i == 0 ? rf_blue : (i == 1 ? rf_cyan : (i == 2 ? rf_yellow : rf_red));
        float facing = 0.72 + 0.28 * sin(angle + 0.4);
        radiance += tint * profile * facing;
    }
    float fold = exp(-pow(c.x / (hw * 0.42), 2.0)) * exp(-pow((fc.y - baseY) / max(edge * 1.4, 1.0), 2.0));
    return (radiance + vec3(rf_fold * fold)) * lightEnvelope * rf_gain;
}

void main() {
    vec2 R = u_resolution;
    vec2 fc = v_uv * R;
    if (activation < 0.003) { outColor = vec4(0.0, 0.0, 0.0, 1.0); return; }
    float ag = easeOutBack(clamp(activation, 0.0, 1.0), overshoot);
    float hwFull = half_w * R.x;
    float hhFull = half_h * R.x;
    float hw = max(mix(pill_half_w, hwFull, ag), 1.0);
    float hh = max(mix(pill_half_h, hhFull, ag), 1.0);
    float s = hh / max(hhFull, 1.0);
    float morph = smoothstep(0.15, 0.8, clamp(activation, 0.0, 1.0));
    vec2 cen = mix(pill_center * R, vec2(center.x * R.x, center.y * R.y), ag);
    vec2 c = fc - cen;
    vec3 un = glassSDF(c, vec2(hw, hh), mix(5.0, 2.0, roundness), morph);
    float sd = un.x;
    float outMask = 1.0 - smoothstep(-30.0, 40.0, sd);

    float t = u_time;
    // the lens of the glass pass: rim bump per channel, zoom about the centre, chromatic shift
    // at the rim; the mapping fades to identity just outside the outline
    float d = max(-sd, 0.0);
    float B = max(refract_band * s, 1.0);
    float dispG = rimDisp(d, B, refract_px * s, rim_peak);
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
    vec3 acc = vec3(0.0);
    for (int k = 0; k < 4; k++) {
        vec2 j = (vec2(float(k & 1), float(k >> 1)) - 0.5) * 0.5;
        acc.r += sheetAt(fcR + dxR * j.x + dyR * j.y, cen, hw, hh, morph, t, footprint).r;
        acc.g += sheetAt(fcG + dxG * j.x + dyG * j.y, cen, hw, hh, morph, t, footprint).g;
        acc.b += sheetAt(fcB + dxB * j.x + dyB * j.y, cen, hw, hh, morph, t, footprint).b;
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
// @param wave_amp label:"Wave Amp" min:0 max:3 default:0.62
// @param zoom label:"Lens Zoom" min:1 max:1.8 default:1.18
// @param refract_px label:"Rim Refract (px)" min:0 max:300 default:78
// @param refract_band label:"Rim Band (px)" min:10 max:400 default:60
// @param rim_peak label:"Rim Peak Pos" min:0.03 max:0.6 default:0.15
// @param aberration label:"Rim Dispersion" min:0 max:0.3 default:0.015
// @param blur_mix label:"Frost Mix" min:0 max:1 default:0.08
// @param glass_tint label:"Glass Darken" min:0 max:1 default:0.5
// @param island_shade label:"Island Shade" min:0 max:1 default:1
// @param island_bottom label:"Island Bottom" min:-1 max:1 default:0.12
// @param island_soft label:"Island Softness" min:0.02 max:1 default:0.38
// @param island_inset label:"Island Inset (px)" min:0 max:60 default:16
// @param wave_refract label:"Wave Refract Mix" min:0 max:1.5 default:0.6
// @param wave_disp label:"Wave Dispersion (px)" min:0 max:30 default:8
// @param sharp_amt label:"Wave Sharp Amt" min:0 max:3 default:1
// @param glow_amt label:"Wave Glow Amt" min:0 max:3 default:0.45
// @param glow_in_black label:"Glow Over Island" min:0 max:1 default:0.45
// @param bottom_rim label:"Bottom Rim" min:0 max:2 default:0.75
// @param rim_w label:"Rim Sheen (px)" min:0.5 max:20 default:4
// @param top_sheen label:"Top Sheen" min:0 max:1 default:0.18
// @param rim_line label:"Rim Dark Line" min:0 max:1 default:0.5
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
    if (d <= 0.0 || d >= B) return 0.0;
    float q = (d / B) / max(u0, 0.01);
    return P * q * exp(1.0 - q);
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
vec3 lensBG(vec2 uv, bool cub) {
    vec3 sharp = cub ? inCubic(uv) : texture(sampler2D(u_in, u_s), uv).rgb;
    return mix(sharp, texture(sampler2D(u_bgblur, u_s), uv).rgb, blur_mix);
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
    float actv = clamp(ag, 0.0, 1.0);
    float hwFull = half_w * R.x;
    float hhFull = half_h * R.x;
    float hw = max(mix(pill_half_w, hwFull, ag), 1.0);
    float hh = max(mix(pill_half_h, hhFull, ag), 1.0);
    float s = hh / max(hhFull, 1.0);
    // the island capsule becomes glass while it grows: outline, black body and rim lighting all follow morph
    float morph = smoothstep(0.15, 0.8, clamp(activation, 0.0, 1.0));
    // ribbons and their spill stay hidden while the island is still solid hardware black
    float waveIn = smoothstep(0.25, 0.65, clamp(activation, 0.0, 1.0));
    vec2 cen = mix(pill_center * R, vec2(center.x * R.x, center.y * R.y), ag);
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
    vec3 outsideCol = bgSharp * (1.0 - shadow) + spill;
    if (sd > coverageWidth) {
        outsideCol += (ign(fc) - 0.5) * (grain_amt * 2.0 / 255.0);
        outColor = vec4(outsideCol, 1.0);
        return;
    }

    float d = max(-sd, 0.0);
    float B = max(refract_band * s, 1.0);
    float dispG = rimDisp(d, B, refract_px * s, rim_peak);
    float dispR = dispG * (1.0 + aberration);
    float dispB = dispG * (1.0 - aberration);
    vec2 cuv = cen / R;
    vec2 gpx = g / R;
    vec2 uvR = cuv + ((v_uv - gpx * dispR) - cuv) / zoom;
    vec2 uvG = cuv + ((v_uv - gpx * dispG) - cuv) / zoom;
    vec2 uvB = cuv + ((v_uv - gpx * dispB) - cuv) / zoom;
    bool cub = d < B;
    vec3 col = vec3(lensBG(uvR, cub).r, lensBG(uvG, cub).g, lensBG(uvB, cub).b);
    col *= mix(vec3(1.0), vec3(0.55, 0.60, 0.70), glass_tint);

    vec2 islHS = max(vec2(hw, hh) - vec2(island_inset * s, island_inset * s * 0.35), vec2(1.0));
    vec3 isl = glassSDF(c, islHS, nExp, morph);
    float islMask = (1.0 - smoothstep(-6.0, 2.0, isl.x));
    float vert = 1.0 - smoothstep(island_bottom - island_soft, island_bottom + island_soft, ny0);
    float black = islMask * mix(1.0, vert * island_shade, morph);
    col = mix(col, vec3(0.008, 0.009, 0.014), black);

    float wm = wave_refract;
    float zw = 1.0 + (zoom - 1.0) * wm;
    float edgeZone = 1.0 - smoothstep(0.0, B, d);
    vec3 sharpW = texture(sampler2D(u_em, u_s), v_uv).rgb * 4.0;
    vec3 glowW = texture(sampler2D(u_glow, u_s), v_uv).rgb * 4.0;
    float bgLum = dot(col, vec3(0.299, 0.587, 0.114));
    float addGain = mix(1.0, 0.6, smoothstep(0.55, 1.0, bgLum));
    float glowVis = 1.0 - black * (1.0 - glow_in_black);
    float rimAtten = mix(1.0, 0.45, edgeZone * edgeZone);
    col += (sharpW * sharp_amt + glowW * glow_amt * glowVis) * (wave_amp * addGain * rimAtten * waveIn);

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
    col += (vec3(0.92, 0.95, 1.0) * bottom_rim + rimGlow * wave_amp * bottom_rim * 1.2) * sheen * morph;
    float topMask = (1.0 - smoothstep(-0.7, 0.1, ny0));
    col += vec3(0.75, 0.80, 0.90) * top_sheen * (1.0 - smoothstep(0.0, 2.5 * s, abs(d - 1.5 * s))) * topMask * morph;

    float aa = 1.0 - smoothstep(-coverageWidth, coverageWidth, sd);
    col = mix(outsideCol, col, aa);
    col += (ign(fc) - 0.5) * (grain_amt * 2.0 / 255.0);
    vec3 hardware = sdRoundRectG(fc - pill_center * R, max(vec2(pill_half_w, pill_half_h) - 3.0, vec2(1.0)), max(min(pill_half_w, pill_half_h) - 3.0, 1.0) * 0.98);
    float hardwareMask = 1.0 - smoothstep(-hardwareAA, hardwareAA, hardware.x);
    col = mix(col, bgSharp, hardwareMask * hardware_keep);
    outColor = vec4(col, 1.0);
}
@endpass
