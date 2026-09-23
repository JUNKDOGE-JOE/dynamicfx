# Coverage readiness: copy, FFX, cache and independent rendering

Final candidate **96dcff77f3b07c1e59cad1785120dcac4d7e3a8dabdcc798ac5ef2ba286a679c**
passes this readiness gate on Windows AE 2026 26.5x89. The native reader remains
`da16133d...`; the diagnostic remains `bc0265d2...`. Both are unchanged. This is
still an acceptance build; the complete production fidelity/MFR/material gate
is not finished and no commit or push was made.

Baseline: public `922845c`, previous production overlays and this frozen source
overlay. Rust 1.97.1; tests and host runs on 2026-09-23. Decisions are ADR-0052
through ADR-0055; the [integration audit](../../11-host-coverage-integration.md)
records current scope and next action.

## Results and failures retained

| Case | Result |
|---|---|
| Original copy/edit/immediate render | FAIL: 42331 wrong float32 alpha words; copied ready=1 points to old reader |
| Resetup-only certificate attempt `9e922d84...` | FAIL: same mismatch; render copy can inherit original flattened bytes |
| Global revocation `6d718619...` | Old pixels refused, but automatic recovery fails on a reused layer ID after reopen |
| Recovery correction `152fb695...` | Copy/FFX recovery exact; independent rendering fails because aerender also sends normal resetup |
| Headless query fix `2d56f597...` | No native render callback: old cached output; not accepted as execution evidence |
| Cache generation correction `9a055c76...` | Prepared independent frame exact |
| Final `96dcff77...`, effective readiness in frame GUID | Warm copy, FFX, independent frame and depth regression pass |

The final warm-copy case changes the triangle while keeping the same bounds,
after warming the original frame, with no purge before the copied frame. Old
authorization is rejected with E60 and 140000 opaque pass-through words; after
idle, all 480000 canvas alpha words match the new original. New and existing
FFX targets behave likewise; existing effect count stays one. These transient
E60 frames are intentionally not described as successful glass output.

Prepared independent aerender uses a new process, MFR OFF, one 32-bpc frame.
It returns 0, logs no E60 and matches all 480000 native words. A missing-reader
project also returns 0 but remains E60/pass-through; pixel/log checks, not exit
status, determine the result. Final ordinary/adjustment × 8/16/32 bpc gives
6 further comparisons, all exact.

## Save-time validation qualification

The `pending` harness initially expected an unverified copy saved in the same
JSX call to fail closed. Instead, AE ran owner validation during saving. The
saved file contains a valid DFXC certificate `238575237021379`; readback shows
owner 54 bound to reader 56 whose source is 54, and the independent frame equals
the new-shape reference. The initial `pending-summary.json` therefore prints
FAIL against an incorrect E60 expectation. It is retained unchanged and is
assessed as **correct save-time validation**, not as a test of accepting zero
authorization. Removing the actual reader and saving a separate `missing`
project supplies the valid negative control: E60, no stale coverage accepted.

Other harness failures are retained: copyToComp inserted relative to selection,
not at index 1; the first script hit a locked reader before applying FFX and
was resumed on the read-back shape ID. The localized TIFF output template was
refused; the existing ASCII Photoshop template was used. A render was attempted
before that project had been saved and yielded no frame. Neither is host
feature acceptance. Failed candidate AEXs/AEPs are local-only backups.

## Verification

```text
python spike/host-outline/verify-coverage-readiness.py docs/audits/evidence/coverage-readiness-20260923
```

The verifier recomputes mismatches from losslessly compressed native alpha
logs, checks E60 evidence and FFX replacement count, and independently verifies
the final depth matrix. It does not rely solely on summary PASS fields. Native
words are placed at their recorded origins on the 800×600 canvas; no PNG
precision is used for the equality claims.

Final local checks: `cargo +1.97.1 test --offline --locked --lib`, the editor
variant, and default without the SDK each pass **249 tests**. SDK-enabled
`cargo +1.97.1 build --release --locked --offline --target
x86_64-pc-windows-msvc` passes. `DYNAMICFX_AESDK_265_ROOT` selects the ignored
local SDK directory for SDK-enabled runs. Earlier passing/failing candidates
are clearly separated by their install records and logs.

[Manifest](manifest.json) hashes frozen source, logs, readbacks and harnesses.
SDK material, AEX/AEP/FFX/output binaries and ignored lockfiles are excluded.
Native logs are byte-preserved gzip; other logs normalize local workspace/home
paths and MCP artifact/recovery identifiers. Windows aerender stdout is decoded
from its local encoding for readable public evidence. Harnesses document the
actual run and must not be blindly replayed against authoring projects.

Remaining: full production animation, 3D, modifiers, ROI/downsample/PAR,
expanded/empty sources, helper tamper/foreign references, true concurrent MFR,
final material and packaging, broader host regression. Image fidelity still
precedes preview-speed optimization; no push until the full agreed gate passes.
