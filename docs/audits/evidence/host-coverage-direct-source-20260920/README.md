# Native source acquisition and return bridge

The original host pixels are available through a cross-layer PF input. A
test-only invisible reader can return those pixels to an effect on the original
adjustment layer without resampling. This acquisition/transport fixture passes
at 8/16/32 bpc; automatic ownership and production liquid glass are not complete.

## Baseline and evidence

Windows AE 2026 26.5x89, public `922845c` plus [frozen source](source/lib.rs).
The final installed diagnostic is 0.0.23, hash recorded in
[installation](raw/install-v023.json). Production DynamicFx.aex remains unchanged.
All AE access uses MCP scripts and native callbacks; restarts use hidden startup,
with the disposable source matrix saved first. No mouse automation or push.

The SDK's [ABI metadata](raw/sdk-abi.json) identifies StreamSuite7 version 12.
[stage.cpp](source/stage.cpp) compiles against the official SDK headers rather
than extending the older Rust vtable by assumption. SDK archives, headers,
binaries and AEP files are excluded from this record. Native logs are losslessly
compressed; the [manifest](manifest.json) records original, normalized and stored
hashes, including local-path and MCP-artifact-ID redactions.

## Findings

- [Source-item matrix](raw/source-item-matrix.json): footage and precomp return
  transparent source pixels; solid returns its own red pixels, not blue
  background. Generated shape returns no source item.
- [Self stage readback](raw/stage-inspect-self.native.log.gz): SOURCE is already
  selected. SOURCE/ONLY_MASKS and atomic layer/stage setters all succeed, but
  self pixel checkout still returns background. The [full self matrix](raw/self-pixel-matrix.json)
  records 140000 differing native alpha words per 800x600 case, at every depth.
- [Cross-layer matrix](raw/stage-pixel-matrix.json): SOURCE and ONLY_MASKS match
  ordinary-layer reference alpha exactly at 8/16/32 bpc, on both parameters.
  This includes partial fill and mask opacity, subtract, anisotropic feather
  and expansion; it does not claim all masks/modifiers/transforms are accepted.
- [Return bridge](raw/bridge-pixel-matrix.json): the reader's input selects
  ONLY_MASKS on the original. Its diagnostic mode 22 copies native pixel bytes
  using recorded origins. The original effect selects ALL_EFFECTS on the reader.
  Reader video is off; no original geometry is copied or rebuilt. Every tested
  alpha word returns unchanged. The image comparison uses native words, not PNG
  precision; 32-bpc comparisons use float bit patterns.
- [Positive-stage refusal](raw/bridge-first-effect-refused.native.log.gz) and
  [visible-reader control](raw/bridge-enabled-stage.native.log.gz) both return
  error 3. The limit query alone therefore did not prove a working connection.
  ALL_EFFECTS (-1) succeeds in the subsequent actual checkout tests.
- [Options v2 plain request](raw/plain-shape-adjustment16.native.log.gz) and
  [options v1 request](raw/plain-options1-shape-adjustment16.native.log.gz) return
  error 3. Neither yielded a receipt. The older upstream/plain failure stays
  retired and is not silently replaced by this result.

## Reproduction and verification

Build the diagnostic with `DYNAMICFX_AESDK_265_ROOT` set to a local SDK 26.5 root:
`cargo test --offline --locked --manifest-path spike/host-outline/native/Cargo.toml`,
then `cargo build --offline --locked --manifest-path spike/host-outline/native/Cargo.toml`.
The [test log](raw/v023-test.log) records 19 passes, including byte-exact origin
and stride-aware copying. An SDK-absent build explicitly refuses stage calls.

The frozen MCP drivers and result files record the exact procedures and fixture
IDs; those IDs belong to the disposable recorded project, not arbitrary projects.
The reference and external fixtures are 800x600 at 25 fps, with triangle vertices
[200,150], [600,150], [400,500], identity layer transforms and blue background.
The mask rectangle is [350,230]..[450,370], subtract mode, 68.75% opacity,
[11.25,7.75] feather and 4.25 expansion; fill opacity is requested at 37.125%.
Reference SOURCE disables the mask, reference ONLY_MASKS enables it. Every frame
is cache-purged and checked at time zero. These are static/identity tests.

Run `python spike/host-outline/verify-source-stage.py
docs/audits/evidence/host-coverage-direct-source-20260920` to independently compare
all 18 external/bridge outputs against native references and verify the six
self-checkout failures. It restores cropped worlds to the recorded 800x600
canvas; malformed, duplicate or absent checkouts fail validation.

## Approved scope and next action

After the full bridge passed, the user explicitly accepted an automatically
maintained internal reading layer. No manual binding or original-shape editing
workflow is introduced. This supersedes the prior no-helper-object restriction;
it does not waive full-scope acceptance or fidelity. No push until tests pass.

Next: implement reader ownership and rebinding in the isolated diagnostic,
then verify duplicate isolation, last-owner cleanup, Undo/Redo, save/reopen and
requested-time invalidation. Shader resource/persistence ABI, FFX, coordinate
transforms/ROI, MFR and older host availability remain open. A single AE 26.5
run does not establish Windows 2023–2026 support or macOS runtime support.
