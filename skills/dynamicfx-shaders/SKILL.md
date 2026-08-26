---
name: dynamicfx-shaders
description: Use when writing, converting, or debugging shaders for the DynamicFX After Effects plugin — porting Shadertoy/GLSL code to it, authoring @dynamicfx envelope shaders from scratch, declaring AE parameter controls (@param), organizing parameters into per-pass panel groups or declaring the canvas boundary (hint:canvas, 0.0.6+), when a Source expression fails to compile or is rejected by AE (E6/E7/E18/E19/E32/E53/E55/E56/E57), when output (glow, halo, shadow, displacement) is cut off at the layer's edges, or when scripting DynamicFx parameters from ExtendScript.
---

# DynamicFX Shaders

## Overview

DynamicFX turns a GLSL shader into a real GPU-accelerated After Effects effect by embedding it as text inside an expression on the effect's numeric `Source` parameter. There is no shader editor UI, no text field, no code panel — the **committed expression string is the single source of truth**.

Core mental model: the final deliverable is always one expression of the shape

```
`<envelope-source>`;0
```

— a backtick-delimited template string containing the shader source, followed by `;0` so the expression evaluates to a number (required because `Source` is a numeric parameter; AE rejects the expression without it).

Applying it: Effect > DynamicFx > DynamicFx, then Alt-click the `Source` stopwatch and paste the wrapped string into the expression editor.

## These do NOT exist in DynamicFX

An AI without this skill reliably invents the following. None of them are real. Do not use them.

| Invented / assumed | Reality |
|---|---|
| `mainImage(out vec4 fragColor, in vec2 fragCoord)` Shadertoy entry point, "compatibility mode" | Every pass is plain GLSL 450: `void main()` writing to `outColor` (see ABI header below). There is no Shadertoy compatibility layer. |
| `iTime`, `iResolution`, `iChannel0`, `iMouse`, `iFrame` as built-ins | None of these exist. Map them: `iTime`→`u_time`, `iResolution.xy`→`u_resolution`, `iChannel0` sampling →`texture(sampler2D(u_in, u_s), uv)`, `iMouse`→a `@param` vec2 point control, `iFrame`→`int(u_frame)`. |
| `// @slider min max default` or similar shorthand annotation | The real syntax is `// @param <id> label:"..." min:0 max:10 default:2` (see @param table below). Misspelled entries reject the whole definition — there is no lenient shorthand. |
| Standalone `uniform float speed;` declarations | User parameters MUST be fields inside the single `FxUniforms` uniform block, after the three reserved fields `u_resolution`, `u_time`, `u_frame`. A parameter declared outside the block is not a parameter. |
| A "Shader Code" text parameter, an "Edit Code" button, a code panel in the Effect Controls UI | Does not exist. The only place shader text lives is inside the `Source` expression string, wrapped `` `...`;0 ``. |
| An "Uniform Bindings" panel, or binding parameters via Slider Control layers/expressions | Does not exist. All parameter binding is declared in-shader via `// @param` comments; AE builds the UI controls from those comments automatically. |
| Omitting `#version 450` | Every `@pass` body must start with `#version 450`. Without it the pass will not compile. |

## Canonical template

This is the upstream Quick Start example, verbatim. Use it as the starting skeleton for any from-scratch shader.

```glsl
`@dynamicfx 1
@graph
pass main: input -> output
@end
@pass main
#version 450
// @param speed label:"Speed" min:0 max:4 default:1
// @param tint label:"Tint" hint:color default:#31C6FF
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float speed;
    vec4 tint;
};
void main() {
    vec4 base = texture(sampler2D(u_in, u_s), v_uv);
    vec3 ramp = tint.rgb * (0.5 + 0.5 * sin(u_time * speed + v_uv.x * 3.0));
    outColor = vec4(mix(base.rgb, ramp, 0.85), 1.0);
}
@endpass
`;0
```

Line-by-line, what's fixed vs. what you edit:

