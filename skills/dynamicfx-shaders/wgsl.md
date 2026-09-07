# WGSL authoring (DynamicFX 0.1.0+)

Select **Setup → Language → WGSL** before committing WGSL to **Source**.
GLSL remains the default. A filename, attribute or source comment does not
change the selected language; all passes in one source use that language.
The source expression is still one backtick string followed by `;0`.

Use [wgsl-field.wgsl](../../examples/wgsl-field.wgsl) for one pass or
[wgsl-multipass.wgsl](../../examples/wgsl-multipass.wgsl) for two passes.
These files contain the whole envelope, without the outer expression wrapper.
The second example writes a field, then reads its neighboring pixels alongside
the original layer. Neither example depends on a browser, WebGPU JavaScript,
or the old feasibility spike's generated GLSL interface.

## Envelope attributes: `@@` is intentional

The existing envelope parser recognizes a directive whenever the first
non-whitespace character of a pass-body line is `@`. Write an additional
leading `@` to pass a literal attribute through to the WGSL compiler:

```wgsl
@@group(0) @binding(0) var u_input: texture_2d<f32>;
@@fragment
fn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {
    // ...
}
```

Only the first `@` on that line is doubled. The inline `@binding` and
`@location` above stay unchanged. Indentation is preserved; a struct member
attribute on its own line also needs escaping, such as `    @@location(0)`.
`// @param` starts with `/`, so it is never escaped. An unescaped `@group`
or `@fragment` at the start of an envelope body line produces **E6**, before
WGSL parsing. There is no `@source` section or new WGSL envelope version.

Raw single-pass WGSL source without an `@dynamicfx` envelope uses ordinary
`@group` and `@fragment` attributes. Do not copy the doubled attributes into
a standalone WGSL compiler. Use an envelope for deliverable examples so the
graph and input order remain explicit.

## Minimal complete Source expression

```wgsl
`@dynamicfx 1
@graph
pass main: input -> output
@end
@pass main
// @param gain label:"Gain" min:0 max:2 default:1
struct FxUniforms {
    u_resolution: vec2<f32>,
    u_time: f32,
    u_frame: f32,
    gain: f32,
}
@@group(0) @binding(0) var u_input: texture_2d<f32>;
@@group(0) @binding(1) var u_sampler: sampler;
@@group(0) @binding(2) var<uniform> fx: FxUniforms;
@@fragment
fn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {
    let source = textureSample(u_input, u_sampler, v_uv);
    return vec4<f32>(source.rgb * fx.gain, source.a);
}
@endpass
`;0
```

WGSL has no `#version`, `layout`, `sampler2D` constructor, or GLSL `outColor`
global. Read block members through the uniform variable (`fx.gain`), and
return the fragment color. Use `let` for an immutable local and `var` when
the value changes. WGSL uses `select(false_value, true_value, condition)`
instead of GLSL's ternary `condition ? true_value : false_value`.

## Fragment ABI

Each pass is an independently compiled module with exactly one entry point:
`@fragment fn main`. It returns one location-0 `vec4<f32>`, directly or through
a single-field output struct. Optional inputs are location-0 `vec2<f32>` UV
(default perspective/center interpolation)
and `@builtin(position) vec4<f32>` fragment position. A generator can omit
UV, the primary texture and sampler. Resources that are declared must still
satisfy the ABI even if shader code never reads them.

| Group / binding | WGSL declaration | Meaning |
|---|---|---|
| 0 / 0 | `var u_input: texture_2d<f32>` | First input in this pass's graph line |
| 0 / 1 | `var u_sampler: sampler` | Shared non-comparison sampler |
| 0 / 2 | `var<uniform> fx: FxUniforms` | Required uniform structure |
| 0 / 3, 4, 5 | Additional `texture_2d<f32>` variables | Second, third, fourth manifest inputs, in order |

The sampler uses linear filtering and clamp-to-edge addressing. Use
`textureSample(texture, sampler, uv)` for filtered color and
`textureLoad(texture, vec2<i32>(x, y), 0)` for exact texels/data.
`vec2<f32>(textureDimensions(texture))` gives the physical texture size;
`fx.u_resolution` gives the logical full-resolution canvas size. There is
one mip level and no author-selected intermediate size or format.

The uniform structure begins with these exact members, types and offsets:

| Member | Type | Byte offset |
|---|---|---|
| `u_resolution` | `vec2<f32>` | 0 |
| `u_time` | `f32` | 8 |
| `u_frame` | `f32` | 12 |

