# DynamicFX

**An open shader runtime for After Effects.** Write GLSL in an ordinary
expression, get a real GPU effect — multi-pass render graphs, keyframeable
parameters, temporal feedback, 8/16/32-bpc, all through normal AE workflows.

DynamicFX is a native AEX plug-in written in Rust (naga + wgpu, DirectX 12).
There is no editor to install, no service to run, no account, and no
telemetry: the committed source on the effect's `Source` parameter is the
single authority for what renders.

## Status

`0.0.3` — pre-release.

| Host (Windows) | Status |
|---|---|
| After Effects 2025 | Verified (full test battery on the release artifact) |
| After Effects 2026 | Verified (full test battery on the release artifact) |
| After Effects 2024 | Not yet verified (no host available) |
| After Effects 2023 | Blocked: AE 23.0 itself fails to launch on the dev machine (with and without the plug-in) |

macOS (Apple Silicon) follows after Windows is stable.

## Install

Copy `DynamicFx.aex` to the version-specific plug-ins folder, e.g.:

```
C:\Program Files\Adobe\Adobe After Effects 2025\Support Files\Plug-ins\DynamicFx\DynamicFx.aex
```

Never install into the shared `Common\Plug-ins\7.0\MediaCore` folder —
Premiere Pro scans it too. Restart After Effects; the effect appears as
**DynamicFx**.

## Quick start

Apply DynamicFx to a layer, then put your shader on the `Source` parameter
as an expression. `Source` is a numeric parameter, so the expression is a
backtick template literal (carrying the source text) followed by `;0` — the
`;0` makes the whole expression evaluate to a number for After Effects.
Paste this complete expression:

```javascript
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

The trailing `` ` `` and `;0` are required — without them After Effects
rejects the expression.

Compile status appears on the effect's `Status` row. `@param` annotations
become real, keyframeable AE controls with stable identity across source
edits; `hint:color default:#RRGGBB` gives a color control its initial value
(6 hex digits imply alpha 1.0, 8 set it explicitly). Multi-pass graphs
declare passes and connections in the `@graph` block; `prev` as a pass
input plus `// @window N` enables temporal feedback (windowed
re-simulation: every frame is self-contained — scrubbing, render-queue
order, and aerender all agree exactly).

## Language guide

Everything DynamicFX adds to GLSL lives in comments and in a small envelope
around your passes. Nothing here is a preprocessor: your shader body is plain
GLSL 450 core, compiled by naga, and the extra syntax only tells DynamicFX how
to wire it to After Effects.

### The envelope

```text
@dynamicfx 1                  ← required first line; `1` is the envelope version
@graph                        ← the pass manifest
pass main: input -> output
@end
@pass main                    ← one section per pass, named in the manifest
#version 450
...your GLSL...
@endpass
```

A source without `@dynamicfx 1` is treated as a single raw pass, so the
shortest possible shader is just GLSL. Once you write the marker, the whole
envelope is required and every violation is reported as `E6` with the line
number.

**Manifest lines** read `pass NAME: INPUT[, INPUT...] -> OUTPUT`. Three names
are reserved and you cannot use them for your own passes or intermediates:

| Name | Meaning |
|---|---|
| `input` | the layer the effect is applied to |
| `output` | the effect's result — exactly one pass writes it |
| `prev` | the previous frame's final output (temporal feedback) |

Any other name is an intermediate: write it in one pass, read it in later ones.
Passes are scheduled from the dependency graph, not from the order you list
them. Limits are 16 passes, 4 inputs per pass, and 15 intermediates.

**Temporal feedback.** Add `prev` as a pass input and put `// @window N`
anywhere in the source. Each frame re-simulates `min(frame + 1, N)` iterations
from black, so every frame is self-contained: scrubbing, render-queue order and
`aerender` all produce identical pixels. Layer and path inputs cannot be
combined with `prev` (diagnostic `E7`) — re-simulating would need the host
resource at every iterated frame, a cost that has not been measured.

### The shader interface

