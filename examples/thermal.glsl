@dynamicfx 1
@graph
pass field: input -> f0
pass fbh: f0 -> f1
pass fbv: f1 -> fsm
pass blurh: input -> b0
pass blurv: b0 -> soft
pass comp: input, soft, fsm -> output
@end
@pass field
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_a;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float glow;
    float heat;
    float speed;
};
float hash21(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123);
}
float vnoise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    float a = hash21(i);
    float b = hash21(i + vec2(1.0, 0.0));
    float c = hash21(i + vec2(0.0, 1.0));
    float d = hash21(i + vec2(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}
float fbm3(vec2 p) {
    return 0.55 * vnoise(p) + 0.28 * vnoise(p * 2.1 + 7.0) + 0.17 * vnoise(p * 4.3 + 13.0);
}
void main() {
    float a = texture(sampler2D(u_a, u_s), v_uv).a;
    float t = u_time * speed;
    vec2 p = (v_uv - 0.5) * vec2(u_resolution.x / u_resolution.y, 1.0);
    vec2 warp = vec2(fbm3(p * 1.4 + vec2(0.0, -t * 0.18)),
                     fbm3(p * 1.4 + vec2(5.2, t * 0.15)));
    float n = fbm3(p * 1.9 + (warp - 0.5) * 1.6);
    vec2 cp = p - vec2(-0.05, 0.02);
    float core = exp(-dot(cp, cp) * 7.0);
    float field = clamp(n * 1.05 + core * 0.55 * heat - 0.22, 0.0, 1.0);
    outColor = vec4(vec3(field), 1.0);
}
@endpass
@pass fbh
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_a;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float glow;
    float heat;
    float speed;
};
void main() {
    float sigma = max(8.0, 30.0 * glow);
    float acc = 0.0;
    float wsum = 0.0;
    for (int i = -24; i <= 24; i++) {
        float x = float(i) * sigma / 24.0;
        float w = exp(-0.5 * (x / sigma) * (x / sigma) * 9.0);
        vec2 off = vec2(x / u_resolution.x, 0.0);
        acc += texture(sampler2D(u_a, u_s), v_uv + off).r * w;
        wsum += w;
    }
    outColor = vec4(vec3(acc / wsum), 1.0);
}
@endpass
@pass fbv
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_a;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float glow;
    float heat;
    float speed;
};
void main() {
    float sigma = max(8.0, 30.0 * glow);
    float acc = 0.0;
    float wsum = 0.0;
    for (int i = -24; i <= 24; i++) {
        float x = float(i) * sigma / 24.0;
        float w = exp(-0.5 * (x / sigma) * (x / sigma) * 9.0);
        vec2 off = vec2(0.0, x / u_resolution.y);
        acc += texture(sampler2D(u_a, u_s), v_uv + off).r * w;
        wsum += w;
    }
    outColor = vec4(vec3(acc / wsum), 1.0);
}
@endpass
@pass blurh
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_a;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float glow;
    float heat;
    float speed;
};
void main() {
    float sigma = max(6.0, 42.0 * glow);
    float acc = 0.0;
    float wsum = 0.0;
    for (int i = -24; i <= 24; i++) {
        float x = float(i) * sigma / 24.0;
        float w = exp(-0.5 * (x / sigma) * (x / sigma) * 9.0);
        vec2 off = vec2(x / u_resolution.x, 0.0);
        acc += texture(sampler2D(u_a, u_s), v_uv + off).a * w;
        wsum += w;
    }
    outColor = vec4(vec3(acc / wsum), 1.0);
}
@endpass
@pass blurv
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_a;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float glow;
    float heat;
    float speed;
};
void main() {
    float sigma = max(6.0, 42.0 * glow);
    float acc = 0.0;
    float wsum = 0.0;
    for (int i = -24; i <= 24; i++) {
        float x = float(i) * sigma / 24.0;
        float w = exp(-0.5 * (x / sigma) * (x / sigma) * 9.0);
        vec2 off = vec2(0.0, x / u_resolution.y);
        acc += texture(sampler2D(u_a, u_s), v_uv + off).r * w;
        wsum += w;
    }
    outColor = vec4(vec3(acc / wsum), 1.0);
}
@endpass
@pass comp
#version 450
// @param glow label:"Glow Radius" min:0.2 max:3 default:1.2
// @param heat label:"Core Heat" min:0 max:2 default:1
// @param speed label:"Heat Speed" min:0 max:4 default:1
// @param grain label:"Grain" min:0 max:0.2 default:0.05
// @param body_deep label:"Body Deep" hint:color default:#2B1B6B
// @param body_main label:"Body Main" hint:color default:#C0357A
// @param body_light label:"Body Light" hint:color default:#F27C38
// @param hot_color label:"Hot Core" hint:color default:#FFD84D
// @param rim_color label:"Rim" hint:color default:#63D9FF
// @param glow_color label:"Glow" hint:color default:#FF8A3D
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_glyph;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float glow;
    float heat;
    float speed;
    float grain;
    vec4 body_deep;
    vec4 body_main;
    vec4 body_light;
    vec4 hot_color;
    vec4 rim_color;
    vec4 glow_color;
};
layout(set = 0, binding = 3) uniform texture2D u_soft;
layout(set = 0, binding = 4) uniform texture2D u_field;
float dhash(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123);
}
vec3 palette(float t) {
    vec3 c0 = body_deep.rgb * 0.25;
    vec3 c1 = body_deep.rgb;
    vec3 c2 = body_main.rgb;
    vec3 c3 = body_light.rgb;
    vec3 c4 = mix(body_light.rgb, hot_color.rgb, 0.65);
    vec3 c5 = hot_color.rgb;
    vec3 c6 = clamp(hot_color.rgb * 0.6 + vec3(0.5), 0.0, 1.2);
    t = clamp(t, 0.0, 1.0);
    if (t < 0.30) { return mix(c0, c1, t / 0.30); }
    if (t < 0.52) { return mix(c1, c2, (t - 0.30) / 0.22); }
    if (t < 0.68) { return mix(c2, c3, (t - 0.52) / 0.16); }
    if (t < 0.80) { return mix(c3, c4, (t - 0.68) / 0.12); }
    if (t < 0.90) { return mix(c4, c5, (t - 0.80) / 0.10); }
    return mix(c5, c6, (t - 0.90) / 0.10);
}
void main() {
    float a = texture(sampler2D(u_glyph, u_s), v_uv).a;
    float s = texture(sampler2D(u_soft, u_s), v_uv).r;
    float fldRaw = texture(sampler2D(u_field, u_s), v_uv).r;
    float fld = smoothstep(0.12, 0.78, fldRaw);
    float band = smoothstep(0.06, 0.50, s) * (1.0 - smoothstep(0.50, 0.94, s));
    float inner = 0.26 + 0.60 * fld + 0.16 * band;
    vec3 body = palette(inner) * a;
    vec3 rimCol = mix(rim_color.rgb, hot_color.rgb, fld);
    vec3 rim = rimCol * band * 1.0;
    float halo = pow(clamp(s * 1.15, 0.0, 1.0), 0.45) * (1.0 - a);
    vec3 glowCol = glow_color.rgb * 0.85 * (1.0 - a);
    vec3 col = body + rim + glowCol;
    float g = dhash(v_uv * u_resolution + vec2(u_frame * 7.3, u_frame * 3.1));
    col *= 1.0 + (g - 0.5) * grain;
    col += (g - 0.5) * (2.0 / 255.0);
    float outA = clamp(a + halo * 0.9 + band * 0.5, 0.0, 1.0);
    outA = clamp(outA + (g - 0.5) * (2.0 / 255.0), 0.0, 1.0);
    outColor = vec4(col, outA);
}
@endpass
