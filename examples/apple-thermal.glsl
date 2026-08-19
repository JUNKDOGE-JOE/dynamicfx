@dynamicfx 1
@graph
pass sh: input -> sh0
pass sv: sh0 -> soft
pass dh: soft -> dh0
pass dv: dh0 -> diff
pass temp: input, soft, diff -> tfield
pass sbh: tfield -> sb0
pass sbv: sb0 -> tsoft
pass th: tfield -> th0
pass tv: th0 -> thalo
pass col: tfield, tsoft, thalo, ramp -> output
@end
@pass sh
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float speed;
    float heat_depth;
    float edge_soft;
    float thickness;
    float wall_heat;
    float rim_heat;
    float line_heat;
    float core_temp;
    float flow_scale;
    float flow_amount;
    float turbulence;
    float contrast;
    float bias_amount;
    float halo;
    float halo_radius;
    float bloom;
    float bloom_radius;
    float grain;
    float extrude_angle;
    float bias_angle;
    int use_ramp;
};
void main() {
    float sigma = max(edge_soft, 0.5);
    float acc = 0.0;
    float ws = 0.0;
    for (int i = -32; i <= 32; i++) {
        float x = float(i) * 3.0 * sigma / 32.0;
        float w = exp(-0.5 * x * x / (sigma * sigma));
        acc += texture(sampler2D(u_in, u_s), v_uv + vec2(x / u_resolution.x, 0.0)).a * w;
        ws += w;
    }
    float dz = fract(sin(dot(floor(v_uv * u_resolution) + 0.5, vec2(127.1, 311.7))) * 43758.5453123);
    outColor = vec4(vec3(acc / ws + (dz - 0.5) / 255.0), 1.0); // dither: keeps sub-level precision as noise for the next blur pass
}
@endpass
@pass sv
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float speed;
    float heat_depth;
    float edge_soft;
    float thickness;
    float wall_heat;
    float rim_heat;
    float line_heat;
    float core_temp;
    float flow_scale;
    float flow_amount;
    float turbulence;
    float contrast;
    float bias_amount;
    float halo;
    float halo_radius;
    float bloom;
    float bloom_radius;
    float grain;
    float extrude_angle;
    float bias_angle;
    int use_ramp;
};
void main() {
    float sigma = max(edge_soft, 0.5);
    float acc = 0.0;
    float ws = 0.0;
    for (int i = -32; i <= 32; i++) {
        float x = float(i) * 3.0 * sigma / 32.0;
        float w = exp(-0.5 * x * x / (sigma * sigma));
        acc += texture(sampler2D(u_in, u_s), v_uv + vec2(0.0, x / u_resolution.y)).r * w;
        ws += w;
    }
    float dz = fract(sin(dot(floor(v_uv * u_resolution) + 0.5, vec2(127.1, 311.7))) * 43758.5453123);
    outColor = vec4(vec3(acc / ws + (dz - 0.5) / 255.0), 1.0); // dither: keeps sub-level precision as noise for the next blur pass
}
@endpass
@pass dh
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float speed;
    float heat_depth;
    float edge_soft;
    float thickness;
    float wall_heat;
    float rim_heat;
    float line_heat;
    float core_temp;
    float flow_scale;
    float flow_amount;
    float turbulence;
    float contrast;
    float bias_amount;
    float halo;
    float halo_radius;
    float bloom;
    float bloom_radius;
    float grain;
    float extrude_angle;
    float bias_angle;
    int use_ramp;
};
void main() {
    float sigma = max(heat_depth, 0.5);
    float acc = 0.0;
    float ws = 0.0;
    for (int i = -32; i <= 32; i++) {
        float x = float(i) * 3.0 * sigma / 32.0;
        float w = exp(-0.5 * x * x / (sigma * sigma));
        acc += texture(sampler2D(u_in, u_s), v_uv + vec2(x / u_resolution.x, 0.0)).r * w;
        ws += w;
    }
    float dz = fract(sin(dot(floor(v_uv * u_resolution) + 0.5, vec2(127.1, 311.7))) * 43758.5453123);
    outColor = vec4(vec3(acc / ws + (dz - 0.5) / 255.0), 1.0); // dither: keeps sub-level precision as noise for the next blur pass
}
@endpass
@pass dv
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float speed;
    float heat_depth;
    float edge_soft;
    float thickness;
    float wall_heat;
    float rim_heat;
    float line_heat;
    float core_temp;
    float flow_scale;
    float flow_amount;
    float turbulence;
    float contrast;
    float bias_amount;
    float halo;
    float halo_radius;
    float bloom;
    float bloom_radius;
    float grain;
    float extrude_angle;
    float bias_angle;
    int use_ramp;
};
void main() {
    float sigma = max(heat_depth, 0.5);
    float acc = 0.0;
    float ws = 0.0;
    for (int i = -32; i <= 32; i++) {
        float x = float(i) * 3.0 * sigma / 32.0;
        float w = exp(-0.5 * x * x / (sigma * sigma));
        acc += texture(sampler2D(u_in, u_s), v_uv + vec2(0.0, x / u_resolution.y)).r * w;
        ws += w;
    }
    float d = acc / ws;
    outColor = vec4(d, fract(d * 8.0), 0.0, 1.0);          // hi/lo encoding: 8x finer field on 8-bpc intermediates
}
@endpass
@pass temp
#version 450
// @param speed label:"Flow Speed" min:0 max:3 default:1
// @param heat_depth label:"Heat Depth (px)" min:4 max:200 default:56
// @param edge_soft label:"Edge Line (px)" min:0.5 max:30 default:5
// @param thickness label:"Wall Thickness (px)" min:0 max:400 default:140
// @param wall_heat label:"Wall Heat" min:0 max:2 default:1
// @param rim_heat label:"Rim Warmth" min:0 max:2 default:0.5
// @param line_heat label:"Edge Line Heat" min:0 max:2 default:0.6
// @param core_temp label:"Core Temp" min:0 max:1 default:0.03
// @param flow_scale label:"Flow Scale" min:0.2 max:6 default:1.6
// @param flow_amount label:"Flow Amount" min:0 max:2 default:1
// @param turbulence label:"Turbulence" min:0 max:2 default:0.6
// @param contrast label:"Contrast" min:0.3 max:3 default:1
// @param bias_amount label:"Heat Bias" min:0 max:1 default:0.15
// @param halo label:"Outer Glow" min:0 max:2 default:1
// @param halo_radius label:"Outer Glow Radius (px)" min:4 max:200 default:70
// @param bloom label:"Softness" min:0 max:1 default:0.4
// @param bloom_radius label:"Softness Radius (px)" min:1 max:120 default:24
// @param grain label:"Grain" min:0 max:0.3 default:0
// @param extrude_angle label:"Wall Direction" hint:angle default:205
// @param bias_angle label:"Heat Bias Direction" hint:angle default:180
// @param use_ramp label:"Use Custom Ramp" hint:bool default:0
// @param ramp label:"Custom Ramp" hint:gradient
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float speed;
    float heat_depth;
    float edge_soft;
    float thickness;
    float wall_heat;
    float rim_heat;
    float line_heat;
    float core_temp;
    float flow_scale;
    float flow_amount;
    float turbulence;
    float contrast;
    float bias_amount;
    float halo;
    float halo_radius;
    float bloom;
    float bloom_radius;
    float grain;
    float extrude_angle;
    float bias_angle;
    int use_ramp;
};
layout(set = 0, binding = 3) uniform texture2D u_soft;
layout(set = 0, binding = 4) uniform texture2D u_diff;
// integer lattice hash (exact and continuous across cells; the classic fract(sin(dot)) hash
// breaks on lattice borders for large arguments because a 1-ulp difference flips the result)
float hash21(vec2 p) {
    uvec2 q = uvec2(ivec2(p) + ivec2(32768));
    uint h = q.x * 0x8da6b343u ^ q.y * 0xd8163841u;
    h ^= h >> 13u;
    h *= 0x5bd1e995u;
    h ^= h >> 15u;
    return float(h & 0x00ffffffu) / 16777216.0;
}
float vnoise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash21(i), hash21(i + vec2(1.0, 0.0)), u.x),
               mix(hash21(i + vec2(0.0, 1.0)), hash21(i + vec2(1.0, 1.0)), u.x), u.y);
}
float fbm(vec2 p) {
    float v = 0.0;
    float amp = 0.5;
    mat2 m = mat2(1.6, 1.2, -1.2, 1.6);
    for (int i = 0; i < 4; i++) {
        v += amp * vnoise(p);
        p = m * p + vec2(3.7, 1.3);
        amp *= 0.5;
    }
    return v / 0.9375;
}
// smoother, lower-detail noise for the regional heat (3 octaves, soft gain)
float fbm3(vec2 p) {
    mat2 m = mat2(1.6, 1.2, -1.2, 1.6);
    float v = 0.60 * vnoise(p);
    p = m * p + vec2(3.7, 1.3);
    v += 0.28 * vnoise(p);
    p = m * p + vec2(1.9, 5.1);
    v += 0.12 * vnoise(p);
    return v;
}
void main() {
    vec2 res = u_resolution;
    float a = texture(sampler2D(u_in, u_s), v_uv).a;          // straight alpha = shape coverage
    float s = texture(sampler2D(u_soft, u_s), v_uv).r;         // small blur of alpha
    vec2 dd = texture(sampler2D(u_diff, u_s), v_uv).rg;       // wide blur of alpha (hi/lo encoded)
    float d = (floor(dd.r * 8.0 - dd.g + 0.5) + dd.g) / 8.0;

    // --- wall band: how much of the shape lies "behind" this pixel along the wall direction ---
    // (samples the soft mask along the extrusion vector; 1 right at a wall-facing contour, 0 one thickness inward)
    float ang = radians(extrude_angle);
    vec2 e = vec2(sin(ang), -cos(ang));                        // AE angle: 0 = up, clockwise; uv y grows downward
    float H = max(thickness, 0.5);
    float wd = 0.0;
    for (int k = 1; k <= 24; k++) {
        wd += 1.0 - texture(sampler2D(u_soft, u_s), v_uv + e * (float(k) / 24.0) * H / res).r;
    }
    wd = clamp(wd / 24.0, 0.0, 1.0);
    // wall heat: hottest at a wall-facing contour, long decay inward (one thickness)
    float hump = pow(wd, 1.5);

    // --- thin edge line on every contour ---
    float ls = (s - 0.5) / 0.18;
    float line = exp(-ls * ls);

    // --- wide diffusion warmth from every contour ---
    float depth = clamp((d - 0.5) * 2.0, 0.0, 1.0);            // 0 at contour, 1 deep inside
    float rimProfile = smoothstep(0.25, 1.0, 1.0 - depth);

    // --- flow: regional heat that drifts around the logo (features ~1/3 of the logo, ~2 s per change) ---
    float t = u_time * speed;
    vec2 p = (v_uv - 0.5) * vec2(res.x / res.y, 1.0);
    vec2 q = p * flow_scale;
    vec2 warp = vec2(fbm(q * 0.9 + vec2(0.0, -t * 0.25)), fbm(q * 0.9 + vec2(4.7, t * 0.2)));
    q += (warp - 0.5) * (0.9 * turbulence);
    float n = fbm3(q * 1.5 + vec2(t * 0.30, -t * 0.22));
    n = clamp((n - 0.5) * 2.4 + 0.5, 0.0, 1.0);
    float heat = mix(1.0 - 0.95 * clamp(flow_amount, 0.0, 1.0), 1.0, smoothstep(0.30, 0.85, n));   // 0.05 (cold) .. 1 (hot), mostly cold

    // --- directional bias ---
    float bang = radians(bias_angle);
    vec2 bdir = vec2(sin(bang), -cos(bang));
    float bias = bias_amount * dot(p, bdir);

    // hot band profile measured from the reference: red-orange at the contour, orange ~20 px in, yellow ~45,
    // cream ~65, light blue ~90, blue ~120 (for heat_depth = 56)
    float band = 0.72 * pow(1.0 - depth, 0.32) + 0.28 * pow(1.0 - depth, 5.0);   // thin red edge, wide yellow/white, long light-blue tail
    float faceFlood = smoothstep(0.70, 1.0, heat);              // only very hot regions flood the face pale/white
    float Thot = 0.85 * rim_heat * 2.0 * band
               + 0.10 * wall_heat * hump                         // lower-left contours run a touch hotter
               + 0.30 * flow_amount * faceFlood * (1.0 - band);
    // cold region: black face with a thin light-blue rim (contour line + a faint wall)
    float Tcold = 0.36 * max(line, wall_heat * pow(hump, 3.0)) + 0.05 * band;
    float T = core_temp + mix(Tcold, Thot, heat) + bias;
    T = clamp((T - 0.5) * contrast + 0.5, 0.0, 1.0);
    outColor = vec4(T, fract(T * 8.0), a, 1.0);                // T (hi/lo), coverage
}
@endpass
@pass sbh
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float speed;
    float heat_depth;
    float edge_soft;
    float thickness;
    float wall_heat;
    float rim_heat;
    float line_heat;
    float core_temp;
    float flow_scale;
    float flow_amount;
    float turbulence;
    float contrast;
    float bias_amount;
    float halo;
    float halo_radius;
    float bloom;
    float bloom_radius;
    float grain;
    float extrude_angle;
    float bias_angle;
    int use_ramp;
};
void main() {
    float sigma = max(bloom_radius, 0.5);
    vec2 acc = vec2(0.0);
    float ws = 0.0;
    for (int i = -24; i <= 24; i++) {
        float x = float(i) * 3.0 * sigma / 24.0;
        float w = exp(-0.5 * x * x / (sigma * sigma));
        vec4 c = texture(sampler2D(u_in, u_s), v_uv + vec2(x / u_resolution.x, 0.0));
        float T = (floor(c.r * 8.0 - c.g + 0.5) + c.g) / 8.0;
        acc += vec2(T * c.b, c.b) * w;
        ws += w;
    }
    outColor = vec4(acc / ws, 0.0, 1.0);                     // diffused temperature (T*a, a): thermal softness
}
@endpass
@pass sbv
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float speed;
    float heat_depth;
    float edge_soft;
    float thickness;
    float wall_heat;
    float rim_heat;
    float line_heat;
    float core_temp;
    float flow_scale;
    float flow_amount;
    float turbulence;
    float contrast;
    float bias_amount;
    float halo;
    float halo_radius;
    float bloom;
    float bloom_radius;
    float grain;
    float extrude_angle;
    float bias_angle;
    int use_ramp;
};
void main() {
    float sigma = max(bloom_radius, 0.5);
    vec2 acc = vec2(0.0);
    float ws = 0.0;
    for (int i = -24; i <= 24; i++) {
        float x = float(i) * 3.0 * sigma / 24.0;
        float w = exp(-0.5 * x * x / (sigma * sigma));
        acc += texture(sampler2D(u_in, u_s), v_uv + vec2(0.0, x / u_resolution.y)).rg * w;
        ws += w;
    }
    outColor = vec4(acc / ws, 0.0, 1.0);                     // diffused temperature (T*a, a): thermal softness
}
@endpass
@pass th
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float speed;
    float heat_depth;
    float edge_soft;
    float thickness;
    float wall_heat;
    float rim_heat;
    float line_heat;
    float core_temp;
    float flow_scale;
    float flow_amount;
    float turbulence;
    float contrast;
    float bias_amount;
    float halo;
    float halo_radius;
    float bloom;
    float bloom_radius;
    float grain;
    float extrude_angle;
    float bias_angle;
    int use_ramp;
};
void main() {
    float sigma = max(halo_radius, 1.0);
    vec2 acc = vec2(0.0);
    float ws = 0.0;
    for (int i = -32; i <= 32; i++) {
        float x = float(i) * 3.0 * sigma / 32.0;
        float w = exp(-0.5 * x * x / (sigma * sigma));
        vec4 c = texture(sampler2D(u_in, u_s), v_uv + vec2(x / u_resolution.x, 0.0));
        float T = (floor(c.r * 8.0 - c.g + 0.5) + c.g) / 8.0;
        acc += vec2(T * c.b, c.b) * w;
        ws += w;
    }
    outColor = vec4(acc / ws, 0.0, 1.0);                     // premultiplied temperature (T*a) and coverage (a), blurred
}
@endpass
@pass tv
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float speed;
    float heat_depth;
    float edge_soft;
    float thickness;
    float wall_heat;
    float rim_heat;
    float line_heat;
    float core_temp;
    float flow_scale;
    float flow_amount;
    float turbulence;
    float contrast;
    float bias_amount;
    float halo;
    float halo_radius;
    float bloom;
    float bloom_radius;
    float grain;
    float extrude_angle;
    float bias_angle;
    int use_ramp;
};
void main() {
    float sigma = max(halo_radius, 1.0);
    vec2 acc = vec2(0.0);
    float ws = 0.0;
    for (int i = -32; i <= 32; i++) {
        float x = float(i) * 3.0 * sigma / 32.0;
        float w = exp(-0.5 * x * x / (sigma * sigma));
        acc += texture(sampler2D(u_in, u_s), v_uv + vec2(0.0, x / u_resolution.y)).rg * w;
        ws += w;
    }
    outColor = vec4(acc / ws, 0.0, 1.0);                     // premultiplied temperature (T*a) and coverage (a), blurred
}
@endpass
@pass col
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float speed;
    float heat_depth;
    float edge_soft;
    float thickness;
    float wall_heat;
    float rim_heat;
    float line_heat;
    float core_temp;
    float flow_scale;
    float flow_amount;
    float turbulence;
    float contrast;
    float bias_amount;
    float halo;
    float halo_radius;
    float bloom;
    float bloom_radius;
    float grain;
    float extrude_angle;
    float bias_angle;
    int use_ramp;
};
layout(set = 0, binding = 3) uniform texture2D u_soft;
layout(set = 0, binding = 4) uniform texture2D u_halo;
layout(set = 0, binding = 5) uniform texture2D u_ramp;
// Palette sampled from the reference footage (cold -> hot):
// black, navy, blue, light blue, pale, cream, yellow, orange, red-orange, salmon, pink-white
vec3 thermal(float t) {
    t = clamp(t, 0.0, 1.0);
    vec3 c0 = vec3(0.000, 0.000, 0.000);
    vec3 c1 = vec3(0.004, 0.000, 0.051);
    vec3 c2 = vec3(0.008, 0.035, 0.404);
    vec3 c3 = vec3(0.043, 0.188, 0.729);
    vec3 c4 = vec3(0.110, 0.424, 0.878);
    vec3 c5 = vec3(0.345, 0.706, 0.937);
    vec3 c6 = vec3(0.682, 0.843, 0.843);
    vec3 c7 = vec3(0.902, 0.847, 0.612);
    vec3 c8 = vec3(0.980, 0.800, 0.404);
    vec3 c9 = vec3(0.992, 0.690, 0.290);
    vec3 cA = vec3(0.992, 0.522, 0.051);
    vec3 cB = vec3(0.980, 0.290, 0.031);
    vec3 cC = vec3(1.000, 0.314, 0.349);
    vec3 cD = vec3(1.000, 0.769, 0.769);
    if (t < 0.06) return mix(c0, c1, t / 0.06);
    if (t < 0.14) return mix(c1, c2, (t - 0.06) / 0.08);
    if (t < 0.22) return mix(c2, c3, (t - 0.14) / 0.08);
    if (t < 0.31) return mix(c3, c4, (t - 0.22) / 0.09);
    if (t < 0.40) return mix(c4, c5, (t - 0.31) / 0.09);
    if (t < 0.48) return mix(c5, c6, (t - 0.40) / 0.08);
    if (t < 0.55) return mix(c6, c7, (t - 0.48) / 0.07);
    if (t < 0.62) return mix(c7, c8, (t - 0.55) / 0.07);
    if (t < 0.70) return mix(c8, c9, (t - 0.62) / 0.08);
    if (t < 0.78) return mix(c9, cA, (t - 0.70) / 0.08);
    if (t < 0.86) return mix(cA, cB, (t - 0.78) / 0.08);
    if (t < 0.93) return mix(cB, cC, (t - 0.86) / 0.07);
    return mix(cC, cD, (t - 0.93) / 0.07);
}
vec3 lookup(float t) {
    if (use_ramp == 1) {
        return texture(sampler2D(u_ramp, u_s), vec2(clamp(t, 0.0, 1.0), 0.5)).rgb;
    }
    return thermal(t);
}
float dhash(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123);
}
void main() {
    vec4 tf = texture(sampler2D(u_in, u_s), v_uv);
    float Tcrisp = (floor(tf.r * 8.0 - tf.g + 0.5) + tf.g) / 8.0;
    float a = tf.b;
    vec2 sb = texture(sampler2D(u_soft, u_s), v_uv).rg;        // diffused (T*a, a)
    float Tsoft = sb.r / max(sb.g, 1e-3);
    float Tin = mix(Tcrisp, Tsoft, clamp(bloom, 0.0, 1.0));
    vec2 hb = texture(sampler2D(u_halo, u_s), v_uv).rg;        // wide-blurred (T*a, a)
    float ab = hb.g;
    float fade = clamp(ab * 2.0, 0.0, 1.0);                    // 1 at the contour, -> 0 outward
    // outer glow: temperature of the nearby shape, cooling with distance; alpha carries the long tail
    float Tout = pow(fade, 0.8) * (0.24 + 0.55 * (hb.r / max(ab, 1e-3)));   // blue base glow + the nearby shape's heat
    float aOut = clamp(halo * pow(fade, 0.5) * 1.6, 0.0, 1.0);
    float outA = max(a, aOut);
    vec3 cIn = lookup(Tin);
    vec3 cOut = lookup(Tout);
    vec3 col = mix(cOut, cIn, a / max(outA, 1e-4));
    float g = dhash(v_uv * u_resolution + vec2(u_frame * 7.3, u_frame * 3.1));
    col *= 1.0 + (g - 0.5) * grain;
    col += (g - 0.5) * (1.5 / 255.0);
    outColor = vec4(col, outA);                                // straight alpha
}
@endpass
