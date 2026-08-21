# DynamicFX test matrix

> **This file is the only authority for verification status.**  
> A prose claim elsewhere is not a test result. Every target-rewrite entry starts as `NOT_RUN` and changes only after a documented run.

## Status definitions

| Status | Meaning |
|---|---|
| `PASS` | The exact documented command/procedure ran successfully with complete required evidence |
| `FAIL` | The documented run completed unsuccessfully; failure evidence is retained |
| `NOT_RUN` | No valid run exists for this target and baseline |
| `BLOCKED` | A named prerequisite prevents execution |
| `CLAIMED_UNVERIFIED` | Historical prose claims success, but required evidence is unavailable |
| `PROTOTYPE_BASELINE` | Result applies only to the pre-rewrite prototype and cannot verify target architecture |

`PENDING_LOG` is an observed script state, not PASS. If used in a raw artifact, the matrix result remains `NOT_RUN`, `FAIL`, or `BLOCKED` until the expected log is collected and verified.

## Required evidence for PASS

Every applicable PASS entry must record:

```text
Baseline commit or exact working-tree diff:
Operating system:
AE year and full version/build:
Plugin artifact path, size/hash, and install destination:
Command or exact UI/script steps:
Date/time:
Expected result:
Observed result:
Raw log/PNG/AEP/report path:
Related milestone audit:
```

A Rust-only run may omit AE fields but must still record toolchain, command, baseline, time, output, and audit.

## Prototype baseline

These entries describe code scheduled for replacement.

| ID | Test | Status | Baseline | Evidence | Scope warning |
|---|---|---|---|---|---|
| PB-RUST-001 | `cargo test --all` | `PROTOTYPE_BASELINE` | `6956cf6` working tree during project-understanding session | Conversation terminal output recorded 19 passed, 0 failed; no repo-local raw log was preserved | Does not verify any target rewrite component |
| PB-AEX-001 | Prototype release AEX build | `CLAIMED_UNVERIFIED` | Prototype docs claim a release build | No complete repo-local build artifact record in this matrix | Do not convert to target PASS |
| PB-AE25-001 | Prototype AE 2025 base render | `CLAIMED_UNVERIFIED` | Prototype docs | No complete host build/log/fixture evidence linked here | Historical context only |
| PB-AE26-001 | Prototype AE 2026 programmatic path | `CLAIMED_UNVERIFIED` | Prototype docs | No complete host build/log/fixture evidence linked here | Historical context only |

## Target rewrite — build and unit layers

### TR-M0-001 — Governance links and decision consistency

- Status: PASS
- Baseline commit/diff: parent commit `fe3ada7` plus the ADR-0009 staging documentation changes committed immediately after this run
- OS: Windows 11 host, validation executed through the repository Python environment
- Rust toolchain: N/A; no Rust build or test was run
- AE year and full version/build: N/A; no AE host was run
- Plugin artifact identity: N/A
- Date/time: 2026-08-10 session; exact wall-clock time was not recorded by the validator
- Command: `python scripts/check_governance.py > docs/audits/00-governance-check.txt 2>&1`
- Expected: required handoff paths exist; all local Markdown links resolve; Mermaid blocks pass structural checks; nine ADRs (0001-0009) are Accepted/indexed; approved clauses exist; target AE host rows contain no PASS; `git diff --check` passes; no `src/**`, `build.rs`, or `Cargo.toml` diff exists
- Observed: 23 Markdown files, 122 local links, 19 Mermaid blocks, 9 Accepted ADRs, 0 errors, `RESULT=PASS`
- Raw artifacts: [governance report](audits/00-governance-check.txt)
- Audit: [M0 Architecture Contract](audits/00-architecture-contract.md)
- Notes: The 2026-08-08 run (22 files, 99 links, 8 ADRs) is preserved at commit `fe3ada7`. This verifies documentation governance only. It does not verify target Rust code, AEX build, AE loading, pixels, persistence, multi-pass, high precision, or performance.

### TR-0036-001 — Publication boundary and governance after single-repo consolidation

- Status: PASS
- Baseline commit/diff: the consolidation commit that first added `docs/`, `CLAUDE.md`, `scripts/m*`, `scripts/f003`, `scripts/spike` and `spike/` to this repository; run against the staged tree immediately before it
- OS: Windows 11 host, Git Bash + repository Python
- Rust toolchain: N/A; no Rust build or test was run
- AE year and full version/build: N/A; no AE host was run
- Plugin artifact identity: N/A; no artifact was built or installed
- Date/time: 2026-08-17
- Command: `python scripts/check_governance.py`, then the four ADR-0036 §"Verification obligations" scans (vendor identity, credentials, withheld-document absence, machine-local paths), all recorded in the raw artifact
- Expected: `RESULT=PASS` with every link resolving after the withheld document's removal; 36 Accepted ADRs indexed; zero matches for the withheld vendor terms and for credential patterns; the withheld document absent from the tree
- Observed: 64 Markdown files, 613 local links, 16 Mermaid blocks, 36 Accepted ADRs, 0 errors, `RESULT=PASS`. Vendor-identity scan: 0 matches. Credential scan: 0 matches. Withheld document: absent. Machine-local paths: 26 files, all original evidence artifacts or their reproduction harnesses, retained deliberately per ADR-0036 §5 and §6.
- Raw artifacts: [publication check](audits/evidence/adr-0036/publication-check.txt)
- Audit: [M0 Architecture Contract](audits/00-architecture-contract.md) (governance surface); decision in [ADR-0036](adr/0036-single-repository-record.md)
- Notes: This verifies the publication boundary and documentation governance only — it proves nothing about runtime code, and it cannot prove that the redacted ADR passages match the archived originals, because the archive is private (ADR-0036 "Costs and risks"). The scan patterns are held in shell variables and deliberately not reproduced in the raw artifact; writing them out would publish the terms the scan exists to exclude.

