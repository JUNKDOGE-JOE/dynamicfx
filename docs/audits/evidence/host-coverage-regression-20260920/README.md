# AE 26.5 coverage acceptance — FAIL

The available diagnostic regression is complete and fails acceptance. The
production coverage feature remains unimplemented. No push or merge followed
this run; the existing remote feature commit remains `922845c`.

Baseline: Windows, AE 2026 **26.5x89** (newer than the prior 26.3x87 run),
production AEX `c91db8c0...`, diagnostic 0.0.9 `f0bd26c4...`, public branch
`codex/host-coverage-probe` at `922845c`. The frozen 20-item test AEP was copied
into a separate workspace output directory; the initially empty AE project was
verified before opening it. MCP handled all project operations. Background
processes used Hidden startup; no foreground activation or mouse input.

## Results and evidence

[Summary](raw/summary.json) lists exact outcomes and limits.
[Local tests](raw/local-tests.json): 228 default + 228 editor + 13 diagnostic.
[Native cases](raw/native-cases.json), [8 bpc](raw/depth-8/native-cases.json),
[16 bpc](raw/depth-16/native-cases.json), [32 bpc](raw/depth-32/native-cases.json)
show pixel-level comparisons. The initial curve was actually straight because
its tangent-array assignment did not persist; corrected tangent readback and
all depth/animation reruns are retained. Its first apparent PASS is not curved
coverage evidence.

- **FAIL:** original add/subtract masks do not affect sampled coverage; maximum
  error 255/255. **FAIL:** 8-bpc curved edge, maximum 16/255.
- **FAIL:** full-size half opacity exhausts the query budget. The
  [result](raw/full-half.result.json) records 5.203 seconds and disabled state.
- **FAIL:** [lifecycle](raw/lifecycle-checks.json) does not delete the carrier
  on last-owner disable; [longer wait](raw/cleanup-longwait-end.json) still
  reports two masks. Undo is inconclusive as an independent bug because the
  expected preceding removal/recreation did not happen. Idle scheduling is a
  lead, not an established cause; no foreground workaround was used.
- **PASS subset:** simple geometry, group transform, small half-opacity cases,
  corrected 16/32-bpc curve and shuffled animation samples. These are expression
  readbacks compared to 8-bit PNGs, not full production shader tests.
- **PASS:** [interactive PF](raw/baseline-pf-comparison.json), 3648 vertices,
  maximum 1/255 against a fresh native reference. [Timing](raw/baseline-pf-timing.json)
  is end-to-end for two images, not an isolated frame benchmark.
- **PASS:** [save/reopen](raw/reopen.json) retains 52 items and ownership state.
- **PASS subset:** [supervised aerender exit](raw/aerender-supervised-exit.json)
  is 0; three PSD frames were actually produced, despite the intended two-frame
  duration (the source comp is 25 fps). [RGBA comparison](raw/aerender-pixel-checks.json)
  is within 1/255. The first valid helper's [PF record](raw/aerender-first-valid-pf-checks.json)
  agrees with reference; the supervised repetition uses cache and has no fresh
  PF record. `-mfr ON 50` is not proof of concurrent plugin rendering because
  the diagnostic has no threaded-rendering capability flag.

## Full feature gate

| Scenario | Gate result | Reason |
|---|---|---|
| HS-01 baseline | BLOCKED | Diagnostic acquisition passes; no shader resource exists |
| HS-02 enumeration | PASS historical | Existing native enumeration retained; not rerun here |
| HS-03 reusable FFX | BLOCKED | No production material/preset for the new resource |
| HS-04 animated shape | BLOCKED | Sampled diagnostic subset passes; full runtime integration absent |
| HS-05 all transforms | NOT_RUN | Group subset passes; full layer/parent/3D matrix unimplemented |
| HS-06 canvas/ROI | BLOCKED | No coverage staging ABI; small fixed-coordinate PF probe fails |
| HS-07 masks/modifiers | FAIL | Original masks and holes are absent |
| HS-08 final adjustment shader | BLOCKED | Probe passes input through; no refraction resource |
| HS-09 cold/MFR/cancel | BLOCKED | Cold diagnostic works; threaded/cancel contract unimplemented |
| HS-10 lifecycle/FFX | FAIL | Cleanup fails; FFX part remains blocked |
| HS-11 compatibility/depth | FAIL | 8-bpc edge error; native float/shader matrix not established |
| HS-12 temporal graph | BLOCKED | No resource integration or temporal contract |
| HS-16 visible alpha | FAIL | Mask, edge and full-size partial-alpha failures |

The gate is not complete feature acceptance. Implementation gaps cannot be
converted into tests that pass. Next action: fix acquisition semantics and the
observed lifecycle behavior, then rerun failing cases before FFX/ABI integration.

## Reproduction and preservation

Raw setup scripts, per-call arguments/results, image pairs, driver revisions,
logs and failed attempts are in [the manifest](manifest.json). For interactive
calls use the repository MCP client; `native_cases.py`, `depth_runner.py`,
`animated_runner.py` and `lifecycle.py` reproduce the recorded fixture routes.
Hard-coded IDs belong only to this disposable project. The original small PF
attempt hit diagnostic fixed coordinates and is not general product evidence.
The TIFF template attempts failed, while the runtime ASCII Photoshop template
succeeded. The initial aerender missing-file error and unsupervised attempt
are kept alongside the supervised run.

Each manifest row contains original and evidence SHA-256 values. Only local
user-directory components and MCP artifact IDs are redacted in evidence text.
Binary pixel files remain unchanged. The original AEP and raw records remain
under the ignored workspace output directory. [Standby](raw/standby.json)
confirms a saved 55-item project, 16 bpc, and 22 reset diagnostic controls.
