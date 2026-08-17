# DynamicFx shader interface

> **Prototype snapshot:** 本文描述尚未发布的重写前 GLSL contract。已确认的多语言、multi-pass、参数和新状态设计以 [ARCHITECTURE.md](ARCHITECTURE.md) 为准；重写不承诺本文所述参数顺序、`SourceChannel` 或 flattened sequence 协议兼容。

Status: the MVP render loop, parameter exposure, multi-shader coexistence, and
render-time default fallback were exercised in AE 2025 (2026-08-01). AE 2025
and AE 2026 are the explicit targets. The current programmatic-add,
idle-recompile, and UI/render source-transport changes pass Rust tests and a
release build; live regression validation in both hosts is still pending.

## How source is supplied

The `Source` parameter carries the shader in its **expression**, wrapped in a
backtick template literal with a trailing `;0` so the expression still
evaluates to a number and AE does not flag a type error:

```js
`#version 450
...shader code...
`;0
```

Editing the expression in AE triggers recompilation automatically. For
programmatic writers (ae-mcp / JSX), a main-thread idle hook scans effect
instances about once per second; only changed source is compiled. Main-thread
sequence setup and the uncommon main-thread render remain fallback entry
points. Writers only need to set the `Source` expression.

`%TEMP%\dynamicfx_source.txt` is disabled by default. To enable this legacy
debug fallback, set `DYNAMICFX_ENABLE_SIDECAR=1` before starting AE (`true`
and `on` are also accepted). Even then it is consulted only while the
instance source state is **Unknown** and no module is available. Once the
plugin observes no expression, a bad wrapper, or a compile failure, the state
is **Inactive**, rendering passes through, and the sidecar cannot mask it.
The file is process-global and cannot safely represent multiple instances;
never use it as a production/programmatic source transport.

## Fragment shader contract

GLSL 450, Vulkan-style separate texture/sampler. The plugin provides a
fullscreen-triangle vertex stage; the user writes only the fragment stage.

```glsl
#version 450
layout(location = 0) in vec2 v_uv;            // 0..1, v = 0 at top of frame
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_input;   // the layer DynamicFx is applied to
layout(set = 0, binding = 1) uniform sampler u_sampler;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;                        // builtin: frame size in pixels
    float u_time;                             // builtin: layer time in seconds
    // ...user uniforms go here, exposed as AE controls
};
void main() {
    vec4 c = texture(sampler2D(u_input, u_sampler), v_uv);
    outColor = c;
}
```

- Pixels are standard straight-alpha RGBA (the plugin swizzles AE's native
  ARGB at the boundary).
