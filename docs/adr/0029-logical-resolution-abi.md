# ADR-0029: `u_resolution` is the logical full-resolution frame size

- Status: Accepted
- Date: 2026-08-14
- Deciders: user (defect report relayed from first users) + assistant session

## Context

Users report shaders changing appearance when the viewer preview resolution
drops below Full. Measured: at Half resolution AE hands the effect a
downsampled buffer (1280×720 comp → 640×360 input world; verbose render log
`in=640x360`), and `u_resolution` reported those physical dims. Any
pixel-based shader math (offsets built from `1/u_resolution`, halftone
pitches, `uv * u_resolution` grids) therefore doubles in visual size — the
16-px stripe probe rendered ~20 stripes instead of 40 (screenshot in
evidence). The AE SDK's own downsample convention says a 4-pixel blur
should behave as a 2-pixel blur at 1/2 downsample — i.e., effects must
compensate.

## Decision

The Shader ABI's `u_resolution` builtin reports the **logical
full-resolution frame size**: physical buffer dims scaled by the inverse of
`in_data.downsample_x/y` (`logical = physical × den / num`; degenerate
ratios fall back to physical). At 100% preview the value is unchanged.
Geometry — buffers, rects, ROI, converters, taps — stays physical
throughout the runtime; only the two floats the shader sees change.

Consequences for shader authors: `uv`-space math (already full-frame per
ADR-0011 §6) plus `u_resolution`-based pixel math is now invariant across
preview resolutions — what you see at 50% is the final look, scaled.
Shaders that want the physical texel size have no builtin for it in v1; if
demanded, a `u_downsample` builtin can be appended by a future ADR without
breaking this one.

## Verification

- Unit: `render::logical_size` ratio math (full/half/third, degenerate
  fallbacks).
- Host: 16-px stripe probe at Half preview resolution — before: ~20 wide
  stripes (physical `u_resolution`); after: 40 stripes visually matching
  Full resolution (screenshots in `evidence/`); regression batteries green
  (at Full resolution the value is bit-identical, so all existing suites
  gate the no-change case).
