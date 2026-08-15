@dynamicfx 1
@graph
pass trail: input, prev -> output
@end
@pass trail
#version 450
// @window 16
// @param spin label:"Orbit Speed" min:0 max:4 default:1
// @param radius label:"Orb Size" min:0.01 max:0.5 default:0.08
// @param orbit label:"Orbit Radius" min:0 max:0.6 default:0.28
// @param decay label:"Trail Persistence" min:0 max:0.99 default:0.86
// @param sweep label:"Start Angle" hint:angle
// @param over label:"Composite Over Layer" hint:bool
// @param core_color label:"Core" hint:color default:#FFF2CC
// @param trail_color label:"Trail" hint:color default:#3FA9FF
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float spin;
    float radius;
    float orbit;
    float decay;
    float sweep;
    int over;
    vec4 core_color;
    vec4 trail_color;
};
layout(set = 0, binding = 3) uniform texture2D u_prev;
void main() {
    // Square up the coordinate system so the orbit is a circle, not an
    // ellipse, on non-square comps.
    float aspect = u_resolution.x / max(u_resolution.y, 1.0);
    vec2 p = (v_uv - 0.5) * vec2(aspect, 1.0);

    // Angles arrive in degrees (ABI v1); u_time is seconds.
    float ang = radians(sweep) + u_time * spin * 2.0;
    vec2 c = vec2(cos(ang), sin(ang)) * orbit;

    float r = max(radius, 1e-4);
    float d = length(p - c);
    float core = exp(-pow(d / r, 2.0) * 2.5);
    float halo = exp(-d / r * 1.6) * 0.55;
    vec3 fresh = core_color.rgb * core + trail_color.rgb * halo;

    // The trail: this frame's orb over a faded copy of the previous frame.
    // `prev` makes the effect temporal — DynamicFX re-simulates the last
    // `@window` frames for every request, so scrubbing, the render queue,
    // and aerender all agree exactly.
    vec3 faded = texture(sampler2D(u_prev, u_s), v_uv).rgb * decay;
    vec3 trail = max(faded, fresh);

    vec4 base = texture(sampler2D(u_in, u_s), v_uv);
    float lum = clamp(max(max(trail.r, trail.g), trail.b), 0.0, 1.0);
    vec3 col = over != 0 ? base.rgb + trail : trail;
    float a = over != 0 ? max(base.a, lum) : lum;
    outColor = vec4(col, a);
}
@endpass
