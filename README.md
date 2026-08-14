# DynamicFX

**An open shader runtime for After Effects.** Write GLSL in an ordinary
expression, get a real GPU effect — multi-pass render graphs, keyframeable
parameters, temporal feedback, 8/16/32-bpc, all through normal AE workflows.

DynamicFX is a native AEX plug-in written in Rust (naga + wgpu, DirectX 12).
There is no editor to install, no service to run, no account, and no
telemetry: the committed source on the effect's `Source` parameter is the
single authority for what renders.

## Status

`0.0.1` — pre-release.

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

## What is verified, concretely

- Pixel exactness at 8/16/32-bpc, including 16-bpc bit-exact multi-pass
  chains, 32-bpc negative/over-white survival, and straight alpha.
- Preview == render queue == aerender, adversarially tested (WYSIWYG).
- Multi-frame rendering enabled; thread-safety proven under measured MFR
  dispatch.
- Per-render cost measured and optimized (cached GPU resources, ROI
  delivery with pixel-identical guarantees, zero per-render log I/O).

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