| ID | Milestone | Test | Status | Evidence/audit |
|---|---|---|---|---|
| TR-M0-001 | M0 | Governance links and decision consistency | [`PASS`](#tr-m0-001--governance-links-and-decision-consistency) | [M0 audit](audits/00-architecture-contract.md); [raw report](audits/00-governance-check.txt) |
| TR-M1-001 | M1 | Target Rust unit suite | [`PASS`](#tr-m1-001--target-rust-unit-suite) | [M1 audit](audits/01-first-frame.md) |
| TR-M1-002 | M1 | Target release AEX build | [`PASS`](#tr-m1-002--target-release-aex-build) | [M1 audit](audits/01-first-frame.md) |
| TR-M1-003 | M1 | Language ID/default GLSL tests | [`PASS`](#tr-m1-003--language-iddefault-glsl-on-the-host) | [M1 audit](audits/01-first-frame.md) |
| TR-M1-004 | M1 | Raw GLSL lowers to one-pass RenderGraph | [`PASS`](#tr-m1-004--raw-glsl-first-frame-through-the-graph-path) | [M1 audit](audits/01-first-frame.md) |
| TR-M2-001 | M2 | Stable ParamId and BindingPlan unit/integration tests | [`PASS`](#tr-m2-001--stable-paramid-and-bindingplan) | [M2 audit](audits/02-keyframed-params.md) |
| TR-M2-002 | M2 | @param annotations, defaults, live alias rename | [`PASS`](#tr-m2-002--param-annotations-defaults-live-alias-rename) | [M2 audit](audits/02-keyframed-params.md) |
| TR-M2-003 | M2 | Value-encoding fixtures (all v1 kinds) + pool overflow | [`PASS`](#tr-m2-003--value-encoding-fixtures-and-pool-overflow) | [M2 audit](audits/02-keyframed-params.md) |
| TR-M3-001 | M3 | StateToken/sequence schema v1 round-trip and corruption | [`PASS`](#tr-m3-001--persistence-and-render-clone) | [M3 audit](audits/03-persistence-render-clone.md) |
| TR-M4-001 | M4 | Graph parser/validator/scheduler unit suite + host run | [`PASS`](#tr-m4-001--multi-pass-graph) | [M4 audit](audits/04-multipass-graph.md) |
| TR-M5-001 | M5 | 8/16/32-bpc pixel fixtures | [`PASS`](#tr-m5-001--81632-bpc-pixel-fixtures) | [05-pixel-formats.md](audits/05-pixel-formats.md) |
| TR-M6-001 | M6 | Temporal windowed re-simulation fixtures | [`PASS`](#tr-m6-001--temporal-windowed-re-simulation-fixtures) | [06-temporal-feedback.md](audits/06-temporal-feedback.md) |
| TR-M7-001 | M7 | Performance baseline (benchmark matrix, per-render spans) | [`PASS`](#tr-m7-001--performance-baseline-benchmark-matrix) | [07-performance-mfr.md](audits/07-performance-mfr.md) |
| TR-M7-002 | M7 | Optimization 1: GPU resource reuse — before/after + full regression | [`PASS`](#tr-m7-002--optimization-1-gpu-resource-reuse) | [07-performance-mfr.md](audits/07-performance-mfr.md) |
| TR-M7-003 | M7 | WYSIWYG preview invalidation after idle compile | [`PASS`](#tr-m7-003--wysiwyg-preview-invalidation-after-idle-compile) | [07-performance-mfr.md](audits/07-performance-mfr.md) |
| TR-M7-004 | M7 | Optimization: ROI final-pass delivery (uv-preserving scissor) | [`PASS`](#tr-m7-004--roi-final-pass-delivery) | [07-performance-mfr.md](audits/07-performance-mfr.md) |
| TR-M7-005 | M7 | MFR eligibility: concurrency measured, stance confirmed | [`PASS`](#tr-m7-005--mfr-eligibility-review) | [07-performance-mfr.md](audits/07-performance-mfr.md) |
| TR-M7-006 | M7 | M7 exit verification (budget enforcement, 1080p matrix, full battery) | [`PASS`](#tr-m7-006--m7-exit-verification) | [07-performance-mfr.md](audits/07-performance-mfr.md) |
| TR-Y26-001 | Host | AE 2026 full-suite host run (M7 exit artifact) | [`PASS`](#tr-y26-001--ae-2026-full-suite-host-run) | [TR-Y26-001](#tr-y26-001--ae-2026-full-suite-host-run) |
| TR-REL-001 | Release | 0.0.1 release verification (both hosts, packaged artifact) | [`PASS`](#tr-rel-001--001-release-verification) | [ADR-0027](adr/0027-0.0.1-prerelease-scope.md) |
| TR-REL-002 | Release | 0.0.2 release verification (both hosts, packaged artifact) | [`PASS`](#tr-rel-002--002-release-verification) | [ADR-0027](adr/0027-0.0.1-prerelease-scope.md) |
| TR-REL-003 | Release | 0.0.3 release verification (both hosts, packaged artifact) | [`PASS`](#tr-rel-003--003-release-verification) | [ADR-0027](adr/0027-0.0.1-prerelease-scope.md) |
| TR-0026-001 | Feature | Color `default:#RRGGBB[AA]` annotation end-to-end | [`PASS`](#tr-0026-001--color-default-annotation) | [ADR-0026](adr/0026-color-parameter-default-annotation.md) |
| TR-0028-001 | Feature | Details button + float-slider precision (first-user feedback) | [`PASS`](#tr-0028-001--details-button-and-slider-precision) | [ADR-0028](adr/0028-details-button-and-slider-precision.md) |
| TR-0029-001 | Feature | Logical-resolution ABI: preview-downsample invariance | [`PASS`](#tr-0029-001--logical-resolution-invariance) | [ADR-0029](adr/0029-logical-resolution-abi.md) |
| TR-0015-001 | Feature | Not-ready marker: E53 `PublicationPending` published and observable | [`PASS`](#tr-0015-001--not-ready-marker-e53) | [ADR-0015](adr/0015-statetoken-and-diagnostics.md) |
| TR-EX-001 | Content | Shipped `examples/` compile through the real frontend | [`PASS`](#tr-ex-001--shipped-examples-compile) | [IMPLEMENTATION_STATUS](IMPLEMENTATION_STATUS.md#agreed-plan-after-m7) |
| TR-0036-001 | Governance | Publication boundary + governance after single-repo consolidation | [`PASS`](#tr-0036-001--publication-boundary-and-governance-after-single-repo-consolidation) | [ADR-0036](adr/0036-single-repository-record.md) |
| TR-0030-001 | Feature | Layer input parameters (`hint:layer`) end-to-end | [`PASS`](#tr-0030-001--layer-input-parameters) | [ADR-0030](adr/0030-layer-input-parameters.md) |
| TR-0031-001 | Feature | Gradient parameters (`hint:gradient`) end-to-end | [`PASS`](#tr-0031-001--gradient-parameters) | [ADR-0031](adr/0031-gradient-parameters.md), [ADR-0032](adr/0032-gradients-are-graph-resources.md) |
| TR-0034-001 | Feature | Point 3D parameters (`hint:point3d`) end-to-end | [`PASS`](#tr-0034-001--point-3d-parameters) | [ADR-0034](adr/0034-point3d-parameters.md) |
| TR-0035-001 | Feature | Path parameters (`hint:path`) end-to-end | [`PASS`](#tr-0035-001--path-parameters) | [ADR-0035](adr/0035-path-parameters.md) |

## Target rewrite — M0 transport feasibility spike

Defined by [ADR-0009](adr/0009-staged-format-adr-acceptance.md). The spike measures AE host transport behavior that gates envelope size limits (ADR-0012), Popup pool viability (ADR-0013), and the M3 StateToken/DefinitionData decisions. Because it measures host behavior rather than target code, the prototype AEX and JSX probe scripts are acceptable instruments; results are host evidence, not prototype-code claims. TR-M0-005 depends partly on plugin implementation specifics, so its prototype-derived portion is indicative only and must be re-verified against target code at M3.

M0 exit requires complete result records on at least one target AE year (2025 recommended); remaining years fold into M1/M3 host evidence.

Instrumentation (code-complete, not yet executed on any AE host):

- Driver: `scripts/spike/run_spike.ps1` (`-Year 2025|2026`; GUI pass runs s0..s5 + s9 through `AfterFX.exe -r` with RESULT_DONE sentinel polling; `-Aerender` replays the saved s4/s5 projects). Scenario scripts: `scripts/spike/s0_init.jsx`, `s1_expr_ceiling.jsx`, `s2_expr_roundtrip.jsx`, `s3_undo_dirty.jsx`, `s4_probe_plugin.jsx`, `s5_aerender_setup.jsx`, `s9_quit.jsx`. Numeric pixel verification: `scripts/spike/check_png.py`.
- Probe instrument: `spike/probe/` crate; artifact `DynamicFxProbe.aex` (336,384 bytes, SHA-256 `9736C8DD1045EFC314399A22F132B1E53CEE78A89C3BE8494A8F487E1F7007AE`), installed 2026-08-11 to `C:\Program Files\Adobe\Adobe After Effects 2025\Support Files\Plug-ins\DynamicFxProbe\`; match name `DynamicFxProbe`; plugin log `%TEMP%\dynamicfx_probe.log`.
- Raw outputs land in `scripts/out/spike/<year>/` (gitignored); when results are recorded, curated logs/PNGs/AEPs are copied to `docs/audits/evidence/m0-transport-spike/` and referenced from result records here.

| ID | Scenario | Status (AE 2025) | Key result |
|---|---|---|---|
| TR-M0-002 | Expression length ceiling on a Slider Control | [`PASS`](#tr-m0-002--expression-length-ceiling) | Byte-exact set+readback at every size 1 KB→16 MB; 16 MB in 235 ms; no host ceiling below the 16 MB probe cap |
| TR-M0-003 | Long-expression save/reopen fidelity | [`PASS`](#tr-m0-003--long-expression-savereopen-fidelity) | All 4 payload variants × 3 sizes (to 1 MB) byte-exact after save+reopen |
| TR-M0-004 | Sequence transport payload capacity | [`PASS`](#tr-m0-004--sequence-transport-payload-capacity) | 16 MB sequence flatten→save→reopen→unflatten crc_ok; arb-param value write via ParamDef ineffective; 33 MB project crashed AE on overlapping open |
| TR-M0-005 | Undo/redo and project-dirty behavior | [`PASS`](#tr-m0-005--undoredo-and-project-dirty-behavior) | Scripted writes set dirty; save clears it; idle recompile does **not** re-dirty a saved project; undo removes committed expression |
| TR-M0-006 | Popup runtime menu mutation | [`PASS`](#tr-m0-006--popup-runtime-menu-mutation) | set_options+PF_UpdateParamUI return Ok in-plugin, but AE keeps the 4 setup items; setValue(5) rejected out of range — menu is fixed at PARAMS_SETUP |
| TR-M0-007 | aerender parity for long expressions | [`PASS`](#tr-m0-007--aerender-parity-for-long-expressions) | 1 MB expression → opacity 37.5; GUI PNG alpha=96 == aerender PSD alpha=96 |

AE 2026 re-verification of these rows is `NOT_RUN` (task tracked; probe reinstall + cold re-run per year). The spike does not exercise the wgpu/DX12 GPU path (ADR-0014) — s5 renders through AE's native opacity pipeline; GPU verification begins at M1 with the real plugin.

### TR-M0-002 — Expression length ceiling

- Status: PASS
- Baseline: working tree at commit `62359f5` + spike scripts (this session)
- OS: Windows 11 (10.0 build, `$.os` = Windows/64)
- AE year/build: After Effects 2025, `app.version` 25.6.6x4, zh_CN
- Command/steps: `pwsh scripts/spike/run_spike.ps1 -Year 2025 -Scenarios s1` → `scripts/spike/s1_expr_ceiling.jsx` on an `ADBE Slider Control` expression, doubling 1 KB→16 MB, byte-compare set vs readback
- Expected: host accepts large committed expressions with exact readback, or reveals a ceiling
- Observed: every size 1 KB…16 MB `setOk=1 match=1`; timings 1 MB=16 ms, 4 MB=68 ms, 16 MB=235 ms; boundary `firstFail=none` (16 MB is the script's explicit cap, not a host limit)
- Raw: [ae2025/s1.log](audits/evidence/m0-transport-spike/ae2025/s1.log)
- Audit: [M0 Architecture Contract](audits/00-architecture-contract.md)

### TR-M0-003 — Long-expression save/reopen fidelity

- Status: PASS
- Baseline/OS/AE: as TR-M0-002
- Command/steps: `... -Scenarios s2` → `s2_expr_roundtrip.jsx`; variants A ASCII, B hostile punctuation (`"'`\{}();#` etc.), C CRLF line endings, D CJK+full-width+emoji+accents; sizes 4 KB / 256 KB / 1 MB; save `.aep`, reopen, byte-compare
- Expected: committed expression survives save/reopen unchanged for every variant/size
- Observed: all 12 combinations `immMatch=1 reopenMatch=1` (byte-exact); no normalization or stabilization pass needed
- Raw: [ae2025/s2.log](audits/evidence/m0-transport-spike/ae2025/s2.log)
- Audit: [M0 Architecture Contract](audits/00-architecture-contract.md)

### TR-M0-004 — Sequence transport payload capacity

- Status: PASS (with two recorded findings)
- Baseline/OS/AE: as TR-M0-002; probe AEX `DynamicFxProbe.aex` SHA-256 `79DD77DFC1F858E1A958B1A2095F513A8A0158ABAD9559EBA7840171803C0665` installed at `…AE 2025\Support Files\Plug-ins\DynamicFxProbe\`
- Command/steps: `... -Scenarios s4 -ProbeKb 16384` → `s4_probe_plugin.jsx`; plugin `flatten()` emits a 16 MB checksummed payload (size from `DFX_PROBE_KB`, read at process start), project saved and reopened, `unflatten()` verifies magic+length+crc
- Expected: sequence schema v1's carrier (sequence flatten) round-trips a large payload intact
- Observed: `SEQ_FLATTEN env_kb=16384 total_bytes=16777232`; `SEQ_UNFLATTEN body_bytes=16777216 magic_ok=true len_ok=true crc_ok=true`; `.aep` grew to 33 MB (payload persisted, ~2× as hex). Finding 1: writing the arb parameter's value through the after-effects-rs `ParamDef` interface never takes effect (`BLOB_UI_READ kb=0`, `param_last_kb=None`) — scripted `setValue` is not delivered as a committed change, so the arb value is not a usable payload path; sequence flatten is. Finding 2: opening `s5.aep` while the 33 MB project was loaded crashed AE (AdobeCrashReport) — large sequence payloads are viable but should be bounded.
- Raw: [ae2025/s4.log](audits/evidence/m0-transport-spike/ae2025/s4.log), [ae2025/probe_key_lines.log](audits/evidence/m0-transport-spike/ae2025/probe_key_lines.log)
- Audit: [M0 Architecture Contract](audits/00-architecture-contract.md)

### TR-M0-005 — Undo/redo and project-dirty behavior

- Status: PASS
- Baseline/OS/AE: as TR-M0-002 (Slider part); prototype `DynamicFx` present for the indicative second half
- Command/steps: `... -Scenarios s3` → `s3_undo_dirty.jsx`; scripted expression writes with `beginUndoGroup`/`executeCommand(16/17)`; prototype half writes a GLSL expression and idles
- Expected: characterize how scripted writes and plugin state publication interact with the undo stack and the project-dirty flag
- Observed: scripted expression write sets `project.dirty=true`; `save` clears it; undo reverts the expression (`111`→empty, `expressionEnabled=false`); redo did not restore the value in this run. Prototype: compile sets dirty, save clears it, and a subsequent idle recompile leaves `dirty=false` (idle republication does **not** re-dirty a saved project); undo removes the committed expression (`exprLen 501→0`). `app.project.dirty` is a readable boolean.
- Raw: [ae2025/s3.log](audits/evidence/m0-transport-spike/ae2025/s3.log)
- Audit: [M0 Architecture Contract](audits/00-architecture-contract.md)

### TR-M0-006 — Popup runtime menu mutation

- Status: PASS (negative result)
- Baseline/OS/AE: as TR-M0-004 (same probe)
- Command/steps: `... -Scenarios s4`; the probe attempts `set_options(5 items)` + `PF_UpdateParamUI` once per process; the script then tries `setValue(5)` on the popup declared with 4 items
- Expected: determine whether a plugin can grow a Popup menu after PARAMS_SETUP
- Observed: in-plugin `POPUP_MUTATE rewrote=true name=Ok ui=Ok` (both calls succeed), but AE keeps the original 4 items — `setValue(5)` fails "值 5 在 1 至 4 的范围外" and the name is unchanged. **A Popup's menu is fixed at PARAMS_SETUP; runtime mutation has no host-visible effect.**
- Raw: [ae2025/s4.log](audits/evidence/m0-transport-spike/ae2025/s4.log), [ae2025/probe_key_lines.log](audits/evidence/m0-transport-spike/ae2025/probe_key_lines.log)
- Audit: [M0 Architecture Contract](audits/00-architecture-contract.md)

### TR-M0-007 — aerender parity for long expressions

- Status: PASS
- Baseline/OS/AE: as TR-M0-002; aerender 25.6.6x4
- Command/steps: `... -Scenarios s5` → `s5_aerender_setup.jsx` builds a comp whose white solid's opacity is driven by a 1 MB expression evaluating to 37.5, saves a GUI frame (`saveFrameToPng`) and a Photoshop-sequence render-queue item; then `aerender -project s5.aep`; alpha compared with `check_png.py` / `check_psd.py`
- Expected: the 1 MB expression evaluates identically in the GUI render and an independent aerender process
- Observed: GUI PNG (16-bit) alpha at (160,120) = 24575/65535 = 96; aerender PSD (8-bit) alpha = 96; `aerender exit=0`; project color space ACES/ACEScg (32-bpc), so RGB reflects linear premultiplied compositing while alpha is the exact opacity value. Parity confirmed on alpha.
- Raw: [ae2025/s5.log](audits/evidence/m0-transport-spike/ae2025/s5.log), [ae2025/s5_gui.png](audits/evidence/m0-transport-spike/ae2025/s5_gui.png), [ae2025/s5_ar_00000.psd](audits/evidence/m0-transport-spike/ae2025/s5_ar_00000.psd), [ae2025/aerender_s5.txt](audits/evidence/m0-transport-spike/ae2025/aerender_s5.txt)
- Audit: [M0 Architecture Contract](audits/00-architecture-contract.md)

#### Cross-cutting host finding

Scripted parameter changes (`property.setValue`, expression writes) do **not** reach the plugin as a `UserChangedParam`/commit — the probe saw `commit=false` and stale reads throughout (`param_last_kb=None`). This matches the prototype's need for a main-thread idle observer (ARCHITECTURE §5.3) and is why the spike drove flatten size through an environment variable rather than a parameter.

## Target rewrite — M1 first frame results (AE 2025)

Shared baseline for TR-M1-001..004:

- Working tree on `codex/stabilize-programmatic-flow` (M0-exit + M1 rewrite changes, uncommitted at run time; parent commit `de401a2`); toolchain pinned `1.97.1` (`rust-toolchain.toml`).
- OS: Windows 11 Pro 10.0.26200.
- Host: After Effects 2025, `AfterFX.exe` file version 25.6.6 (same install as the M0 spike host, `app.version` 25.6.6x4), zh_CN. aerender 25.6.6.
- Artifact: `dynamicfx.dll` → `DynamicFx.aex`, 8,160,256 bytes, SHA-256 `BDDB51F1A349ED3ED96F4A18587C687A2986BE40759F4361DC6734999ECC58D4`, installed 2026-08-12 via `scripts/install.bat 2025` (elevated) to `C:\Program Files\Adobe\Adobe After Effects 2025\Support Files\Plug-ins\DynamicFx\DynamicFx.aex`; installed-file hash verified equal to the build artifact.
- GPU (ADR-0014 §3/§6): adapter "NVIDIA GeForce RTX 5080", backend Dx12, driver 32.0.15.9621 (from the plugin log's `pipeline built` line).
- Harness: `pwsh scripts/m1/run_m1.ps1 -Year 2025` (warm AE, scenarios m1a-m1e + quit), then `-Aerender`, then `-Checks`. Raw outputs in `scripts/out/m1/2025/`; curated copies in [audits/evidence/m1-first-frame/ae2025/](audits/evidence/m1-first-frame/ae2025/). An earlier same-day run of a previous artifact (`6E8F2345…`) is preserved in `scripts/out/m1/2025_run1_6e8f2345/` (superseded by the status-text fix; its pixel checks also passed).

### TR-M1-001 — Target Rust unit suite

- Status: PASS
- Command: `cargo test --all` (2026-08-12)
- Expected: every domain/host contract test green
- Observed: `38 passed; 0 failed` (envelope classifier, Language registry, ParamId grammar/pools/BindingPlan, ABI reflection, topology order, source extraction)
- Audit: [M1 First Frame](audits/01-first-frame.md)

### TR-M1-002 — Target release AEX build

- Status: PASS
- Command: `cargo build --release` (2026-08-12)
- Expected: clean release cdylib
- Observed: `Finished release`, zero warnings; artifact identity as in the shared baseline above
- Audit: [M1 First Frame](audits/01-first-frame.md)

### TR-M1-003 — Language ID/default GLSL on the host

- Status: PASS
- Command/steps: harness scenario `m1a_apply.jsx`
- Expected: one `addProperty("DynamicFx")` succeeds with an intact property tree; Language defaults to position 1 (GLSL); menu has no position 2; save/reopen keeps the popup value
- Observed: `add_ok=1`; 110 properties (109 declared + AE's built-in Compositing Options); heads in declared order (Language, Source, Compile, Status, State Token, then pools); `lang_value=1`; `setValue(2)` rejected (`popup_reject=1`, value still 1); after save+reopen `props=110`, `lang_value=1`
- Raw: [ae2025/m1a.log](audits/evidence/m1-first-frame/ae2025/m1a.log)
- Audit: [M1 First Frame](audits/01-first-frame.md)

### TR-M1-004 — Raw GLSL first frame through the graph path

- Status: PASS
- Command/steps: scenarios `m1b_write.jsx` (scripted gradient-shader expression; no UCP fires — exercises the §5.3 idle observer), `m1c_check.jsx` (GUI frame), `m1d_invalid.jsx`/`m1e_verify.jsx` (invalid source, pass-through, restore, stage aerender); numeric verification via `check_png.py`/`check_psd_rgb.py`
- Expected: the committed source compiles through classify → GLSL frontend → one-pass graph → BindingPlan → SPIR-V → DX12 and produces a non-pass-through 8-bpc frame matching `outColor = vec4(v_uv, 0, 1)`; invalid source yields a diagnostic and byte-exact pass-through
- Observed:
  - idle observer chain in the plugin log: `idle observation: compiled: 1 pass, 0 params` → `idle state token updated: 00042eec230341ec` → `definition resolved from process registry` → `pipeline built (adapter="NVIDIA GeForce RTX 5080" backend=Dx12 …)`;
  - gradient PNG numeric probes (tolerance 3): (16,16) = 13,17,0 vs expected 13,18,0; (160,120) = 128,128,0 exact; (304,224) = 243,239,0 exact — UV origin/texel-center/row-order conventions (ADR-0011 §6) confirmed;
  - invalid source: `Status: GLSL error: Error { kin` (PF names truncate at 31 chars), token republished as 0, pass-through frame exact (10,200,30);
  - aerender leg: exit 0, output PSD = pass-through (10,200,30) at every probe — the fresh aerender process has no session registry, so the render clone fails closed as designed; shader output under aerender requires M3 persistence (recorded as a known limitation, not a defect);
  - the compiled status text lands on the next UI refresh after idle observation (asynchronous; `m1c` read it before the refresh, `m1e` after — see audit finding).
- Raw: [m1b.log](audits/evidence/m1-first-frame/ae2025/m1b.log), [m1c.log](audits/evidence/m1-first-frame/ae2025/m1c.log), [m1d.log](audits/evidence/m1-first-frame/ae2025/m1d.log), [m1e.log](audits/evidence/m1-first-frame/ae2025/m1e.log), [m1c_gui.png](audits/evidence/m1-first-frame/ae2025/m1c_gui.png), [m1d_invalid.png](audits/evidence/m1-first-frame/ae2025/m1d_invalid.png), [m1_ar_00000.psd](audits/evidence/m1-first-frame/ae2025/m1_ar_00000.psd), [checks.txt](audits/evidence/m1-first-frame/ae2025/checks.txt), [dynamicfx_plugin.log](audits/evidence/m1-first-frame/ae2025/dynamicfx_plugin.log), [aerender_m1.txt](audits/evidence/m1-first-frame/ae2025/aerender_m1.txt)
- Audit: [M1 First Frame](audits/01-first-frame.md)

## Target rewrite — M2 keyframed-parameter results (AE 2025)

### TR-M2-001 — Stable ParamId and BindingPlan

- Status: PASS (unit core + first host integration; M2 continues — annotation grammar, defaults, and non-float value-encoding fixtures are not covered by this record)
- Baseline: working tree on `codex/stabilize-programmatic-flow` at commit `86b7c7a` + idle slot-UI publication (uncommitted at run time); toolchain 1.97.1
- OS/Host: Windows 11 Pro 10.0.26200; After Effects 2025 (25.6.6, zh_CN)
- Artifact: `DynamicFx.aex` 8,199,680 bytes, SHA-256 `BB9B17F010024F7CDF10CE6C5A2D32D3051FB1BCBC38A146F8476BEA65DC4F56`, installed via elevated `scripts/install.bat 2025`; installed hash verified
- Commands: `cargo test --all` (44 passed — includes six reuse/alias tests: reorder stability, rename with/without alias, kind-change reallocation, hole filling, vec4 atomic pairing, reuse overflow); `pwsh scripts/m2/run_m2.ps1 -Year 2025` then `-Checks`
- Expected: a scripted shader with `float gain` binds to Float slot 0 with the ParamId as its AEGP-applied label (unbound slots keep default names, hidden); keyframes 0.0@t0 → 0.8@t0.8s interpolate in rendered pixels; a source edit declaring a new `extra` parameter BEFORE `gain` keeps gain's slot and keyframes (IDs, not declaration order)
- Observed: slot names `gain`/`Float 02` applied by the idle observer via AEGP (no UI callback involved); `numKeys=2`; frames at t=0 / 0.4 / 0.8 = (0,0,0) / (102,102,102) / (204,204,204) — all exact; after the edit: slots `gain`/`extra`, `numKeys=2` intact, t=0.4 frame (102,102,102) exact with `extra` at default 0; plugin log chain `compiled: 1 pass, 1 params` → `idle slot ui applied: 1 bound` → token, then `2 params` → `2 bound` → new token after the edit
- Raw: [ae2025/](audits/evidence/m2-keyframed/ae2025/) — m2a-d logs, four PNGs, [checks.txt](audits/evidence/m2-keyframed/ae2025/checks.txt), [dynamicfx_plugin.log](audits/evidence/m2-keyframed/ae2025/dynamicfx_plugin.log)
- Audit: [M2 Keyframed Parameters](audits/02-keyframed-params.md)

### TR-M2-002 — @param annotations, defaults, live alias rename

- Status: PASS
- Baseline: working tree at `a06a437` + annotation slice (uncommitted at run time); toolchain 1.97.1; OS/host as TR-M2-001
- Artifact: `DynamicFx.aex` 8,238,592 bytes, SHA-256 `8D0E5E043528A65A9A56D3BC6032CF4D05AEBA1EAFAB6F299072A0071F4117D1`, installed via elevated `scripts/install.bat 2025`; installed hash verified
- Commands: `cargo test --all` (50 passed — annotation parser tests fix the grammar per ADR-0013; merge/inconsistency tests in the GLSL frontend); `pwsh scripts/m2/run_m2.ps1 -Year 2025 -Scenarios e,f,g,q` then `-Checks`
- Expected: `// @param level label:"Master Level" min:0 max:2 default:0.5` labels the slot, writes the default into the fresh binding (idle/AEGP), and the frame renders at the default with no stream ever touched; renaming to `volume` with `alias:level default:0.9` keeps the slot, keeps both keyframes, applies the new label, and does NOT apply the new default to the inherited binding
- Observed: slot named `Master Level`, stream value 0.5 (script-read), default frame gray 127/127/127 (0.5·255 = 127.5) — the defaults-before-committed-streams criterion holds; after rename: slot named `Volume`, `numKeys=2`, t=0.4 frame gray 127 exact (keyframes win); plugin log `1 bound, 1 defaults written` on first bind and `1 bound, 0 defaults written` on the inherited re-bind
- Raw: [ae2025-annotation/](audits/evidence/m2-keyframed/ae2025-annotation/) — m2e-g logs, two PNGs, checks.txt, plugin log
- Audit: [M2 Keyframed Parameters](audits/02-keyframed-params.md)

### TR-M2-003 — Value-encoding fixtures and pool overflow

- Status: PASS
- Baseline: working tree at `6d5c90e` + kind-fixture slice (uncommitted at run time); toolchain 1.97.1; OS/host as TR-M2-001
- Artifact: `DynamicFx.aex` 8,238,592 bytes, SHA-256 `3D11B511682ABD850A183C77B2261B021E6FD35CCBE89F640FBD862D8B2FAF8F`, installed via elevated `scripts/install.bat 2025`; installed hash verified
- Commands: `cargo test --all` (51 passed — the multi-kind fixture shader is pinned in the unit suite, which is what surfaced naga's rejection of std140 `bool` before any host run); `pwsh scripts/m2/run_m2.ps1 -Year 2025 -Scenarios h,h2,i,j,q` then `-Checks`
- Expected: one shader using every v1 kind renders five vertical bands whose pixel values pin the value encodings (ADR-0013 §8); a 49-float shader atomically rejects with a diagnostic, a zeroed token, and byte-exact pass-through
- Observed, all probes exact/within tolerance:
  - int passes the integer value (count default 3 → 3/10 band = 77);
  - bool is an `int` member + `hint:bool` (std140 has no host-shareable bool — naga `NonHostShareable`; this pins ADR-0011's "bool as i32" surface form) mapping to a Checkbox, 0/1 encoding (default 1 → 255);
  - color passes AE's RGB as 0..1 straight (script-set 1/0.5/0.25 → 255,128,64);
  - point passes pixels normalized by the render extent (240,60 in 320×240 → 0.75,0.25 → 191,64,0);
  - angle passes degrees (default 90 → 90/360 band = 64);
  - annotation labels/defaults across pools: `Count`(3)/`flag`(1)/`sweep`(90) applied by the idle observer; color/point annotation defaults are v1-rejected by design (scalar-only, fail closed);
  - overflow: `Status: definition rejected: Po…`, token 0, frame = solid (10,200,30) exact — the atomic pool-overflow criterion has host evidence
- Raw: [ae2025-kinds/](audits/evidence/m2-keyframed/ae2025-kinds/) — m2h/h2/i/j logs, `m2h_kinds.png`, `m2i_overflow.png`, checks.txt, plugin log
- Audit: [M2 Keyframed Parameters](audits/02-keyframed-params.md)

## Target rewrite — M3 persistence results (AE 2025)

### TR-M3-001 — Persistence and render clone

- Status: PASS
- Baseline: working tree at `80ddb99` + M3 implementation (uncommitted at run time); toolchain 1.97.1; blake3 1.8.6; OS/host as TR-M2-001
- Artifact: `DynamicFx.aex` 8,325,632 bytes, SHA-256 `82CEA1AA80C530A880F136DBC58839C7F583AEEB11B400BD7FF302C7475BD311`, installed via elevated `scripts/install.bat 2025`; installed hash verified. (Earlier same-day run of `C91C1A77…` archived — it exposed the slot-UI-restore defect fixed in this artifact.)
- Commands: `cargo test --all` (68 passed — token encode/decode incl. corrupt cases, snapshot golden/every-byte-flip/schema-unknown/truncation/budget, fingerprint golden vector, diagnostic registry guard); `pwsh scripts/m3/run_m3.ps1 -Year 2025 -Session4` (four AE sessions), then `-Aerender`, then `-Checks` (nine probes)
- Expected and observed (all pixel probes exact):
  - save → **fresh AE process** → open → render with **no Compile click**: (51,51,0) at t=0.4 via the snapshot path (`definition rebuilt from snapshot` in the plugin log); keyframes aligned (`numKeys=2`); slot label `gain` restored by the idle spot-check (defect found in run 1: stream renames do not persist and the token-change gate skipped republication — fixed by a one-read staleness probe);
  - **aerender renders the shader**: output PSD (51,51,0) exact — closes TR-M1-004's pass-through limitation; the render-clone path performs no AEGP calls (snapshot resolution is pure computation);
  - **corruption**: one byte flipped inside the raw `DFXS` payload of a copied project → snapshot discarded (`SnapshotCorrupt`), expression path recovers automatically, frame exact again;
  - **duplicate isolation**: layer duplicate holds independent state — original keeps 2 keyframes → (51,51,0), duplicate static 0.9 → (115,115,0);
  - **torn token**: scripted overwrite with a well-formed-but-wrong Active word (5) → registry miss → `token/snapshot fingerprint mismatch; snapshot wins` → frame exact; idle corrected the stream back to the true word;
  - **token diagnostic state**: during an invalid edit the stream carried word 70 = `Invalid(E17 GlslParse)` — render clones see the real code;
  - **undo**: measured host fact — the plugin's AEGP token write occupies exactly one undo entry (`AEGP_SetStreamValue` is always undoable), so converging past an invalid edit took two Undo presses; after them rendering converged to (51,51,0) with no Compile click;
  - **dirty**: `app.project.dirty == false` immediately after save AND after a further idle window — republication of unchanged state never re-dirties (extends TR-M0-005 to target code).
- Raw: [ae2025/](audits/evidence/m3-persistence/ae2025/) — twelve scenario logs, seven PNGs, the aerender PSD, three plugin logs, checks.txt
- Audit: [M3 Persistence and Render Clone](audits/03-persistence-render-clone.md)

## Target rewrite — M4 multi-pass results (AE 2025)

### TR-M4-001 — Multi-pass graph

- Status: PASS
- Baseline: working tree at `4e15e99` + M4 implementation (uncommitted at run time); toolchain 1.97.1; OS/host as TR-M2-001
- Artifact: `DynamicFx.aex` 8,411,648 bytes, SHA-256 `F0AFAE743F9981593FA8EAA97F2DDBC1CB787648F6FAB85E0CF6AD9611AFEBCB`, installed via elevated `scripts/install.bat 2025`; installed hash verified. Two superseded same-day runs archived: `76ECE8A4…` (exposed the idle-sync defect below), `358E0B48…` (pre-evidence-logging).
- Commands: `cargo test --all` (82 passed — grammar goldens + every ADR-0018 §3 rule line-numbered, `@@` escape round-trip, limit boundaries, plan goldens per ADR-0020 incl. chain=2/diamond/determinism, multi-input budget, pinned sampling-pass fixture); `pwsh scripts/m4/run_m4.ps1 -Year 2025` (two AE sessions: alias on, then `DYNAMICFX_NO_ALIAS=1`), then `-Checks` (eight probes)
- Expected and observed (all pixel probes exact):
  - two-pass envelope (gradient×gain → invert): center (191,191,255), (32,120)=(242,191,255) — multi-pass output verified and distinct from the one-pass module;
  - three-pass double-invert chain equals the plain generator at (64,64,0) — and its plan uses **2 physical intermediates** (logged: `plan 3 step(s), 2 physical intermediate(s), ~600 KiB transient`), the ADR-0020 chain golden live;
  - raw module and the same module as a one-pass envelope probe identically at (64,64,0) — the ADR-0018 raw/envelope identity obligation;
  - cyclic graph fails closed: `Status: E6 envelope line 3: cyc…`, token word 26 = Invalid(E6), byte-exact pass-through;
  - `DYNAMICFX_NO_ALIAS=1` session probes identically to the aliased run at both points — the ADR-0020 §5 A/B obligation;
  - per-pass pipelines really build (`pipelines built: 2/3 pass(es)` with adapter identity), and plan shape + peak transient memory are logged per ADR-0020 §6.
- Finding (fixed in `358E0B48…`): the idle token sync still carried the M1-era rule "every envelope → Invalid(E3)", so successful multi-pass compiles published a lying Invalid token and clones passed through; caught immediately because pass-through values differ from the expected probes. Run 1 archived as evidence.
- Raw: [ae2025/](audits/evidence/m4-multipass/ae2025/) — eight scenario logs, six PNGs, checks.txt, plugin log
- Audit: [M4 Multi-pass Graph](audits/04-multipass-graph.md)

## Target rewrite — M5 pixel-format results (AE 2025)

### TR-M5-001 — 8/16/32-bpc pixel fixtures

- Status: `PASS` (all 22 gated probes; 2 record-only entries)
- Date: 2026-08-13
- Baseline: working tree at the M5 exit commit (`feat: M5 pixel formats` + this docs commit); Rust 1.97.1 (pinned)
- Artifact: `target/release/dynamicfx.dll` 8,432,128 bytes, SHA-256 `D9E916378D44EC33DF7E242C3D92CBE22568467F617E1A4B3ED0FD80E993E4A7`, installed as `C:\Program Files\Adobe\Adobe After Effects 2025\Support Files\Plug-ins\DynamicFx\DynamicFx.aex` (installed-file hash verified identical before the run)
- Host: After Effects 2025 v25.6.6 zh_CN, Windows 11 Pro 10.0.26200, NVIDIA GeForce RTX 5080 (Dx12, driver 32.0.15.9621)
- Host precondition: new-project color engine flipped from the OCIO default to classic (AE prefs `colorManagementSystem 1→0`, `pcms 01→00`; backup kept beside the prefs file). Under OCIO, scripting `bitsPerChannel = 16` is refused ("不支持 OCIO 颜色管理模式") and classic `workingSpace` control is inert — both measured, see [m5x.log](audits/evidence/m5-pixel-formats/ae2025/m5x.log)
- Commands: `pwsh scripts/m5/run_m5.ps1 -Year 2025` (one warm session; `m5x_cmprobe.jsx` host probe + `m5all.jsx` scheduleTask state machine: setup → arm sampleImage probes post-compile → per-depth reads at 16/32/8 → alpha probes per depth → managed/unmanaged color pair), then `-Checks` (`check_probes.py` over `m5all.log`)
- Expected/observed (all in sampleImage's normalized units; full print in [checks.txt](audits/evidence/m5-pixel-formats/ae2025/checks.txt)):
  - 16-bpc three-pass chain: p100 = 10291/32768 = 0.31405640 exact; p9 = 973/32768 exact; white = 1.0 exact; staircase pair (9,10) distinct (973 ≠ 1075) through two Rgba32Float physical intermediates. Tolerance 2e-4 rejects the 8-bit value 80/255 (off by 3.4e-4);
  - 16-bpc single-pass (HDR generator) + ADR-0022 boundary clamp: 2.0 → 1.0, −0.5 → 0.0, 1.0 → 1.0, ramp exact;
  - 32-bpc single-pass and three-pass chain: over-white 2.0 and negative −0.5 survive bit-exact end to end (chain inverts map 2.0 → −1.0 → 2.0); ramp = 0.31406248 (fp32);
  - 8-bpc canary on the same shader: exact 8-bit values; the (9,10) pair collapses as predicted (validates the 16-bpc staircase discriminates);
  - Alpha semantics (ADR-0021 §4 measured contract): AE delivers **straight** alpha at 8/16/32-bpc (probe shader sees R = 1.0 against a 50%-alpha red precomp whose pre-effect buffer reads r = 1.0, a = 0.5);
  - Color pair (ADR-0021 §5): unmanaged 32-bpc carries 2.0 exactly; sRGB-managed (classic engine) also reads 2.0 — no transform applied to float renders on this host (recorded).
- Visible artifacts: [m5_chain16_00000.psd](audits/evidence/m5-pixel-formats/ae2025/m5_chain16_00000.psd) (16-bpc chain render; file is 8-bit because the output-module `Depth` key is read-only from scripting — measured, artifact-only), [m5_hdr32_preview.png](audits/evidence/m5-pixel-formats/ae2025/m5_hdr32_preview.png) (the 32-bpc PSD export fails with a modal "unexpected export error \1" on this host — measured, recorded)
- Raw evidence: [m5all.log](audits/evidence/m5-pixel-formats/ae2025/m5all.log), [m5x.log](audits/evidence/m5-pixel-formats/ae2025/m5x.log), [checks.txt](audits/evidence/m5-pixel-formats/ae2025/checks.txt), [dynamicfx_plugin.log](audits/evidence/m5-pixel-formats/ae2025/dynamicfx_plugin.log) (per-render `render enter` lines show depth, ROI window, and token state; superseded failing runs are archived timestamped under `scripts/out/m5/2025/`)
- Findings during verification (fixed same day, details in the [M5 audit](audits/05-pixel-formats.md)): wgpu refuses `Rgba16Unorm` as a render target (→ [ADR-0022](adr/0022-16bpc-working-format-f32.md)); AE requires SmartFX result rects ⊆ request while `max_result_rect` must both contain the result and advertise the full frame or AE caches "empty" for unrequested regions; ROI requests can start outside the layer (negative origins); sliders evaluated before the idle compile cache their results effectively permanently in a scripted session
- Related audit: [05-pixel-formats.md](audits/05-pixel-formats.md)

## Target rewrite — M6 temporal results (AE 2025)

### TR-M6-001 — Temporal windowed re-simulation fixtures

- Status: `PASS` (all 52 gated probes: 22 shuffled interactive reads, 25 MFR render-queue frames, 25 aerender frames — reported as 30 CHECK lines with sequence summaries)
- Date: 2026-08-13
- Baseline: working tree at the M6 exit commits; Rust 1.97.1 (pinned)
- Artifact: `target/release/dynamicfx.dll` 8,438,272 bytes, SHA-256 `C7023854D43FB5D3120BB6257C54F991BC130321784351695B2665D784330711`, installed to `…\Adobe After Effects 2025\Support Files\Plug-ins\DynamicFx\DynamicFx.aex` (installed-file hash verified)
- Host: After Effects 2025 v25.6.6 zh_CN, Windows 11 Pro 10.0.26200, RTX 5080 / Dx12 / 32.0.15.9621; MFR at host defaults
- Commands: `pwsh scripts/m6/run_m6.ps1 -Year 2025` (scheduleTask fixture `m6all.jsx`), then `-QuitAE`, `-Aerender`, `-Checks` (`check_m6.py`)
- The law under test (ADR-0025): value(F) = `min(F+1, W) × step`, exact at any depth, in any evaluation order, identically across interactive probes, the MFR-concurrent render queue, and a fresh aerender process.
- Expected/observed ([checks.txt](audits/evidence/m6-temporal/ae2025/checks.txt)):
  - shuffled interactive reads (8-bpc W=16 step 4/255; 32-bpc W=16 step 1/64): ramp, plateau, backwards, and repeated frames all exact — including frame 30 after frame 12 and re-reads;
  - `@window 8` recompile: plateau follows W (frame 20 → 8/64, frame 3 → 4/64) — W is part of the definition;
  - render-queue PSD sequence frames 0..24: **25/25 exact** under measured out-of-order MFR dispatch;
  - aerender PSD sequence frames 0..24: **25/25 exact** in a fresh process resolving from the snapshot;
  - Effect Controls shows no MFR warning icon ([screenshot](audits/evidence/m6-temporal/ae2025/m6_no_mfr_warning_effect_controls.png)); 96 windowed renders logged.
- Raw evidence: [m6all.log](audits/evidence/m6-temporal/ae2025/m6all.log), [checks.txt](audits/evidence/m6-temporal/ae2025/checks.txt), [dynamicfx_plugin.log](audits/evidence/m6-temporal/ae2025/dynamicfx_plugin.log), sample PSDs (rq/ar frames 0/15/24); superseded failing runs archived under `scripts/out/m6/2025/`
- Findings during verification (details in the [M6 audit](audits/06-temporal-feedback.md)): the ADR-0023 session-chain model measured dead on the render path (out-of-order MFR dispatch; clone fragmentation in the no-flag A/B; render-context frame numbering) → ADR-0025; **snapshot stored only the first pass body** (M4-latent: envelope sources reopened as raw single-pass effects) → fixed to the exact committed text, caught by the aerender leg
- Related audit: [06-temporal-feedback.md](audits/06-temporal-feedback.md)

## Target rewrite — M7 performance results (AE 2025)

### TR-M7-001 — Performance baseline (benchmark matrix)

- Status: `PASS` (baseline captured: 8 scenes × 25 RQ frames, 275 perf lines, all scenes `DONE`, zero uncompiled renders in the measured run)
- Date: 2026-08-13
- Baseline: working tree at M7 Phase-1 instrumentation (perf spans + `scripts/m7/` harness), on top of M6 exit commit `f96a330`; Rust 1.97.1 (pinned)
- Artifact: `target/release/dynamicfx.dll` 8,433,664 bytes, SHA-256 `D9C8B9FFBFF2B786E7B326C9DEB4517F720E89E919E6C925F0A8DBA93DC80855`, installed to `…\Adobe After Effects 2025\Support Files\Plug-ins\DynamicFx\DynamicFx.aex`
- Host: After Effects 2025 v25.6.6 zh_CN, Windows 11 Pro 10.0.26200, RTX 5080 / Dx12 / 32.0.15.9621; MFR at host defaults; render queue → Photoshop PSD sequences
- Commands: `pwsh scripts/m7/run_m7.ps1 -Year 2025` (cold AE with `DYNAMICFX_PERF=1`; scheduleTask fixture `m7bench.jsx` with StateToken readiness polling), then `-QuitAE`, then `-Summarize` (`summarize_perf.py`)
- Spans per render (ms): AE→working conversion (`conv_in`), GPU prep+upload (`upload`), submit+wait (`gpu`), buffer map+copy (`readback`), working→AE write-back (`conv_out`), and `total`; temporal scenes accumulate upload/gpu/readback across all window iterations of the frame.
- Median/p95 baseline ([summary.md](audits/evidence/m7-perf/ae2025/summary.md), CSV alongside):
  - grad720 8-bpc: total 3.61/4.45; grad4k 8-bpc: 26.74/29.94 (conv+upload+readback ≈ 86% of total; GPU 1.49 p50)
  - thermal720 (6-pass): 3.87/4.62; thermal4k: 33.77/39.67 (GPU only 3.53 p50 — data movement dominates at 4K)
  - temporal `@window 16`: 17.57/24.82 (upload 11.48 p50 — per-iteration re-upload is the windowed-cost hotspot)
  - multi (4 instances): 6.60/8.66 per render, 100 renders
  - grad720 32-bpc: 11.37/13.12; grad4k 32-bpc: 136.95/174.92 (upload 47.90 p50 for ~132 MB/frame ≈ 2.7 GB/s effective — far below PCIe capability)
  - RQ wall-clock per frame (host overhead included): 47.9 ms at 720p/8 (20.9 fps) vs 3.61 ms in-plugin — AE-side pipeline dominates small frames.
- Raw evidence: [m7bench.log](audits/evidence/m7-perf/ae2025/m7bench.log), [dynamicfx_plugin.log](audits/evidence/m7-perf/ae2025/dynamicfx_plugin.log), [summary.md](audits/evidence/m7-perf/ae2025/summary.md), [summary.csv](audits/evidence/m7-perf/ae2025/summary.csv)
- Failed runs preserved: run 1 (20 s blind wait → 50 `token=0 compiled=false` passthrough renders, first two scenes black: [m7bench_run1_20s.log](audits/evidence/m7-perf/ae2025/m7bench_run1_20s.log), [dynamicfx_plugin_run1_20s.log](audits/evidence/m7-perf/ae2025/dynamicfx_plugin_run1_20s.log)); run 3 hit the same failure when a host modal ("save before closing") blocked the idle window. The fixture now polls each instance's StateToken stream (property 5, non-zero ⇔ render-resolvable) instead of blind-waiting; the measured run reported `READY 9/9`.
- Related audit: [07-performance-mfr.md](audits/07-performance-mfr.md)

### TR-M7-002 — Optimization 1: GPU resource reuse

- Status: `PASS` (before/after benchmark pair on the same matrix, plus the complete M1-M6 host regression battery green on the changed artifact — 88 numeric probes, `fails=0` in every suite)
- Date: 2026-08-13
- Baseline: TR-M7-001 commit `9036003` (before) vs the item-1 working tree (after); Rust 1.97.1 (pinned)
- Artifact (after): `target/release/dynamicfx.dll` 8,440,320 bytes, SHA-256 `E675B8202EB7B9908A72289178CD6F1AEA328E893CF5CFB1223ECD0AF51C7959`, installed-file hash verified at `…\Adobe After Effects 2025\Support Files\Plug-ins\DynamicFx\DynamicFx.aex`
- Host: identical to TR-M7-001 (AE 2025 v25.6.6 zh_CN, Windows 11 Pro 10.0.26200, RTX 5080 / Dx12 / 32.0.15.9621; MFR defaults)
- Change under test: per-instance `FrameCache` (input/intermediate/output textures, readback buffer, sampler, temporal ping/pong reused across renders, keyed by token/depth/size/plan shape; guarded by the instance lock held for the whole render); the temporal window loop moved inside one execute — single input upload per frame, iteration 0 samples a never-written zero texture (black start without clears); conversion scratch buffers reused; 256-aligned rows upload without repacking.
- Commands: `battery.ps1` (session orchestration): `scripts/m7/run_m7.ps1 -Year 2025` (+`-QuitAE`, `-Summarize`), then the full `scripts/m1..m6` drivers with their aerender/Session4/checks stages.
- Total p50/p95 ms per render, before → after ([summary.md](audits/evidence/m7-perf/ae2025/summary.md) → [summary_item1_after.md](audits/evidence/m7-perf/ae2025/summary_item1_after.md)):
  - grad720/8: 3.61/4.45 → 2.08/2.95 (−42% p50); grad4k/8: 26.74/29.94 → 15.56/19.75 (−42%)
  - thermal720: 3.87/4.62 → 2.48/3.17 (−36%); thermal4k: 33.77/39.67 → 15.84/19.91 (−53%)
  - temporal `@window 16`: 17.57/24.82 → 3.28/4.67 (−81% — windowed cost now ~1.6× a plain render)
  - multi ×4: 6.60/8.66 → 4.22/6.98 (−36%)
  - grad720/32: 11.37/13.12 → 6.13/8.77 (−46%); grad4k/32: 136.95/174.92 → 72.23/84.81 (−47%)
- Correctness regression: M1 (pixel + aerender dual-branch), M2 (keyframes), M3 (persistence + Session4 + aerender), M4 (multi-pass), M5 (22-probe depth matrix: 16-bpc chain exact, 32-bpc over/neg preserved, straight alpha ×3), M6 (temporal law exact, RQ 25/25, aerender 25/25, 96 windowed renders) — all `fails=0` on the new artifact: [regression_battery.log](audits/evidence/m7-perf/ae2025/regression_battery.log) (+ per-suite outputs under `scripts/out/`)
- Raw after-run evidence: [summary_item1_after.md](audits/evidence/m7-perf/ae2025/summary_item1_after.md), [summary_item1_after.csv](audits/evidence/m7-perf/ae2025/summary_item1_after.csv), [dynamicfx_plugin_item1_after.log](audits/evidence/m7-perf/ae2025/dynamicfx_plugin_item1_after.log)
- Related audit: [07-performance-mfr.md](audits/07-performance-mfr.md)

### TR-M7-003 — WYSIWYG preview invalidation after idle compile

- Status: `PASS` (verified by measurement on the shipping path — no code change required; the standing user constraint "preview must be WYSIWYG" holds in the adversarial construction)
- Date: 2026-08-13
- Baseline/artifact: the TR-M7-002 + log-policy build (commit `605caf6` working tree), installed AEX; host identical to TR-M7-001
- Adversarial construction (all steps via the local panel `/exec` warm-session channel, no process launches): commit a BLUE-emitting source and, inside the same synchronous script (idle observer provably not yet run), force a frame render — it enters AE's frame cache as the uncompiled GRAY passthrough ([wys2_precompile.png](audits/evidence/m7-perf/ae2025/wysiwyg/wys2_precompile.png)); the layer is deselected (Effect Controls empty, so no `UpdateParamsUi` path exists) and no property is touched afterwards.
- Observed: after the idle compile published the token (log: `idle state token updated: Active(…)` → `definition resolved from process registry` → `pipelines built` — [excerpt](audits/evidence/m7-perf/ae2025/wysiwyg/invalidation_log_excerpt.txt)), the comp viewer refreshed to BLUE on its own (observed on-screen via session screenshots), and the renderer truth is BLUE ([wys2_postcompile.png](audits/evidence/m7-perf/ae2025/wysiwyg/wys2_postcompile.png)).
- Mechanism: the idle bridge's AEGP mirror of the StateToken into the parameter stream is itself a dependency-graph change — AE invalidates frames rendered against the uncompiled instance. Stale preview is possible only if the mirror write is missing, which is the already-diagnosed publication-latency scenario (audit 07 findings), not a separate cache defect.
- The M5-era note "frames rendered pre-compile stay cached until an ordinary dirty" does not reproduce on this build and is superseded by this record for the current artifact.
- Fixture: [wys2_setup.jsx](audits/evidence/m7-perf/ae2025/wysiwyg/wys2_setup.jsx)
- Related audit: [07-performance-mfr.md](audits/07-performance-mfr.md)

### TR-M7-004 — ROI final-pass delivery

- Status: `PASS` (WYSIWYG-safe by construction; full M1-M6 regression green; benchmark matrix flat; small-request speedup demonstrated)
- Date: 2026-08-13
- Baseline/artifact: ROI build `3014AED8…` (short hash; `target/release/dynamicfx.dll`), installed over the TR-M7-002/log-policy line; host identical to TR-M7-001
- Change under test: only the delivered final image narrows to AE's requested window — the FINAL pass (last window iteration) gets a scissor rect and only that rect is read back / converted. uv mapping, the full-frame input upload, every intermediate pass, and intermediate temporal iterations are untouched, so covered pixels are bit-identical to a full render (fragments are pure per-pixel functions of unchanged inputs). `DYNAMICFX_NO_ROI=1` escape hatch forces full delivery for A/B.
- Equivalence gates: full M1-M6 battery green on this artifact ([regression_battery_roi.log](audits/evidence/m7-perf/ae2025/regression_battery_roi.log)) — including M5's 22-probe depth matrix whose `sampleImage` probes are themselves ~11×11 ROI requests, and M6's temporal law (RQ 25/25, aerender 25/25). Benchmark matrix flat vs the log-policy build ([summary_roi_after.md](audits/evidence/m7-perf/ae2025/summary_roi_after.md)) — render-queue requests are padded ⊇ full frame, so `rect=` equals the full frame there by design.
- Demonstrated win ([roi_demo_perf_lines.txt](audits/evidence/m7-perf/ae2025/roi_demo_perf_lines.txt)): 4K 1-pass comp, downstream `sampleImage` probes — AE requests arrive as 11×11; delivered `rect=11x11` renders total 4.0-4.6 ms vs 15.4 ms steady-state full-frame (~3.5×; the remaining cost is the full-frame input conversion+upload that correctness mandates while shaders may sample anywhere). Probe values track coordinates correctly through the scissored path.
- Related audit: [07-performance-mfr.md](audits/07-performance-mfr.md)


### TR-M7-005 — MFR eligibility review

- Status: `PASS` (concurrency measured; decision recorded: threaded-rendering stance unchanged, intra-instance parallelism deliberately not widened)
- Date: 2026-08-13
- Baseline/artifact: ROI build + per-render `t0=` epoch-ms instrumentation; host identical to TR-M7-001; benchmark matrix re-run with interval-overlap analysis ([mfr_concurrency.md](audits/evidence/m7-perf/ae2025/mfr_concurrency.md), raw log [dynamicfx_plugin_t0.log](audits/evidence/m7-perf/ae2025/dynamicfx_plugin_t0.log))
- Method: every render logs `[t0, t0+total]`; per scene compute work (Σ durations), busy (interval union), average concurrency (work/busy), peak overlap depth, and plugin share of RQ wall.
- Measured: single-instance scenes average concurrency 1.00-1.06 (peak 2 — the two sequence clones occasionally overlap; AE effectively hands one frame at a time per instance). The 4-instance scene averages 2.70 with peak depth 5 — cross-instance parallelism flows freely through per-instance state. Plugin busy time is 5-46% of RQ wall (46% worst case at 4K/32-bpc); the wall is host-bound everywhere.
- Decision (no code change): keep `SUPPORTS_THREADED_RENDERING` for all graph classes — thread safety is proven by the M6 temporal suite and the M1-M6 batteries on every M7 artifact (per-clone instance locks; ADR-0025 renders are self-contained). Do NOT widen intra-instance parallelism: AE does not dispatch enough per-instance work to use it (measured 1.0×), cross-instance parallelism already works (2.7×), and per-lane GPU resources would multiply VRAM for no user-visible gain.
- Related audit: [07-performance-mfr.md](audits/07-performance-mfr.md)

### TR-M7-006 — M7 exit verification

- Status: `PASS` (all five ROADMAP exit criteria verified on one artifact)
- Date: 2026-08-14
- Artifact: `target/release/dynamicfx.dll` SHA-256 `4AD318E6A0BFD35BE5B1ADCCDC8EDBB68C1A90551C870C5082702192EACD8C0C`, installed to `…\Adobe After Effects 2025\Support Files\Plug-ins\DynamicFx\DynamicFx.aex`; Rust 1.97.1 (pinned); host identical to TR-M7-001
- Exit criteria mapping:
  - benchmark matrix covers 1080p/4K, one/many instances, one/many passes: 10 scenes — gradient 720p/1080p/4K at 8+32-bpc, thermal 6-pass 720p/4K, temporal `@window 16`, 4-instance — 25 RQ frames each, 325 perf lines all assigned ([summary_m7_exit.md](audits/evidence/m7-perf/ae2025/summary_m7_exit.md))
  - memory and cache budgets are enforced: per-instance frame-cache budget (`DYNAMICFX_CACHE_CAP_MB`, default 2048) with transient-resource fallback and diagnostic; byte math and thresholds unit-tested (`render::budget_tests`, 94-test suite green); transient aliasing (ADR-0020) continues to bound per-render intermediates
  - ROI produces equivalent pixels in covered regions: TR-M7-004
  - MFR enabled only for graph classes proven thread-safe: TR-M7-005 (+ M6 temporal suite)
  - performance claims include baseline, hardware, host, commit, raw report: TR-M7-001/002/004 record chains
- Identical-image verification on this artifact: full M1-M6 battery green ([regression_battery_exit.log](audits/evidence/m7-perf/ae2025/regression_battery_exit.log)) — M1 pixel/aerender, M2 keyframes, M3 persistence+Session4+aerender, M4 multi-pass, M5 22-probe depth matrix, M6 temporal law (RQ 25/25, aerender 25/25)
- Related audit: [07-performance-mfr.md](audits/07-performance-mfr.md)

## Target rewrite — Windows AE host matrix

All cells are independent. A PASS in one AE year must not be copied to another. AE 2026 cells reference the year-specific full-suite run (TR-Y26-001) executed with the M7 exit artifact.

| Scenario | AE 2023 | AE 2024 | AE 2025 | AE 2026 | aerender evidence |
|---|---|---|---|---|---|
| Plugin load and effect discovery | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-m1-003--language-iddefault-glsl-on-the-host) | [`PASS`](#tr-y26-001--ae-2026-full-suite-host-run) | `NOT_RUN` |
| Single `addProperty("DynamicFx")` | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-m1-003--language-iddefault-glsl-on-the-host) | [`PASS`](#tr-y26-001--ae-2026-full-suite-host-run) | N/A |
| Language defaults to GLSL | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-m1-003--language-iddefault-glsl-on-the-host) | [`PASS`](#tr-y26-001--ae-2026-full-suite-host-run) | N/A |
| Expression-only GLSL first frame | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-m1-004--raw-glsl-first-frame-through-the-graph-path) | [`PASS`](#tr-y26-001--ae-2026-full-suite-host-run) | [`PASS`](#tr-m3-001--persistence-and-render-clone) (shader via snapshot) |
| Invalid source pass-through/diagnostic | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-m1-004--raw-glsl-first-frame-through-the-graph-path) | [`PASS`](#tr-y26-001--ae-2026-full-suite-host-run) | `NOT_RUN` |
| Keyframed parameters at defined times | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-m2-001--stable-paramid-and-bindingplan) | [`PASS`](#tr-y26-001--ae-2026-full-suite-host-run) | `NOT_RUN` |
| Save/reopen without Compile click | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-m3-001--persistence-and-render-clone) | [`PASS`](#tr-y26-001--ae-2026-full-suite-host-run) | [`PASS`](#tr-m3-001--persistence-and-render-clone) |
| UI/render clone registry hit/miss | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-m3-001--persistence-and-render-clone) | [`PASS`](#tr-y26-001--ae-2026-full-suite-host-run) | `NOT_RUN` |
| Two-pass blur graph | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-m4-001--multi-pass-graph) (invert chain) | [`PASS`](#tr-y26-001--ae-2026-full-suite-host-run) | `NOT_RUN` |
| 16-bpc precision | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-m5-001--81632-bpc-pixel-fixtures) | [`PASS`](#tr-y26-001--ae-2026-full-suite-host-run) | `NOT_RUN` |
| 32-bpc negative/over-white | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-m5-001--81632-bpc-pixel-fixtures) | [`PASS`](#tr-y26-001--ae-2026-full-suite-host-run) | `NOT_RUN` |
| Temporal feedback continuity/reset | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-m6-001--temporal-windowed-re-simulation-fixtures) (windowed law) | [`PASS`](#tr-y26-001--ae-2026-full-suite-host-run) | [`PASS`](#tr-m6-001--temporal-windowed-re-simulation-fixtures) |
| SmartRender/ROI | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-m7-004--roi-final-pass-delivery) | [`PASS`](#tr-y26-001--ae-2026-full-suite-host-run) | `NOT_RUN` |
| Stateless MFR | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-m7-005--mfr-eligibility-review) | [`PASS`](#tr-y26-001--ae-2026-full-suite-host-run) | `NOT_RUN` |
| Pool float>1/negative, int>10 unclamped (ADR-0037) | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-0037-001--pool-valid-range-float1-negative-int10) | [`PASS`](#tr-0037-001--pool-valid-range-float1-negative-int10) | N/A |
| Copied instance keeps its own slot mapping (same source, different BindingPlan) | `NOT_RUN` | `NOT_RUN` | [`FAIL`](#tr-bind-002--copied-instance-corrupts-slot-mapping-field-defect) (field, 0.0.4) | [`FAIL`](#tr-bind-002--copied-instance-corrupts-slot-mapping-field-defect) (field, 0.0.4) | `NOT_RUN` |
| Interrupted preview leaves no cached frame missing a layer | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-cache-001--interrupted-render-poisons-the-frame-cache-field-defect) (fix build `cfccd5d`; `FAIL` on released 0.0.4) | `NOT_RUN` |

### TR-BIND-002 — Copied instance corrupts slot mapping (field defect)

- Status: `FAIL` — observed in the field on the 0.0.4 artifact, AE 2025 (25.6.6x4), 2026-08-19. Not yet reproduced by a scripted harness leg (`NOT_RUN`); the fix is unscheduled. Public issue [#6](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/6).
- Date: 2026-08-19
- Baseline: installed 0.0.4 release artifact (per [TR-REL-004](#tr-rel-004--004-release-verification); installed hash not re-verified in this session); working tree on `main` at `b1cb0f7` (docs only touched).
- Scenario: one instance whose `BindingPlan` was **migrated** (IDs added over ~15 in-place source edits, so slot order ≠ GLSL declaration order) was copy/pasted to a layer in another comp; the pasted instance was compiled fresh (declaration-order slots). Same source → same `DefinitionHash`.
- Observed: the pasted instance rendered black-and-white noise, then coloured noise after `Compile`; the original instance flickered and then rendered with its **parameter roles permuted** — 16 of 18 float slots and both angle slots read through the fresh declaration-order table while the AE streams kept their old-slot values, four of them clamped to the new sliders' ranges (`bloom_radius` 24 read as `contrast` → 3, `thickness` 140 as `halo` → 2, `wall_heat` 1 as `halo_radius` → 4, `halo_radius` 70 as `grain` → 0.3). Status stayed `compiled: 10 passes, 22 params` — no diagnostic. Full readback, the pre-incident values, the slot-by-slot mapping and the log excerpt (`definition resolved from process registry` → `pipelines built` alternating between the two instances' sizes, each followed by `idle slot ui applied` on the original) are in [`docs/audits/evidence/field-20260819-copy-instance/`](audits/evidence/field-20260819-copy-instance/README.md).
- Reading: the render/UI path resolves the definition — including the slot table used for stream reads and slot UI — from a process-wide registry keyed by `DefinitionHash`; two instances of the same source with different plans cannot both be right, and whichever compiled last owns the entry. TR-M3-001's "duplicate isolation" probe passed because both duplicates carried identical fresh plans. This contradicts the intent of ADR-0005/ADR-0013 (per-instance stable ParamIds, per-instance plan persisted by ADR-0016) and must be fixed in the runtime, not worked around in shaders.
- Repair applied to the user's project (not a fix): remove + re-add the effect with the same expression → fresh declaration-order plan matching the registry entry; render normal again (evidence PNG in the folder).
- Harness leg to author: two instances of one source, A migrated (compile with params `p1..p3`, then insert `p0` first in the block and re-commit → A keeps `p1..p3` in F01..F03, `p0` in F04), B fresh from copy/paste or `addProperty` + same expression; render both with distinguishable per-slot values and assert each instance reads its own values; then repeat with B compiled after A and A after B.
- Cause (from the source, 2026-08-19): the session registry is keyed by the source fingerprint only (`session_token` → `registry()`), `registry_insert` **replaces** an entry for the same source, and the stored `CompiledEffect.definition.binding` is the per-instance plan from `binding::build_with_reuse`; `resolve_transported_definition` (render clones; a UI instance re-created by the copy/paste flatten/resetup round-trip) therefore adopts the other instance's slot table.
- Related: [ADR-0005](adr/0005-stable-parameter-ids.md), [ADR-0013](adr/0013-paramid-grammar-and-pools.md), [ADR-0016](adr/0016-sequence-schema-v1.md), [TR-M3-001](#tr-m3-001--persistence-and-render-clone); public issue [#6](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/6).
- Field confirmation 2026-08-21 (AE 2026, 0.0.4): the user's `prism` sample carries two instances of one source; copy/paste, re-commit, param edits and `layer.duplicate()` were exercised by script. Both instances kept their **own** values throughout and 5 purged single-frame renders at each step were bit-identical — because both plans are **fresh declaration-order** (the corruption above needs a *migrated* plan). The registry still churned (`resolved from process registry` 67→72, `pipelines built` 71→76 as the instances alternated ownership), which is the **flicker** the user reports ("复制会导致原效果和复制效果闪烁"). Same root cause; no separate fix track. Evidence: [`docs/audits/evidence/field-20260821-prism-sample/`](audits/evidence/field-20260821-prism-sample/README.md) (Finding B).

### TR-CACHE-001 — interrupted render poisons the frame cache (field defect)

- Status: **`FAIL` on the released 0.0.4 artifact; `PASS` on the fix build (`cfccd5d`), host-verified on AE 2026 the same day.** Observed in the field on 0.0.4, AE 2026 (26.3x87), 2026-08-21, reproduced by a scripted harness, fixed, and re-verified on the host. Public issue [#7](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/7). The FAIL evidence on the released artifact is retained below; do not treat the fix as shipped (0.0.4 in the wild still has the bug — re-release is a separate step).
- Date: 2026-08-21
- Baseline: installed 0.0.4 release artifact `DynamicFx.aex` SHA-256 `BFE1AB9FBE20F64E9098599C57F89B9D721C9DA8735F24F480384F85E5B858C3` at `C:\Program Files\Adobe\Adobe After Effects 2026\Support Files\Plug-ins\DynamicFx\` (installed hash re-verified equal to [TR-REL-004](#tr-rel-004--004-release-verification) this session); working tree on `main` at `3d1950c` (docs only touched). OS Windows 11 Pro 10.0.26200; GPU `NVIDIA GeForce RTX 5080`, backend Dx12, driver `32.0.15.9621`.
- Host/project: AE 2026 (26.3x87) — the sample was saved by AE 2026 and does not open in AE 2025. Comp `test` 1920×1080 30fps 480f, 8-bpc; two "solid" layers each with a `DynamicFx` 1-pass/12-param **Chromatic Dispersion** flare instance over fractal-noise + blurs (L1 `Status: idle`, L2 `compiled`, one shared registry entry `token=3763537382884453`).
- Scenario: purge all caches → run an interrupted-preview loop (`mf_preview_interrupt.jsx`: 8 cycles of *play, then move the CTI* over work area frames 90–209) → sample the 120 work-area frames from cache without purging → re-read the dips, then purge and re-render them.
- Observed: 7 work-area frames rendered with luminance ≈0.008–0.021 below both neighbours from cache (f111/153/154/155/156/181/183). Layer-hidden decomposition matched each cached bad frame to **Layer 1 hidden** at mean |diff| = **0.00000** — i.e. one DynamicFx layer delivered transparent black for that frame. The poisoned frame **persists on cache re-read** and **recovers after `app.purge(ALL_CACHES)`**. Three clean Render-Queue passes rendered earlier this session (`rq_baseline` 480f, `rq_pass2`/`rq_pass3` 120f) are **bit-identical** where they overlap (`max |diff| = 0.0000`, 0 frames differ) — batch/`aerender` output never interrupts and is unaffected. Each interrupted preview added `smart render input checkout failed: InterruptCancel` pairs to the plug-in log (38→48→58 over the session).
- Reading: in `src/lib.rs` `Command::SmartRender`, an input `checkout_layer_pixels(0)` returning `InterruptCancel` is logged and flattened to `checked_out = None`, which then takes the **same branch as a genuine empty input** (ADR-0030 §5 adjustment-over-nothing): it fills the output transparent black and returns `Ok(())`. Returning `Ok` tells AE the layer rendered, so AE caches the transparent-black output; the composited frame is then missing that layer's flare until the region is re-rendered or the cache is purged. A cancel and an empty input are conflated — the CLAUDE.md "never convert a failure into pass-through without a stable diagnostic code" rule, applied to `InterruptCancel`.
- Fix direction (runtime; not applied): in the `SmartRender` arm return `Err(Error::InterruptCancel)` when the input checkout is aborted (AE discards the frame) instead of falling into `fill(0)`; keep `fill(0)` only for the true `Ok(None)` no-input case. Harness leg to author: over an interrupted work-area preview, assert no cache-served frame differs from its purged re-render (per-layer decomposition). Record `FAIL` on 0.0.4 first, then green after the fix.
- Workaround: purge the cache after interrupting a preview, or export through the Render Queue / `aerender` (batch never interrupts).
- Evidence: [`docs/audits/evidence/field-20260821-prism-sample/`](audits/evidence/field-20260821-prism-sample/README.md) (Finding A) — `cached-frame-missing-layer1.png`, `reproduction-report.txt`, `dynamicfx.log.interrupt-window.txt`, and the harness scripts.
- Related: [ADR-0030](adr/0030-layer-input-parameters.md) §5 (the empty-input branch being mis-shared), M7 SmartRender rows [TR-M7-004](#tr-m7-004--roi-final-pass-delivery)/[TR-M7-005](#tr-m7-005--mfr-eligibility-review); public issue [#7](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/7).
- **Fix — `PASS`, host-verified on AE 2026, 2026-08-21.** Commit `cfccd5d`: in `src/lib.rs` `Command::SmartRender`, an aborted input checkout now checks the bound layer parameters back in, clears the per-frame thread-locals, and returns `Err(e)` (propagating `InterruptCancel`) so AE discards the frame; `fill(0)` runs only for a true `Ok(None)`. Implemented by `gpt-5.6-sol`, reviewed and verified by Fable.
  - Baseline: working tree on `main` at `cfccd5d`. Build: `cargo build --release` (Rust 1.97.1) → `dynamicfx.dll` 8,544,768 B, SHA-256 `24E963FB19E735252A5D21CFBBF48864A597D38A7A38D461F6B3B9A34F3D22F2`, installed via `scripts/install.bat 2026` (elevated) to `C:\Program Files\Adobe\Adobe After Effects 2026\Support Files\Plug-ins\DynamicFx\DynamicFx.aex`, installed-hash re-verified equal. Note: the build is not byte-reproducible (see TR-REL-003/004), so this exact artifact identity is what was tested; a future rebuild will differ.
  - Unit: `cargo test` → **131 passed, 0 failed** (Fable's own run).
  - Host: Windows 11 Pro 10.0.26200, AE 2026 (26.3x87), GPU `NVIDIA GeForce RTX 5080` Dx12 driver `32.0.15.9621`. Procedure: open a fresh copy of the sample → `app.purge(ALL_CACHES)` → interrupted-preview loop (`mf_preview_interrupt.jsx`, 8 play/CTI-move cycles over frames 90–209) → sample the 120 work-area frames from cache → flag frames >0.006 luma below their neighbour median. **Three rounds → 0 dips each** (pre-fix baseline on 0.0.4 was 7), while `smart render input checkout failed: InterruptCancel` still appears each round (62→71→76 total) — interrupts still happen; they are now propagated, not swallowed. Evidence: [`docs/audits/evidence/field-20260821-prism-sample/`](audits/evidence/field-20260821-prism-sample/README.md) (`verify-report-fix-build.txt`, `dynamicfx.log.post-fix-verified.txt`, `verify_fix.py`, `verify_rounds.py`).
  - Harness leg still to add as a permanent regression check: assert over an interrupted work-area preview that no cache-served frame differs from its purged re-render (per-layer decomposition). Currently a manual host procedure, not an automated test.

### TR-0029-001 — logical-resolution invariance

- Status: `PASS`
- Date: 2026-08-14
- Baseline: post-0.0.2-candidate tree (ADR-0029); 96 unit tests green (`render::logical_size_tests`)
- Host: AE 2025 v25.6.6; verbose render log proves the downsampled path (`in=640x360` for a 1280×720 comp at Half preview)
- Defect (user-reported, reproduced): with physical `u_resolution`, a 16-px stripe probe rendered ~half the stripes at Half preview (before-screenshot in session); pixel-based shader math scaled with preview resolution.
- Fix: `u_resolution` = logical full-resolution size (`physical × den / num` from `downsample_x/y`); geometry stays physical.
- Observed: middle-row half-period transitions — Full 1280×720: **159**; Half 640×360 (render-queue at Resolution=Half through the same downsample path): **159**. Identical structure at both resolutions ([transition_counts.txt](audits/evidence/adr-0029/transition_counts.txt), artifacts alongside).
- Related ADR: [0029](adr/0029-logical-resolution-abi.md)

### TR-0028-001 — Details button and slider precision

- Status: `PASS`
- Date: 2026-08-14
- Baseline: post-0.0.1 working tree (ADR-0028); Rust 1.97.1; 95 unit tests green (topology contract now pins 110 entries with `Details` last; head stream indexes unchanged); PIPL subversion 5
- Host: AE 2025 v25.6.6, warm-session verification via the panel `/exec` channel + one manual click
- Changes: pool Float sliders set `Precision::Hundredths` (root cause: unset precision = zeroed field = integer stepping — measured against the StateToken slider that sets it explicitly); `Details` Button appended after all pool slots; click pops a task-modal dialog (Win32 `MessageBoxW`, host layer only) with the full status text + diagnostic code.
- Observed: broken-envelope instance published `Invalid(6)` with full text `envelope line 2: expected @graph before any pass section`; effect shows `Details` at scripting index 110; the dialog displayed the complete message and code (user-confirmed screenshot in session, text transcribed exactly).
- Regression: m1 (+aerender), m2, m3 (+Session4 +aerender) green on the new artifact — including reopening projects saved under the 109-entry topology (append-only compatibility exercised for real).
- Related ADR: [0028](adr/0028-details-button-and-slider-precision.md)

### TR-0026-001 — color default annotation

- Status: `PASS`
- Date: 2026-08-14
- Baseline: post-0.0.1 working tree (ADR-0026 implementation); Rust 1.97.1; 95 unit tests green (`annotation::tests::color_hex_defaults`, glsl merge-rule updates)
- Host: AE 2025 v25.6.6, warm-session verification via the local panel `/exec` channel
- Change: `@param … hint:color default:#RRGGBB[AA]` — hex decode to normalized components (6 digits imply alpha 1.0); hex requires `hint:color`; color default rejects `min`/`max`; vec3 colors reject a 4th component (no alpha companion slot); the M2-era "default is scalar-only" gate lifted for colors; AEGP color-stream writer added (`RawStreamSuite6::set_color`, fresh bindings only — inherited bindings keep user values).
- Observed (zero script writes, annotations only): `Tint` stream = (1, 0.19607843, 0) ≡ `#FF3200` exact; `GlowC` = (0, 0.50196081, 1) ≡ `#0080FF`; alpha companions `Tint A` = 1.0, `GlowC A` = 0.50196 (= `80` hex); rendered frame shows both colors with their alphas applied.
- Regression: m2 (params/annotations/defaults suite) and m3 (persistence + Session4 + aerender) drivers green on the new artifact; m3 aerender leg exact.
- Related ADR: [0026](adr/0026-color-parameter-default-annotation.md)

### TR-0030-001 — Layer input parameters

- Status: `NOT_RUN` — no host leg exists. The unit evidence below is real and does not verify any AE behaviour.
- Date: 2026-08-15
- Baseline: working tree on `codex/stabilize-programmatic-flow` (uncommitted)
- Implements: [ADR-0030](adr/0030-layer-input-parameters.md) in full — `PoolKind::Layer` (capacity 4), `hint:layer` annotation, layer names as read-only graph resources, comp-space checkout, `None` → transparent black, `E7` refusal of the temporal combination.
- Code paths: `src/binding.rs` (`GROWTH_POOLS`), `src/host/params.rs` (`LayerDef`, declaration order), `src/persistence.rs` (kind byte 6), `src/frontend/annotation.rs` (`hint:layer`), `src/frontend/grammar.rs` (external resources), `src/plan.rs` (`TexSlot::External`), `src/render.rs` (external texture upload), `src/lib.rs` (PreRender `checkout_layer`, SmartRender `checkout_layer_pixels`).
- **Topology risk closed by test:** `Details` occupies index 109 in every project saved by 0.0.2. Growing `V1_POOLS` would have slid it to 117 and silently repointed a released parameter stream, so the new pools are declared *after* `Details`. `host::params::tests::released_prefix_is_frozen_through_details` pins the first 110 positions.
- Commands actually run (2026-08-15): `cargo test --all` → **116 passed; 0 failed**, including five `layer_param_tests` (legal writer-less input, binds to the Layer pool, cannot be written or name a pass, `E7` on the temporal combination, rejected when the id is also an FxUniforms member); `cargo build --release` → zero warnings; `python scripts/check_governance.py` → `PASS`.
- What the host leg must still show: a displacement-style graph reading a second layer renders as expected; `None` leaves the input untouched; a referenced layer with a non-identity transform samples aligned as composited (the ADR §4 claim — **untested and the highest-risk assumption in this change**); an animated referenced layer updates per frame; duplicated instances do not share the reference; save/reopen and aerender reproduce; M1-M7 batteries stay green.

- **Host run 2026-08-16 — `PASS`, after this run found and closed two defects that made the feature non-functional.** `f003a` unassigned renders `rgb(200,40,40)` (the effect layer's own colour, untouched) and assigned renders `rgb(20,180,220)` — exactly the referenced solid. Evidence: [`f003a_none_00000.psd`](audits/evidence/f003-20260816/f003a_none_00000.psd), [`f003a_assigned_00000.psd`](audits/evidence/f003-20260816/f003a_assigned_00000.psd), [`f003a.log`](audits/evidence/f003-20260816/f003a.log).
- **Defect 1 — `PF_CHECKOUT_LAYER` was handed the wrong index.** `ExternalSource::Layer` stored the *declaration position* (110) where AE wants the *parameter index* (111; the implicit input layer occupies 0). Index 110 is the ADR-0028 `Details` button, so AE answered `BadCallbackParameter` and every layer read returned nothing, every frame, silently. **Layer inputs had never worked.**
- **Defect 2 — external layer pixels were uploaded unconverted.** A checked-out layer arrives in AE's own layout (ARGB, and 8 bytes per pixel at 16-bpc); the working format is RGBA. The bytes went straight to the GPU, so the shader read `(a,r,g,b)`. Confirmed arithmetically before the fix: the cyan solid `(20,180,220)` composited to `rgb(247,23,161)`, which is exactly `mix(self, (255,20,180)/255, 220/255)`. Now converted at the encode site through the same converters the effect's own input uses.
- **Why the earlier run passed anyway (and why the leg was rewritten).** The shader returned the referenced layer verbatim, and the referenced solid sat *underneath* the effect layer in the comp — so "transparent black over the source solid" and "a successful read of that source solid" produced identical pixels. A completely broken checkout scored a pass. The leg now composites the side layer over the effect's own input, so the two phases must differ and each must be the right colour.

### TR-0031-001 — Gradient parameters

- Status: `NOT_RUN` — code-complete including [ADR-0031](adr/0031-gradient-parameters.md) §7, with **no host leg**. The editor is the least verifiable surface in this change: none of its drawing, hit-testing, or picker behaviour has run inside After Effects.
- Date: 2026-08-15
- Baseline: working tree on `codex/stabilize-programmatic-flow` (uncommitted)
- Implemented: §1 (`PoolKind::Gradient`, capacity 4), §3 (8-stop value format, `E54` fail-closed validation, serde persistence), §4 (straight-sRGB interpolation), §5 (256×1 LUT), §6 (keyframe interpolation by union resampling, wired through `ArbitraryData` and the `ArbitraryCallback` dispatch), and [ADR-0032](adr/0032-gradients-are-graph-resources.md) (graph-resource binding).
- §7 implemented in `src/host/gradient_ui.rs` plus the `Command::Event` handler in `src/lib.rs`: `ParamUIFlags::CONTROL` with a 46 px control area, a Drawbot ramp drawn as one-pixel columns, handle strip with selection ring, hit-test that lets handles win over the bar, click-empty-bar-to-add (colour sampled from the ramp so adding a stop never changes the picture), drag-to-move with re-sorting, double-click into `PF_AppColorPickerDialog` (alpha preserved — the picker carries none), and Delete. Every gesture validates before committing, so the editor cannot author an `E54`.
- Deviation from §5, deliberate: the LUT is baked into the render's **working format** rather than a fixed `Rgba32Float`. At 32-bpc that *is* float, satisfying §5's stated reason (not quantizing what the rest of the pipeline preserves); at 8-bpc a float texture would require `FLOAT32_FILTERABLE`, which `Depth::required_features` only guarantees for the deep formats. Recorded here rather than as a superseding ADR because the decision's purpose is met — flag it if that reading is wrong.
- Commands actually run (2026-08-15): `cargo test --all` → **116 passed; 0 failed**, including eight `gradient::tests` (default ramp, every malformed shape rejected and never repaired, ends hold rather than fade, coincident stops make a hard edge, texel-centre LUT, union resampling across differing stop counts, cap never exceeded, serde round-trip) and three `gradient_param_tests` (binds to the Gradient pool, shares one binding rule with a layer input in the same pass, cannot be written or name a pass).
- **Second defect, found only by binary measurement (2026-08-15, AE 2025).** With `CustomUI` added, AE then refused to load the effect: *"global out-flags mismatch — code 6008444, PiPL 6003F44"*. Root cause is upstream: `pipl` 0.1.1 serializes the PiPL as an RC **string literal** of hex escapes under `#pragma code_page(65001)`, so every byte >= 0x80 is code-page converted into `?` (0x3F) on the way into the binary. `OutFlags::CustomUI` is bit 15, which makes byte 1 of the little-endian out-flags word `0x84` — **the first byte in this project's history to cross 0x7F**, which is why the latent upstream bug surfaced now and never before. A first hypothesis (stale incremental build) was **refuted**: `cargo clean` reproduced it exactly, and the truth came from decoding the `eGLO` property straight out of the DLL, not from reasoning. Fixed in [build.rs](../build.rs) `repair_pipl_resource()` — `pipl::build_pipl` is public, so the correct bytes are written to `pipl.bin` and re-emitted through an RC **file reference**, which is copied verbatim; no fork or patched dependency. Verified by measurement: the built DLL now carries exactly one `eGLO` = `0x6008444`, matching the code side.
- **First defect, found by the first host run and fixed (2026-08-15, AE 2025):** `PF_PUI_CONTROL` on a parameter requires `PF_OutFlag_CUSTOM_UI` in the effect's *global* out-flags. Without it AE rejects the whole effect at PARAMS_SETUP — `addProperty("DynamicFx")` throws `effect: no custom ui outflag, but param has ui_width or ui_height or PF_PUI_TOPIC/CONTROL flags` and **no instance can be created at all**. Every unit test, the warning-free release build, and the governance check were green against this build; only the host caught it. Fixed in [build.rs](../build.rs) (`OutFlags::CustomUI`). First-run evidence: `scripts/out/f003/2025/f003a.log` at artifact `17A0D1CC…D18A`; the fixed artifact is `7E06113FFE9EC9FCA47FF1D9440D8B52FB236ADA05FF3764D95B5934DC37E647`.
- Editor unit tests (8): position↔x round-trip with out-of-frame clamping, handle-vs-bar hit resolution, adding a stop leaves the visible ramp unchanged, the 8-stop cap refuses rather than overflows, dragging past a neighbour renumbers and stays sorted, dragging onto an occupied position lands after it (coincident stops are a legal hard edge), the last stop cannot be removed, selection is scoped per parameter slot.
- What the host leg must still show, and why it matters most here: the control actually draws in the Effect Controls panel at all (`CONTROL` + `ui_height` is untested); a click lands where the drawing suggests (the event's `screen_point` is assumed to share `current_frame`'s coordinate space — **the single most likely thing to be wrong**); drag tracking continues after `set_send_drag`; the colour picker returns and the stop updates; Delete removes; edits survive save/reopen; a keyframed gradient animates and matches the unit-tested interpolation; a duplicated instance does not share the value.

- **Update 2026-08-16 — the §7 editor was removed, so this record's editor content is now historical.** Reproducing the shipping reference effect's parameter declaration byte for byte (`param_type = 11`, `ui_flags = 0x82`, 200x80) still crashed AE 2025 on expand, with **zero editor log lines written** — so the fault is not in the declaration, and no further host cycles were spent on it. Deleted: `src/host/gradient_ui.rs`, the `Event` and `ArbitraryCallback` command arms, the `gradient::Canvas` value, the `serde` dependency, the `%TEMP%` level switch, and `PF_OutFlag_CUSTOM_UI` (PiPL out-flags now `0x06000444`, verified by decoding `eGLO` out of the built DLL). `Pool(Gradient, g)` survives as an inert, permanently invisible float — the binding anchor, holding its declaration index. [ADR-0033](adr/0033-gradient-stops-are-ordinary-parameters.md) §6 explicitly permitted this, which is the property it was written to buy. Two presentation changes follow from having no selector: the `Stops` count now drives how many stop groups are on screen, and its default drops from 8 to 2. The eight editor unit tests listed above went with the editor; every value test remains.

- **Host run 2026-08-16 — `PASS` for the ADR-0033 value path.** `f003b` renders a 256x32 frame that is row-constant and linear black-to-white: x=0 -> `(0,0,0)`, 64 -> `(64,64,64)`, 128 -> `(128,128,128)`, 192 -> `(192,192,192)`, 255 -> `(255,255,255)`. That is the new two-stop default (ADR-0033 erratum) sampled through the LUT. Evidence: [`f003b_default_00000.psd`](audits/evidence/f003-20260816/f003b_default_00000.psd), [`f003b.log`](audits/evidence/f003-20260816/f003b.log). The removed §7 editor is out of scope by construction; nothing in this row depends on it.

### TR-0034-001 — Point 3D parameters

- Status: `NOT_RUN` — code-complete, no host leg has run. The unit evidence below is real and verifies no AE behaviour.
- Date: 2026-08-16
- Baseline: working tree on `codex/stabilize-programmatic-flow` (uncommitted)
- Implements: [ADR-0034](adr/0034-point3d-parameters.md) in full — `PoolKind::Point3D` (capacity 8, appended after the ADR-0033 growth), `hint:point3d` retyping a `vec3` while an un-annotated `vec3` stays a Colour, `x,y` normalized to the frame with `z` in pixels, annotation defaults refused as for Point 2D.
- Code paths: `src/binding.rs` (`GROWTH_POOLS`, `PoolKind::Point3D`), `src/definition/param.rs` (`ShaderParamType::Point3D`), `src/frontend/annotation.rs` (`Hint::Point3D`), `src/frontend/glsl.rs` (retype and reject), `src/host/params.rs` (`Point3DDef`), `src/persistence.rs` (kind byte 8), `src/lib.rs` (`read_bound_values` encoding).
- Commands actually run (2026-08-16): `cargo test` → **128 passed; 0 failed**, including `point3d_needs_the_hint_and_a_vec3` (retypes a `vec3`, leaves an un-annotated `vec3` a Colour, routes to the Point3D pool, keeps the 3-word block layout, and rejects `float`/`int`/`vec2`/`vec4` members) and `point3d_defaults_are_refused`; `cargo build --release` → zero warnings; `python scripts/check_governance.py` → `PASS`.
- Host leg authored but not run: [`scripts/f003/f003f_point3d.jsx`](../scripts/f003/f003f_point3d.jsx), wired into `run_f003.ps1` as leg `f`. It probes property 117, logs `DISTINCT_CONTROLS` (the two `vec3`s must produce different `propertyValueType`s — that is the decision), drives the point to (40, 60, 50) in a 160x120 comp, keyframes the stream, and renders.
- **Open question the host leg must settle, not a bug:** whether PF reads a Point 3D's *declared* `x/y` default as a percentage of the frame (as it does for Point 2D) or as absolute pixels. The SDK header is not vendored with the crate and no run has measured it. The declared default (50, 50, 0) lands somewhere visible and draggable under either reading, so it is safe to ship unresolved — but it must not be written down as known.
- What else the host leg must show: the control appears as an AE 3D point widget; moving it changes the render; `x,y` arrive normalized and `z` in pixels as ADR-0034 §3 documents; the keyframed stream animates; save/reopen restores; the M1-M7 batteries stay green, since the topology grew by 8 parameters.

- **Host run 2026-08-16 — `PASS`.** `f003f` renders `rgb(64,127,127)` where the shader encodes `(probe.x, probe.y, probe.z/100)` and the point was driven to `(40, 60, 50)` in a 160x120 comp. `40/160 = 0.25 -> 64`, `60/120 = 0.5 -> 127`, `50/100 = 0.5 -> 127`. **ADR-0034 §3 is confirmed on the host: `x,y` arrive normalized to the frame and `z` arrives in pixels.**
- The decision itself is confirmed by `DISTINCT_CONTROLS 1`: the annotated `vec3` reports `propertyValueType` 6413 (ThreeD spatial) and the un-annotated `vec3` beside it reports 6418 (colour), from one shader, in one read. `KEYS 2` confirms the single animatable stream that motivated the kind.
- Evidence: [`f003f_point3d_00000.psd`](audits/evidence/f003-20260816/f003f_point3d_00000.psd), [`f003f.log`](audits/evidence/f003-20260816/f003f.log).
- **Still unmeasured:** the *declared default's* units. The leg sets the value explicitly, so it says nothing about whether PF reads a declared `x/y` default as a percentage or as pixels. That question stays open in IMPLEMENTATION_STATUS.

### TR-0035-001 — Path parameters

- **Host run 2026-08-16 — `PASS`.** Windows 11 Pro 10.0.26200, After Effects 2025 (`C:\Program Files\Adobe\Adobe After Effects 2025`), artifact `A1F156ABDF988E01B7628E4C8A3C7E7EA09309D306C726CAC095C80266016672` installed at `Support Files\Plug-ins\DynamicFx\DynamicFx.aex`. Command: `pwsh scripts/f003/run_f003.ps1 -Year 2025 -Scenarios @('a','b','c','f','g')`. Baseline: working tree on `codex/stabilize-programmatic-flow` (uncommitted). Evidence: [`docs/audits/evidence/f003-20260816/`](audits/evidence/f003-20260816/). Same run also produced M2 (`a..d`, `e..j`) and M3 (`a..f`) green against the grown 177-property topology, which is what rules out a repointed released parameter stream.

- Status: `NOT_RUN` — code-complete, no host leg has run. **This feature's riskiest surface is entirely unverified**: nothing in it has ever checked out an AE path.
- Date: 2026-08-16
- Baseline: working tree on `codex/stabilize-programmatic-flow` (uncommitted)
- Implements: [ADR-0035](adr/0035-path-parameters.md) in full — `PoolKind::Path` (capacity 2), `hint:path` declaring a graph resource through the existing ADR-0030/0032 rule, the N x 2 `Rgba32Float` vertex texture, count-from-`textureSize`, the 1 x 2 zero texture for an unassigned selector, Beziers delivered unflattened, and the `E7` refusal of the temporal combination.
- Code paths: `src/path.rs` (new — the pure texel encoding), `src/binding.rs`, `src/definition/param.rs`, `src/frontend/annotation.rs` (`hint:path`, `path_param_names`), `src/frontend/grammar.rs` (external names), `src/host/params.rs` (`PathDef`), `src/persistence.rs` (kind byte 9), `src/render.rs` (per-external texture format), `src/lib.rs` (`read_path` checkout, encode site, `ExternalSource::Path`).
- Commands actually run (2026-08-16): `cargo test` → **128 passed; 0 failed**, including four `path::tests` (vertex-to-texel round trip with tangents and frame normalization, the empty path's documented 1 x 2 zero texture, no NaN from a degenerate frame size, the vertex cap enforced not exceeded) and four `path_param_tests` (binds to the Path pool and reaches the render side as `ExternalSource::Path`, cannot be written or name a pass, `E7` on the temporal combination naming both the input and its kind, rejected when the id is also an FxUniforms member).
- **Deviation from ADR-0035's stated cost, recorded rather than hidden.** Its Costs section says an `Rgba32Float` path texture can be "fetched with `texelFetch`, not `texture()`" at 8-bpc. That does not actually dodge the constraint: naga's GLSL frontend pairs the texture with a sampler binding either way, so the bind group still requires `FLOAT32_FILTERABLE` however the shader reads it. The implementation therefore checks the feature at upload time and binds the documented zero texture with a log line when the adapter lacks it, rather than failing bind-group validation and taking the whole render down. Every DX12 adapter this project has run on offers the feature, so the practical effect is nil — but the reasoning in the ADR is wrong, and this row rather than a silent code comment is where that is said.
- Host leg authored but not run: [`scripts/f003/f003g_path.jsx`](../scripts/f003/f003g_path.jsx), wired into `run_f003.ps1` as leg `g`. It draws a four-corner closed rectangle mask at known pixel positions, renders the **unassigned** selector first (§5's obligation is the one that costs a user their project when it is wrong), then assigns and renders again. Channels carry vertex 0's normalized x/y and the vertex count, so one PNG answers count, position and normalization together.
- What the host leg must show: a mask drawn on the layer appears in the selector; `textureSize` reports 5 for a closed four-corner rectangle (N segments means N+1 vertices); vertex 0 lands at (40/160, 30/120); editing the mask changes the next frame; an animated mask updates per frame; an unassigned selector renders rather than failing; save/reopen and aerender reproduce; the M1-M7 batteries stay green.
- **Unresolved semantic, deliberately not asserted:** whether `PF_PathVertex`'s `tan_in_*`/`tan_out_*` are offsets from the vertex or absolute handle positions. `src/path.rs` divides them by the frame either way and documents the ambiguity; the host leg is what settles it, and until then no shader should be told which reading applies.

- **Host run 2026-08-16 — `PASS`.** `f003g` draws a closed four-corner rectangle mask at `(40,30),(120,30),(120,90),(40,90)` on a 160x120 layer and paints `(vertex0.x, vertex0.y, vertexCount/16)`.
  - Unassigned selector: `rgb(0,0,16)` — one zero vertex, count 1. **ADR-0035 §5's `1x2` zero texture, confirmed**, and the render did not fail.
  - Assigned: `rgb(64,64,80)` — `40/160 = 0.25 -> 64`, `30/120 = 0.25 -> 64`, `5/16 -> 80`. **ADR-0035 §3 (normalized vertex texels) and §4 (`textureSize` IS the count) both confirmed**, including N segments yielding N+1 vertices for a closed path.
  - The plug-in log corroborates independently: `path 0: id=0 vertices=0` then `path 0: id=1 vertices=5`.
- The shader reads with `texelFetch`, and the adapter (RTX 5080, DX12) carries `FLOAT32_FILTERABLE`, so the `Rgba32Float` binding path is exercised as designed. An adapter without the feature is still untested.
- Evidence: [`f003g_none_00000.psd`](audits/evidence/f003-20260816/f003g_none_00000.psd), [`f003g_assigned_00000.psd`](audits/evidence/f003-20260816/f003g_assigned_00000.psd), [`f003g.log`](audits/evidence/f003-20260816/f003g.log).
- **Defect this leg found, affecting layers and gradients too.** A render clone resolved its definition *inside* `render`, which is **after** the SmartRender arm stages external resources — so on a clone's first frame `local.compiled` was `None`, nothing was staged, and every external silently bound the frame-sized zero texture. Visible here as a path reading "unassigned" on the first render and correctly on every one after. Resolution is now hoisted into `resolve_transported_definition` and run before staging.
- **A second defect that the first fix caused, reported from interactive use the same day.** Resolving in SmartRender alone made PreRender and SmartRender disagree about how many layers exist this frame: PreRender skipped the checkout, SmartRender asked for the pixels, and AE aborted with *"Node received more checkout requests than expected"*. Both sides now resolve, and PreRender records exactly the ids AE accepted so SmartRender can never request or check in a superset.
- **Harness defect found and fixed, which had already invalidated one run.** After Effects' *disk* cache survives a restart and is not keyed on the plug-in binary, so a rebuilt AEX rendering a byte-identical comp gets the previous build's frames served back — no render call, nothing in the plug-in log, and a PSD that reads as fresh evidence. `f003RenderPsd` now calls `app.purge(PurgeTarget.ALL_CACHES)` before every render and logs `PURGE ok=1`.

### TR-0037-001 — Pool valid range (float>1, negative, int>10)

- Status: `PASS` on AE 2025 and AE 2026 (host run 2026-08-19; both years recorded in the host-run block below). Unit + governance green; the pre-fix defect measurement (0.0.3) is preserved below.
- Date: 2026-08-19
- Baseline: working tree on `main` at `52f65e5` + ADR-0037 implementation (uncommitted at the time of this record); Rust 1.97.1
- Implements: [ADR-0037](adr/0037-pool-valid-range-and-slider-range.md) — `POOL_FLOAT_VALID_RANGE` / `POOL_INT_VALID_RANGE` (±10⁹) registered at `PARAMS_SETUP` for the Float and Integer pools, slider defaults `0..1` / `0..10` unchanged, `configure_slots` treating `@param min:/max:` as the slider range and resetting an un-ranged binding to the display default + wide valid range; the runtime still passes values raw.
- Code paths: `src/host/params.rs` (constants, pool declarations), `src/lib.rs` (`configure_slots`).
- **The defect, as measured before the fix** (public issue [#5](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/5); AE 2026 v26.3, Windows 11, 0.0.3 artifact `20868E2D…`, 8-bpc, pixel readback of a per-uniform column ramp): `min:2 max:200` at 40 → shader `1.004` (= 1.0 after 8-bit quantisation); `min:0.15 max:4` at 3.0, `min:0 max:3` at 2.5, `min:0.4 max:12` at 6.0, `min:1.2 max:24` at 9.0, `min:0.2 max:2` at 1.8, `min:0 max:3` at 2.25 → all `1.004`; `min:0 max:1` at 0.32 → 0.314, `min:0 max:2` at 0.70 → 0.706 (values ≤ 1 exact); `hint:angle` at 135 → 135. Host-side `setValue(0.3)` on the `2..200` slot rejected as out of range, so AE held the declared range and the correct value; the raw evidence table lives in the issue (its artifacts are outside this repository).
- Cause: `PF_UpdateParamUI` changes only `slider_min/slider_max/precision/display_flags` (SDK header, `PF_ParamUtilsSuite3`); the valid range registered at `PARAMS_SETUP` (`0..1` Float, `0..10` Integer) is what AE clamps rendered values to. `read_bound_values` and the std140 packing pass values raw.
- Why the M2 rows missed it: TR-M2-002/003 exercised 0.5 on `min:0 max:2`, an int at 3, an angle at 90 — every probe inside the registered range.
- Commands actually run (2026-08-19): `cargo test` → **129 passed; 0 failed**, including `host::params::tests::pool_valid_ranges_are_wide_symmetric_and_exact` (wide, symmetric, `f32`-exact, display defaults inside) and the extended `growth_pool_property_indexes_match_the_harness` (Float 0 → property 6, Float 1 → 7, Integer 0 → 54, the indexes the leg drives); `cargo build --release` → zero warnings; `python scripts/check_governance.py` → `PASS`.
- Host leg authored, not run: [`scripts/f003/f003h_range.jsx`](../scripts/f003/f003h_range.jsx) (runner leg `h`), gate [`scripts/f003/f003h_check.py`](../scripts/f003/f003h_check.py). One shader declares `wide min:2 max:200 default:40`, `neg min:-1 max:1 default:-0.6`, `count min:0 max:100 default:60` (int) and paints `(wide/200, (neg+1)/2, count/100)`; every expected 8-bit value is an exact integer and far from the clamped result.
- What the host leg must show, per year (AE 2025 and AE 2026 separately), with `PURGE ok=1` logged before every render:
  - `f003h_defaults` = `rgb(51,51,153)` — the annotation defaults 40 / −0.6 / 60 arrive unclamped (the 0.0.3 artifact renders `rgb(1,128,26)`);
  - `f003h_set` = `rgb(191,153,204)` after `setValue` 150 / 0.2 / 80 (`ASSIGN ok=3/3`);
  - `RANGE` / `TYPING_BOUND` lines recorded (measurement of the host's typing courtesy, not a gate);
  - `f003h_thermal` — `examples/thermal.glsl` at its defaults with `glow` = 1.2 reaching the shader (`THERMAL_RANGE glow … value=1.2`): visual evidence PSD, first host sighting of the intended palette;
  - regression: `run_m2.ps1` (parameters, annotations, defaults, kinds) and `run_m3.ps1` (persistence incl. `-Session4` reopen of a project saved under the old ranges) green on the new artifact.
- **Host run 2026-08-19 — `PASS` on AE 2025 and AE 2026 (recorded separately below).** Windows 11 Pro 10.0.26200; artifact `DynamicFx.aex` (`dynamicfx.dll` 8,544,768 B) SHA-256 `BFE1AB9FBE20F64E9098599C57F89B9D721C9DA8735F24F480384F85E5B858C3`, installed at each year's `Support Files\Plug-ins\DynamicFx\` and installed-hash verified; GPU adapter `NVIDIA GeForce RTX 5080`, backend Dx12, driver `32.0.15.9621` (from the plug-in log). Toolchain 1.97.1; baseline working tree on `main` at `52f65e5` + ADR-0037 (uncommitted; the binary predates the comment-only ADR correction below and is behaviour-identical to it).
  - **AE 2025 (v25.6.6):** `f003h_defaults` = `rgb(51,51,153)` (wide=40, neg=−0.6, count=60 arrive unclamped — the 0.0.3 artifact renders `rgb(1,128,26)`); `f003h_set` = `rgb(191,153,204)` after `setValue` 150 / 0.2 / 80 (`ASSIGN ok=3/3`); `f003h_check.py` → `RESULT PASS`. `THERMAL_RANGE glow … value=1.20000004768372` and the palette rendered (visual PSD — first host sighting of thermal's intended colours). Regression on the same artifact: M2 battery `a..j` — 11 logic legs plus all 12 pixel probes PASS; M3 `-Session4` + `-Aerender` — reopen-without-Compile `(51,51,0)`, corruption recovery `(51,51,0)`, duplicate isolation `(51,51,0)`/`(115,115,0)`, torn-token→snapshot `(51,51,0)`, invalid pass-through `(10,200,30)`, aerender PSD `(51,51,0)` — all PASS. Evidence: [`docs/audits/evidence/adr-0037/ae2025/`](audits/evidence/adr-0037/ae2025/).
  - **AE 2026 (v26.3):** `f003h_defaults` = `rgb(51,51,153)`, `f003h_set` = `rgb(191,153,204)`, `f003h_check.py` → `RESULT PASS`; `THERMAL_RANGE glow … value=1.2` and palette rendered. Regression: M2 `a..j` 11 legs + 12 pixel probes PASS; M3 `-Session4` + `-Aerender` PASS (same probe values as 2025). Evidence: [`docs/audits/evidence/adr-0037/ae2026/`](audits/evidence/adr-0037/ae2026/).
  - **Measured, and it corrected the ADR:** after a binding declared `min:2 max:200`, `setValue(0.3)` on that slot was **accepted** on both years (`TYPING_BOUND … accepted`), not rejected. So `PF_UpdateParamUI`'s valid-field write is a no-op for scripting/typing as well as for rendering — the host uses the wide `PARAMS_SETUP` range on both paths. [ADR-0037](adr/0037-pool-valid-range-and-slider-range.md) §2 and the `configure_slots` comment were corrected to state this (the pre-run text had hypothesised a typing courtesy from the 0.0.3 `setValue` rejection).
- Related ADR: [0037](adr/0037-pool-valid-range-and-slider-range.md); issue [#5](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/5)

### TR-EX-001 — Shipped examples compile

- Status: `PASS` for the compile contract only. **The visual result is `NOT_RUN`** — see the boundary below; do not read this row as "the examples look right".
- Date: 2026-08-15
- Baseline: working tree on `codex/stabilize-programmatic-flow` (uncommitted at the time of this record)
- Artifacts: [`examples/thermal.glsl`](../examples/thermal.glsl) (6-pass graph, 10 params), [`examples/orb.glsl`](../examples/orb.glsl) (1 pass, `prev` + `@window 16`, 8 params), [`examples/README.md`](../examples/README.md)
- Command: `cargo test --all` → **99 passed; 0 failed**, including `example_tests::thermal_example_compiles` and `example_tests::orb_example_compiles`. Both feed the file's exact bytes (`include_str!`) to `evaluate_committed_source` — the same classify → envelope grammar → GLSL frontend → lowering path the plug-in runs — and assert `Diag::Ok` with a compiled effect. A grammar, ABI, or annotation change that would break a user's copy-paste now fails the build.
- Also run: `cargo build --release` (zero warnings); `python scripts/check_governance.py` → `PASS` (58 files, 485 links, 0 errors — `examples/` added to the scanned set in this change, since it is a public surface).
- Provenance: `thermal.glsl` is the M7 benchmark's thermal-A shader ported verbatim from [`scripts/m7/m7_lib.jsxinc`](../scripts/m7/m7_lib.jsxinc) — proven to render under TR-M7-001…006 — with `default:#RRGGBB` values added per [ADR-0026](adr/0026-color-parameter-default-annotation.md). `orb.glsl` is new; its temporal shape follows the TR-M6-001 fixture.
- **Verification boundary (deliberate):** the benchmark never set parameter values, so the thermal shader has only ever rendered with the pre-ADR-0026 all-white color defaults — **its intended palette has never been seen on a host**. The chosen hex defaults and `orb.glsl`'s appearance are authored judgements. Both need a visual pass on AE 2025/2026 at the 0.0.3 batch exit, alongside a check that `orb.glsl`'s angle/checkbox controls drive what their labels claim.
- **2026-08-19 — third example added: [`examples/apple-thermal.glsl`](../examples/apple-thermal.glsl)** (10 passes, 22 params: 18 float, 2 `hint:angle`, 1 `hint:bool`, 1 `hint:gradient`; graph `sh, sv, dh, dv, temp, sbh, sbv, th, tv, col`). Compile contract: `example_tests::apple_thermal_example_compiles` added next to the other two; `cargo test` → **130 passed; 0 failed** (baseline: working tree on `main` at `b1cb0f7`, Rust 1.97.1). **Visual: `PASS` on AE 2025** — rendered on Windows 11 Pro 10.0.26200, After Effects 25.6.6x4, with the installed 0.0.4 artifact `DynamicFx.aex` SHA-256 `BFE1AB9FBE20F64E9098599C57F89B9D721C9DA8735F24F480384F85E5B858C3` at `Support Files\Plug-ins\DynamicFx\` (hash re-verified 2026-08-19), 8-bpc comp, on a 1024x1024 precomp holding a 512-px logo with alpha; frames rendered by `comp.saveFrameToPng` after `app.purge(ALL_CACHES)` at t = 0..8 s (1 fps) at the shader's annotation defaults, all 22 controls bound (`Status: compiled: 10 passes, 22 params`). Expected: black top face, hot contour bands drifting over time, thin light-blue edge line where cold, blue outer glow warming next to hot regions, alpha = source coverage inside the shape and a glow alpha outside (measured 226/157/81/36 at 10/40/80/120 px from the contour). Observed: as expected — evidence [`docs/audits/evidence/examples/apple-thermal-ae2025/`](audits/evidence/examples/apple-thermal-ae2025/) (`frame_t2s.png`, `timeline_0-8s_1fps.png`). AE 2026: `NOT_RUN`. The `Use Custom Ramp` path was exercised once during authoring (default black→white gradient renders a greyscale field) but not re-recorded on the final source — treat it as `CLAIMED_UNVERIFIED` for this file.

### TR-REL-004 — 0.0.4 release verification

- Status: `PASS` — the packaged artifact is the exact binary host-verified on both years by [TR-0037-001](#tr-0037-001--pool-valid-range-float1-negative-int10).
- Date: 2026-08-19
- Baseline: working tree on `main` at `52f65e5` + ADR-0037; `Cargo.toml` bumped `0.0.3` → `0.0.4`; **PiPL subversion unchanged at 5** — ADR-0037 changes no topology or out-flag, so no cache-busting bump is needed (or wanted: a bump would force AE to re-read the PIPL for a functionally unrelated reason).
- OS: Windows 11 Pro 10.0.26200. GPU: NVIDIA GeForce RTX 5080, DX12, driver 32.0.15.9621.
- Artifact: `DynamicFx.aex` SHA-256 `BFE1AB9FBE20F64E9098599C57F89B9D721C9DA8735F24F480384F85E5B858C3` (8,544,768 B), packaged as `DynamicFX-0.0.4-win-x64.zip` (build output; not in git — the hashes here are its identity) SHA-256 `956F988704BFA4FF187FE08DE53F1A38EFEECAA2E078CA9B70B96E94EA2CA8EC`, containing the aex + README + LICENSE + `INSTALL.txt` + `SHA256SUMS` + `examples/` (root layout matching 0.0.3). Published at <https://github.com/JUNKDOGE-JOE/dynamicfx/releases/tag/v0.0.4> as a pre-release; tag `v0.0.4` on `main`.
- **The version is not embedded in the binary, so no rebuild was made for the bump.** `CARGO_PKG_VERSION` is used nowhere in `src/` or `build.rs`, the PiPL `AE_Effect_Version` is a hardcoded literal (`build.rs`), and the DLL contains no `0.0.x` string. Bumping `Cargo.toml` therefore changes zero binary bytes: the `BFE1AB9F…` artifact verified under TR-0037-001 (built while `Cargo.toml` read `0.0.3`) **is** the 0.0.4 binary, packaged as-is. This honours the not-byte-reproducible constraint recorded in TR-REL-003 (verify the file, package that same file, never rebuild in between) without the 0.0.3-era complication of a version-driven rebuild.
- Host verification of the functional change: [TR-0037-001](#tr-0037-001--pool-valid-range-float1-negative-int10) `PASS` on AE 2025 (v25.6.6) and AE 2026 (v26.3) against this exact artifact — `f003h` renders the previously-clamped float/int values unclamped, `examples/thermal.glsl` renders its intended palette, and the M2 + M3 regression batteries stay green on both years. Evidence: [`docs/audits/evidence/adr-0037/{ae2025,ae2026}/`](audits/evidence/adr-0037/).
- Unit evidence on the same tree: `cargo test` → **129 passed; 0 failed**; `cargo build --release` → zero warnings (the release DLL predates the comment-only ADR-0037 §2 correction and is behaviour-identical to it); `python scripts/check_governance.py` → `PASS` (37 accepted ADRs).
- **Not verified by this release** (unchanged from 0.0.3): After Effects 2024 (unprovisioned by decision) and 2023 (host will not launch); a mask/path input on an adapter lacking `FLOAT32_FILTERABLE`; the Point 3D *declared default's* units. Each is recorded in [IMPLEMENTATION_STATUS](IMPLEMENTATION_STATUS.md).

### TR-REL-003 — 0.0.3 release verification

- Status: `PASS` — the packaged artifact carries fresh evidence on both verified hosts.
- Date: 2026-08-16
- Baseline: working tree on `codex/stabilize-programmatic-flow` (uncommitted), Cargo `0.0.3`, PiPL subversion 5
- OS: Windows 11 Pro 10.0.26200. GPU: NVIDIA GeForce RTX 5080, DX12, driver 32.0.15.9621.
- Artifact: `DynamicFx.aex` SHA-256 `20868e2d52d2cd9397b987d9f120c92b8b346cf316f3920279a1e017213c99fd`, packaged as `DynamicFX-0.0.3-win-x64.zip` (build output; not in git — the hashes here are its identity) SHA-256 `c5c4e4a503b80c8e5b2278dbc865dc5458bc341debae35ea132bff25c1f3c06f`, containing the aex + README + LICENSE + `INSTALL.txt` + `SHA256SUMS` + `examples/`. Published at <https://github.com/JUNKDOGE-JOE/dynamicfx/releases/tag/v0.0.3> as a pre-release; private tag `v0.0.3` on `dynamicfx-dev` marks the source. Installed to `…\Adobe After Effects <year>\Support Files\Plug-ins\DynamicFx\DynamicFx.aex` on 2025 and 2026, hashes verified equal to the build.
- **This exact artifact was verified, not an equivalent one.** The feature evidence recorded above (TR-0030/0031/0034/0035, TR-0015) was gathered on `A1F156AB…`; the version bump to 0.0.3 produced a new binary, so every host leg was re-run against `20868E2D…` rather than an equivalence argument being made.

Commands and results:

| Host | Suite | Result |
|---|---|---|
| AE 2025 | `run_f003.ps1 -Scenarios @('a','b','c','f','g')` | all five `RESULT_DONE`, pixels below |
| AE 2025 | `run_m2.ps1 -Scenarios @('a'..'j')` | 11/11, incl. `M2F slot1=[Master Level] value=0.5 numKeys=2` (alias inheritance) and `M2J status=[Status: E32 definition rejected]` (pool overflow fails closed) |
| AE 2025 | `run_m3.ps1` | 6/6, incl. `M3D slot1=[gain] numKeys=2` and `M3F` after reopen |
| AE 2026 | `run_f003.ps1 -Scenarios @('a','b','c','f','g')` | all five `RESULT_DONE`, pixels identical to 2025 |

Rendered pixels, identical on both hosts (centre texel, 160x120 comp):

| Render | Expected | Observed |
|---|---|---|
| `f003a_none` (layer unassigned) | `rgb(200,40,40)` — effect layer untouched | `rgb(200,40,40)` |
| `f003a_assigned` | `rgb(20,180,220)` — the referenced solid | `rgb(20,180,220)` |
| `f003b_default` (gradient) | linear black→white, row-constant | `0/64/128/192/255` at x=`0/64/128/192/255` |
| `f003f_point3d` | `rgb(64,128,128)` from `(40,60,50)` | `rgb(64,127,127)` (0.5 rounds down) |
| `f003g_none` (mask unassigned) | `rgb(0,0,16)` — 1 zero vertex | `rgb(0,0,16)` |
| `f003g_assigned` | `rgb(64,64,80)` — vertex0 `(0.25,0.25)`, 5 vertices | `rgb(64,64,80)` |

- Unit evidence on the same tree: `cargo test` → **128 passed; 0 failed**; `cargo build --release` → zero warnings; `python scripts/check_governance.py` → `PASS` (35 accepted ADRs).
- Evidence files: [`docs/audits/evidence/f003-20260816/`](audits/evidence/f003-20260816/) (2025 PSDs and logs from the feature run) and `scripts/out/f003/{2025,2026}/` for this artifact's run.
- **Harness flake, recorded because it will recur:** the first f003 leg after a cold `AfterFX.exe -r` sometimes runs its script body while `app.scheduleTask` callbacks never fire, so the leg logs its setup and stalls to a 240 s timeout. It happened once per host here and both times the leg passed on an immediate re-run against the warm instance. This is the M0 spike finding the runner's warm-start exists to avoid; the warm-start window is evidently still too short after a forced kill. Not a product defect — no plug-in code runs in the stalled window.
- **The build is not byte-reproducible, and this is now a release-procedure constraint.** After the host runs, a documentation-only edit to a doc comment produced a different binary hash — and *reverting that edit and rebuilding produced a third hash*, not the original. So `20868E2D…` cannot be re-derived by rebuilding the tagged source; it exists only as the file that was built, installed, tested and packaged. Consequence for every future release: **verify the artifact, then package that same file, and never rebuild in between.** Recorded here rather than fixed — making the build reproducible (`--remap-path-prefix`, one codegen unit, a pinned `SOURCE_DATE_EPOCH`) is worth doing, but it is a change to the build, not to this release.
- **Not verified by this release:** After Effects 2024 (unprovisioned by decision) and 2023 (host will not launch); a mask input on an adapter lacking `FLOAT32_FILTERABLE`; the Point 3D *declared default's* units. Each is recorded in [IMPLEMENTATION_STATUS](IMPLEMENTATION_STATUS.md).

### TR-0015-001 — Not-ready marker (E53)

- Status: `NOT_RUN` — the host leg has no run. Unit evidence below is real; it does not verify the AE behavior and must not be read as if it does.
- Date: 2026-08-15
- Baseline: working tree on `codex/stabilize-programmatic-flow` (uncommitted at the time of this record)
- Contract under test: a committed source whose definition is unpublished must be distinguishable, from the render side, from an instance that was never authored. Before this change both presented identically — StateToken 0, no snapshot, and a `Source` slider reading its own 0.0 default whether or not the `` `…`;0 `` expression is present ([source.rs](../src/source.rs), [params.rs](../src/host/params.rs)) — so pass-through was indistinguishable from "nothing here". `Diag::PublicationPending = 53` is appended inside the runtime/transport family that [ADR-0015](adr/0015-statetoken-and-diagnostics.md) §4 pre-partitions for exactly this (48-63); no new ADR is required and no existing code is renumbered.
- Code paths: [`src/diagnostics.rs`](../src/diagnostics.rs) (code + registry + family guard), [`src/host/idle.rs`](../src/host/idle.rs) `sync_state_token` (publishes E53 where it previously returned without writing; the pending mark fills an empty stream only and never overwrites a saved Active word or a more specific diagnostic), [`src/lib.rs`](../src/lib.rs) `observe_core` (a refused `registry_insert` now reports E53 instead of leaving a success status over a pass-through render).
- Commands actually run (2026-08-15): `cargo test --all` → **97 passed; 0 failed**, including the new `token_tests::pending_publication_is_distinguishable_from_never_authored`; `cargo build --release` → **finished, zero warnings**; `python scripts/check_governance.py` → **RESULT=PASS** (57 files, 467 local links, 0 errors).
- Fixture consequence, already applied: E53 encodes to a **non-zero** word, so the readiness idiom `property(5).value !== 0` would score a pending instance as ready. [`scripts/m7/m7bench.jsx`](../scripts/m7/m7bench.jsx) — the only site that *gates* behavior on it — now tests `% 4 === 1` (the Active state) instead. Evidence scripts that merely log `token=` are unaffected.
- What the host leg must still show: (1) an instance stuck unpublished publishes E53 rather than leaving the stream at 0; (2) `Show Full Status` names it; (3) a normal compile never transits E53 (it would dirty the project for a sub-second state); (4) reopening a project does not clobber a saved Active token with E53; (5) the M1-M7 batteries stay green.
- Related: the scripted readiness contract this makes pollable is documented in the public [README](../README.md#scripting-wait-for-readiness-before-you-render).

- **Host run 2026-08-16 — `PASS`.** `f003c`: `RESULT F003C ready=1 saw_e53=0`. The negative obligation is the point — an ordinary compile reaches `Active` without ever transiting `E53`, so the marker does not fire on the happy path. Evidence: [`f003c.log`](audits/evidence/f003-20260816/f003c.log). Host details as in TR-0035-001's run record.

### TR-REL-002 — 0.0.2 release verification

- Status: `PASS`
- Date: 2026-08-14
- Artifact: `DynamicFx.aex` (Cargo 0.0.2, PIPL subversion 5) SHA-256 `79dfb58ada87845d66c303e9c6f6396e83939d4da61a269d3da482f155123931`; packaged as `DynamicFX-0.0.2-win-x64.zip`
- Contents since 0.0.1: ADR-0026 color `default:#RRGGBB[AA]` (TR-0026-001), ADR-0028 Details button + float-slider precision (TR-0028-001), ADR-0029 logical-resolution ABI (TR-0029-001); README teaches the complete expression form; Cargo license field aligned to MIT.
- Verification: full M1-M7 driver batteries on THIS artifact — AE 2025: all suites `PASS` (battery.log 19:39); AE 2026 v26.3: all suites `PASS` (battery2026.log 19:54); 96 unit tests; governance `PASS`. Old-topology (0.0.1) projects reopen unchanged — exercised by the m3 suites.
- Scope: ADR-0027 pre-release host subset (2025/2026 verified; 2023 `BLOCKED` by host; 2024 no host).

### TR-REL-001 — 0.0.1 release verification

- Status: `PASS` (the released artifact carries fresh full batteries on both verified hosts)
- Date: 2026-08-14
- Artifact: `DynamicFx.aex` (= `target/release/dynamicfx.dll` at Cargo version 0.0.1) SHA-256 `da071181cd9ef126b50a233ee4aa60f81089f14038662ae42a2ccd42ed77c1b6`; packaged as `DynamicFX-0.0.1-win-x64.zip` (aex + INSTALL + LICENSE + SHA256SUMS)
- Verification: complete M1-M7 driver batteries re-run on THIS artifact — AE 2025 v25.6.6: all suites `PASS` (battery.log 15:24); AE 2026 v26.3: all suites `PASS` (battery2026.log 15:39); 94 unit tests green; governance `PASS`
- Scope: ADR-0027 pre-release host subset (2025/2026 verified; 2023 `BLOCKED` by host; 2024 no host)
- Distribution: public curated repository + GitHub Release v0.0.1 (pre-release); internal governance corpus remains in the private development repository per ADR-0027

### TR-Y26-001 — AE 2026 full-suite host run

- Status: `PASS` (the complete M1-M7 verification battery green on AE 2026 with the unmodified M7 exit artifact — zero year-specific changes)
- Date: 2026-08-14
- Artifact: SHA-256 `4AD318E6A0BFD35BE5B1ADCCDC8EDBB68C1A90551C870C5082702192EACD8C0C`, installed via `scripts/install.bat 2026` to `…\Adobe After Effects 2026\Support Files\Plug-ins\DynamicFx\DynamicFx.aex`
- Host: After Effects 2026 v26.3 (FileVersion 26.3) zh_CN, Windows 11 Pro 10.0.26200, RTX 5080 / Dx12 / 32.0.15.9621; MFR at host defaults
- Commands: the same driver batteries as AE 2025 with `-Year 2026` — `run_m7.ps1` (10-scene benchmark + summarize), `run_m1` (+`-Aerender`, `-Checks`), `run_m2`, `run_m3` (+`-Session4`, `-Aerender`, `-Checks`), `run_m4`, `run_m5`, `run_m6` (+`-QuitAE`, `-Aerender`, `-Checks`)
- Observed: every suite `PASS` ([battery2026.log](audits/evidence/host-ae2026/battery2026.log)); m5/m6 numeric gates `fails=0`; benchmark totals match AE 2025 within noise ([m7_bench_2026.md](audits/evidence/host-ae2026/m7_bench_2026.md), 325 perf lines all assigned); aerender legs (m1/m3/m6) green in the 2026 render engine
- Caveat preserved: a session-tooling watchdog closed two AE progress windows during the m3 leg ("保存项目", "正在执行脚本 m3b_save.jsx…"); the operations underneath completed and the m3 numeric checks passed — the watchdog was killed mid-run and is retired from suite runs
- Per-suite raw outputs under `scripts/out/m1..m7/2026/` (gitignored); curated: [evidence/host-ae2026/](audits/evidence/host-ae2026/)

## Result record template

When a matrix cell changes, add a result record below or in the related audit and link it from the cell.

```markdown
### RESULT-ID — descriptive name

- Status: PASS | FAIL | NOT_RUN | BLOCKED | CLAIMED_UNVERIFIED
- Baseline commit/diff:
- OS:
- Rust toolchain:
- AE year and full version/build:
- Plugin artifact identity:
- Install destination:
- Date/time:
- Command or exact steps:
- Expected:
- Observed:
- Raw artifacts:
- Audit:
- Notes/blockers:
```

## Evidence retention rules

- Keep failing logs when a later run passes.
- Do not overwrite an output artifact without preserving which baseline produced it.
- Prefer repo-relative artifact paths. If an artifact must remain outside the repo, record an absolute path plus file hash and explain retention.
- A screenshot alone cannot prove parameter values, host version, or pixel correctness; pair it with logs/fixture metadata.
- Image correctness tests require numeric comparison and tolerance, not only visual inspection.
- Performance results require hardware, adapter/backend, resolution, graph, instance count, cache state, and commit.