| Line(s) | Fixed (never change) | Editable |
|---|---|---|
| `` `@dynamicfx 1 `` | Literal text and version `1` | — |
| `@graph` / `@end` | Literal | The `pass ...` line(s) between them |
| `pass main: input -> output` | Keyword order `pass NAME: INPUT[, INPUT] -> OUTPUT` | `main`, input list, output name (as long as an envelope-consistent name) |
| `@pass main` / `@endpass` | Literal, name must match graph | — |
| `#version 450` | Literal, must be first line of pass body | — |
| `// @param ...` lines | `// @param` prefix and entry syntax | id, label, min/max/default/hint/alias values |
| `layout(...) in/out/uniform ...` block through `};` | Every line except the user-parameter fields added inside `FxUniforms` | Which extra fields you add inside `FxUniforms`, matched 1:1 with `@param` lines |
| `void main() { ... }` | Function signature `void main()`, writing to `outColor` | Everything inside the body |
| `` `;0 `` | Literal, mandatory suffix | — |

## Authoring from scratch — checklist

1. **Envelope**: write `@dynamicfx 1` as line 1, then `@graph` ... `pass main: input -> output` ... `@end`. Add more `pass` lines if using multiple passes, `prev`, or extra inputs.
2. **Graph**: for each pass, declare `pass NAME: IN1[, IN2...] -> OUT`. Exactly one pass across the whole graph must write `output`. Do not use `input`, `output`, or `prev` as a pass name.
3. **`@pass` blocks**: one `@pass NAME` / `@endpass` pair per graph entry, same names, same order not required but names must match exactly.
4. **ABI header**: in every pass body, `#version 450` first, then the fixed `v_uv` in/`outColor` out layout lines, then `u_in`/`u_s` at binding 0/1, then `FxUniforms` at binding 2 with `u_resolution, u_time, u_frame` first and in that order.
5. **`@param` for every user-facing value**: one `// @param` comment per uniform field you add to `FxUniforms`, placed near its declaration. Never leave a magic number un-parameterized if the user should be able to animate/tweak it.
6. **Extra inputs**: any additional pass input beyond the first binds sequentially at binding 3, 4, 5 (manifest order). `hint:layer`/`hint:gradient`/`hint:path` parameters are separate from this — see reference.md.
7. **Body**: write the GLSL logic, sampling with `texture(sampler2D(u_in, u_s), uv)`.
8. **Wrap**: enclose the whole thing in backticks and append `;0`. Paste into the `Source` expression.

## Parameter groups (0.0.6+): the panel follows your uniform blocks

Since 0.0.6 the Effect Controls panel is grouped: `Setup` (Language/Source/Compile/Status/Details, always expanded) → `Main` (collapsed; shared parameters, gradients nested inside) → one group per pass, shown under the pass's OWN NAME from `@graph` and hidden while empty. **You control the grouping entirely through uniform-block membership:**

- A parameter whose member appears in **exactly one pass's** `FxUniforms` block (in a graph of ≥2 passes) lands in **that pass's group**.
- A member repeated in two or more blocks — or any parameter of a single-pass shader — lands in **Main**.
- `layer` / `gradient` / `point3d` / `path` parameters are always Main.

### The authoring pattern (and the anti-pattern)

Give every pass its OWN block containing only the three reserved heads plus what that pass actually reads:

```glsl
`@dynamicfx 1
@graph
pass warm: input -> t
pass cool: t -> output
@end
@pass warm
#version 450
// @param shared_gain label:"Shared Gain" min:0 max:2 default:1
// @param warm_tint label:"Warm Tint" min:0 max:1 default:0.8
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float shared_gain;
    float warm_tint;
};
void main() { vec4 c = texture(sampler2D(u_in, u_s), v_uv);
    outColor = vec4(c.r * shared_gain + warm_tint * 0.2, c.g, c.b, c.a); }
@endpass
@pass cool
#version 450
// @param cool_tint label:"Cool Tint" min:0 max:1 default:0.6
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;
layout(set = 0, binding = 1) uniform sampler u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float shared_gain;
    float cool_tint;
};
void main() { vec4 c = texture(sampler2D(u_in, u_s), v_uv);
    outColor = vec4(c.r, c.g, c.b * shared_gain + cool_tint * 0.2, c.a); }
@endpass
`;0
```

Panel result (verified live): `Main` holds Shared Gain; a group named **warm** holds Warm Tint; a group named **cool** holds Cool Tint; nothing else shows. Note `shared_gain`'s MEMBER is in both blocks but its `@param` line appears ONCE — **`@param` annotations are source-wide unique per name; re-declaring one in a second pass is `E19` dup**. The member repeats freely; the annotation never does.

The anti-pattern is copy-pasting one mega-block into every pass (the pre-0.0.6 habit): every parameter then counts as shared → the whole panel collapses into Main, and every pass uploads the full block. Slim blocks group better AND run leaner.

### Bank capacities, spill, and structure limits