- Compilation: GLSL → naga (parse + validate + reflect) → SPIR-V → wgpu.
- Compile errors are reported on the `Status` parameter label (truncated to
  AE's 31-char param-name limit) and in full to `%TEMP%\dynamicfx.log`.

## `@param` annotations — exposing uniforms as AE controls

Every extra member of the FxUniforms block becomes an AE control. The
annotation line supplies UI metadata; the token count depends on the kind:

```glsl
// @param u_speed float 0.0 10.0 3.0 Speed      // float slider: min max default
// @param u_count int 1 16 4 Count              // int slider:   min max default
// @param u_enable bool 1 Enabled               // checkbox:     default (0/1)
// @param u_tint color 1.0 0.5 0.2 Tint         // color picker: r g b (0..1)
// @param u_center point 0.5 0.5 Center         // 2D point:     x y (0..1 of frame)
```

Rules:

- The kind token is optional for `float`/`int`/`vec2`/`vec3`/`vec4` members;
  it is then derived from the GLSL type (f32→float slider, i32→int slider,
  vec2→point, vec3/vec4→color). `bool` on an f32/i32 member makes a checkbox.
- Without an annotation the member still gets a control, labeled with the
  member name and 0..1 range.
- Pool sizes (per effect instance): 16 float, 4 int, 4 bool, 6 color,
  4 point. Unused pool slots are hidden via the DynamicStream API.
- Control values are read at render time, so **keyframing works**.
- `vec4` colors also receive the picker's alpha as the 4th component.

### Default-value commit quirk

AE only honors programmatic value changes in a `UserChangedParam`/event
context. When a new shader is compiled, names/ranges/visibility apply
immediately, but default VALUES may wait for the next user gesture; the
plugin backfills them automatically on the next `UserChangedParam` — the
**Compile button is the deterministic way to force this**. Programmatic
flows should simply set values via ae-mcp.

Rendering is immune to this window: each instance's hidden, non-time-varying
`SourceChannel` FloatSlider carries one atomic source-identity/commit token.
The token is an exactly represented integer containing a 51-bit stable FNV-1a
source hash and two low state bits. Until the UI thread reports the streams
committed, the render path uses the annotation defaults stored in the registry
build at compile time, so an uncommitted instance renders with correct
defaults instead of pool zeros (which used to produce NaN white frames, e.g.
a `Levels`-style divide by zero). Flattened sequence data persists the same
source/commit identity for project reload and render-side reconstruction.

## Current limitations

- **Programmatic-path fixes await live host validation.** The idle hook now
  notices expression-only writes without a poke/render, and a source-less new
  instance defers slot hiding so `addProperty("DynamicFx")` can finish building
  its property tree. Both fixes pass the Rust suite but still need the included
  regression script run once in AE 2025 and once in AE 2026.
- The current PiPL disables Multi-Frame Rendering. Idle updates the target
  instance on AE's main thread. UI and render projects are still separate, so
  the supported handoff is already the hidden single-FloatSlider
  `SourceChannel` plus flattened state; render-side callbacks do not make AEGP
  calls.
- `%TEMP%\dynamicfx_source.txt` is an opt-in, Unknown-state-only debug escape
  hatch and is not instance-safe (see "How source is supplied").
- 8-bit precision: 16bpc (U15) input is downconverted to 8 bpc, rendered,
  and converted back. 32 bpc projects pass through unchanged. (P3: RGBA16F)
- Whole-frame CPU readback per render; no texture caching, no GPU surface
  interop, no multi-pass. (P2: SmartRender)
- `u_time` uses layer time; preview drives it continuously because the
  plugin declares `NonParamVary`.
- No dropdown/enum, angle, layer(texture) inputs, or gradient-ramp editor
  (the last one needs arbitrary-data + custom UI; N color stops ≈ N color
  controls for now).

## Multi-instance model (P0)

- Compiled shaders live in a process-wide **registry keyed by source
  hash** — N different shaders coexist, identical sources share one build.
- Each instance publishes one atomically copied token through a hidden,
  non-time-varying FloatSlider parameter (`SourceChannel`). Its 51-bit source
  hash occupies the high bits and its two low bits encode state: the complete
  word is `0` for uninitialized, `(hash << 2) | 1` for active/uncommitted,
  `(hash << 2) | 2` for active/committed, and `3` for clear. Every valid word
  is at most `2^53 - 1`, so f64 copies it exactly. Render-project copies read
  it fresh and resolve their own build from the registry. A clear state
  suppresses stale registry, legacy, and sidecar data.
- Version-3 flattened sequence data persists explicit Unknown/Active/Inactive
  state, the active source/hash, and the committed flag. It rebuilds a missing
  registry entry after reload and is the other half of the supported
  UI/render handoff.
- Hidden arbitrary `SourceData` is retained only for old-project parameter
  ordering/callback compatibility. For a legacy project it may recover the
  committed bit only when its stored hash matches a recomputed legacy
  fingerprint of the flattened active source; its hash never selects a shader.
- The opt-in sidecar is consulted only for Unknown state and is not part of
  the supported multi-instance programmatic flow.
- Slot (control) values are per-instance AE params and work everywhere;
  only shader identity/state needs the FloatSlider + flatten transport.

## Diagnostics

- `%TEMP%\dynamicfx.log` — compile, idle synchronization, lifecycle, and error
  events are logged by default. High-frequency/per-frame render lines are
  suppressed so logging does not slow previews; set
  `DYNAMICFX_VERBOSE_LOG=1` before starting AE to include them (`true` and
  `on` are also accepted).
- If the shader fails to compile, the effect passes the input through
  unchanged and `Status` shows the error.
