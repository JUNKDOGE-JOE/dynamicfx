#version 450
// DynamicFX example — canvas expansion (ADR-0039).
//
// `reach` is both the visual halo radius AND the canvas authority: the
// `hint:canvas` annotation grows the layer's render canvas by that many
// logical pixels per side, so the halo is never clipped at the layer edge.
// Remove the annotation and the same shader clips exactly like 0.0.5 did
// (or expands to whatever an upstream Grow Bounds provides).
//
// @param reach label:"Reach (px)" min:0 max:512 default:160 hint:canvas
// @param halo_color label:"Halo" hint:color default:#FF7A1A
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float reach;
    vec4 halo_color;
};

void main() {
    vec4 base = texture(sampler2D(u_in, u_s), v_uv);

    // Ring-march the input's alpha out to `reach` logical pixels. The halo
    // is the strongest alpha seen, faded by the distance it was found at —
    // a cheap dilate that makes the canvas boundary directly visible.
    float px = reach / max(u_resolution.x, 1.0);
    float py = reach / max(u_resolution.y, 1.0);
    float glow = 0.0;
    const int RINGS = 12;
    const int TAPS = 16;
    for (int r = 1; r <= RINGS; r++) {
        float t = float(r) / float(RINGS);
        float falloff = 1.0 - t;
        for (int a = 0; a < TAPS; a++) {
            float ang = 6.2831853 * (float(a) + 0.5 * float(r % 2)) / float(TAPS);
            vec2 off = vec2(cos(ang) * px, sin(ang) * py) * t;
            float alpha = texture(sampler2D(u_in, u_s), v_uv + off).a;
            glow = max(glow, alpha * falloff);
        }
    }

    vec3 col = base.rgb + halo_color.rgb * glow * (1.0 - base.a);
    float a_out = max(base.a, glow * halo_color.a);
    outColor = vec4(col, a_out);
}