Each of the FIRST TWELVE passes gets a private bank: 8 floats, 2 ints, 2 checkboxes, 3 colors, 2 points, 1 angle. A pass's exclusive params beyond its bank **spill to Main** gracefully (the Status line reports "(N spilled to Main)") — never a compile error. Consequences for design:

- A mega-pass that exclusively owns 30+ params (a do-everything "finish" composite) will show a partial group + a Main overflow. That's working-as-designed, but if grouping matters to the look's UX, distribute the LOGIC across the passes that already exist — never add passes just for grouping (see Pass economy).
- Angles are the scarcest bank resource (1 per pass); a pass with five angle knobs keeps four in Main.
- Passes 13+ of a 16-pass graph get no bank (params → Main).
- Name your passes for humans: the group header IS the pass name (31-char cap). `pass glow:` reads better than `pass p7:`.

### Stability rule (why an edited shader may look "un-grouped")

A parameter that already owns a slot **keeps it across source edits** — keyframes outrank regrouping. Re-shuffling members between blocks on a live instance will NOT visually move already-bound rows; the grouped layout fully applies to fresh binds. To re-group an existing tuned instance: capture values by LABEL, delete the effect, re-apply, re-commit, restore by label (worked recipe in reference.md → "Reshaping an existing instance").

## Quick reference

**Reserved uniforms (fixed order, always present):**

| Name | Type | Meaning |
|---|---|---|
| `v_uv` | `vec2` (in) | 0..1 UV covering the CANVAS — the layer's own frame unless expanded (see "Canvas expansion"); nothing exists outside it |
| `outColor` | `vec4` (out) | pass output color |
| `u_in` | `texture2D`, binding 0 | first pass input |
| `u_s` | `sampler`, binding 1 | sampler for all texture inputs |
| `u_resolution` | `vec2`, in `FxUniforms` | logical full-resolution size of the CANVAS (= layer frame when unexpanded; constant across preview res) |
| `u_time` | `float`, in `FxUniforms` | layer time in seconds |
| `u_frame` | `float`, in `FxUniforms` | layer time in frames |