User fields follow. The runtime reflects the original WGSL member offsets
and block span, including legal `@align`/`@size` attributes; do not guess a
Rust struct layout or manually insert dummy members. A `vec3<f32>` aligns
to 16 bytes but occupies 12, so a following scalar may share the remaining
four bytes. Structure/variable names are presentation; the bindings and
reserved head members define the ABI. `FxUniforms` and `fx` are the recommended
authoring names. Explicit padding must keep the block within the runtime's
65,536-byte uniform binding limit.

User types are `f32`, `i32`, `vec2<f32>`, `vec3<f32>`, and `vec4<f32>`.
Use **`i32` plus `hint:bool`** for a checkbox, and test `fx.enabled != 0`.
Do not put a WGSL `bool` in a uniform block. Arrays, matrices, nested user
structures, `u32` parameters, storage resources, texture/binding arrays,
comparison samplers, pipeline `override`s and additional groups are outside
this ABI. Vertex/compute entry points, extra entry points, depth/sample-mask
outputs and other fragment inputs are also rejected.

## Parameters, graphs and image quality

The existing [annotation syntax and controls](reference.md) apply unchanged.
Use `vec4<f32>` with `hint:color default:#RRGGBB` for a color with opacity.
For a `vec3<f32>` color use three numeric default components, for example
`default:0.2,0.6,1`; the existing hex syntax expands to four components.
Point controls arrive canvas-normalized; point/point3d `default:` annotations
are unsupported. Use scalar coordinate controls if an explicit authored
initial position is needed.

Annotations are unique across the entire source, while a shared uniform
member may appear in multiple pass blocks. Declare only the members a pass
uses; this determines the existing Main/per-pass groups. Extra resource hints
(`layer`, `gradient`, `path`) go in the manifest, not in the uniform struct.
For `pass glow: light, input -> output`, bind `light` at 0 and original
`input` at 3. The shader variable names do not route these textures.

Gradient LUT sampling uses
`textureSample(u_ramp, u_sampler, vec2<f32>(t, 0.5))`. For a path vertex use
`textureLoad(u_path, vec2<i32>(vertex_index, row), 0)` and query the width
with `textureDimensions(u_path).x`. Existing canvas, temporal-window,
resource-count and `prev` coexistence limits remain unchanged.

Read [quality.md](quality.md) for the full sampling policy. The WGSL names
for GLSL `dFdx`/`dFdy` are `dpdx`/`dpdy`; `fwidth` keeps its name. Evaluate
derivatives and implicit-derivative texture samples in uniform control flow.
Use signed-distance coverage for contours, and filter unresolved procedural
frequencies independently from texture filtering. The WGSL examples use
low-frequency continuous analytic fields, not random lattice values.

Choose units deliberately: `vec2<f32>(1.0) / vec2<f32>(textureDimensions(t))`
steps by a physical texel; `vec2<f32>(radius_px) / fx.u_resolution` retains
a logical-pixel radius in Full/Half/Quarter previews. `fwidth(distance)`
measures the rendered contour footprint at each preview size. These solve
different problems; contour AA does not reconstruct a minified background.

The runtime carries AE's alpha and working-space values without a conversion.
The field example preserves source alpha; the multipass example explicitly
returns opaque output. Its intermediate stores coverage-weighted RGB, so
filtering RGB and alpha together does not pull arbitrary unweighted colors
across the edge. For soft light, prefer 16/32-bpc: 8-bpc quantizes every
intermediate, and 32-bpc preserves HDR values above one.

## Validation and diagnostics

Use the [shared validation checklist](SKILL.md#validation-checklist), selecting
WGSL and the WGSL ABI instead of the GLSL header. Verify actual renders at
Full/Half/Quarter, all intended depths, and multiple requested times.
Compilation alone proves neither appearance nor AE host support.

- **E6**: envelope parsing, commonly a missing second `@` on an attribute.
- **E21**: WGSL parse or language-validation error.
- **E18**: shared ABI violation, including a texture not supplied by the graph.
- **E19**: annotation, parameter type, or cross-pass declaration error.
- **E20**: shared GPU artifact emission error.

Status identifies the pass and source location where available. Preserve the
committed source while fixing diagnostics. Changing Language does not
translate source; older GLSL-only releases cannot render a WGSL snapshot.
The 0.1.0 host/release verification is tracked separately in
[TEST_MATRIX](../../docs/TEST_MATRIX.md).