Every pass declares the same fixed head. Descriptor set 0 only:

```glsl
layout(location = 0) in  vec2 v_uv;      // 0..1 across the frame
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_in;   // manifest input 1
layout(set = 0, binding = 1) uniform sampler   u_s;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2  u_resolution;   // logical full-resolution frame size, always
    float u_time;         // layer time in seconds
    float u_frame;        // layer time in frames
    // ...your parameters follow, in any order...
};
```

The three head members are required, in that order, with those names. Your own
uniforms follow them and become AE controls. `u_resolution` is the *logical*
full-resolution size even at half or quarter preview, so a shader looks the same
at every preview resolution.

A pass with more than one manifest input takes the extra ones at bindings
`3`, `4`, `5` — in manifest order:

```text
pass compose: blurred, mask_layer -> output
```

```glsl
layout(set = 0, binding = 3) uniform texture2D u_mask;   // = mask_layer
```

Declaring a binding the manifest does not feed is an `E18`.

### Declaring parameters

One comment line per parameter, anywhere in the source:

```text
// @param <id> [entry ...]
```

| Entry | Meaning |
|---|---|
| `label:"Some Text"` | the row name in Effect Controls (quotes optional if one word) |
| `min:<number>` `max:<number>` | slider range |
| `default:<number>[,<number>...]` | initial value, 1-4 components |
| `default:#RRGGBB` / `default:#RRGGBBAA` | colour initial value (`hint:color` only) |
| `alias:<id>[,<id>...]` | previous names, so renaming a uniform keeps its keyframes |
| `hint:<kind>` | pick a different AE control — see the table below |

A typo in any entry rejects the definition rather than being ignored: silently
dropping `mim:0` would leave you wondering why the range never applied. An
annotation whose id matches no uniform is ignored, so commenting a uniform out
does not break the build.

**`alias:` is how you rename things safely.** Rename the uniform, add
`alias:old_name`, and the parameter keeps its AE slot, its value and its
keyframes across the recompile.

### Parameter types

The GLSL type picks the control; `hint:` overrides it where a type is ambiguous.

| Declare | Hint | AE control | Value the shader receives | Slots |
|---|---|---|---|---|
| `float x;` | — | Slider | as shown | 48 |
| `float x;` | `hint:angle` | Angle dial | degrees | 8 |
| `int x;` | — | Integer slider | as shown | 8 |
| `int x;` | `hint:bool` | Checkbox | `0` or `1` | 16 |
| `vec2 x;` | — | Point (crosshair) | normalized to the frame (`0..1`) | 12 |
| `vec3 x;` | — | Colour | RGB `0..1` | 12 |
| `vec3 x;` | `hint:point3d` | 3D point | `x,y` normalized to the frame; **`z` in pixels** | 8 |
| `vec4 x;` | — | Colour + Opacity | RGB `0..1`, alpha in `.a` | 12 + 48 |
| *(not a uniform)* | `hint:layer` | Layer menu | `texture2D`, comp-space | 4 |
| *(not a uniform)* | `hint:gradient` | Stop rows | `texture2D`, 256×1 LUT | 2 |
| *(not a uniform)* | `hint:path` | Mask menu | `texture2D`, `N×2` vertices | 2 |

Notes that will save you time:

- **`vec3` is a colour by default.** Spatial `vec3`s must say `hint:point3d`,
  because flipping the default would silently retype every existing shader's
  colours.
- **Point 3D's `z` is not normalized.** There is no third frame dimension to
  divide by, and picking one (height? the diagonal?) would be a convention you
  could not predict. `u_resolution` is in the head if you want to scale it
  yourself.
- **`vec4` costs two slots** — a Colour plus a Float for alpha — and they are
  allocated together or not at all.
- **Point and Point 3D take no `default:`.** The AE control's own default
  applies.
