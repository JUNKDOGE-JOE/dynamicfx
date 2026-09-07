# Shader image quality

Use this when authoring procedural effects or diagnosing blocks, grain, banding,
shimmer and jagged contours. Separate the scene's intended structure from the
sampling and precision used to render it. More octaves and more passes do not
automatically make a shader finer.

## Establish which stage creates the artifact

1. Record the committed source, DynamicFX build, project depth, preview scale,
   input size and color settings. Render the same frame at Full, Half, Quarter.
2. Replace the body with a UV ramp and then an input copy. If those fail, inspect
   upload/format/sampler/host geometry before changing procedural math.
3. For a generated field, render its scalar value directly, before thresholds,
   palette lookup and glow. Check a 200–400% crop across a suspected cell edge.
4. Switch to 16 or 32 bpc. If bands disappear, find the intermediate that stores
   the small signal. If only edges improve with supersampling, fix sampling.
5. Change one source of frequency at a time: grain, octaves, warp, thin rim,
   palette contrast. Keep evidence for the failing and improved versions.

The runtime uses one full-resolution texture per pass at the current preview
size, with one mip level and one sample per pixel. It does not provide an
automatic procedural antialiasing pass or a mip pyramid. The ABI specifies a
linear, clamp-to-edge sampler; builds through `d4477ab` accidentally created the
wgpu default nearest sampler. The 2026-09-08 fix makes the contract explicit.
An old installed build may therefore show blocky texture displacement or a
stepped 256-texel gradient LUT even with correct shader math.

## Three coordinate scales

```glsl
vec2 logicalPixel = v_uv * u_resolution; // stable geometry across previews
vec2 inputTexel = 1.0 / vec2(textureSize(sampler2D(u_in,u_s),0));
vec2 uvFootprint = fwidth(v_uv); // current output pixel, not logical pixel
```

Use logical pixels for sizes such as a 40-pixel blur radius. Use actual texels
for a one-texel stencil or LUT addressing. Use derivatives for coverage and
frequency filtering. At Half resolution one output pixel covers approximately
two logical pixels, so `1/u_resolution` alone is too small for edge AA.

## Cover the pixel at contours

For a signed distance `d` that is negative inside:

```glsl
float width = max(fwidth(d), 0.00001);
float coverage = clamp(0.5 - d / width, 0.0, 1.0);
```

This is a practical coverage approximation, not exact integration. For a thin
stroke, subtract the coverage of its two boundaries, as in `siri-glow.glsl`,
instead of just widening `abs(d) < lineWidth`. Compute derivatives outside
pixel-dependent branches, `discard`, and divergent loop exits. Do not take a
derivative of another derivative. Never reverse `smoothstep` edges: use
`1.0-smoothstep(low,high,x)`; GLSL leaves `edge0 >= edge1` undefined.

MSAA on a fullscreen triangle would sample that triangle's boundary, not the
implicit ring drawn by fragment math. For difficult warped/discontinuous
functions, integrate several subpixel evaluations or compare to a supersampled
reference. A blur can hide a jagged edge while also destroying the intended rim.

## Smooth fields and grain are different signals

Use a deterministic integer lattice hash for procedural noise and then
interpolate. A raw `hash(floor(p))` is constant throughout each cell. Cubic
interpolation removes value and first-derivative discontinuities; the quintic
weight `f*f*f*(f*(f*6-15)+10)` additionally makes second derivatives continuous
at lattice boundaries. This matters when noise drives normals or deformation.

`fract(sin(dot(p,...))*largeNumber)` is compact but sensitive to float rounding
and backend transcendental evaluation; it is not evidence of a renderer tile
bug by itself. The same corner must hash identically from each adjacent cell.
Avoid extremely large coordinate/time arguments; integer hashes still cannot
restore spatial detail already lost in floating-point coordinates.

For fBm, compute `dFdx(p)` and `dFdy(p)` once, and transform both derivatives
with every octave's rotation/scale. Attenuate amplitudes as their footprint
grows. The example's smooth fade over 0.75–2.0 lattice units per pixel was
checked against an 8× supersampled reference. It is a useful approximation,
not a universal exact low-pass filter. Nonlinear `abs`, `pow`, threshold and
domain warp can create new frequencies after filtering and need another check.
Keep signed, zero-mean contributions so dropping an octave does not change
the brightness; do not renormalize removed detail back to full contrast.

Final grain can hash `ivec2(gl_FragCoord.xy)` because individual physical pixels
are exactly the intended scale. Keep it low-amplitude and distinguish it from
fluid-field noise. Lock the grain in time unless animation is an intentional
part of the look. Randomizing every frame trades visible structure for flicker.

## Gradients, blur taps and packed fields

Linear sampling smooths adjacent texture texels. It does not fix a blur kernel
whose tap spacing exceeds the feature width: sparse taps can leave repeated
lobes or rings. Increase tap density, use a separable blur, or validate a
careful sampling approximation. Do not introduce per-pixel tap jitter as the
default fix; it trades structured error for noise and can shimmer in animation.

For categorical values, path vertices and packed data, use `texelFetch` with
explicit bounds. `texture()` uses the shared sampler. Never interpolate a
wrapped low channel such as `fract(T*8)` and then decode it: the wrap creates a
false value at fractional sample positions. Decode each of the four texels,
then interpolate the decoded value. If temperature carries coverage, multiply
by coverage before filtering; unpremultiply only after the convolution when
the consumer needs a straight value. `apple-thermal.glsl` demonstrates this.

## Precision and compositing

An 8-bpc project uses `Rgba8Unorm` at every pass boundary. A scalar in 0–0.01
only has about four levels there. Thresholding or multiplying it in the next
pass exposes those steps. 16/32-bpc projects use `Rgba32Float` internally;
16-bpc quantizes to AE U15 once at the output boundary. Dither at final output
can disguise a final rounding step but cannot reconstruct an earlier lost field.

The runtime neither changes the color space nor converts alpha. For a new
straight-alpha generator composited over a straight-alpha input, form the
premultiplied numerator, combine alpha, then divide safely. Do not multiply
straight RGB by alpha twice. Keep nonlinear color transforms explicit and
consistent with AE's working space; don't add a guessed gamma correction.

## Minimum visual acceptance

- Render Full/Half/Quarter with logical size held fixed and inspect a corner,
  diagonal, low-contrast gradient and noise crop.
- Compare 8-bpc and 16/32-bpc when the graph carries smooth intermediate fields.
- Check at least three times and the most demanding documented parameter values.
- Check input alpha, intended margins, and pixels outside the main shape.
- Keep shader identity, actual backend, raw frames and numeric tolerances.
  Call the result headless GPU verification until the same artifact runs in AE.

The repository's [quality report](../../docs/shader-quality.md) and
[headless runner](../../scripts/quality/README.md) contain measured examples.

## Primary references

- [GLSL 4.60 specification](https://registry.khronos.org/OpenGL/specs/gl/GLSLangSpec.4.60.pdf): derivative and `smoothstep` semantics.
- [Physically Based Rendering, Noise](https://www.pbr-book.org/3ed-2018/Texture/Noise): lattice interpolation and filtering fBm by pixel footprint.
- [WebGPU sampler descriptor](https://gpuweb.github.io/gpuweb/#dictdef-gpusamplerdescriptor): filter modes and default nearest sampling.
