# Native Apple Silicon AE acceptance — 2026-09-08

**Result: PASS for the documented native AE 2026 subset on final artifact e239.**
The first binary loaded and rendered natively, but a real Half viewport check
exposed a canvas regression. The corrected binary was installed after macOS
authentication and passed fresh Full/Half/Quarter, depth, persistence and
independent aerender checks. The first-binary failure remains recorded.

## Baseline and identity

- Fetched `origin/main`: `d4477ab6256d5fa712993f378b85c21fbe94bdf9` (public 0.0.6).
  Work branch: `codex/macos-shader-quality`; prior local WIP branch preserved.
- Host: macOS 26.5.2 build 25F84, native arm64, Apple M5; AE 2026 **26.3x87**.
- Rust/Cargo: 1.97.1 via installed `stable` alias; Naga/wgpu 29.0.4.
- Previously installed first binary: SHA-256 `b3bd67cf021b453a8616a8e5e3c89a444da7822d80276b4c55629afdafd3c480`, 7,535,968 bytes.
- Installed and accepted corrected binary: SHA-256 `e239ae457ab33e1bc9466d5ab0bff8b81c08861d072eb5e667b5cd8bc6ae8b0b`, 7,536,144 bytes.
- Destination: `/Applications/Adobe After Effects 2026/Plug-ins/DynamicFx/DynamicFx.plugin`.
- [Complete artifact identities](build-identities.json), [source hashes](source-manifest.json),
  [native source diff](native-source.patch), [build/test commands and logs](build/README.md).
  Actual source/build context is kept distinct from packaging-time metadata.

## What ran

The first installation used the verified installer after AE had quit and
preserved the previous plugin. macOS required administrator authentication
for Adobe's root-owned plugin directory. The source, installed binary and PiPL
hashes matched; [install record](build/install-ae2026.json).

The [host harness](../../../../scripts/macos/ae_smoke.py) authored expression-only
GLSL into six temporary compositions, waited for published State Token words,
and used AE `sampleImage` plus numeric checks. The original AppleScript call
waited without a verified outcome and its client was interrupted. Subsequent
JSX ran through the already available local ae-mcp transport. This is only a
test driver; DynamicFX does not depend on that bridge. Exact JSX, outcomes,
fixtures and failures are retained in [first-run/](first-run/README.md).

| Obligation | First b3 artifact | Corrected e239 artifact |
|---|---|---|
| ARM resource/symbol/signature/build | PASS | PASS |
| Default/editor unit suites | 181 / 181 PASS; Siri compile separately PASS | 184 / 184 PASS each |
| Native discovery, addProperty, expression publication | PASS | PASS, including new clean demo |
| One/three pass, 8/16/32-bpc and HDR | PASS, 22 numeric checks | PASS, 22 numeric checks |
| Temporal window random-order samples; invalid source passthrough | PASS | PASS |
| Two keyed values; save and reopen readback | PASS | PASS, 22 repeated checks and saved keys |
| Fresh independent aerender, odd 321×239 analytic fixture | PASS, PSD pixels verified | PASS, every pixel at 321×239 / 161×120 / 81×60 |
| Complex Siri example Full viewport | PASS, visibly complete frame | PASS, complete frame |
| Complex Siri example Half viewport | FAIL, only top-left portion visible | PASS, complete frame |
| Quarter viewport / reduced-resolution numeric export | NOT_RUN | PASS, complete frame; 39 export checks |
| Native Details button | NOT_RUN | PASS, compiled / 1 pass / 12 params / E0 |

The live preview reproduction used `DFX_siri` at 1280×720, switched the AE
Composition resolution menu from Full to Half and back, and reproduced a
complete Full rim versus cropped Half rim. Source, parameters and time were
unchanged. This was a viewport observation, separate from `saveFrameToPng`
exports; those exports do not prove the viewport behavior. Source analysis
found `PF_InData.width/height` are logical/full-resolution but ADR-0039 canvas
resolution consumed them as physical dimensions. The fix converts each axis
once before checkout/canvas resolution, preserving physical rectangles and
existing margin rules. Regression tests include Full/Half/Quarter, odd sizes,
nonuniform ratios, upstream bounds and declared expansion. Odd dimensions
retain the existing outward rounding and reconstructed logical-size policy.

## Final host gate completed

The [final-run record](final-run/README.md) preserves installed identity,
exact JSX and numeric results. The final binary repeated the six-fixture
capture and saved-project readback rather than inheriting first-run PASS.
Actual Full→Half→Quarter Composition-menu changes at t=1 preserved all four
rims; verbose geometry showed the corresponding 1280×720, 640×360 and
320×180 physical canvases. Screenshots were inspected in the live tool
conversation; no whole-window captures containing unrelated panel history
are published. Manual observations are paired with geometry and pixel evidence.

A separate aerender invocation used fresh source content (three-pass UV ramp,
blue 0.625, never previewed before the saved run) and the complete Siri example.
It exited 0 and wrote 12 PSD files across Full/Half/Quarter. The odd-sized ramp
matched its analytic per-pixel oracle within one U8 level at 321×239, 161×120
and 81×60. The [export checker](../../../../scripts/macos/check_exports.py)
passed 39 checks. Full native Siri versus same-time float production-renderer
reference differed by at most 0.0019607831; Half/Quarter versus box-reduced
native Full had mean errors 0.00070544/0.00090892 and p99 errors
0.00490196/0.01593137. These are fixture-specific comparisons, not a universal
quality or exact downsample-equivalence claim.

The final clean one-composition demo was newly authored with the accepted
plugin, published, and saved at 32-bpc / Full:
`scripts/out/macos/ae2026/Siri-Glow-macOS.aep`. All earlier temporary projects
remain saved. The final installation backs up the first b3 bundle, as recorded
in the new install JSON. The native Details dialog was also opened and
confirmed `compiled: 1 pass, 12 params`, E0, then dismissed normally.

The corrected local bundle and extraction-verified archive are ready at
`target/macos-arm64/DynamicFx.plugin` and
`scripts/out/macos/DynamicFX-0.0.6-macos-arm64-local.zip`. These are local ad-hoc
signed development artifacts, not a notarized release. AE 2024, Intel/Rosetta,
Windows acceptance of the current diff, Path downsample semantics, full MFR
performance coverage and public distribution were not validated by this run.