- **Known limitation:** `hint:layer`, `hint:gradient`, `hint:point3d` and
  `hint:path` controls show a generic row name (`Mask 01`) instead of your
  `label:`. Their value, keyframes and rendering are unaffected. After Effects
  ignores the name update for these control types, and the one alternative
  route freezes the host, so it is left alone until that is understood.

### Layers, gradients and masks

These three are not `FxUniforms` members. You declare them with an annotation
and name them in the graph; DynamicFX feeds them as textures:

```text
@graph
pass main: input, depth_map, heat_ramp, outline -> output
@end
@pass main
#version 450
// @param depth_map  label:"Depth Map" hint:layer
// @param heat_ramp  label:"Heat Ramp"  hint:gradient
// @param outline    label:"Outline"    hint:path
layout(set = 0, binding = 3) uniform texture2D u_depth;
layout(set = 0, binding = 4) uniform texture2D u_ramp;
layout(set = 0, binding = 5) uniform texture2D u_path;
```

They are read-only: a graph resource fed by a parameter can never be a pass
name or a pass output (`E6`).

**Layer inputs** arrive as the referenced layer composited into the same frame
rect, so `v_uv` means the same thing for `u_in` and for a layer input. An
unassigned menu binds transparent black rather than failing.

**Gradients** are ordinary AE rows: a `Stops` count plus a Position, Colour and
Alpha row per stop, up to 8. All of it keyframes, copies and pastes like any
other parameter. Sample the baked ramp with `texture(..., vec2(t, 0.5))`.

**Masks** arrive as an `N × 2` `Rgba32Float` texture of the mask's own vertices:

| Texel | Row 0 | Row 1 |
|---|---|---|
| `i` | `(x, y, tan_out_x, tan_out_y)` | `(tan_in_x, tan_in_y, 0, 1)` |

All coordinates are normalized to the frame, exactly like a `hint:point`
parameter. **The vertex count is the texture width** — read it with
`textureSize(...).x` — so the count can never disagree with the data. Use
`texelFetch`, not `texture()`: a vertex is a value to read exactly, not to
interpolate between. A closed path repeats vertex 0 at the end, so a four-corner
rectangle reports 5 vertices. An unassigned menu binds a `1 × 2` zero texture,
so handle a one-vertex path rather than dividing by `count - 1`.

```glsl
int   n  = textureSize(sampler2D(u_path, u_s), 0).x;
vec2  v0 = texelFetch(sampler2D(u_path, u_s), ivec2(0, 0), 0).xy;
```

### Capacity

Parameters are allocated from fixed pools, which is what keeps their AE
identity stable across source edits. The per-shader ceilings are the "Slots"
column above. Exceeding one rejects the definition with `E32` rather than
silently dropping a control.

## Examples

[`examples/`](examples/) has complete, working shaders to paste in:

- [`thermal.glsl`](examples/thermal.glsl) — a six-pass heat signature: warped
  fBm field, two separable blur chains, palette compositing. Shows multi-pass
  graphs and effect-wide parameters.
- [`orb.glsl`](examples/orb.glsl) — an orbiting light with a decaying trail.
  Shows temporal feedback (`prev` + `@window`), plus angle and checkbox
  controls.

Both are compiled by the test suite on every build, so they cannot drift out
of sync with the grammar. See [`examples/README.md`](examples/README.md) for
how to apply one.

## AI assistant skill

`skills/dynamicfx-shaders/` is a skill for AI coding assistants (Claude Code, Cursor, and similar) that teaches them the envelope syntax, the shader ABI, `@param` declarations, and how to port existing Shadertoy/GLSL shaders to DynamicFX. One-line install from your project root:

```bash
mkdir -p .claude/skills/dynamicfx-shaders && for f in SKILL.md porting.md reference.md; do curl -fsSL "https://raw.githubusercontent.com/JUNKDOGE-JOE/dynamicfx/main/skills/dynamicfx-shaders/$f" -o ".claude/skills/dynamicfx-shaders/$f"; done
```