**Reserved pass/graph names:** `input` (source layer), `output` (must be written by exactly one pass), `prev` (previous frame's output, feedback only). None of these may be used as a custom pass name or as an output target for a parameter-fed resource (layer/gradient/path) — violating this is `E6`. `dfx_` is a reserved identifier prefix — do not name anything with it. A pass need not read `input` at all — `prev` may be a pass's only input (e.g. `pass gen: prev -> t`). An empty input list (`pass name: -> out`) is invalid (E6), and `prev` can never be written to.

**`@param` entries:** `label:"text"`, `min:`, `max:` (slider range, give both — a display range, not a clamp; the shader receives the row's value as shown, so `clamp()` in the shader if a hard limit matters), `default:<value>` (number, `#RRGGBB`, or `#RRGGBBAA` for `hint:color`), `alias:<old-id>` (renames while preserving keyframes), `hint:angle|color|bool|layer|gradient|point3d|path`. Any misspelled entry (e.g. `mim:0`) rejects the whole `@param` definition — parsing is fail-closed, never silently ignored. Full type-mapping and pool-capacity table: see reference.md. Declare real ranges (`min:2 max:200`): the 0.0.3 habit of declaring everything `min:0 max:1` and remapping inside the shader was a workaround for a since-fixed render-side clamp (ADR-0037), not a rule.

**Capacity limits:** 16 passes max, 4 inputs per pass max, 15 intermediate textures max, 4 MiB max source size. Per-type parameter pool ceilings are in reference.md; exceeding any one pool rejects the whole shader with `E32`.

**Feedback (`prev`):** requires `// @window N` (default 16, max 64) inside the pass that consumes `prev`. Each frame is deterministically re-simulated from black for `min(frame+1, N)` iterations — preview, render queue, and `aerender` all match bit-for-bit. `prev` cannot coexist with `layer`/`path` inputs in the same graph (`E7`).

**Error codes:**

| Code | Meaning |
|---|---|
| E6 | Envelope syntax violation (reported with line number), or a parameter-fed resource (layer/gradient/path) used as a pass name/output |
| E7 | `prev` feedback combined with a `layer`/`path` input in the same graph |
| E18 | A texture binding declared in a pass that the `@graph` manifest doesn't feed |
| E19 | An `@param` line is malformed — or the same parameter name is annotated in more than one pass (annotations are source-wide unique; repeat the MEMBER, never the annotation) |
| E32 | A parameter pool exceeded its slot ceiling |
| E53 | PublicationPending — compiled but not yet published; not renderable yet |
| E55 | Two `hint:canvas` declarations in one source (exactly one allowed) |
| E56 | `hint:canvas` on a non-float parameter |
| E57 | Canvas expansion past the GPU texture limit (falls back to the layer frame; logged, never a crash) |

Full diagnostics (Status line, Show Full Status, log file, scripting readiness poll) are in reference.md.

## Canvas expansion (0.0.6+): `hint:canvas`, or an upstream Grow Bounds

Since **0.0.6** a shader can own its canvas. Declare exactly ONE float `@param` with `hint:canvas` and the drawable area becomes the layer frame grown by that parameter's value in logical pixels per side (keyframeable); the shader still reads it as an ordinary uniform, so a glow radius can be its own canvas authority — `examples/reach-ring.glsl` is the pattern. With NO declaration, the canvas is the layer frame **unioned with whatever an upstream buffer-expanding effect provides** (a Grow Bounds above DynamicFx now works; before 0.0.6 it was ignored — TR-BOUNDS-001). A declaration REPLACES the upstream signal: the author's boundary wins, even under a bigger Grow Bounds.

What the shader sees on an expanded canvas is exactly a padded precomp: `u_resolution`/`v_uv` span the canvas, the input sits centered with transparent margins (not clamp-to-edge), and points / 3D points / mask vertices stay on the same visual pixels. Diagnostics: `E55` two canvas declarations, `E56` `hint:canvas` on a non-float, `E57` expansion past the GPU texture limit (falls back to the plain layer frame, logged). Costs: VRAM/render time scale with canvas area, and changing the canvas size resets temporal (`@window`) history. On builds **before 0.0.6** none of this exists — the canvas is the bare layer frame and only a padded precomp helps.

What to do:

- **Declare the reach as the canvas.** `// @param reach label:"Reach (px)" min:0 max:512 default:160 hint:canvas` — one line replaces the whole padded-precomp workaround. (The precomp remains the only route on pre-0.0.6 installs: source into a comp larger by the shader's maximum reach on every side, effect onto the precomp.)
- **Expose reach as pixel `@param`s** (`label:"Glow Radius (px)"`) so the needed margin is explicit, and state it in the shader's header comment: `// needs ≥ N px of transparent margin around the source`.
- **Build content from the source, not from the canvas.** Derive geometry from the input's alpha (blurred/distance fields of `texture(sampler2D(u_in, u_s), uv).a`) and pixel sizes from `u_resolution`, never from fixed `uv` constants — then the same shader looks the same on the bare layer and on its padded precomp, only with room to breathe. Padding changes the inside too: blur/halo passes near the edge sample the margin instead of clamp-to-edge pixels (measured mean |diff| 20/255 inside the logo between the clipped and the padded instance).
- When a user reports "the effect is cut off at the edges", ask what the layer's own size is before touching the shader.

## Pass economy (previews feel slow = count the passes first)

Frame cost ~ `passes x canvas area` (every pass is a full-canvas draw;
`@window` multiplies by the temporal window). Author with as FEW passes as
the algorithm truly needs: merge stages that never read an intermediate
result (chained per-pixel color math is one pass, not three); keep passes
that earn their keep (a separable blur's H/V pair beats one-pass O(r^2) —
never merge those); question octave counts (does the third blur octave read
on screen?). A declared `hint:canvas` reach raises every pass's price —
default it to what the look needs. Per-pass downsampled intermediates do not
exist in v1; a lower-resolution pyramid must be faked with fewer octaves.

## Validation checklist

Run this before delivering any DynamicFX shader (new or ported):

- [ ] First line is exactly `@dynamicfx 1`
- [ ] Exactly one pass across the whole graph writes `output`
- [ ] Every `@pass NAME` has a matching `pass NAME: ...` line in `@graph`, and vice versa
- [ ] Every pass body starts with `#version 450`
- [ ] ABI header present in every pass, with `u_resolution`, `u_time`, `u_frame` first and in that order inside `FxUniforms`
- [ ] Every user-tunable value is a field inside `FxUniforms` (never a standalone `uniform`) and has a matching `// @param` comment
- [ ] Extra pass inputs bind sequentially at binding 3, 4, 5 in the same order they appear in the `@graph` manifest line
- [ ] If a pass uses `prev`, it has `// @window N` (N ≤ 64) and the graph has no `layer`/`path` input
- [ ] No `input`/`output`/`prev` used as a custom pass name; no `dfx_`-prefixed identifiers
- [ ] Entire source is wrapped in backticks with the final expression ending `` `;0 ``
- [ ] If the shader paints beyond the source pixels (glow/halo/shadow/displacement), it declares `hint:canvas` on its reach parameter (0.0.6+), or the delivery states the padded-precomp margin for older installs
- [ ] Multi-pass: each pass's `FxUniforms` holds only the heads + members that pass reads (grouping follows block membership); every `@param` name is annotated exactly once across the whole source
- [ ] Pass names are human-readable (they are the panel group headers) and the pass count is the algorithm's minimum (see Pass economy)

## Common mistakes

| Symptom | Cause | Fix |
|---|---|---|
| AE rejects the expression outright | Missing trailing `;0`, or unmatched backtick | End with `` `;0 `` exactly; verify only one opening and one closing backtick around the whole source |
| Compile error citing `mainImage` or "no main function" | Kept a Shadertoy `mainImage` signature | Rewrite as `void main()` writing `outColor`, per the ABI header |
| Undeclared identifier `iTime`/`iResolution`/`iChannel0` | Left Shadertoy built-ins in place | Map to `u_time` / `u_resolution` / `texture(sampler2D(u_in, u_s), uv)` — see porting.md |
| `@param` line silently has no effect / whole shader rejected | Typo'd entry key (`mim:`, `lable:`) | Fix the entry key; parsing is fail-closed, re-check spelling against reference.md's entry table |
| Parameter doesn't appear in Effect Controls | Declared a standalone `uniform`, not a field inside `FxUniforms` | Move the field inside `FxUniforms`, keep the `// @param` comment near it |
| `E6` on submit | Envelope malformed: pass name mismatch, missing `@end`/`@endpass`, no pass writing `output`, or reserved name misused | Re-check `@graph` vs. `@pass` names line by line; confirm exactly one `-> output` |
| `E7` on submit | `prev` used together with a `layer` or `path` input in the same graph | Remove one of the two, or split into a graph that doesn't mix them |
| `E18` on submit | A `layout(... binding = N) uniform texture2D` declared for an input the `@graph` line doesn't list | Add the missing input to the pass's manifest line, matching binding order |
| `E32` on submit | Too many parameters of one type (see reference.md pool table) | Merge or drop parameters of that type; pools are per-type, not shared |
| Status shows `E53` / effect passes through unrendered | PublicationPending — shader compiled but not yet published; a few-second window after submit | Wait; for scripting, poll readiness on the `"State Token (internal)"` row BY NAME (`value % 4 === 1`) instead of assuming immediate readiness — see reference.md |
| Shader looks vertically flipped vs. the original | Coordinate origin convention differs from the source (e.g. Shadertoy) | Try `uv.y = 1.0 - uv.y`; verify visually, there is no documented universal rule |
| Output cut off in a hard square/rectangle at the layer's edges (halo, glow, shadow, displaced pixels vanish) | The canvas defaults to the layer's own frame | 0.0.6+: declare `hint:canvas` on the reach parameter (or put a Grow Bounds above an undeclared shader); pre-0.0.6: precompose with transparent margin — see "Canvas expansion" |
| `E19 @param line N: dup` on a multi-pass source | The same parameter name is annotated in more than one pass | Keep ONE `@param` line per name (anywhere in the source); repeat only the uniform MEMBER in other blocks |
| Nine passes but only a few groups in the panel | Groups follow parameter EXCLUSIVITY, not pass existence — passes with no exclusive params have empty (auto-hidden) groups; shared params live in Main | Working as designed; to populate a pass's group, declare its params only in that pass's block |
| Panel shows fewer grouped rows than expected after editing a live instance | Bound parameters keep their slots across source edits (keyframes outrank regrouping) | Re-add the effect fresh and restore values by label — recipe in reference.md |
| ExtendScript that worked pre-0.0.6 now throws / reads wrong rows | Numeric property indexes shifted once in 0.0.6 (group rows occupy positions) | Address parameters BY NAME (names/matchNames are stable); never hardcode indexes — see reference.md scripting section |

## Further reading

**Converting existing shaders? REQUIRED: read porting.md** — symbol mapping table, structural decisions (single vs. multi-pass, feedback loops), parameterization conventions, a full worked Shadertoy→DynamicFX example, and the batch-conversion workflow.

**Full parameter tables, limits, scripting, diagnostics: reference.md** — complete `@param` type/hint mapping with pool capacities, gradient/path texture read patterns, all capacity limits, full E-code table, diagnostic channels, scripting readiness polling, plugin install/platform support.
