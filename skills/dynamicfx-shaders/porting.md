# DynamicFX Porting Guide

Guidance for converting existing shaders (chiefly Shadertoy) into DynamicFX's `@dynamicfx` envelope + ABI v1 format. Read this after SKILL.md's canonical template and validation checklist.

## Symbol mapping

| Source (Shadertoy / generic GLSL) | DynamicFX equivalent | Notes |
|---|---|---|
| `void mainImage(out vec4 fragColor, in vec2 fragCoord)` | `void main()` writing `outColor` | Signature and output variable both change |
| `fragCoord` | `v_uv * u_resolution` | Only needed if pixel-space coords are required; most logic can work directly in UV space |
| `fragCoord / iResolution.xy` | `v_uv` | Same normalized UV, use `v_uv` directly instead of recomputing |
| `iResolution.xy` | `u_resolution` | `iResolution` is `vec3` in Shadertoy; DynamicFX has no `.z` (pixel aspect) equivalent — usually safe to drop |
| `iTime` | `u_time` | Seconds, layer time |
| `iFrame` | `int(u_frame)` | `u_frame` is a float; cast if an int is needed |
| `texture(iChannel0, uv)` / `texture2D(iChannel0, uv)` | `texture(sampler2D(u_in, u_s), uv)` | `u_in`/`u_s` are separate texture/sampler objects combined at the call site |
| Extra `iChannel1..3` | Extra pass input (binding 3/4/5) or a `hint:layer` parameter | Depends on whether the source is another pass in the same graph or a user-selected layer — see "Structural classification" below |
| `iMouse` | A `// @param` `vec2` control (0..1 point) | DynamicFX has no live pointer input; expose mouse-driven values as a keyframeable parameter instead. Multiply by `u_resolution` if pixel space is needed |
| `iDate`, audio (`iChannel` waveform/FFT textures), keyboard textures, cubemaps | No equivalent | Flag for manual decision; cannot be auto-ported |
| `iChannelResolution[n]` | `textureSize(sampler2D(u_in, u_s), 0)` (or the relevant input's sampler) | |
| `precision ...;` qualifiers | Delete | Not used in GLSL 450 core-profile ABI |
| `texture2D(sampler, uv)` (GLSL ES 1.00 call form) | `texture(sampler2D(tex, samp), uv)` | Also add `#version 450` as the first line of the pass body |

## Coordinate system note

Shadertoy's `fragCoord`/`iResolution` convention places the origin at the bottom-left. Whether DynamicFX's `v_uv` origin matches is not documented upstream — **verify visually after conversion**. If the image appears vertically flipped, apply `uv.y = 1.0 - uv.y` as an empirical fix, not as an assumed default.

**Canvas size is the layer, not the screen.** A Shadertoy shader paints the whole viewport; in DynamicFX the equivalent is the layer's own frame, and nothing outside it exists (SKILL.md → "Canvas = the layer's own frame"). Ports of bloom/glow/halo/shadow or displacement effects should declare their reach with `hint:canvas` (0.0.6+, SKILL.md → "Canvas expansion"); on older installs precompose the source with transparent margin instead.

## Structural classification

Classify each source shader before converting:

| Source structure | DynamicFX target |
|---|---|
| Single Shadertoy `mainImage`, no extra buffers | Single-pass envelope (`pass main: input -> output`) |
| Multiple Shadertoy buffers (Buffer A, B, C, D feeding the main image) | Multi-pass `@graph`: one `@pass` per buffer, buffer names become intermediate pass/texture names, main image becomes the pass that writes `output` |
| A buffer reads its own previous frame (feedback loop) | `prev` input on that pass + `// @window N`. Note this is a **windowed re-simulation**, not infinite accumulation: history beyond `N` frames (max 64) disappears every re-render. Evaluate whether the original effect depends on unbounded history — if so, flag this as a semantic gap rather than silently truncating it. `prev` cannot be combined with a `layer`/`path` input in the same graph (E7). If the buffer never samples the source layer, `prev` may be the pass's only input (`pass main: prev -> output`) |
| Uses `iMouse`, keyboard, or audio input | Downgrade interactive input to a `@param` (mouse → point parameter) or mark as **not portable** if there's no reasonable static/keyframeable substitute |

## Parameterization conventions

When porting, scan the source for magic constants that a user would plausibly want to tune: speeds, intensities, radii, thresholds, colors. Promote each to a `// @param`:

- Numeric magic constant `X` → `min:0 max:<4×X> default:<X>` as a starting range (empirical convention, not a spec rule — adjust to taste).
- Color constants (`vec3(r,g,b)` literals used as a fixed color) → `hint:color default:#RRGGBB`.
- Respect the per-type parameter pool ceilings (see reference.md). If a port would exceed a pool, merge related constants into fewer parameters or drop the least useful ones — do not attempt to "fit" everything and trigger `E32`.
- **Place members per pass (0.0.6+).** Give each pass its own `FxUniforms` block containing only the heads plus what that pass reads — the panel groups parameters under the pass that exclusively owns them, and copy-pasting one mega-block into every pass collapses the whole panel into `Main`. Annotate each `@param` name exactly once across the source (a second annotation is `E19`); repeating the MEMBER in several blocks is fine and makes the parameter shared/`Main`.
- **If the effect paints beyond the source pixels** (glow/bloom/halo/shadow/displacement — most Shadertoy full-viewport looks do once confined to a layer), pick its reach parameter and add `hint:canvas` to it so the port isn't clipped at the layer frame. Multi-pass ports: name passes for humans — the names are the panel group headers.

## Worked example

Input (Shadertoy):

```glsl
void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    vec2 uv = fragCoord / iResolution.xy;
    vec4 tex = texture(iChannel0, uv);
    float wave = sin(uv.x * 10.0 + iTime * 2.0) * 0.5 + 0.5;
    float speed = 2.0;
    fragColor = vec4(mix(tex.rgb, vec3(wave, 0.2, 1.0 - wave), 0.3), tex.a);
}
```

Mapping applied:
- `mainImage(out fragColor, in fragCoord)` → `void main()` / `outColor`
- `fragCoord / iResolution.xy` → `v_uv` directly (already normalized)
- `iChannel0` sampling → `texture(sampler2D(u_in, u_s), uv)` (the layer itself is the only input, bound as `u_in`)
- `iTime` → `u_time`
- local `float speed = 2.0;` constant → promoted to a `// @param` slider (`min:0 max:8 default:2`, range = 4x convention) and referenced as `speed` from `FxUniforms`
- `tex.a` preserved unchanged
- wrapped in the full envelope, ABI header, and `` `;0 `` expression suffix

Output (final DynamicFX `Source` expression):

```glsl
`@dynamicfx 1
@graph
pass main: input -> output
@end
@pass main
#version 450
// @param speed label:"Speed" min:0 max:8 default:2
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float speed;
};
void main() {
    vec2 uv = v_uv;
    vec4 tex = texture(sampler2D(u_in, u_s), uv);
    float wave = sin(uv.x * 10.0 + u_time * speed) * 0.5 + 0.5;
    outColor = vec4(mix(tex.rgb, vec3(wave, 0.2, 1.0 - wave), 0.3), tex.a);
}
@endpass
`;0
```

## Batch conversion workflow

When porting many shaders at once:

1. **Inventory**: list every input file/snippet. For each, classify per the "Structural classification" table above (single-pass / multi-pass / feedback / needs-interactive-downgrade / not-portable).
2. **Convert individually**: for each shader, produce the complete envelope source and save it as `<name>.glsl`. State the delivery format explicitly: the file's full content wrapped as `` `<file-content>`;0 `` is what gets pasted into the `Source` expression.
3. **Validate each**: run every converted file through SKILL.md's Validation checklist before considering it done.
4. **Report**: summarize results in three buckets — converted successfully, converted with a flagged manual decision (e.g. windowed feedback, downgraded mouse input), and not portable (no DynamicFX equivalent exists). Never silently change semantics (e.g. truncating infinite feedback history, dropping mouse interactivity) without calling it out to the user.