Or paste this to your assistant: *Download SKILL.md, porting.md and reference.md from https://raw.githubusercontent.com/JUNKDOGE-JOE/dynamicfx/main/skills/dynamicfx-shaders/ and save all three into .claude/skills/dynamicfx-shaders/ in this project, then confirm the skill is installed.*

See [skills/dynamicfx-shaders/INSTALL.md](skills/dynamicfx-shaders/INSTALL.md) for details.

## Scripting: wait for readiness before you render

If a script applies DynamicFx and writes the `Source` expression, it must
wait for the effect to become *renderable* before starting a render. Writing
an expression does not itself compile anything: After Effects sends no
effect selector when a script changes an expression, so DynamicFX picks the
change up on a main-thread idle pass instead.

The consequence that bites: **idle passes cannot run while a script holds the
main thread.** A `$.sleep(20000)` after the writes does not give the plug-in
its turn — it denies it one. Rendering then produces frames that pass the
layer through untouched, because no definition has been published yet. A
modal dialog left open does the same thing.

Poll with `app.scheduleTask`, which returns control to After Effects between
checks:

`app.scheduleTask` takes a string of code to evaluate later, so keep the
effect list in a global and schedule a named, no-argument step. This is the
shape used by this project's own benchmark harness:

```javascript
// Property 5 is the effect's StateToken: `(payload << 2) | state`. State 1
// (`% 4 === 1`) means a definition is published and render clones can
// resolve it. Non-zero alone is NOT readiness — a published diagnostic is
// also non-zero, including E53 "publication pending", which is exactly the
// not-yet-renderable case.
function dfxIsReady(fx) {
    try { return fx.property(5).value % 4 === 1; } catch (e) { return false; }
}

var DFX_FX = [];        // fill with your DynamicFx effect property groups
var DFX_TRIES = 0;

function dfxPoll() {
    var ready = 0;
    for (var i = 0; i < DFX_FX.length; i++) {
        if (dfxIsReady(DFX_FX[i])) { ready++; }
    }
    if (ready === DFX_FX.length) {
        dfxRender();                                  // your next step
    } else if (++DFX_TRIES > 60) {                    // 60 × 500 ms = 30 s
        alert("DynamicFX not ready: " + ready + "/" + DFX_FX.length);
    } else {
        app.scheduleTask("dfxPoll()", 500, false);
    }
}

// After applying the effects and writing every Source expression:
dfxPoll();
```

Reading the `Status` row (property 4) gives the human-readable reason when a
poll times out; the `Show Full Status` button prints the untruncated text
with its `E<code>` diagnostic.

## What is verified, concretely

- Pixel exactness at 8/16/32-bpc, including 16-bpc bit-exact multi-pass
  chains, 32-bpc negative/over-white survival, and straight alpha.
- Preview == render queue == aerender, adversarially tested (WYSIWYG).
- Multi-frame rendering enabled; thread-safety proven under measured MFR
  dispatch.
- Per-render cost measured and optimized (cached GPU resources, ROI
  delivery with pixel-identical guarantees, zero per-render log I/O).
- Every parameter kind above checked on the host by rendering a frame whose
  pixels encode the value, then reading those pixels back — not by asserting
  that a control appeared. A point driven to `(40, 60, 50)` in a 160×120 comp
  must render `rgb(64, 128, 128)`; a four-corner mask must report 5 vertices
  and place vertex 0 at `(0.25, 0.25)`. Those are the actual passing numbers.

Equally concretely, here is what is **not** verified: After Effects 2024 and
2023 (no host available), and the mask path on a GPU without
`FLOAT32_FILTERABLE` — every adapter tested has it, and without it a mask input
binds an empty texture and says so in the log rather than rendering wrong.

## Build from source

```
rustup toolchain install (pinned by rust-toolchain.toml)
cargo build --release
scripts\install.bat 2025   (run as administrator)
```

## Sponsors

Thank you to everyone supporting DynamicFX!

| Sponsor | Amount |
|---|---|
| **PAO** — our first sponsor! | ¥5 |

## License

MIT — see [LICENSE](LICENSE).
