# DynamicFX examples

Complete, working shaders you can paste into an effect. Each file is the
shader *source*; After Effects needs it wrapped as an expression.

## How to use one

1. Apply **DynamicFx** to a layer.
2. Select the matching **Language**: GLSL for `.glsl`, WGSL for `.wgsl`
   (WGSL requires 0.1.0+). GLSL is the default; filenames do not set it.
3. Alt-click (Option-click) the stopwatch on the `Source` parameter to open
   the expression field.
4. Type a backtick `` ` ``, paste the whole file, then type `` ` `` and `;0`.

The result looks like this — the backticks carry the source text verbatim,
and the `;0` makes the expression evaluate to a number, which is what the
numeric `Source` parameter requires:

```javascript
`@dynamicfx 1
@graph
pass trail: input, prev -> output
@end
...the rest of the file...
@endpass
`;0
```

Click away from the field to commit. The `Status` row reports the compile
result; `Show Full Status` prints the untruncated text with its `E<code>`
diagnostic if something is wrong. The declared `@param` controls appear
underneath as ordinary, keyframeable AE properties.

WGSL examples intentionally contain `@@group` and `@@fragment`: the envelope
removes one leading `@` before compiling each pass. Keep them doubled when
pasting the whole example. Inline attributes such as `@binding` stay single.
See the [WGSL authoring guide](../skills/dynamicfx-shaders/wgsl.md) for the
complete interface, parameter types and raw-source distinction.

