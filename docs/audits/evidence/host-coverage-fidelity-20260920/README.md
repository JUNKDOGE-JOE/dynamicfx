# Coverage fidelity investigation — exact baseline, delivery blocked

The standalone diagnostic now has a verified exact sampling/transport baseline.
It is not the production coverage feature. Full scope and fidelity-first delivery
remain required; no commit, push, release or main update occurred.

Baseline: Windows, AE 2026 26.5x89, public `922845c` plus isolated diagnostic
changes. Current installed probe 0.0.14 SHA-256
`6734a384d3f538aebf8d09725f9db3f359d9448aa893ed14ae81eb8c5ecd1e71`;
production remains `c91db8c0...`. Every project action uses MCP on a copied
scratch project. All replacement installations verify AE is closed, keep a
backup, verify hashes and use Hidden startup. No foreground activation command.

## Correctness findings

The adaptive sampler's area average can quantize to zero while individual edge
pixels remain visible. [Direct oracle](raw/direct-8-checks.json) initially records
the curve; both saved CSV/PNG pairs later contain all 12288 pixels and compare
exactly. The first two-case call timed out at 40 seconds; its outputs arrived
later, followed by a successful host read. It was not replayed.

The [coordinate probe](raw/coordinate-precision.result.json) shows that direct
alpha coordinates and fractional-width encoding lose precision. The new
[exact expression](source/coverage-exact-expression.js) samples each pixel and
merges only identical float32 values. Its three-point rectangle records split
float32 bit patterns into integer/half-integer symbols. AE introduces small
coordinate noise; the [decoder](source/verify-exact-coverage.py) validates a
bounded rounding margin and rejects malformed, overlapping or out-of-bounds
records. The [native PF result](raw/exact-native-checks.json) matches all 12288
oracle float32 words. [Small constants](raw/packed-float-test.result.json)
include 1e-10 and values next to 1, which the old coordinate encoding loses.

[Curve 16](raw/exact16-curve-checks.json), [half 16](raw/exact16-half-checks.json),
[curve 32](raw/exact32-curve-checks.json) and [half 32](raw/exact32-half-checks.json)
all match native upstream AE reference worlds bit-for-bit. Those worlds are
cropped to content bounds; comparisons restore their recorded origin into the
128x96 canvas. These tests compare expression-path data with native references,
not production shader outputs or full MFR transport.

## Precision-preserving speed experiment

A full-frame sample warms the same source raster before individual pixel reads.
It does not supply alpha values to the result or authorize skipping pixels.
[Cold checks](raw/warm-cold-suite-checks.json) preserve the verified 8/16/32-bpc
curve/half data. The cold half-opacity expression changes from
29.888 seconds to 1.463 seconds. Earlier unwarmed observations and the separate
experimental warm-up expression are retained; the current source includes the
validated warm-up. Do not generalize this single comparison to a preview FPS.

The original large half-opacity failure is now reproduced on an 800x600 clone
with the same 50% fill. [Full-frame comparison](raw/exact-full-half-checks.json)
is exact across 480000 pixels, encoded in 3746 vertices. The recorded expression
takes **91.529 seconds**, so this is a correctness result and an unacceptable
interactive cost. The sampler is still a diagnostic with pixel/data budgets;
its results do not accept higher resolutions or all shapes.

## Native masks and remaining API gap

PF_MaskWorldWithPath supplies add/subtract mask pixels without render-thread
AEGP access. [Native 16-bit checks](raw/native16-mask-checks.json) match the two
zero-feather, full-opacity reference cases exactly. This diagnostic fixes those
parameters; it does not read arbitrary animated mask metadata or implement the
complete mask stack.

[Expansion +8](raw/expansion-checks.json) fails at 834 pixels, reaching 32768
native alpha units. The API omits mask expansion, also reported in Adobe's
[SDK discussion](https://community.adobe.com/questions-529/how-do-you-get-the-pixels-inside-a-layer-mask-30214).
Our current-host reproduction is the acceptance evidence. postEffect=true
[sampling](raw/post-effect-probe.result.json) returns adjustment background
alpha, so it does not solve the missing original coverage.

The faster source-geometry raster experiment is also not accepted: native
16-bit differences and float-conversion model differences remain in
[geometry checks](raw/geometry-raster-checks.json),
[quantization experiments](raw/geometry-quantization-experiments.json) and
[float models](raw/geometry-float-models.json). No approximate geometry shortcut
has replaced the exact sampler.

## Product boundary awaiting a decision

The original requirement excludes helper layers/precomps. Accurate full-frame
sampling remains slow, while native mask rasterization lacks expansion. The user
has been asked whether automatically managed internal reference objects are
allowed. No answer is assumed; no production proxy or additional required user
workflow has been introduced. Test-only reference layers already serve as
independent native oracles, not as a completed automatic proxy implementation.
Automatic ownership, expression context, FFX reconstruction, transforms and MFR
would still require implementation and separate evidence if that route is allowed.

## Verification and preservation

[17 native tests](raw/v014-test.log), [build](raw/v014-build.log) and
[codec tests](raw/exact-warm-tests.log) pass. The 0.0.11/12/13 intermediate
binaries are preserved in the ignored output directory, with install hashes.
The [manifest](manifest.json) freezes raw records and final source snapshots;
local user components and MCP artifact IDs are redacted with original/evidence
hashes. Binary test images are unchanged. Native-world alpha arrays retain full
integer/float32-bit data rather than only 8-bit exports.

[Standby](raw/standby.result.json) records a saved 69-item, 16-bpc test project
with 34 diagnostic request/log controls reset. Expensive exact expressions are
disabled after their reads. AE remains open; the production plugin is unchanged.
