# Production coverage first frame and initial ownership

**PASS within the recorded fixture scope.** Production `hint:coverage` reaches
the real shader graph through an automatically managed native reader. Six
ordinary/adjustment × 8/16/32-bpc comparisons each match all 480000 native alpha
words. Eight ownership cases and one immediately rendered, same-process reopen
frame also pass. Copy/FFX-before-idle readiness and complete delivery acceptance
remain open; this is not a final glass preset or release.

Baseline is public `922845c` plus the
[resource foundation](../coverage-integration-20260923/manifest.json),
[stage adapter](../coverage-stage-20260923/manifest.json) and this source overlay.
Windows, Rust 1.97.1, AE 2026 26.5x89; date 2026-09-23. Installed artifacts are
listed in [installation readback](raw/installed.json). Main `e72ac937...` and
reader `da16133d...` are installed only in the AE 2026 DynamicFx directory.
The original main AEX was backed up; the independent diagnostic was unchanged.

## Pixel method

The [shader](harness/coverage.glsl) fetches coverage by texel and writes it to
output alpha. A native diagnostic placed after DynamicFx records that output.
An ordinary AE shape without DynamicFx supplies the reference alpha. The source
triangle has 37.125% fill, subtract mask, 68.75% mask opacity, anisotropic feather
and 4.25px expansion. Readback is native words, not PNG values. Cropped worlds
are placed at their recorded origins in the common 800×600 canvas. The verifier
also requires nonzero partial alpha, an actual hole and transparent exterior.
A negative control disables DynamicFx: all 140000 words in the checked-out
world become opaque background alpha and differ from the reference. The
effect is then restored and the project saved. Pass-through cannot satisfy
the positive comparison.
The first two control captures contained two different id-0 frames in the
same log window. The first script's PASS inspected only the first frame and
is not accepted; the strict verifier rejected it. Disabling the diagnostic's
self selector alone did not isolate the window. The accepted final capture
temporarily silences diagnostic output in the other compositions, contains
exactly one opaque frame, and restores all probe modes afterwards. Both mixed
captures remain in the evidence.

Run from repository root:

```text
python spike/host-outline/verify-production-reader.py docs/audits/evidence/coverage-owner-20260923
```

The [verifier](../../../../spike/host-outline/verify-production-reader.py) reads
compressed native logs and raw lifecycle snapshots independently of the summary
PASS fields. It checks all 6 frame pairs, reopened float32 frame and 8 ownership
cases. Native logs are losslessly gzipped; their captured bytes are unmodified.

## Host workflow and limits

MCP confirmed an empty, unmodified project before scheduled quit and hidden
restart. First fixture creation timed out while loading effects; the bridge
recovered, native creation logs and readbacks confirmed completion, and creation
was not replayed. The first read-only inspection hit an AE no-value group and
was corrected. No mouse or foreground operation was used.

Ownership checks cover creation, shared reader, first/last opt-out, unused-source
cleanup, Undo/Redo, re-enable and duplicate original with separate links after
idle. The saved baseline is restored at the end; the lifecycle variant has its
own ignored AEP copy. The reopened frame is requested in the same JSX call,
without a separate idle wait, but is not cold-process/aerender acceptance.

Still NOT_RUN: immediate copy/FFX authorization, helper transform/time edits,
full foreign-reference protection, owner deletion, empty alpha, large sources,
production animation/3D/modifiers, ROI/downsample/PAR, real MFR and final material
packaging. The source keeps schema v1 while that clone protocol is designed.

## Local checks and evidence

SDK-enabled default/editor and SDK-absent default tests: 241 each. Native reader
tests: 3. Both Release builds pass. SDK builds set `DYNAMICFX_AESDK_265_ROOT` to
the ignored local SDK root. Main command is `cargo +1.97.1 build --release
--locked --offline --target x86_64-pc-windows-msvc`; reader adds
`--manifest-path coverage-reader/Cargo.toml --target-dir
scripts/out/coverage-owner-20260923/reader-target`. Library tests use
`cargo +1.97.1 test --offline --locked --lib` with the recorded feature setting;
reader tests use its manifest and target directory. Initial compile failures
are retained alongside final passing logs.

[Manifest](manifest.json) hashes the source, readbacks, logs and local harness.
SDK headers, AEX/AEP binaries and local lockfiles are excluded. Workspace/home
paths and MCP artifact/recovery identifiers are normalized in public logs.
Harness files document the actual run, but their local paths must be supplied
for reproduction; do not replay mutation scripts on an authoring project.
See the [integration audit](../../11-host-coverage-integration.md).