If you are applying effects from a script rather than by hand, read
[Scripting: wait for readiness before you render](../README.md#scripting-wait-for-readiness-before-you-render)
first — writing an expression does not compile it, and a script that holds
the main thread prevents the compile from ever happening.

## The examples

### [`wgsl-field.wgsl`](wgsl-field.wgsl) — single-pass analytic light field

A flowing cyan/violet disc with a narrow highlighted rim. It samples the
source, exposes speed/radius/rim/colors/amount plus an `i32` checkbox, and
preserves input alpha. Apply it to a comp-sized solid or footage. Continuous
trigonometric fields avoid random cell boundaries; `fwidth` coverage follows
the rendered pixel footprint while the logical shape remains fixed at reduced
preview resolutions.

### [`wgsl-multipass.wgsl`](wgsl-multipass.wgsl) — field plus sampled glow

The **field** pass samples the source and writes an animated light field with
antialiased coverage to `light`. The **glow** pass reads a nine-tap neighborhood
from `light` at binding 0 and the original layer at binding 3. The intermediate
contains coverage-weighted RGB and alpha; the final output is explicitly
opaque, so use footage or a comp-sized solid.

Controls appear in their owning pass groups because each uniform block only
declares the members that pass uses. **Glow Radius (px)** uses logical canvas
pixels; it is a compact binomial kernel for a small soft edge, not a wide
Gaussian blur. Use 16/32-bpc for smooth intermediates, and 32-bpc when preserving
additive light above white matters. Both passes use the production WGSL ABI.

### [`thermal.glsl`](thermal.glsl) — six-pass heat signature

A thermal/infrared look built as a real multi-pass graph: a warped fBm heat
field, two separable blur chains at different radii, and a compositing pass
that maps everything through a six-stop palette.

Demonstrates multi-pass graphs (six passes, two independent blur chains
feeding one compositor), effect-wide parameters shared across passes, and
`hint:color default:#RRGGBB` controls.

Drive it with a layer that has an **alpha channel** — text or a logo works
well. The graph reads the input's alpha as the heat source, so a fully
opaque solid gives a flat result.

### [`orb.glsl`](orb.glsl) — orbiting light with a temporal trail

A glowing orb circling the frame, leaving a decaying trail. One pass that
reads both the layer and `prev` (the previous frame's output).

Demonstrates temporal feedback: `prev` as a pass input plus `// @window 16`.
DynamicFX re-simulates the last 16 frames for every request, so scrubbing,
the render queue, and aerender all produce exactly the same pixels — there
is no hidden playback state to get out of sync. It also shows `hint:angle`
and `hint:bool` controls alongside colors.

Turn **Composite Over Layer** off to see the orb alone on transparency.

### [`apple-thermal.glsl`](apple-thermal.glsl) — event-style thermal logo (ten passes)

A thermal-camera look in the style of a well-known 2025 event invitation:
black top face, hot bands that drift along parts of the contour (red-orange
edge → yellow → white → light-blue tail), a thin light-blue edge line where
the shape is cold, and a wide blue glow outside the shape that warms up next
to hot regions.

Built as ten passes: two separable blurs of the alpha (a small one for the
edge line, a wide one for the diffusion field, hi/lo-encoded so the smooth
field survives 8-bpc intermediates), a temperature pass (regional-heat noise
× a contour-band profile, plus a "wall direction" boost so the lower-left
contours run hotter), a medium blur of the temperature for softness, a wide
blur of the temperature for the outer glow, and one colouring pass that maps
temperature through a palette (`Use Custom Ramp` swaps in a `hint:gradient`).

Demonstrates a 10-pass graph with two independent blur chains over a scalar
field, hi/lo encoding of a smooth field on 8-bpc intermediates, an integer
lattice hash for value noise (the `fract(sin(dot))` hash breaks on cell
borders for large arguments), `hint:angle`, `hint:bool` and `hint:gradient`
controls, and an output whose alpha extends beyond the source (the glow) while
the shape's own coverage is kept.

Drive it with a logo or text that has an **alpha channel**, and give the
layer canvas margin — the glow can only be drawn inside the layer's extent, so
precompose a 512-px logo into a 1024-px comp (or use a padded PNG). Pixel
controls (`Heat Depth`, `Wall Thickness`, `Outer Glow Radius`) are tuned for a
logo about 400 px across; scale them with your artwork.

### [`ink-bleed.glsl`](ink-bleed.glsl) — nine-pass analog chromatic bleed

An analog, organic "wet ink" titling look from one effect: the source's ink
diffuses into a three-octave blur pyramid whose radius differs per RGB
channel, elongated along a smear axis, with turbulence-driven color patches
tinted into the ink *before* the blur so the colors spread organically. The
same turbulence melts the source into the cloud (smooth or 45° halftone
screen), erodes it away (Dissolve), and warps it. On top: a thresholded
multi-layer glow with an inner→outer tint gradient, film halation, an
anamorphic flare, ink-drip streaks, echo ghosts, paper fiber, edge-ink rings
and midtone film grain.

Demonstrates a 9-pass graph whose blur pyramid is shared by two consumers
(the bleed cloud and the glow read the same three octaves at different
weights), per-channel Gaussian sigmas in one separable pass, pre-blur
linearization behind a `Linear Light` checkbox (what makes small highlights
bloom photographically), per-pixel tap jitter instead of more taps, int
sliders + `hint:bool` checkboxes + an angle-driven `Evolution`, and NaN
discipline: every `pow()` base is clamped strictly positive, because a base
that hits exactly 0 NaNs on some GPU stacks and `0 * NaN` poisons a whole
accumulator column (it renders as a razor-straight black seam).

Works directly on footage for a full-frame analog wash, or as a titling
effect: black comp-sized solid with `adjustmentLayer` on, above white text,
with an opaque black backdrop at the bottom of the stack. The output replaces
the frame with alpha 1, so the carrier solid must be black. Pixel controls
are tuned for 1080p titles; scale Bleed Amount / Glow Radius / Turbulence
Scale with your frame.

### [`siri-glow.glsl`](siri-glow.glsl) — smooth aurora screen rim

A single-pass, Siri-inspired perimeter glow with a rounded inner frame,
animated cyan/violet/coral color flow, soft halo, and subtle final dither.
It is an authored visual study of the glowing-edge motif, not a claimed
reproduction of a particular iOS version. Apply it to a comp-sized black solid
or footage. The rim sits inside the canvas, so no expansion is needed.

The shader demonstrates logical-pixel geometry with derivative-based edge
coverage, quintic integer-lattice noise, and pixel-footprint filtering of fBm
octaves. It is designed to stay smooth at Full, Half and Quarter preview;
16/32-bpc is preferable for the soft glow. Set **Dither (1/255)** to 0 when
comparing raw color values. **Flow Detail (px)** controls the size of the fluid
field; **Rim Width (px)** and **Glow Width (px)** control separate structures.

Headless GPU verification and known limits, including the corrected packed
temperature sampling in `apple-thermal.glsl`, are recorded in
[Shader quality](../docs/shader-quality.md). Compilation and headless rendering
do not imply an After Effects host acceptance result.

## Verification status

The GLSL examples are compiled through the real frontend by
`cargo test example_tests`; the WGSL pair uses
`cargo test shipped_wgsl_examples_compile`. Both read the exact shipped files,
so a grammar, ABI, or annotation change cannot silently break them.
Those tests prove they **compile**; the
palettes and default values are authored choices and are checked visually at
release time, not by the test. `apple-thermal.glsl` was rendered on
After Effects 2025 with the 0.0.4 build when it was added (evidence under
`docs/audits/evidence/examples/`). `ink-bleed.glsl` was authored and
visually checked on After Effects 2026 with the 0.0.4 build; its check used
licensed footage, so no evidence bundle is committed for it.

The WGSL examples also have a reproducible production-renderer check in
[`scripts/quality/run_wgsl_examples.py`](../scripts/quality/run_wgsl_examples.py).
It records actual adapter/source identities, original float buffers,
8/16/32 working-depth comparisons, Full/Half/Quarter images and deterministic
repeat requests. The reduced-size comparisons are descriptive measurements,
not proof of perfect antialiasing. Its 16-bpc run checks the f32 working
buffer; it does not test AE's U15 boundary. The 0.1.0 real-AE and platform
results are tracked separately in [TEST_MATRIX](../docs/TEST_MATRIX.md).
