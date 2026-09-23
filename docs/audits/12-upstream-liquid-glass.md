# Licensed upstream Liquid Glass port

The user requested an existing GitHub material. The delivered shader adapts
`iyinchao/liquid-glass-studio` at `f7b28c36305a862f5cffed3ddd51511cf1204f56`.
The five-pass graph uses native coverage for raster distance, blurs the background,
then applies the upstream optical and light formulas. The native plug-ins remain
the previously accepted pair. [Attribution and changes](../../examples/liquid-glass-upstream.md),
[ADR-0059](../adr/0059-licensed-liquid-glass-port.md).

## Acceptance

- 254 Rust tests pass, including compilation of the five-pass example.
- 26 GPU cases pass: CPU distance reference for rectangle/ring/disconnected
  geometry across 8/16/32 bpc, packed depth identity, nine material depth/scale
  combinations, Amount zero, partial opacity, extreme linear HDR, filtered-color
  reference and isolated Snell displacement reference.
- 27 formal AE 26.5x89 frames pass: project depths 8/16/32, Full/Half/Quarter,
  times 0/1.5/3 seconds. These Photoshop outputs are 8-bit display evidence,
  not a claim that the exported files preserve 32-bit HDR precision.
- Native sampleImage checks pass: partial alpha is applied once (maximum error
  0.00000502), exterior and hole interiors remain exact. FFX adapts to a renamed
  arbitrary Bezier. Applying the script twice leaves one effect.
- The editable local project is saved at 32 bpc, with three original shapes,
  an empty render queue and no regression sampler/Bezier. Background readers
  stay managed by the plug-in.

## Failure history and corrections

The initial GPU inverse-trig displacement differed from the CPU reference by
0.002992 pixels. Direct displacement output isolated this from texture filtering.
The mathematically equivalent sine/cosine ratio reduces the error to 0.00000423
pixels; filtered color error is 0.00000212. This keeps the upstream optical model.
An early diagnostic shader left unreachable code and failed Naga validation;
the diagnostic was corrected. SciPy was unavailable, so the independent distance
reference uses NumPy directly. One Rust invocation pointed at the SDK container
instead of its Windows root; the corrected invocation passes all 254 tests.
Initial failure logs remain in the evidence set.

## Limits

Distance is measured from raster alpha support and capped at 64 logical pixels.
It is not an analytic reconstruction of paths; reduced-resolution previews have
reduced spatial detail. Explicit Linear RGB Input selects linear-light handling.
This shader-only change does not expand the previous host/platform acceptance.
Full MIT notices for both upstream authors are embedded in the source and FFX,
and included as separate package files.

[Evidence and hashes](evidence/liquid-glass-upstream-20260923/README.md),
[test matrix](../TEST_MATRIX.md#tr-liquid-glass-upstream-008--licensed-material-port).
