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
| TR-MAC-001 | Post-M7 | Native Apple Silicon AE 2026 | [`PASS`](#tr-mac-001--native-apple-silicon-ae-2026) | [Native evidence](audits/evidence/macos-arm64-20260908/README.md) |
| TR-QUALITY-001 | Quality | Headless Metal sampling, procedural AA/noise and precision fixtures | [`PASS`](#tr-quality-001--headless-metal-shader-quality) (named GPU fixtures only) | [Quality evidence](audits/evidence/shader-quality-20260908/README.md) |
| TR-WGSL-001 | Research | WGSL frontend feasibility and equivalent GLSL/WGSL Metal pixels | [`PASS`](#tr-wgsl-001--wgsl-feasibility-on-metal) (non-shipping spike only) | [WGSL evidence](../spike/wgsl/evidence/run.json) |

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
| Copied instance keeps its own slot mapping (same source, different BindingPlan) | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-bind-002--copied-instance-corrupts-slot-mapping-field-defect) (fix build `FF1197D9…`; `FAIL` in the field on 0.0.4) | [`PASS`](#tr-bind-002--copied-instance-corrupts-slot-mapping-field-defect) (fix build `FF1197D9…`; `FAIL` on 0.0.4 / `24E963FB…`) | `NOT_RUN` |
| Interrupted preview leaves no cached frame missing a layer | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` | [`PASS`](#tr-cache-001--interrupted-render-poisons-the-frame-cache-field-defect) (fix build `cfccd5d`; `FAIL` on released 0.0.4) | `NOT_RUN` |

### TR-BIND-002 — Copied instance corrupts slot mapping (field defect)

- Status: `FAIL` — observed in the field on the 0.0.4 artifact, AE 2025 (25.6.6x4), 2026-08-19, and **reproduced by the scripted harness leg on 2026-08-21** (AE 2026 26.3x87, pre-fix artifact `24E963FB…`, both compile orders, reopen and `aerender` legs — records below). **Fixed: [ADR-0038](adr/0038-registry-key-per-binding-plan.md) (Accepted, mechanism B + plan-token transport) — the harness leg is `PASS` on the fix build `FF1197D9…` on AE 2026 (19:58) and AE 2025 (20:19), 2026-08-21 (records below); the M2/M3 batteries on the fix build are `PASS` on both years ([TR-0038-001](#tr-0038-001--m2m3-batteries-on-the-adr-0038-fix-build)).** Public issue [#6](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/6).
- Date: 2026-08-19
- Baseline: installed 0.0.4 release artifact (per [TR-REL-004](#tr-rel-004--004-release-verification); installed hash not re-verified in this session); working tree on `main` at `b1cb0f7` (docs only touched).
- Scenario: one instance whose `BindingPlan` was **migrated** (IDs added over ~15 in-place source edits, so slot order ≠ GLSL declaration order) was copy/pasted to a layer in another comp; the pasted instance was compiled fresh (declaration-order slots). Same source → same `DefinitionHash`.
- Observed: the pasted instance rendered black-and-white noise, then coloured noise after `Compile`; the original instance flickered and then rendered with its **parameter roles permuted** — 16 of 18 float slots and both angle slots read through the fresh declaration-order table while the AE streams kept their old-slot values, four of them clamped to the new sliders' ranges (`bloom_radius` 24 read as `contrast` → 3, `thickness` 140 as `halo` → 2, `wall_heat` 1 as `halo_radius` → 4, `halo_radius` 70 as `grain` → 0.3). Status stayed `compiled: 10 passes, 22 params` — no diagnostic. Full readback, the pre-incident values, the slot-by-slot mapping and the log excerpt (`definition resolved from process registry` → `pipelines built` alternating between the two instances' sizes, each followed by `idle slot ui applied` on the original) are in [`docs/audits/evidence/field-20260819-copy-instance/`](audits/evidence/field-20260819-copy-instance/README.md).
- Reading: the render/UI path resolves the definition — including the slot table used for stream reads and slot UI — from a process-wide registry keyed by `DefinitionHash`; two instances of the same source with different plans cannot both be right, and whichever compiled last owns the entry. TR-M3-001's "duplicate isolation" probe passed because both duplicates carried identical fresh plans. This contradicts the intent of ADR-0005/ADR-0013 (per-instance stable ParamIds, per-instance plan persisted by ADR-0016) and must be fixed in the runtime, not worked around in shaders.
- Repair applied to the user's project (not a fix): remove + re-add the effect with the same expression → fresh declaration-order plan matching the registry entry; render normal again (evidence PNG in the folder).
- Harness leg (authored 2026-08-21): [`scripts/bind/tr_bind_002.py`](../scripts/bind/tr_bind_002.py) — A migrated (v1 `p1 p2 p3 / a1 / texB`, then v2 inserts `p0`, `a0`, `texA` in front → A keeps `p1..p3` in F01..F03, `p0` in F04, `a1`/`a0` in Angle 01/02, `texB`/`texA` in Layer 01/02), B fresh (`addProperty` + the same v2 text → declaration order); per-instance distinguishable values on 4 floats, 2 angles and **2 layer inputs**; slot names read back after idle ticks; each instance rendered alone twice with the four quadrants encoding `(p0,p1,p2)`, `(p3,a0/360,a1/360)`, `texA`, `texB`; both compile orders; `--reopen` adds same-session reopen and `aerender` passes. Procedure and readouts: [`docs/audits/evidence/tr-bind-002/`](audits/evidence/tr-bind-002/README.md).
- **Pre-fix harness run — `FAIL` (defect reproduced), 2026-08-21 18:19 local, AE 2026 26.3x87, Windows 11 Pro 10.0.26200, installed artifact `24E963FB…` (the `cfccd5d` build, whose registry code is identical to released 0.0.4 — the only difference is the SmartRender checkout arm, not exercised here):** order A→B corrupted **A** (slot names renamed to B's labels by the idle observer; `TL=(51,76,102)` for `(26,51,76)`, `TR=(25,51,25)` for `(102,26,51)`, **layer inputs swapped** green/red); order B→A corrupted **B** symmetrically (names renamed to `p1 p2 p3 p0 / a1 a0`; `TL=(204,127,153)` for `(128,153,178)`; layers swapped yellow/blue). The instance that compiled last was always correct. Report + 8 PNGs: [`docs/audits/evidence/tr-bind-002/prefix-24E963FB/`](audits/evidence/tr-bind-002/prefix-24E963FB/report.txt).
- **Pre-fix reopen/`aerender` legs — `FAIL`** (2026-08-21 18:30, same host/artifact): same-session reopen kept A corrupted; under `aerender` the first clone rebuilt from its snapshot and the second adopted that entry by fingerprint (B rendered through A's table). Evidence: [`prefix-24E963FB/reopen-aerender/`](audits/evidence/tr-bind-002/prefix-24E963FB/reopen-aerender/report.txt).
- **Fix build run 1 — `6E4E80A6…` (registry per plan, snapshot-carried identity only) — partial, 2026-08-21 19:49, AE 2026 26.3x87:** slot names stayed correct in both orders (idle half fixed), reopen `PASS`, `aerender` `PASS` (`rebuilt from snapshot` +2), but in every warm leg the first-compiled instance rendered through the other's plan with `resolved by latest entry … clone carries no plan` ×2 — warm render clones carry no snapshot (AE keeps the flattened copy taken at `addProperty`; a compile is not a change it sees). Led to ADR-0038 §7. Evidence: [`fix-6E4E80A6-run1/`](audits/evidence/tr-bind-002/fix-6E4E80A6-run1/report.txt).
- **Fix build run 2 — `FF1197D9…` (plan-token transport) — `PASS`, 2026-08-21 19:58 local, Windows 11 Pro 10.0.26200, AE 2026 26.3x87, artifact `dynamicfx.dll` 8,564,736 B SHA-256 `FF1197D9DB09CCE81D8E2CE26AC4FD3FA752FEA8705109FBCA8D9725670A0344` installed at `…\Adobe After Effects 2026\Support Files\Plug-ins\DynamicFx\` (hash verified), working tree `6386e87` + the uncommitted ADR-0038 implementation, `cargo test` 147 passed. Command `python scripts/bind/tr_bind_002.py --reopen`. Expected: every instance reads its own floats/angles/layers in both orders, after reopen and under `aerender`, slot names unchanged, strict log lines at zero. Observed: exactly that — 0 mismatches over 48 quadrant probes, `resolved from process registry` +2 per warm leg and after reopen, `rebuilt from snapshot` +2 under `aerender`, all strict counters 0. Evidence: [`fix-FF1197D9-run2/`](audits/evidence/tr-bind-002/fix-FF1197D9-run2/report.txt); procedure in [`docs/audits/evidence/tr-bind-002/README.md`](audits/evidence/tr-bind-002/README.md).
- **Fix build on AE 2025 — `PASS`, 2026-08-21 20:19 local, Windows 11 Pro 10.0.26200, AE 2025 25.6.6x4, same artifact `FF1197D9…` installed at `…\Adobe After Effects 2025\Support Files\Plug-ins\DynamicFx\` (hash verified), same command and expectations.** Observed: 0 mismatches over all 48 quadrant probes, slot names matched each instance's own plan throughout, strict counters 0 in every leg; reopen `resolved from process registry` +2, `aerender` `rebuilt from snapshot` +2. Evidence: [`fix-FF1197D9-ae2025/`](audits/evidence/tr-bind-002/fix-FF1197D9-ae2025/report.txt).
- Cause (from the source, 2026-08-19): the session registry is keyed by the source fingerprint only (`session_token` → `registry()`), `registry_insert` **replaces** an entry for the same source, and the stored `CompiledEffect.definition.binding` is the per-instance plan from `binding::build_with_reuse`; `resolve_transported_definition` (render clones; a UI instance re-created by the copy/paste flatten/resetup round-trip) therefore adopts the other instance's slot table.
- Related: [ADR-0005](adr/0005-stable-parameter-ids.md), [ADR-0013](adr/0013-paramid-grammar-and-pools.md), [ADR-0016](adr/0016-sequence-schema-v1.md), [TR-M3-001](#tr-m3-001--persistence-and-render-clone); public issue [#6](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/6). **Fix: [ADR-0038](adr/0038-registry-key-per-binding-plan.md) Accepted 2026-08-21 (mechanism B — registry keyed by `(source, plan identity)` with session lineage aliases; the idle observer applies slot UI from the instance's own artifact; `StateToken`/schema unchanged), implemented in the working tree; fix-build host verification pending.**
- Field confirmation 2026-08-21 (AE 2026, 0.0.4): the user's `prism` sample carries two instances of one source; copy/paste, re-commit, param edits and `layer.duplicate()` were exercised by script. Both instances kept their **own** values throughout and 5 purged single-frame renders at each step were bit-identical — because both plans are **fresh declaration-order** (the corruption above needs a *migrated* plan). The registry still churned (`resolved from process registry` 67→72, `pipelines built` 71→76 as the instances alternated ownership), which is the **flicker** the user reports ("复制会导致原效果和复制效果闪烁"). Same root cause; no separate fix track. Evidence: [`docs/audits/evidence/field-20260821-prism-sample/`](audits/evidence/field-20260821-prism-sample/README.md) (Finding B).

### TR-0038-001 — M2/M3 batteries on the ADR-0038 fix build

- Status: `PASS` on **AE 2026 and AE 2025** — the complete M3 battery (three GUI sessions, session 4, `aerender`, numeric checks) and the complete M2 battery (scenarios a–d, e–j, numeric checks) green on the fix build with zero year-specific changes.
- Date: 2026-08-21, 20:07–20:12 local (AE 2026) and 20:13–20:19 local (AE 2025); Windows 11 Pro 10.0.26200.
- Artifact: `dynamicfx.dll` 8,564,736 B, SHA-256 `FF1197D9DB09CCE81D8E2CE26AC4FD3FA752FEA8705109FBCA8D9725670A0344`, installed on both years at `…\Adobe After Effects <year>\Support Files\Plug-ins\DynamicFx\DynamicFx.aex` (elevated copy with the installer's checks — AE closed, no MediaCore copy — because `install.bat`'s `net session` probe fails on this machine even elevated; installed hashes verified). Working tree `6386e87` + the uncommitted ADR-0038 implementation; `cargo test` 147 passed.
- Host: AE 2026 26.3x87 and AE 2025 25.6.6x4; RTX 5080 / Dx12.
- Commands: per year, `scripts/m3/run_m3.ps1 -Year <y> -Session4` (runs sessions 1–4), `-Aerender`, `-Checks`; `scripts/m2/run_m2.ps1 -Year <y>`, `-Scenarios e,f,g,h,h2,i,j,q`, `-Checks` — chained so that **each runner starts After Effects itself**: the runners hand their output folder to AE through `DFX_M3_OUT`/`DFX_M2_OUT`, which an AE started by hand never inherits; the first chain attempt on AE 2026 (20:00) had AE already open and timed out on `m3a` for exactly that reason (its `m3a.log` went to `scripts/out/m3/dev/`); the chain was restarted with AE closed and its log overwritten.
- Observed: every step exit 0 on both years; M3 probes `m3c_reopen`, `m3d_t04`, `m3f_recover`, `m3g_layer1`/`layer2`, `m3h1_torn`, `m3h4_undo`, `m3h3_invalid`, `aerender PSD` all `exit=0`; M2 probes `m2b` ×3, `m2d`, `m2e`, `m2g`, `m2h_kinds` ×5, `m2i_overflow` all `exit=0`; `m3h4 dirty_after_save=false` and `m3h5 dirty=false` — the new hidden `PlanToken` stream (ADR-0038 §7) does not dirty the project; `m2j` rejects the overflow with `E32` and token word 130; no timeouts.
- Evidence: [`battery-2026.log`](audits/evidence/tr-bind-002/battery-2026.log), [`battery-2025.log`](audits/evidence/tr-bind-002/battery-2025.log); raw per-suite outputs under `scripts/out/m2|m3/<year>/` (gitignored).
- Related: [TR-BIND-002](#tr-bind-002--copied-instance-corrupts-slot-mapping-field-defect), [TR-M2-001](#tr-m2-001--stable-paramid-and-bindingplan), [TR-M3-001](#tr-m3-001--persistence-and-render-clone), [TR-Y26-001](#tr-y26-001--ae-2026-full-suite-host-run).

### TR-BOUNDS-001 — Shader canvas is the layer frame (field observation)

- Status: **`PASS`** — contract-conformance measurement on the released 0.0.5 artifact on AE 2026: the runtime behaves exactly as [ADR-0011](adr/0011-shader-abi-v1-core.md) §5 specifies (the canvas is the layer's own frame), so the field-reported clipping is a contract limitation, not a defect. No code changed. Canvas expansion is recorded as an unscheduled, ADR-sized feature candidate in [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) (*Recorded, not scheduled*) and filed as public issue [#8](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/8) (enhancement, 2026-08-21).
- Date: 2026-08-21
- Baseline: installed 0.0.5 release artifact `DynamicFx.aex` (8,564,736 B) SHA-256 `FF1197D9DB09CCE81D8E2CE26AC4FD3FA752FEA8705109FBCA8D9725670A0344` at `C:\Program Files\Adobe\Adobe After Effects 2026\Support Files\Plug-ins\DynamicFx\` (hash re-verified equal to [TR-REL-005](#tr-rel-005--005-release-verification) this session); working tree on `main` at `8da89d8` (docs only touched). OS Windows 11 Pro 10.0.26200.
- Host/project: AE 2026 (26.3x87), the user's `AppleVison` project (8-bpc; not part of the repository). Field question: the `apple-thermal` shader (10 passes, 22 params — the shipped `examples/apple-thermal.glsl` defaults) renders fully on the 1024×1024 padded precomp `apple-logo (pad 1024)` but is clipped to a hard square on the 512×512 `apple-logo.png` footage layer — "plugin defect or shader defect?"
- Procedure (warm session via ae-mcp `ae_exec` + `ae_previewFrame`): new test comp `DFX 边界测试 (可删除)` 3072×1024; three fresh `DynamicFx` instances of the same source (expression copied from the tuned instance; all 94 copyable parameter values verified identical), one per 1024-wide tile: **A** the 512×512 footage + DynamicFx; **B** the same footage + `Red Giant GrowBounds` (Pixels 256, an upstream buffer-expanding effect) + DynamicFx; **C** the padded precomp + DynamicFx. `app.purge(ALL_CACHES)` → `comp.saveFrameToPng` at t = 2.0 s → `analyze.py` (alpha bbox per tile, pixels outside the central 512×512 logo square, tile diffs).
- Expected (ADR-0011 §5; `src/lib.rs` `Command::SmartPreRender` checks out `0,0,width,height` of the layer and declares `max_result_rect` = that frame; the PiPL sets no `PF_OutFlag_I_EXPAND_BUFFER`): A and B confined to the 512×512 frame, C free to use its 1024×1024 frame.
- Observed: **A** visible bbox exactly 512×512 (x256..768, y256..768), 0 px outside the square. **B** pixel-identical to A (A–C and B–C diffs coincide: mean |diff| 13.80, 437,288 px > 8) — an upstream buffer expansion does **not** enlarge the canvas. **C** bbox 766×855 (x119..885, y92..947), 253,110 visible px (49.1 %) outside the square; maximum reach 179 px beyond the logo, so the 256 px pad suffices. Inside the logo square A differs from C by mean |diff| 20.07/255 (the blur/halo passes sample the margin instead of clamp-to-edge pixels). Adobe's own `ADBE Grow Bounds` is absent from this AE 2026 install (1,536 effects enumerated; only the Red Giant utility matches), hence the stand-in; the mechanism is the same.
- Reading: not a shader bug (nothing in GLSL can address pixels outside `v_uv` ∈ [0,1]²) and not a runtime bug (the canvas is the layer frame by Accepted contract). A runtime feature would need an ADR covering an explicit stable expansion parameter (ADR-0013 topology), the `u_resolution`/`v_uv` canvas definition and `input` placement (ADR-0011 §5, ADR-0029), ADR-0030 §4 comp-space alignment, temporal extents (ADR-0023/0024), extents in the identity keys (ADR-0007/0017) and the SmartFX `max_result_rect`/`EXPAND_BUFFER` protocol (TR-M7-004). A request-derived canvas was rejected on sight — the ROI request varies per render and a canvas that moves with it breaks WYSIWYG.
- Workaround (documented in the `dynamicfx-shaders` skill): precompose the source with transparent margin ≥ the shader's maximum pixel reach and apply DynamicFx to the precomp; expose reach as pixel `@param`s so the margin is explicit.
- Evidence: [`docs/audits/evidence/field-20260821-layer-bounds/`](audits/evidence/field-20260821-layer-bounds/README.md) — `abc-t2.0-a-clipped-b-growbounds-c-padded-precomp.png`, `analysis-report.txt`, `analyze.py`.
- Related: [ADR-0011](adr/0011-shader-abi-v1-core.md) §5, [ADR-0029](adr/0029-logical-resolution-abi.md), [ADR-0030](adr/0030-layer-input-parameters.md) §4, [TR-M7-004](#tr-m7-004--roi-final-pass-delivery).

### TR-GRP-001 — Parameter stream matching across layout change (spike)

- Status: **PASS** — the spike ran to a definite verdict: **`ID_MATCH`**. After Effects re-attaches a saved project's effect parameter streams by `PF_ParamDef.uu.id` (via the id-derived stream matchName), **not** by declaration index.
- Date: 2026-08-26
- Baseline pair: released 0.0.5 artifact `DynamicFx.aex` (8,564,736 B) SHA-256 `FF1197D9DB09CCE81D8E2CE26AC4FD3FA752FEA8705109FBCA8D9725670A0344` (authored `baseline.aep`), then the spike build `5C6C3B4D9D81A13725BAA15C54920A22D8DCBD7998850C95147C1A25255CCA06` (8,574,976 B) — repo commit `8da89d8` + `spike.patch` (a "Floats" topic wrapped around the Float pool: GROUP_START at released position 5, GROUP_END after Float 48; floats +1, everything later +2; all pre-existing `ParamKey` Debug strings, hence ids, unchanged), toolchain 1.97.1, carried `Cargo.lock`, worktree `AePlugin_Dynamicfx-spike-grp`. Swap by elevated copy with hashes logged; 0.0.5 restored from the hash-verified backup afterward. OS Windows 11 Pro 10.0.26200.
- Host: AE 2025 (25.6.6x4), launched per phase; AE 2026 untouched. Driver [`scripts/grp/tr_grp_001.py`](../scripts/grp/tr_grp_001.py) over the ae-mcp panel `/exec` channel.
- Command: `tr_grp_001.py phase1` (0.0.5: probe project + dump) → `quit` → swap → `phase2` (spike build: reopen + dump) → `verdict` → `quit` → restore.
- Expected: discriminate `ID_MATCH` (13 named probes keep values/keyframes/expression despite the shift) from `INDEX_MATCH`/reset (probes land on neighbours or defaults).
- Observed: **13/13 probes KEPT** — every parameter kind (float, int, bool, color, point, angle, layer, point 3D, gradient-stop rows), the 2-keyframe stream (1.25@0s / 2.5@1s exact) and the expression (`'0.123'`) all followed their **unchanged matchNames** (`DynamicFx-<uu.id>` in decimal, e.g. `Float 01` = `DynamicFx-447494404`) while flat property indexes shifted +1/+2 exactly as declared. Secondary: scripting/AEGP see the topic **flat** — the group start is an inert `NO_VALUE` top-level row (`"Floats"`, `DynamicFx--733460086`), floats remain top-level siblings, so flat stream-index arithmetic survives grouping.
- Consequence: grouping released parameters does **not** break saved projects, provided every shipped `ParamKey` Debug string (the id source) is frozen. The user-approved one-time-break fallback is unnecessary. Decision to adopt id-stable identity: [ADR-0040](adr/0040-parameter-groups-and-id-identity.md) (Proposed).
- Limits: measured on AE 2025 only in this spike; the ADR-0040 verification obligations require the grouped production build to repeat the reopen check on AE 2025 **and** 2026 before release. AE 2023/2024 remain outside the measured set (hosts unavailable; release gate unchanged).
- Evidence: [`docs/audits/evidence/spike-20260826-param-group-matching/`](audits/evidence/spike-20260826-param-group-matching/README.md) — `baseline.aep`, `baseline.json`, `after_spike.json`, `verdict.txt`, `spike.patch`.
- Related: [ADR-0013](adr/0013-paramid-grammar-and-pools.md) §5, [ADR-0028](adr/0028-details-button-and-slider-precision.md), [ADR-0040](adr/0040-parameter-groups-and-id-identity.md) (Proposed), [ADR-0039](adr/0039-canvas-expansion.md) (Proposed, unrelated mechanism but same session).

### TR-0039-001 — Canvas expansion host legs

- Status: **PASS** on AE 2025 and AE 2026 — all five ADR-0039 legs, with the two equivalence legs **bit-identical** to their padded-precomp references (mean abs diff 0.000, peak 0), exceeding the "comparable" acceptance bar.
- Date: 2026-08-26
- Baseline: batch build `DynamicFx.aex` 8,589,824 B SHA-256 `8A472BFE3D51C7418AFF227137B929F656AC0CA8FDBBEB72F5E24207533925E0` (working tree `main`@`8da89d8` + accepted slices 1–3, uncommitted; unit evidence `cargo test` 166 passed, zero warnings), installed by user-elevated copy with 0.0.5 backed up and restored+re-verified (`FF1197D9…`) after the pass. OS Windows 11 Pro 10.0.26200; AE 2025 (25.6.6x4), AE 2026 (26.3x87).
- Command: `python scripts/canvas/tr_0039.py all` (AE 2025) / `… --out docs/audits/evidence/hostpass-20260826-006/canvas-ae2026` (AE 2026), ae-mcp panel `/exec`, `app.purge(ALL_CACHES)` before every render read.
- Expected/observed: **L1** undeclared+plain = released clipping (bbox exactly the layer rect); **L2** undeclared + Red Giant GrowBounds 256 ≡ 256-padded precomp (0.000/0 — TR-BOUNDS-001's no-op tile B becomes the positive test); **L3** declared `hint:canvas` 160 ≡ its padded reference (0.000/0); **L4** declared 64 under GrowBounds 256 stays bound to 64 (reach 58, crop visible); **L5** keyframed reach 0→200: t=0 exactly clipped, t=1 reach 180. Identical numbers on both years.
- Evidence: [`docs/audits/evidence/hostpass-20260826-006/`](audits/evidence/hostpass-20260826-006/README.md) — `canvas-ae2025/`, `canvas-ae2026/` (reports + PNGs), harness [`scripts/canvas/tr_0039.py`](../scripts/canvas/tr_0039.py), shader `examples/reach-ring.glsl`.
- Related: [ADR-0039](adr/0039-canvas-expansion.md) (Accepted), [TR-BOUNDS-001](#tr-bounds-001--shader-canvas-is-the-layer-frame-field-observation) (closed by this feature; issue #8), [TR-0040-001](#tr-0040-001--grouped-topology-host-legs) (same artifact/pass).

### TR-0040-001 — Grouped topology host legs

- Status: **PASS** on AE 2025 and AE 2026 — live index map verified, 0.0.5 project reopen 13/13 on both years, per-pass allocator demonstrated, M2 (12/12) + M3 (4/4 + aerender) batteries green per year on the re-pinned harnesses, scale measured.
- Date: 2026-08-26
- Baseline: the TR-0039-001 artifact/environment (one shared pass, one install/restore cycle).
- Commands: live index dump; `scripts/grp/repin_indexes.py apply` (26 harness sites); reopen legs against the spike `baseline.aep` (verdicts in evidence); `apple-thermal` compile/render + a 2-pass allocator demo; `run_m2.ps1`/`run_m3.ps1` per year (runner-started AE per the recorded discipline); `phase_f` measurement probes.
- Observed: 424 top-level rows exactly per the slice-3 table (`Details`@6, `Main`@8 flat, `Float 01`@9, `P01 Float 01`@185); reopen **13/13 KEPT** per year (keyframes and expression exact); `apple-thermal` "compiled: 10 passes, 22", all params correctly `Main` (its passes share one uniform block), renders on both years; 2-pass demo splits `shared→Main@9, warm→Pass 1@185, cool→Pass 2@205` with labels; batteries all green, zero TIMEOUT/FAIL in accepted runs; measurements — `addProperty` 5→9/8 ms, exec-RTT delta in noise, one-instance `.aep` 170,213→287,737/294,311 B (**≈+117 KB per instance**, the accepted pass-bank price). Empty pass-group headers remain visible on single-pass shaders (the accepted ADR-0040 §6 fallback; hiding is a recorded follow-up).
- Incidents recorded in the evidence README (spike-file overwrite → driver overwrite-guard added; a killed runner → AE crash-recovery dialog → one discarded M2 attempt; an AE 2026 crash while wedged in a quit — no evidence affected).
- Evidence: [`docs/audits/evidence/hostpass-20260826-006/`](audits/evidence/hostpass-20260826-006/README.md) — reopen dumps/verdicts, compile dumps/PNGs, allocator demos, measurements, battery outputs under `scripts/out/m2|m3/<year>/`.
- Related: [ADR-0040](adr/0040-parameter-groups-and-id-identity.md) (Accepted), [TR-GRP-001](#tr-grp-001--parameter-stream-matching-across-layout-change-spike) (the mechanism spike), [TR-0039-001](#tr-0039-001--canvas-expansion-host-legs).

### TR-0041-001 — Panel polish and the final 0.0.6 artifact re-verification

- Status: **PASS** on AE 2025 and AE 2026 — the complete batch leg set repeated on the release-gating artifact, plus the three ADR-0041 behaviors verified live.
- Date: 2026-08-26
- Baseline: **final artifact** `DynamicFx.aex` 8,613,888 B SHA-256 `9E438A6444394EA8…` (full hash in the evidence README/swap log), built from the working tree carrying slices 1–4; `cargo test` **170 passed**, release build warning-free (Fable's own runs). Superseded intermediates recorded in the evidence README's round structure (`8A472BFE…` R1, `54B5F0AF…` R2, `3E732238…` diagnostic, never release-gating). Installed/restored by the user-elevated procedure with full hash logging; 0.0.5 `FF1197D9…` restored and re-verified after the pass.
- Observed (per year unless noted): index map — `Setup`@1 (expanded), heads shifted +1, `Details`@7, `Main`@10, `Float 01`@11, first bank slot@187, 426 total; ADR-0041 items — `Setup` group present, **empty pass groups hidden** (panel photograph: a 2-pass shader shows only `Setup`/`Main`/`warm`/`cool`), **pass groups renamed to envelope names** (`warm`/`cool`; apple-thermal's `th`/`tv` observed); reopen of the 0.0.5 `baseline.aep` **13/13** both years; canvas legs all five, equivalences bit-identical (0.000/0) both years; M2 **12/12** and M3 **4/4 + aerender exit 0** both years on name-based harnesses; measurements — `addProperty` median 8–9 ms (0.0.5 baseline 5 ms), RTT deltas within noise, one-instance `.aep` 289,637/296,211 B (0.0.5: 170,213 B).
- Known/accepted: `Setup`'s initial collapse state showed collapsed in one instance despite the expanded declaration (cosmetic, user-adjustable, recorded as an observation); AE 2026 raises a version-conversion modal for 2025-saved projects (host behavior, harness accommodates); the m2h2/m2b/m2d harness scenarios now address bound slots by label (two stale-literal failures during R2/R3 were harness faults, not plugin faults — the diagnostic detour is in the evidence README).
- Evidence: [`docs/audits/evidence/hostpass-20260826-006/`](audits/evidence/hostpass-20260826-006/README.md) round-structure note and `*-r2-*`/`*-r3-*` files; battery outputs under `scripts/out/m2|m3/<year>/`.
- Related: [ADR-0041](adr/0041-panel-polish.md) (Proposed → acceptance requested with this evidence), [ADR-0039](adr/0039-canvas-expansion.md), [ADR-0040](adr/0040-parameter-groups-and-id-identity.md), [TR-0039-001](#tr-0039-001--canvas-expansion-host-legs), [TR-0040-001](#tr-0040-001--grouped-topology-host-legs) (R1 records; their leg sets were repeated in R3 on the final artifact).

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

### TR-CUI-001 — Custom-UI crash bisection (spike)

- Evidence availability note (2026-09-08): the `scripts/out/cui/` paths below identify gitignored local artifacts from the original Windows runs; they are absent from this clone and are written as literal paths, not repository links. Historical verdicts are retained as recorded and have not been re-verified from those artifacts in this session.
- Status (AE 2025 25.6.6x4, 2026-08-28): **U0 `PASS`; U1 `PASS`; U2 with `register_ui` `PASS`; U2 without `register_ui` `FAIL` — After Effects dies, reproduced 3/3 on a byte-identical artifact.** The 2026-08-15 crash class is reproduced and pinned to one missing call (Round 3 below). **Root cause identified at code level (Round 4):** a **null-pointer access violation inside After Effects' own Effect Controls paint path** (`AfterFXLib!CEffects::UpdateInvalidParams`, `mov rcx,[rbx]` with `rbx=0`), escalated by Windows to **`C000041D STATUS_FATAL_USER_CALLBACK_EXCEPTION`** because the paint runs inside a kernel-dispatched window-procedure callback. **The dump harness is `FAIL` for exactly that reason:** WER `LocalDumps` captured **nothing** in any of the three deaths and Windows logged no `Application Error`/`Application Hang` event, so the "crash → dump" branch of the verdict table below is **not reachable through WER**; the fault was captured instead with `procdump` attached in first-chance mode. AE 2026: `NOT_RUN`. This row records the crash-class analysis and the experiment matrix for the deferred custom-controls track (the [ADR-0031](adr/0031-gradient-parameters.md) §7 editor, removed 2026-08-16 after crashing AE 2025 on expand — see [TR-0031-001](#tr-0031-001--gradient-parameters) and [ADR-0033](adr/0033-gradient-stops-are-ordinary-parameters.md) Decision 6/Outcome).
- Date: 2026-08-28 (design + instrument build; no host contact)
- **What the 2026-08 record already establishes.** The crash reproduced with the reference effect's declaration byte for byte (`param_type = 11`, `ui_flags = 0x82`, 200x80) with zero plugin log lines and no custom-UI event delivered, so the declaration is exonerated; the fault lives in the callback/event protocol between AE and the `after-effects` 0.4 crate, or in AE's own handling of what that protocol returned. The editor implementation itself was never committed (working tree only, deleted the next day); TR-0031-001's prose and ADR-0033's Outcome are the surviving record of it.
- **New finding 1 — the crash was a hard fault, not a Rust panic.** `build.rs` has emitted the crate's `catch_panics` cfg since 2026-08-04/05 (`6956cf6`/`fde0c8a`, pre-dating all editor work), and with it every panic inside `EffectMain` — including the bare `assert!`s in the crate's arbitrary-data dispatch and the `unreachable!()` on unknown event types — is caught at the FFI boundary and surfaced as an error message with AE alive. The observed instant, silent process death is therefore an access violation (or equivalent), which log-line bisection could never localize. **The missing instrument in every 2026-08-15/16 cycle was a crash dump** (faulting module + stack), not more logging. (Boundary: the AEGP idle-hook entry is not under that wrapper, but the crash path — expand → arbitrary-callback/event — is EffectMain's.)
- **New finding 2 — arbitrary data without custom UI is already proven on this host stack.** [TR-M0-004](#tr-m0-004--sequence-transport-payload-capacity) ran `spike/probe`'s `PF_Param_ARBITRARY_DATA` parameter through the same crate 0.4 with a `Vec<u8>`-carrying value type (heap pointers inside AE handles) and the full seven-callback set to 16 MB flatten/unflatten on AE 2025 without instability. The crash therefore requires the custom-UI dimension — `PF_OutFlag_CUSTOM_UI` + `PF_PUI_CONTROL` + the `PF_Cmd_EVENT` pipeline — alone or in combination with arbitrary data; neither the arb protocol nor heap-pointers-in-handles is an independent cause.
- **New finding 3 — upstream state (reviewed 2026-08-28; fork divergence measured by git, superseding an initial wrong web-surface read the same day).** crates.io `after-effects` is still 0.4.0 (2026-04-23), the pinned version; the upstream repo's post-0.4.0 log (through July 2026) shows handle-layer cleanup (their issue #6: `host_[un]lock_handle` no-ops removed, merged as their PR #112) and platform `PF_Handle` cast fixes, but no custom-UI or arbitrary-data crash fix — "upgrade and hope" is not available. **The recorded `exaecut-after-effects` fork-diff review is hereby answered with data** (clone at `E:\Code\_refs\after-effects`, remotes `origin` = virtualritz, `exaecut`; `git rev-list --left-right --count origin/master...exaecut/master` → **32 / 17**): virtualritz remains the active mainline (32 commits ahead, including the July 2026 handle work); the Exaecut fork's 17 own commits are Premiere-side additions (VideoSegmentRender suite, `prRect` field-width fix, VideoSequenceParser, `guid_mix_in_ptr`), the crates-rename/publish CI, and one generic robustness fix ("forward unknown host commands without panicking", `4d93141`) — **nothing custom-UI-relevant is ahead of mainline there**; its crates.io releases are a convenience snapshot (last upstream sync 2026-06-03). Upstream ships examples for exactly the two shapes at issue, unexercised on this machine: `custom_ecw_ui` (ECW custom UI over a **standard** Color parameter — arbitrary data not involved on AE hosts) and `color_grid`/`histo_grid` (arbitrary data + custom UI, value types carrying `String`).
- **Design consequence, independent of the verdict:** since ADR-0033 the gradient value lives entirely in ordinary parameters, a revived editor needs only a drawing surface plus events — the standard-parameter custom-UI shape, which never touches the arbitrary-data protocol. If leg U1 below passes, the editor track can proceed on that shape regardless of the arb-side verdict, and ADR-0033's Outcome already permits reviving the editor as pure presentation (no new value ADR; the global `CustomUI` out-flag and per-param UI flags are not persistent project state).
- Instrument: extend `spike/probe` (never-shipped diagnostic effect, own match name, `catch_panics` already on) with the UI legs, param sets selected per artifact at build time (env-var → `rustc-cfg`, since the PiPL out-flags live in `build.rs`); the `CustomUI` out-flag rides every leg artifact (a control-flagged param without it is rejected at PARAMS_SETUP — TR-0031-001 first defect). Every host leg runs under a crash-dump harness (`procdump -e -ma` on `AfterFX.exe`, or WER LocalDumps) so any death yields faulting module + stack; the probe's `%TEMP%\dynamicfx_probe.log` remains the command/event log.
- Legs (AE 2025 first; AE 2026 only after a verdict):
  - **U0** — build upstream's `custom_ecw_ui` and `color_grid` examples from **current master** (the state any fix/PR would land on; user direction 2026-08-28 — do not pin research artifacts to the stale 0.4.0 snapshot), install, apply, expose the rows. Outcome either way is decisive: an existence proof that each shape works on this host, or a clean upstream repro with a dump. A 0.4.0-era build is only added later if a verdict specifically needs the pinned-dependency vintage reproduced.
  - **U1** — probe leg: standard Float parameter + `ParamUIFlags::CONTROL` + ui width/height, Draw handler fills a rectangle via Drawbot, every event logged. No arbitrary data involved.
  - **U2** — probe leg: the existing `ProbeBlob` arb parameter gains `CONTROL | DONT_ERASE_CONTROL` + ui size (the 2026-08-15 shape), same Draw handler. (This alters the M0-era probe param presentation; TR-M0-004's record pins the artifact that ran then, so no evidence is invalidated.)
  - **U2a** — U2 with the Draw handler muted to log-and-return (no Drawbot acquisition), toggled at runtime: separates "dies before/without any event" from "dies in the draw path".
  - **U2b** — U2 with a POD value type (`[f32; 8]`-class, no heap pointers): separates value-representation sensitivity from the UI flags.
- Verdict table: **U1 fails** → the event pipeline itself is broken on Windows in the crate — fork + fix + PR upstream (the queued plan's fork condition is met); editor blocked until fixed. **U1 passes, U2 fails** → arb×UI interaction — the editor proceeds on the U1 shape (standard-param canvas), the arb fix becomes optional upstream work. **All pass** → the 2026-08-15 crash needed something the minimal shape lacks; bisect from the working probe toward the removed editor's specifics (46 px control, color-picker call, drag tracking, `%TEMP%` logging switch) with a dump at every step.
- **Instrument record — probe legs (built 2026-08-28).** Implemented by `gpt-5.6-sol` from a self-contained brief and accepted after Fable's line-by-line diff review; every acceptance check re-run by Fable directly: all five `DFXP_LEGS` combinations (unset, `u1`, `u2`, `u2b`, `u1,u2,u2b`) build zero-warning via [`spike/probe/build_legs.ps1`](../spike/probe/build_legs.ps1); the PiPL repair was ported from the root `build.rs` mechanism verbatim (winres overwrites pipl's `resource.rc`, one PiPL reaches the linker); Fable's own `eGLO` decode of the artifacts reads base `0x06000440` (byte-identical to the pre-change probe) and every legged build `0x06008440` (+`CustomUI` bit 15, the code-page-vulnerable `0x84` byte intact). Base topology is unchanged with `MuteDraw` appended at the tail (the declared exception). Artifacts (`spike/probe/out/`, gitignored; hashes are their identity): base `29C48AC9…0E92`, u1 `B802112C…BA8E`, u2 `1147FA7D…178F`, u2b `CE5C9760…6E14`, u1-u2-u2b `53D02439…526B`.
- **Instrument record + measured finding — U0 upstream examples (2026-08-28).** Built from upstream master `9fb3fc1` (clone `E:\Code\_refs\after-effects`). **As shipped, both Windows example DLLs are dead on arrival: their PiPL `eGLO` reads `40 3f 00 02` (LE `0x02003F40`) — the `0x3F` is a literal `?`, the same pipl 0.1.1 RC code-page corruption TR-0031-001 diagnosed, sitting exactly where the `CustomUI`-carrying byte belongs — so After Effects would refuse them with the global out-flags mismatch and upstream's Windows custom-UI examples cannot have been loading as built.** This independently raises the prior that the crate's Windows custom-UI path is under-tested, and is an upstream PR candidate on its own (the `repair_pipl_resource` file-reference fix), regardless of the crash verdict. After applying that repair locally in the scratch clone (both example `build.rs` files + `winres` build-dep), both DLLs read `40 84 00 02` (LE `0x02008440`), matching their four declared flags exactly. Runnable U0 artifacts: `custom_ecw_ui.dll` `8D0BA260…1A767`, `colorgrid.dll` `94EEC66D…2E48` (in the clone's `target/release/`).
- Host-run prerequisites (satisfied 2026-08-28): elevated setup `scripts/out/cui/cui_setup_elevated.ps1` — Modify ACL on the probe/samples plugin folders + WER `LocalDumps` for `AfterFX.exe` → `scripts/out/cui/dumps` (minidump, count 10); driver [`scripts/cui/tr_cui_001.py`](../scripts/cui/tr_cui_001.py) (install/samples/start/leg/quit/forensics/status over the ae-mcp panel).
- **Round 1 — AE 2025 25.6.6x4, 2026-08-28 (partial; two hard results, crash still not localized).** Probe variants `base` (`29C48AC9…`) and `u1` (`B802112C…`) installed and exercised; leg driver applies the probe to a fresh solid and lets the Effect Controls panel render.
  - **Result A — U1 does NOT crash AE.** With the `u1` artifact (standard Float parameter carrying `ParamUIFlags::CONTROL` + 200×80 canvas + a Drawbot `paint_rect` Draw handler), the effect was created (`SEQ_SETUP` logged), the panel rendered the `U1 Std Canvas` row, and **AE stayed alive with zero minidumps** through the whole leg until an unrelated (non-crash) shutdown. This is the standard-parameter custom-UI shape ADR-0033 leaves open for a revived gradient editor — its host stability is now evidenced on one year, **short of the Draw-path confirmation** (see the gap below).
  - **Result B — a bare arbitrary-data parameter (no custom UI) raises a modal on this host.** Both variants carry the M0-era `Blob` parameter (`PF_Param_ARBITRARY_DATA`, `ParamUIFlags::empty()`); when the Effect Controls panel renders it, localized AE 2025 pops a modal **"效果控件不受支持！" ("effect control not supported")** — not a crash (AE survives; clicking through it restores the panel), but it blocks the main thread and times out the JSX bridge. This is a real, reproducible finding (an un-drawn arbitrary parameter is not a valid ECW control here) and it is **instrument noise** that must be removed before the crash legs can run clean.
  - **The gap that blocks the verdict.** The probe log carried **no `EVT` line at all** across the whole round: the Draw event never reached the plug-in because the custom-UI row sat **collapsed**, and AE sends no Draw to a collapsed control. The 2026-08-15 crash fired precisely on **expanding** the row; reproducing it needs that expansion with the Draw path actually invoked. Round 1 could not get there — the modal-B interruptions plus a fragile manual expand (a mis-click raised a Save-copy dialog and the session ended) prevented a clean expand+Draw observation.
  - **Result C — THE DECISIVE ONE: the custom-UI + Drawbot pipeline is fully healthy on this host, for BOTH parameter shapes.** Same session, U0 legs run after purging the collapse/modal problem by choosing controls that paint on load:
    - **`Custom_ECW_UI`** (standard `Color` parameter + `ParamUIFlags::CONTROL` + Drawbot **image and text**): applied clean (`props:2`, JSX **did not** time out — no "unsupported" modal), and the Effect Controls panel **drew the sample's crab image and instructional text** in its canvas. AE alive, **0 dumps**. This is the exact shape a revived gradient editor would take (ADR-0033 puts the value in ordinary parameters), demonstrated working end-to-end by upstream's own sample.
    - **`ColorGrid`** (arbitrary-data parameter + custom UI + Drawbot **rects**): applied clean (`props:2`, no modal), the panel **drew the full 3×3 colour grid** and the comp viewport rendered it. AE alive, **0 dumps**.
    - Both paint **on load with no expand**, so the Draw event demonstrably reaches the plug-in and the Drawbot acquire→`surface()`→`paint_rect`/`draw_image` chain completes on AE 2025 25.6.6x4. Screenshots captured in-session.
  - **What Result C settles.** The arbitrary-data + custom-UI + Drawbot combination — the very shape the 2026-08-15 gradient editor used — **works on this exact host through crate 0.4's API**. So the 2026-08-15 hard crash was **not** caused by the API shape, the crate, or the host; it was a defect in that specific deleted implementation (candidates unchanged: the 46 px control area, the `PF_AppColorPickerDialog` call, drag tracking via `set_send_drag`, the `%TEMP%` level switch, or the assumed `screen_point`↔`current_frame` coordinate space). This is the TR-CUI-001 verdict table's **"all pass"** branch: bisect from a working control toward the removed editor's specifics. **And the editor-revival path is now backed by host evidence** — Result A (our own U1 standard-param custom UI loads and does not crash) plus Result C's ECW (the same shape drawing image+text+click upstream) confirm ADR-0033's standard-parameter canvas is viable on-host with **no new value ADR** required.
  - **Result B refined by contrast.** ColorGrid's arbitrary parameter carries a **drawn** custom UI and raised **no** modal, where the probe's `Blob` (arbitrary, `ParamUIFlags::empty()`, no draw) did — so the "效果控件不受支持" modal is specifically an **un-drawn** arbitrary parameter, not arbitrary data as such. Independently, this means the probe's `u2` leg (Blob **+ CONTROL**) should also be modal-free; confirming that on-host is the one remaining probe-side check, now low-priority since U0 already answers the core question.
  - **Round-1 verdict:** the core research question is answered — custom UI is viable on-host in both shapes and the old crash is implementation-specific. Optional follow-ups (not blocking): the probe `u1/u2/u2b` fine-grained legs (our own event/PiPL-repair code on-host; `u1` already loaded clean, proving the PiPL `CustomUI` out-flag repair works on-host), and an upstream PR for the pipl code-page fix (both example DLLs ship dead-on-Windows without it — new finding above). Evidence dir: `scripts/out/cui/` (`legs.log`; `dumps/` empty = zero crashes all round).
- **Round 2 — `PF_REGISTER_UI` isolated as the custom-UI event master switch (AE 2025 25.6.6x4, 2026-08-28 17:08–17:40).** Three results, recorded from the session's on-disk handoff (`scripts/out/cui/HANDOFF.md`) plus `legs.log` and the probe log; the chat transcript of that session was summarised three times and is not evidence.
  - **Finding 1 — `PF_REGISTER_UI` is the on/off switch for custom-UI events.** The **same byte-identical artifact** was run with the registration call taken only by an environment variable (`DFX_PROBE_NO_REGUI=1` skips it), giving a clean A/B:

    | leg | `register_ui` | `EVT` lines | `DRAW painted` | AE | dumps |
    |---|---|---|---|---|---|
    | u1 (standard Color param + `CONTROL`) | called | 58 | 20 | alive | 0 |
    | u2 (arb param + `CONTROL\|DONT_ERASE`) | called | 3 | 1 | alive | 0 |
    | u2 (**same artifact**) | **skipped** | **0** | **0** | *— see the correction in Round 3* | 0 |

    Without the call the parameters still declare, `SEQ_SETUP`/`RENDER_READ` still run, and there is no error or warning — **Draw is simply never delivered**. Probe log evidence: `REGISTER_UI res=Ok(())` / `REGISTER_UI skipped (DFX_PROBE_NO_REGUI)` (pids 48428 / 4112 / 24128). Implementation: `spike/probe/src/lib.rs`, end of `params_setup()`, `#[cfg(cui_legs)]` block.
  - **Finding 2 — this project has never called `register_ui`.** A whole-repo scan (code + every ADR + scripts) puts **every** hit inside the vendored upstream copy under `.setup/after-effects-ref/`; all three upstream custom-UI samples call it, DynamicFX never has, and [ADR-0031](adr/0031-gradient-parameters.md) §7's editor design never mentions it. The 2026-08-15 crash site was recorded as "zero plugin log lines, no custom-UI event delivered" — word for word the signature of the third row above. **Inference boundary (important):** `gradient_ui.rs` never entered git (no commit, stash, or dangling object in the archived `AePlugin_Dynamicfx_dev_archive` holds it), so "the 2026-08-15 editor omitted `register_ui`" is **reconstructed from surrounding records, not read from the source**. Corroboration: that record says the reproduction copied the reference effect's *declaration* byte for byte (`param_type=11`, `ui_flags=0x82`, 200x80) — the declaration was copied, the `params_setup` registration call was not.
  - **Finding 3 — `screen_point` and `current_frame` share one coordinate space (measured).** On the u1 leg with canvas `frame=(52,189,332,269)`: a click at the canvas centre reported `sp=(190,230)` against a predicted (192,229); a click at 13.8 % from the left reported `sp=(90,214)` against a predicted (90.7,214) — **error ≤ 2 px, same space**. Separately the reported frame width is **280**, not the declared **200**: AE stretches the control to the panel column width, so **an editor must paint from `current_frame`, never from its declared size** (upstream's samples do the same). The u1 leg also passed Draw / Click / the drag stream (`set_send_drag` effective, `last=true` terminator, automatic redraw after the drag) / AdjustCursor.

- **Round 3 — the unregistered control area kills After Effects, and WER cannot see it (AE 2025 25.6.6x4, 2026-08-28 18:26–18:58).** The queued single action was "in the no-register arm, put real interaction on the arb control area (expand/collapse the row, click the control, drag the panel width to force a repaint) and see whether AE survives or produces a dump." **The gestures turned out to be unreachable: AE dies before the control area is ever interactive.**
  - Baseline: working tree on `main` at `97c6700`, with `spike/probe/{Cargo.toml,build.rs,src/lib.rs}` modified and `spike/probe/{.gitignore,build_legs.ps1}` untracked (the probe instrument is deliberately uncommitted). OS Windows 11 Pro 10.0.26200. Host After Effects 2025 **25.6.6x4** (`app.version`). Artifact `DynamicFxProbe.aex` SHA-256 `E55A3DB3C086B63F361D9C28386FB9DB38E201B28651A7CA33189516D14A88FF` = `spike/probe/out/DynamicFxProbe-u2.aex`, installed at `C:\Program Files\Adobe\Adobe After Effects 2025\Support Files\Plug-ins\DynamicFxProbe\DynamicFxProbe.aex`. WER `LocalDumps` for `AfterFX.exe` armed throughout (`DumpFolder` = `scripts/out/cui/dumps`, `DumpType` 1, `DumpCount` 10 — re-verified in the registry at 18:29).
  - Procedure per run: `DFX_PROBE_NO_REGUI=1 python scripts/cui/tr_cui_001.py start` (the variable must be set on the launching process — AE inherits it), dismiss AE's 「崩溃修复选项」 modal with **继续** (never safe mode), then `python scripts/cui/tr_cui_001.py leg probe`, then observe. Arm confirmed per process in the probe log (`REGISTER_UI skipped (DFX_PROBE_NO_REGUI)`).
  - **Result D — AE dies in the no-register arm; 3/3.** Deaths at pids 34120, 52704 and 12656. Sequence, identical every time: the effect is applied (`props:7`, `REGISTER_UI skipped`, **zero `EVT`/`DRAW` lines ever**) → within roughly 8 s AE raises a native 「保存副本」 ("Save a Copy") file browser, default name `“无标题项目”复制.aep`, which blocks the main thread and times out the JSX bridge → dismissing it (**取消**) terminates the process. **The Effect Controls panel never draws a single effect row**, so no expand/collapse, no click on the control, and no panel-width drag was ever possible — the queued gesture set is moot.
  - **Result E — no gesture is required; applying the effect is sufficient.** Isolated cleanly: a healthy session ran the driver's leg **with the single `addProperty('DynamicFxProbe')` line removed** (same new project, same save, same comp, same solid, same viewer, same selection) and stayed **alive and dialog-free for 105 s** with zero plugin log lines; applying **only** the effect into that same living session produced the 「保存副本」 dialog within 8 s, and dismissing it killed AE. So the trigger is the effect's arrival in the Effect Controls panel, not any user interaction with it.
  - **Result F — the cause is the missing `register_ui` call, not the parameter shape.** The same artifact launched **without** `DFX_PROBE_NO_REGUI` (so `register_ui` **is** called) applied cleanly and logged `REGISTER_UI res=Ok(())`, `EVT NewContext`, `EVT Draw`, `DRAW begin idx=2`, `DRAW drawbot_acquired`, `DRAW painted`; the `Probe Blob` row **painted its blue canvas** in the panel and **AE was still alive and healthy 105 s later** (pid 20980). Byte-identical binary, one environment variable apart: **an arb parameter carrying `PF_PUI_CONTROL` while the plug-in never calls `PF_REGISTER_UI` is fatal to AE 2025; adding the call makes the identical parameter safe.**
  - **Result G — the dump instrument is blind to this crash class.** Across all three deaths: `scripts/out/cui/dumps/` stayed **empty**, and the Windows `Application` log recorded **no `Application Error` (1000) and no `Application Hang` (1002)** event for `AfterFX.exe` at any point in the session. No `WerFault.exe` was ever attached. **This was inferred here and is now MEASURED in Round 4, which confirms it:** the paint fault is escalated to a fatal user-callback exception and routed through `KERNELBASE!UnhandledExceptionFilter` into Adobe's own `dvacore!…PanicFromUnrecoverableSignalHandler`, which raises the 「保存副本」 prompt (`AfterFXLib!CPDir::DoSaveCopyAs`) as its rescue step before the process goes down — so WER never sees a normal unhandled exception. Consequences: (a) **WER `LocalDumps` is the wrong instrument for this project's AE crashes** and the 2026-08-15 "no dump, no log line" record is explained without needing any extra failure; (b) fault evidence must be taken **while the post-fault dialog is still open**. Done here: `procdump -ma` against the live process with the dialog up → `scripts/out/cui/faultdump/AfterFX-FAULT-dialog-up-12656.dmp` (5,695,673,153 B, 18:53). **Not yet analysed — no debugger is installed on this machine** (no `cdb`/`windbg`/Windows SDK Debuggers; only `dbghelp.dll` 10.0.26100.8328), so the faulting module and stack remain **unread**: the crash is localized to a *call-level* cause by A/B, not yet to a code address.
  - **Correction to Round 2's Finding 1 table.** Its third row read "AE alive" for the unregistered u2 leg. That reading came from the driver checking process existence about 3 s after apply (`cmd_leg` → `time.sleep(3)` → `ae_running()`), which is **true but misleading**: at that moment AE is already faulted and merely sitting in its 「保存副本」 dialog, so the process still exists. The unregistered arm does **not** survive. Corroborating field observation: that session's AE (pid 24128, unregistered arm, last plugin log line 17:34:15) was found **still running but with a wedged UI thread** at 18:26 — `Responding=False`, a DWM `Ghost` "(未响应)" window, panel `/health` OK but `/exec` timing out, main window still enabled and **no `#32770` dialog present**, 0 dumps and no event-log entry. Preserved before it was killed: `scripts/out/cui/hang/AfterFX-HANG-24128.dmp` (5,647,933,912 B, 18:33, `procdump -ma`). This is **consistent with** a post-fault state left unattended for ~52 min, but the exact wedge was **not attributed** (also unread, same missing-debugger limitation) — recorded as an observation, not as a second reproduction.
  - **Limits of this round, stated plainly.** The "AE’s own handler swallows the fault" mechanism is an **inference** from three consistent negatives (no dump, no event, no `WerFault`) plus the dialog's behaviour — it is not confirmed by a read stack. The no-dialog control ran **105 s**, which rules out a fast trigger but not a long-period timer. AE 2026 was not touched. The gesture matrix the round set out to run (expand/collapse, click, panel-width drag on an unregistered control area) is **`NOT_RUN` and unreachable by construction** on AE 2025.
  - Evidence: `scripts/out/cui/legs.log` (records `hang-observed`, `gesture-check` rows `baseline-after-start`, `cancel-save-a-copy-dialog-REPRO2`, `CONTROL-no-effect-105s-no-dialog`, `APPLY-ONLY-no-gesture-then-cancel-REPRO3`, `CONTROL-registered-arm-105s-ALIVE-canvas-painted`), `%TEMP%\dynamicfx_probe.log` (per-pid `REGISTER_UI` lines for 34120 / 52704 / 12656 / 20980), the two `procdump` artifacts above, and `scripts/out/cui/dumps/` — **empty, which is itself the Result G evidence**. Dump files are build output under the gitignored `scripts/out/`; the hashes/sizes and paths here are their identity.
  - **What this changes for the editor track.** Round 1's "all pass → bisect toward the removed editor's specifics" conclusion still holds for the *shapes* (U0/U1 remain `PASS`), but the bisection list gains a candidate that outranks the rest and is **cheap to check first**: the 2026-08-15 editor's missing `PF_REGISTER_UI` call, which alone reproduces a fatal, dumpless death with the same signature the crash record describes. A revived editor **must** call `register_ui` in `params_setup` for every build that declares `PF_PUI_CONTROL`; a regression test at that boundary is worth more than more logging.

- **Round 4 — root cause read from a debugger (AE 2025 25.6.6x4, 2026-08-28 20:15–20:35). This closes TR-CUI-001's core question.**
  - **Instrument added.** Windows SDK **10.0.26100.7705** Debugging Tools installed (`cdb.exe` at `C:\Program Files (x86)\Windows Kits\10\Debuggers\x64\`). Note for reproduction: `winget install --id Microsoft.WindowsSDK.10.0.26100 --override "/features …"` **fails with exit 1000** — winget passes its own `-burn.*` arguments after the override and `winsdksetup.exe` rejects the mixed command line (`ERROR: Command line error near "C:/Program". Switch expected.`). Running winget's cached installer directly works: `"%TEMP%\WinGet\Microsoft.WindowsSDK.10.0.26100.10.0.26100.7705\winsdksetup.exe" /features OptionId.WindowsDesktopDebuggers /quiet /norestart`. Symbols: Microsoft public server only, cached at `scripts/out/cui/symcache`.
  - **Capture method (no synthetic input).** AE relaunched in the unregistered arm (pid 57428, `REGISTER_UI skipped`), its main window moved onto the secondary monitor with `SetWindowPos` so the run never covered the primary screen, then `procdump -accepteula -ma -t -e 1 -f ACCESS_VIOLATION -n 10 57428` attached to the live process. The effect was applied through the normal driver leg, and the modal was dismissed **programmatically** — `PostMessage(hDlg, WM_COMMAND, IDCANCEL, 0)`, the same message the Cancel button posts — so no mouse or keyboard input was synthesised at any point.
  - **Measured fault.** procdump logged the same event **ten times** (once per repaint attempt, until `-n 10` was exhausted): `Exception: C0000005.ACCESS_VIOLATION` followed by `Unhandled: C000041D`. Exception context (`.ecxr`):

    ```
    AfterFXLib!CEffects::UpdateInvalidParams+0x5db1:
    00007ffd`56233e51 488b0b   mov rcx,qword ptr [rbx]   ds:00000000`00000000=????????????????
    rbx=0000000000000000
    ```

    A **null-pointer read**, inside After Effects' own code, on the Effect Controls paint path.
  - **The full chain, in one stack** (`k 60` on the dialog-up dump, innermost first):

    ```
    AfterFXLib!CEffects::UpdateInvalidParams+0x5db1      <-- AV, null deref
    AfterFXLib!CTopicPane::Draw / CECStream::UpdatedValueChanged / CTopicPane::DrawSelf
    AfterFXLib!CTopicArray::Iterate / COutlineDir::GetOutlinePano
    AfterFXLib!ecp::internal::CECStreamV2::…                (effect-control-panel stream)
    AfterFXLib!CPane::DrawAll / CPane::Pane_Draw / CChore::Perform
    dvaui!…UI_Node::UI_Draw* / OS_Window::UI_Paint          (WM_PAINT)
    ntdll!KiUserExceptionDispatch / RtlDispatchException
    ntdll!KiUserCallbackDispatcherHandler
    ntdll!LdrpLogFatalUserCallbackException               <-- C000041D, non-recoverable
    KERNELBASE!UnhandledExceptionFilter
    dvacore!…PanicFromUnrecoverableSignalHandler           <-- Adobe's panic handler
    AfterFXLib!CPDir::DoSaveCopyAs -> FILE!FILE_DoSaveDialog -> comdlg32!CFileOpenSave::Show
    ```

  - **What this establishes.** (1) The crash is **inside After Effects**, not in DynamicFX, the probe, or the `after-effects` crate — but it is **provoked** by the plug-in declaring `PF_PUI_CONTROL` on a parameter while never calling `PF_REGISTER_UI`: AE paints that row expecting a registered custom-UI record, finds null, and dereferences it without a check. (2) The death is **not a normal unhandled exception**: because the faulting frame runs inside a kernel-dispatched user callback, Windows converts it to `STATUS_FATAL_USER_CALLBACK_EXCEPTION`, which is why **WER `LocalDumps` and the Application event log see nothing** — the instrument's blindness is now explained, not merely observed. (3) The 「保存副本」 dialog is **AE's crash-rescue save**, raised by Adobe's panic handler; it is not auto-save, which is why it appears only when the effect is applied and why dismissing it is followed by process death.
  - **Correction to Round 3's reading of the first dump.** An intermediate note in this session said the dialog-up stack carried "no exception-dispatch frame", and briefly retracted Result G's inference on that basis. That was an artifact of the sweep command `~* kL 12` truncating at 12 frames: the exception frames sit **below** the cut. The full `k 60` above shows `KiUserExceptionDispatch`, `LdrpLogFatalUserCallbackException`, `UnhandledExceptionFilter` and Adobe's panic handler all present. Result G's inference stands, now upgraded to measurement.
  - **Symbol caveat, stated plainly.** Only Microsoft public symbols exist for this stack; `AfterFXLib` ships no public PDB, so every `AfterFXLib!…` name is a **nearest-exported-symbol approximation** (hence offsets such as `+0x5db1`) and the private function names are almost certainly different. What is exact: the faulting **module**, the faulting **instruction and register state**, the `dvaui`/`ntdll`/`KERNELBASE`/`comdlg32` frames (public symbols), and the paint-path shape corroborated by several independent frames naming pane/topic drawing.
  - **Behaviour under a debugger differs — do not generalise from it.** With procdump attached in first-chance mode, AE **survived** the whole sequence (still alive and responding minutes later), because the debugger takes the first-chance exception before the fatal-callback escalation completes. The 3/3 deaths in Round 3 were all **without** a debugger attached. Any future repro that reports "AE did not die" must state whether a debugger was attached.
  - Evidence: `scripts/out/cui/faultdump/` — `AfterFX-AV.dmp` (first-chance AV capture, 5.08 GB) and `AfterFX-FAULT-dialog-up-12656.dmp` (5.30 GB, carries the complete fault-to-dialog chain), with `analysis/01_analyze.txt`, `02_threads.txt`, `03_av_ecxr.txt`, `04_dialog_deepstack.txt`. Nine further byte-equivalent first-chance captures written by `-n 10` were **deleted after analysis** (they were repeat captures of the same repaint fault, ~46 GB); the two retained dumps and the four text analyses are the record. `!analyze -v` on a **manual** procdump is misleading here — it reports procdump's own injected breakpoint (`80000003`, `win32u!NtUserMsgWaitForMultipleObjectsEx`) because a manual dump carries no exception record; use `.ecxr` plus a deep `k`.
  - **Consequence for the editor track.** A revived custom-UI editor must call `PF_REGISTER_UI` in `params_setup` for **every** build that declares `PF_PUI_CONTROL`; omitting it is not a degraded-but-safe state, it is a host-fatal one. This is also an **upstream report candidate**: the `after-effects` crate lets a plug-in set `ParamUIFlags::CONTROL` with no compile-time or runtime guard that `register_ui` was ever called, and the failure mode is a silent host kill with no dump.
- Original host-run prerequisites (now met): elevated copy into AE 2025 `Support Files\Plug-ins\`; a crash-dump source (WER `LocalDumps` chosen over the uninstalled procdump); AE started by the runner.
- Related: [TR-0031-001](#tr-0031-001--gradient-parameters), [ADR-0031](adr/0031-gradient-parameters.md) §7, [ADR-0032](adr/0032-gradients-are-graph-resources.md), [ADR-0033](adr/0033-gradient-stops-are-ordinary-parameters.md), [TR-M0-004](#tr-m0-004--sequence-transport-payload-capacity), `spike/probe/`.

### TR-0042-001 — Gradient editor (ADR-0042): probe-leg prelude and editor host legs

- Status: **probe legs u3 / u4 / u146 all `PASS` on AE 2025 (2026-08-29)** — the ADR-0042 Decision 6 stage-2 gate is **open** (event-path sibling-parameter commits behave like hand edits, including on a keyframed stream), the color-picker modal is host-safe from the Click path, and a 46 px control area draws and tracks. **Editor host legs `PASS` on AE 2025 25.6.6x4 build 4 AND AE 2026 26.3x87 (2026-08-29, artifact `BB39AC11…`):** interactive draw/drag/undo/keyframe legs, cold reopen, M2 12/12 + M3 4/4 + aerender per year, and the editor-on↔off interchange in both directions with zero value loss. One v1 cosmetic recorded: the canvas repaints when a drag ends (the viewport follows live).
- Date: 2026-08-29, 19:51–19:58 (driver `legs.log` timestamps; install 16:01, first start wedged on AE's crash-recovery modal from the TR-CUI-001 fatal rounds — dismissed 「继续」 by the user, never safe mode).
- Baseline: `main` at `7b84dc0` plus the uncommitted slice-2 probe diff (spike/probe u3/u4/u146 legs + driver `VARIANTS`); the artifact hash is the identity.
- Artifact: `DynamicFxProbe-u3-u4-u146.aex` SHA-256 `844DD9D5DCEE0FC6DFE510D005A750D00CAB818E31054197CEE4AE4E3E41DA29`, installed by the driver (post-copy hash re-verified) at `C:\Program Files\Adobe\Adobe After Effects 2025\Support Files\Plug-ins\DynamicFxProbe\DynamicFxProbe.aex`. PiPL decode of the working-tree build set: base `eGLO 0x06000440` (no leg strings in the binary), every legged artifact `0x06008440`.
- OS: Windows 11 Pro 10.0.26200. Host: After Effects 2025 **25.6.6x4 build 4** (`app.version` read in-session). Registration armed (no `DFX_PROBE_NO_REGUI`); WER LocalDumps still configured from TR-CUI-001.
- Procedure: `tr_cui_001.py install u3u4u146` → `start` → `leg probe` (fresh project, comp `cui`, solid, apply with layer selected) → interactive gestures by the user in two rounds → panel JSX readbacks between rounds → `quit`. Round 1: repeated clicks/drags on `U3 Canvas`, click+drag on `U146 Canvas`. Round 2: keyframe armed by script (`setValueAtTime(0, 0)`, CTI parked at 1 s), one isolated click on `U3 Canvas`, then picker pick + picker cancel on `U4 Canvas`.
- **u3 — event-path sibling commit (the stage-2 gate): `PASS`.** Every gesture delivers `CLICK` plus a `DRAG last=true` terminator and both write paths fire; values land exactly (click `sp=(294,·)` in `frame=(52,189,332,269)` → 0.864286 = (294−52)/280); the committed value is visible in the ECW and to scripting (final scripted read 1.0, consistent with a last click clamped at the right edge). **Keyframed stream:** with one key at t=0 and CTI at 1 s, a single canvas click produced `numKeys` 1→2, keys `[[0, 0], [1, 0.528571]]`, the t=0 key untouched — exactly a hand edit's semantics. **Undo/redo (isolated gesture):** one undo removes precisely the new keyframe; redo restores it — via command id **2035** on this host (id 17 is a silent no-op here; instrument note for every future scripted undo probe).
- **u4 — `PF_AppColorPickerDialog` from the Click path: `PASS`.** Pick → `res=picked color=(0.296275,0.106668,0.359477,1.0)`; second click → cancel → `InterruptCancel` with the seed echoed. AE alive through both modals, the panel bridge resumed after each, no rescue dialog, no dump.
- **u146 — 46 px control area: `PASS`.** Declared 200×46 renders as `frame=(52,425,332,471)` — height honored at 46 px, width stretched to 280 like every canvas; `DRAW painted` on apply; click and drag stream logged. (The old editor's height class is exonerated as a crash suspect, as Round 3/4 already implied.)
- Also observed: all three canvases painted on apply (`DRAW painted` idx 7/9/10 — Color+CONTROL needs no expand); zero crashes across the session; AE quit clean via the driver.
- **Residual (recorded, non-gating):** under round 1's rapid gesture burst (~8 click+drag pairs in ~3 s) the scripted two-undo trajectory read 1.0 → 0 → 0.935714, which no per-gesture-entry model explains cleanly; the isolated-gesture semantics are clean, so the suspects are script-read racing around `executeCommand` or entry coalescing under burst. Re-observe during the editor host leg before relying on burst-undo granularity; a plain single-edit undo is measured sound.
- Evidence: `scripts/out/cui/legs.log` (install/start/leg/quit records with hashes), `%TEMP%\dynamicfx_probe.log` (per-pid `U3_*`/`U4_PICKER`/`CLICK`/`DRAG`/`DRAW` lines), in-session JSX readback JSON. Run outputs live under the gitignored `scripts/out/`; the hashes and this row are their identity.
- **Editor host legs — `PASS` on both years (2026-08-29, 21:13–21:52).** Artifacts: the first candidate `12D5D9DAFA87855E…` (8,643,072 B) ran one drag leg on AE 2025 and measured the drag-repaint cadence — the viewport follows every committed write live, the canvas repaints only when the drag ends; Fable added `EventOutFlags::UPDATE_NOW` on the drag path and the **final candidate `BB39AC1156F52F02F79B3E8FB0AFC6BEBC01B450C91F9A066A151C5636994E3E` (8,643,584 B, eGLO `0x06008644`, PiPL 1.6)** went to both years (install script with pre/post hash checks; 181 tests green both flavors, zero-warning builds). `UPDATE_NOW` did **not** achieve mid-drag canvas repaint — consistent mechanism: AE repaints rows whose values changed, and the canvas value never changes by design — so release-repaint is the recorded v1 presentation behavior; follow-up hypothesis: `update_param_ui` on the canvas from the drag path (probe-testable). Mid-pass the user granted Modify ACLs on both `Plug-ins\DynamicFx` folders (`grant_acl_and_install.ps1`, one elevation); an unelevated copy+hash swap was then verified from the plain shell — artifact swaps need no elevation from here on.
- Per-year scene: fresh 512×128 comp, black solid, a one-pass `hint:gradient` envelope (the TR-0031-001 shape) committed through the Source expression; idle-compiled `compiled: 1 pass`, token Active; the editor rows live at the re-pinned indexes — 129 `Gradient 01`, **130 `Preview` (the canvas)**, 131 anchor, 133+ stops; `numProperties` 428 (427 declared + Compositing Options).
  - **AE 2025:** canvas paints ramp+ticks; a left-tick drag commits live to the render; one undo snaps value, canvas and viewport back together (user-confirmed); on a keyframed stream (key@t=0, CTI 2 s) the drag produced `numKeys` 1→2, keys `[[0, 0], [2, 0.443514…]]`, the t=0 key untouched, and the value read 0.2395 at a user-scrubbed CTI 1.08 s — the correct interpolation. Cold reopen: keys/values restored exactly, token Active renders pre-idle, the Status label renames on the next idle pass.
  - **AE 2026 (the first custom-UI datum on this year):** load/registration/compile clean; a static drag persisted (`pos2` 0.476190…); undo confirmed; the disambiguated left-tick drag with the right stop parked mid-ramp produced key `[2, 0.47619047760963]` — **equal to the neighbor's position to the last digit: the monotone clamp measured on-host**. Cold reopen restored everything.
  - **Batteries:** M2 **12/12** and M3 **4/4** (reopen/t04/recover/aerender; the same Session-4 `SKIP` shape as the 0.0.6 pass) on **both** years, invoked exactly per the hostpass-20260826-006 README sequence.
  - **Interchange (AE 2025, both directions):** the editor-saved project in the off flavor (`AC85FA0D3AEEC133…`, PiPL 1.5 = the shipped 0.0.6 words): 426 props, no `Preview`, **the editor-made keyframes intact to the digit at the off indexes** (id stream matching across the flavor change), token Active; saved from off, reopened in the editor flavor: 428 props, `Preview` back at 130, keys `[[0, 0], [2, 0.443514…]]` and values intact. The ADR-0042 Decision 1 decoration property is proven in both directions.
- Evidence-shape note: the 0.0.5 `baseline.aep` 13-probe reopen was **not repeated** for this pass; coverage argument: every pre-existing stream's id is unchanged in both flavors (golden tables pinned per flavor), and the bidirectional interchange above measured live id re-matching across the actual topology delta, on real editor-authored keyframes.
- Residuals: the canvas release-repaint behavior (above, recorded); the growth-pool anchor label reverting to its pool name after reopen is the pre-existing 2026-08-16 limitation, unrelated to the editor; the probe prelude's burst-undo anomaly did not reproduce — single-gesture undo/redo was clean on both years.
- Evidence: `scripts/out/hostpass-0042/` (both artifacts + `SHA256SUMS`, `ed42_2025.aep`, `ed42_2025_offsaved.aep`, `ed42_2026.aep`, the install/swap/grant scripts), `scripts/out/m2/<year>/` and `scripts/out/m3/<year>/` battery logs/PNGs/PSDs, and the in-session JSX readbacks quoted above (gitignored run outputs; the hashes are their identity).
- **Release candidate: `BB39AC11…`, preserved as `scripts/out/hostpass-0042/DynamicFx-editor.aex` — package from that file, never rebuild** (the TR-REL-003 discipline; unlike 0.0.6 the verified bytes live in an archived copy, not only in `target/release/`).
- **Field addendum (2026-08-29, AE 2026 26.3x87, the user's `examples/流体渐变（已转换）.aep`):** six DynamicFx instances (one 21,940-char multi-pass shader with `@param ramp … hint:gradient` "Custom Ramp" + a `use_ramp` toggle and a hand-written 14-node `thermal()` palette) reopened under the editor artifact — 428 rows, `Preview` canvas present, tokens Active, renders. **A/B regression: `PASS`** — two comps (the 3840×2160 main and the 3072×1024 bounds comp incl. its GrowBounds/pad layers), same frame t=2 s, purge-before-render, rendered on the editor flavor then the off flavor via unelevated swaps: **pixel-identical both comps, byte-equal PNG sizes** — the editor does not change rendering on a real project. **The palette-transplant test (seed `thermal()`'s nodes into the 8-stop control, flip `use_ramp`): `BLOCKED`** — scripted `setValue` on stop rows 3+ is refused (the hidden-row refusal, by design), and a scripted write to `Stops` commits the value (read back 8) but never reaches the plugin as a UserChangedParam (the TR-M0-005 class), so the visibility re-application that would unhide rows 3–8 never runs; the only unblocking path is a real hand gesture on the `Stops` slider, and the user shelved the test instead. Also observed and un-attributed: these reopened instances' `Status` labels never renamed during the whole session (tokens Active, renders fine — the label/unhide idle service did not touch them; the ed42 scenes the same day renamed within seconds). The user's project file was closed without saving — mtime 2026-08-21 preserved, untouched.
- **Outcome — the editor is SHELVED (user decision 2026-08-29):** no 0.0.7 release; the capability stays in-repo behind the `editor` cargo feature with **default off** — the default build ships the 0.0.6 surface again (re-verified after the flip: 181 tests green on both flavors, both release builds zero-warning). Implementation method and re-enable path: [docs/gradient-editor.md](gradient-editor.md). Shipping the editor later re-runs this row's release gate on the artifact that ships.
- Related: [ADR-0042](adr/0042-gradient-editor-presentation-contract.md), [TR-CUI-001](#tr-cui-001--custom-ui-crash-bisection-spike), [ADR-0033](adr/0033-gradient-stops-are-ordinary-parameters.md).

### TR-REL-006 — 0.0.6 release verification

- Status: **PASS** — the package carries the exact host-verified artifact, byte-identical (no rebuild between verification and packaging).
- Date: 2026-08-26
- Package: `dist/DynamicFX-0.0.6-win-x64.zip` SHA-256 `C5A401F8F2AD272F2DD0178F1258C850357F5B65E861021A412435D9B57FB23D`; inside, `DynamicFx.aex` 8,613,888 B SHA-256 `9E438A6444394EA8…` — equal to the [TR-0041-001](#tr-0041-001--panel-polish-and-the-final-006-artifact-re-verification) release-gating artifact (full hash chain in the hostpass evidence README and swap log). Contents: aex, README (0.0.6 state incl. `hint:canvas`, grouping, and the numeric-index scripting note), INSTALL.txt (version string updated), LICENSE, SHA256SUMS, five `examples/*.glsl` incl. the new [`reach-ring.glsl`](../examples/reach-ring.glsl).
- Gate evidence: TR-0041-001 (both years, final artifact) + the R1 leg records TR-0039-001/TR-0040-001; `cargo test` 170 passed and warning-free release build at packaging time; publication-boundary scan over the commit set recorded clean (credentials 0, machine-local user paths 0, vendor-internals markers 0) in `IMPLEMENTATION_STATUS.md` before the push.
- Release: tag `v0.0.6` (pre-release) with the zip attached; closes [issue #8](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/8).

### TR-REL-005 — 0.0.5 release verification

- Status: `PASS` — the packaged artifact is the exact binary host-verified on both years by the TR-BIND-002 fix-build runs and [TR-0038-001](#tr-0038-001--m2m3-batteries-on-the-adr-0038-fix-build); TR-CACHE-001 carries an explicit equivalence argument (below) rather than a re-run on this artifact.
- Date: 2026-08-21
- Baseline: `main` at `4155837` (ADR-0038 Accepted at `e071d76`) + this release commit; `Cargo.toml` bumped `0.0.4` → `0.0.5`; **PiPL subversion unchanged at 5** — ADR-0038 §7 appends one hidden parameter at the tail and changes no out-flag; both hosts read the appended parameter on the unbumped PiPL (the plan-token mirror and resolution worked in every harness leg), and 0.0.3 appended the ADR-0030/0031/0034/0035 pools the same way.
- OS: Windows 11 Pro 10.0.26200. GPU: NVIDIA GeForce RTX 5080, DX12.
- Artifact: `DynamicFx.aex` SHA-256 `FF1197D9DB09CCE81D8E2CE26AC4FD3FA752FEA8705109FBCA8D9725670A0344` (8,564,736 B), packaged as `DynamicFX-0.0.5-win-x64.zip` (build output; not in git — the hashes here are its identity) SHA-256 `0393938262886E93E6B9AB715042DE0861641402BEB391EA65DC7E9C35035123` (3,156,791 B), containing the aex + README + LICENSE + `INSTALL.txt` + `SHA256SUMS` + `examples/` (the four tracked shaders and their README), root layout matching 0.0.4. Published at <https://github.com/JUNKDOGE-JOE/dynamicfx/releases/tag/v0.0.5> as a pre-release; tag `v0.0.5` on `main`.
- **The version is not embedded in the binary** (established for TR-REL-004: `CARGO_PKG_VERSION` unused, PiPL version a literal, no `0.0.x` string in the DLL), so the `FF1197D9…` artifact built while `Cargo.toml` read `0.0.4` **is** the 0.0.5 binary: verified, then packaged, never rebuilt in between (the TR-REL-003 constraint).
- Host verification of the functional changes against this exact artifact: TR-BIND-002 harness `PASS` on AE 2026 26.3x87 and AE 2025 25.6.6x4 (pre-fix `FAIL` recorded first); M2/M3 batteries `PASS` on both years (TR-0038-001); TR-CACHE-001 was **not re-run** on this artifact: its host verification (3 interrupted-preview rounds, 0 dropped-layer frames) was on the `24E963FB…` build of `cfccd5d`, and at packaging time the only After Effects 2026 instance on the machine was the owner's own session with an unsaved project, which the procedure must not touch. Equivalence: `git diff 6386e87..HEAD -- src/lib.rs` changes exactly one thing inside the `SmartPreRender`/`SmartRender` arms — the `if local.compiled.is_none()` gate around `resolve_transported_definition` is removed — and changes zero lines of the checkout/cancel branch (`smart render input checkout failed` → check-in, thread-local clear, `Err(e)` propagated); the resolution call runs before the input checkout and does not feed the cancel branch, so the behaviour TR-CACHE-001 verified is the same code path in this artifact. The M2/M3 batteries and the harness `aerender`/reopen legs rendered through the same arms on this artifact without a fault.
- Unit evidence on the same tree: `cargo test` → **147 passed; 0 failed**; `cargo build --release` → zero warnings; `python scripts/check_governance.py` → `PASS` (38 accepted ADRs).
- **Not verified by this release** (unchanged from 0.0.4): After Effects 2024 (unprovisioned) and 2023 (host will not launch); a mask/path input on an adapter lacking `FLOAT32_FILTERABLE`; the Point 3D *declared default's* units.

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

### TR-MAC-001 — Native Apple Silicon AE 2026

- Status: **`PASS` for the documented native AE 2026 subset on corrected artifact e239**. First-binary Half preview **FAIL** is retained. The final binary was installed and independently repeated the host checks below; first-binary results were not transferred.
- Baseline: fetched `origin/main` `d4477ab6256d5fa712993f378b85c21fbe94bdf9` plus the [recorded native diff](audits/evidence/macos-arm64-20260908/native-source.patch) and [source hashes](audits/evidence/macos-arm64-20260908/source-manifest.json), branch `codex/macos-shader-quality`.
- Date/time: 2026-09-08 Asia/Shanghai; original logs retain timestamps. OS: **macOS 26.5.2 build 25F84**, native **arm64 / Apple M5 / Metal**. AE **2026 26.3x87**, build 87. Rust/Cargo **1.97.1** via `stable`, naga/wgpu **29.0.4**.
- Artifacts: first signed binary **b3bd67cf021b453a8616a8e5e3c89a444da7822d80276b4c55629afdafd3c480**, 7,535,968 B; corrected **e239ae457ab33e1bc9466d5ab0bff8b81c08861d072eb5e667b5cd8bc6ae8b0b**, 7,536,144 B. Same PiPL SHA-256 **a42ceeda2455c4775ce88fcc9269341daa731c1e2bb754debe4ebcd7099898bb**. [Full identities](audits/evidence/macos-arm64-20260908/build-identities.json). Destination `/Applications/Adobe After Effects 2026/Plug-ins/DynamicFx/DynamicFx.plugin`; first installed hash verified with backup. Corrected installation completed after user authentication; installed binary and PiPL hashes match, with the first artifact backed up. [Final install record](audits/evidence/macos-arm64-20260908/final-run/install-ae2026-geometry.json).
- Commands: `RUSTUP_TOOLCHAIN=stable bash scripts/build-macos.sh`; `cargo +stable test --target aarch64-apple-darwin` and the same with `--features editor`; `python3 scripts/macos.py verify`; verified installer for explicit year 2026. [Build record](audits/evidence/macos-arm64-20260908/build/README.md) preserves two compile failures (signed error constant, user32 link), successful native builds and **184/184 default + 184/184 editor tests** after geometry repair. ARM-only PiPL, exported `EffectMain`, system dependencies, ad-hoc signature and archive-extraction hashes passed.
- First-binary host procedure: [native harness](../scripts/macos/ae_smoke.py) `setup`, `capture`, `keyframes`, `reopen`, `capture --name reopened-capture`, using `--transport aemcp` after the initial public AppleScript call had an unverified outcome. Exact submitted JSX, publication tokens, all numeric values, two-key saved readback and **22 passing checks before and after reopen** are retained in [first-run](audits/evidence/macos-arm64-20260908/first-run/README.md). Fixture coverage: one/three-pass UV, 8/16/32-bpc, unclamped float (2, -0.5, 0.123456), four random-order temporal samples, invalid-source passthrough, parameters and persistence. The bridge drives AE scripts only and is not a plugin dependency.
- First-binary independent aerender: a fresh, unpreviewed three-pass **321×239** source with changed blue 0.375 rendered through `/Applications/Adobe After Effects 2026/aerender -project <native-aerender-fresh.aep> -rqindex 1`; two PSD frames and five analytic pixel probes passed. [Log](audits/evidence/macos-arm64-20260908/first-run/aerender-fresh.log), [numeric check](audits/evidence/macos-arm64-20260908/first-run/aerender-fresh-checks.json). The earlier missing frame-placeholder failure is retained and the harness now uses `[#####]`.
- Reproduced failure: real Composition-menu Full→Half→Full on the 1280×720 Siri example showed a complete Full rim but only the top-left portion at Half. `PF_InData.width/height` remain logical, while the canvas and worlds are physical. The correction converts each axis before base checkout, canvas resolution, containment and SmartRender fallback; tests cover ratios, outward rounding and expansion. New verbose `smart geometry:` logs support the pending real-host regression check. PNG-helper exports are not proof of preview resolution; its 32-bpc darkening is separately documented with sampleImage/headless parity and correct 8-bpc export.
- Final-artifact host repeat: `final-open`, `final-capture`, `final-keyframes`, `final-reopen`, `final-reopened-capture`, and `final-keys-read` ran on e239. **22/22 checks before and after reopening**, with saved keys read without rewriting them. A newly added instance in the clean final demo also published successfully. [Final raw evidence](audits/evidence/macos-arm64-20260908/final-run/README.md).
- Final Full/Half/Quarter: CUA switched the actual Composition resolution menu at t=1 on the 1280×720 Siri scene; all three retained complete four-sided rims. Verbose geometry records 1280×720 / 640×360 / 320×180 canvases and matching upstream worlds. The native Details button opened `compiled: 1 pass, 12 params`, E0, then dismissed normally (AE supplies its standard ReportInfo dialog wrapper).
- Final independent export: `/Applications/Adobe After Effects 2026/aerender -project <final-aerender.aep>` exited **0** and produced **12 PSD frames**, two per Full/Half/Quarter for a newly authored, unpreviewed three-pass UV ramp (blue 0.625) and the Siri example. The odd 321×239 ramp became 161×120 and 81×60; every RGB pixel matched its analytic oracle within one U8 level. [check_exports.py](../scripts/macos/check_exports.py) passed **39/39 checks**, including file/depth/alpha completeness, four illuminated Siri edges and dark center. Full native Siri versus same-time production F32 reference max error **0.0019607831**, consistent with U8 quantization. Native Half/Quarter versus box-reduced native Full: mean error **0.00070544 / 0.00090892**, p99 **0.00490196 / 0.01593137** (fixture-specific limits 0.003 mean / 0.03 p99). [Numbers and output hashes](audits/evidence/macos-arm64-20260908/final-run/final-export-checks.json).
- Delivery: clean one-composition 32-bpc Full-preview demo saved locally as `scripts/out/macos/ae2026/Siri-Glow-macOS.aep`; exact setup/save and publication readback retained. Final plugin archive was not rebuilt or re-signed after acceptance. AE 2024, Intel/Rosetta, current-diff Windows/DX12, Path downsample semantics and full performance/MFR matrix remain **NOT_RUN**. No release/notarization claimed. Protocol: [ADR-0043](adr/0043-apple-silicon-host-protocol.md).

### TR-QUALITY-001 — Headless Metal Shader quality

- Status: **`PASS` for 45 named headless GPU renders and their quantitative gates**. Windows/DX12: **`NOT_RUN`** for this change. Independent texture-minification behavior: **`NOT_RUN`**. This row does not certify an AE host, plugin installation, or arbitrary shader quality; macOS AE acceptance is recorded separately.
- Date/time: 2026-09-07 20:43 UTC / 2026-09-08 04:43 Asia/Shanghai. OS: **macOS 26.5.2 arm64**; GPU **Apple M5**, actual backend **Metal**. Rust `1.97.1`; naga/wgpu `29.0.4`.
- Baseline and instrument: HEAD `d4477ab6256d5fa712993f378b85c21fbe94bdf9` plus the recorded working-tree changes. [`scripts/quality`](../scripts/quality/README.md) imports the production frontend, definition, binding, plan and renderer modules directly. The current executable uses the fixed working-tree `render.rs`; its comparison executable substitutes only `render.rs` from `d4477ab`, with the historical diagnostic `DYNAMICFX_BACKEND=all` override. Both returned Metal adapter identities. Compiled-in renderer BLAKE3 hashes and executable/source SHA-256 hashes are in [summary.json](audits/evidence/shader-quality-20260908/summary.json). No AE SDK is linked and no AE process is operated by the instrument.
- Exact final command: `python3 scripts/quality/run.py --binary scripts/out/quality/dynamicfx-quality-current --baseline scripts/out/quality/dynamicfx-quality-before --out scripts/out/quality/run5` (bundled Python with NumPy/Pillow; native GPU access). Build/source-substitution commands are in the [runner README](../scripts/quality/README.md). Plugin artifact/install destination/AE build: **not applicable to this headless test**.
- Expected: linear sampling at tested fractional coordinates; procedural coverage and noise filtering closer to an 8×8 supersampled reference; preservation of small intermediate fields at float working depth; decode-before-filter correctness for packed thermal data; finite output from the complete examples.
- Observed sampling: F32 2×2 checker center changed **0 → 0.5**, matching the bilinear oracle; quarter-texel native-scale displacement mean error **0.25 → 0**; 256×1 LUT magnified to 1024 pixels mean error **0.00097656 → 1.36e-8**, distinct sampled values **256 → 1022** (two clamped endpoint pairs). The center/offset/LUT probes also passed at U8 and U15 working depths with the documented quantization tolerance. U8 final output still has only 256 representable levels. These probes cover center sampling, fractional native-scale offsets and magnification, **not independent minification**.
- Observed procedural/precision fixtures: circle-coverage mean error versus an 8×8 supersampled hard mask fell **69.0% at Full, 68.3% at Half and 70.2% at Quarter**. Filtered fBm RGB RMSE fell **0.02698049 → 0.02040683 (24.4%)** against the supersampled unfiltered field. Mean adjacent-pixel jump at lattice boundaries fell **0.333624 → 0.00001084** with quintic interpolation (a continuity measurement, not an image-equivalence claim). A 0–0.01 intermediate amplified ×100 retained **4 levels at U8 versus 1024 at U15/F32**. Packed-temperature midpoint error was **0.0625 before decode-before-filter versus 0 after**.
- Complete examples: `siri-glow.glsl` rendered at 1280×720, t=0/2/4 s, U8/U15/F32, plus 640×360 and 320×180 with logical size fixed to 1280×720; all components finite. Half/Quarter RGB mean error versus box-reduced Full was **0.000380 / 0.000599**, with 99th-percentile absolute error **0.00535 / 0.01577**; this is not pixel identity. Corrected `apple-thermal.glsl` completed a **10-pass 256×256 F32 smoke** with checker input and unassigned external gradient; this is compilation/execution evidence, not artistic acceptance on logo footage.
- Evidence: [record and identities](audits/evidence/shader-quality-20260908/README.md), [metrics](audits/evidence/shader-quality-20260908/summary.json), [commands/stdout/stderr/exit](audits/evidence/shader-quality-20260908/renders.log), [full example](audits/evidence/shader-quality-20260908/siri-t2.png), [enlarged preview crops](audits/evidence/shader-quality-20260908/preview-corners.png), [retained failures](audits/evidence/shader-quality-20260908/failed-runs.txt). Raw f32 frames and generated GLSL remain in ignored `scripts/out/quality/run5/`; previous failing/iterative runs remain alongside. Interpretation and code changes: [shader-quality.md](shader-quality.md).
- Limits: U15 tests inspect `Rgba32Float` working data **before AE's final U15 conversion**. This row does not test AE color management, host alpha behavior, project persistence, MFR, temporal continuity, another GPU, Windows, or every shader parameter combination. The initial sandbox adapter-enumeration failure, malformed LUT fixture and overly aggressive first noise cutoff remain recorded; later PASS does not erase them.

### TR-WGSL-001 — WGSL feasibility on Metal

- Status: **`PASS` for the non-shipping WGSL spike**. Production WGSL frontend/menu integration is **not implemented**: the plugin's Language menu remains GLSL-only. AE WGSL save/reopen, Undo/Redo, keyframes, Layer/Gradient/Path resources, temporal/MFR and Windows/DX12 are **`NOT_RUN`**. These results do not grant AE WGSL support or cross-GPU bit identity.
- Date/time: 2026-09-07 **20:38:21–20:38:25 UTC** / 2026-09-08 **04:38:21–04:38:25 Asia/Shanghai**. OS: **macOS 26.5.2 / build 25F84, arm64**; GPU **Apple M5**, **Metal**, `FLOAT32_FILTERABLE` granted. Rust `1.97.1`; naga/wgpu pinned at `29.0.4`.
- Baseline/instrument: `d4477ab6256d5fa712993f378b85c21fbe94bdf9` plus the working-tree Metal renderer fix. [`spike/wgsl`](../spike/wgsl/README.md) imports production definition/binding/graph/renderer modules. WGSL is parsed to Naga IR; a temporary generated GLSL **interface schema** reuses existing annotation/reflection rules, while the original WGSL IR is emitted to SPIR-V and rendered through production wgpu/Metal. This temporary adapter is not a shipped `WgslFrontend`. Per-file source hashes and raw-output hashes are in [run.json](../spike/wgsl/evidence/run.json). Plugin artifact/install destination/AE build: **not applicable**; the instrument neither links the AE SDK nor launches AE.
- Command actually run: `python3 spike/wgsl/run.py`, using Cargo `+stable --offline --locked` and native GPU access. [tests.log](../spike/wgsl/evidence/tests.log): **89 passed, 0 failed**, comprising **8 WGSL-specific tests and 81 imported production-module tests**, not the complete plugin suite. Covered WGSL probes include reflection/offsets and annotations, supported uniform members and tail span, `@@` envelope escaping, graph lowering, manifest-bound resources, compatible language switch/alias slot inheritance, rejection of unsupported ABI forms, and raw-source error locations.
- GPU expected/observed: **six 256×128 GLSL/WGSL pairs** — 1 and 2 passes at U8/U15/F32 working depths — were **byte-identical with max absolute error 0**. The single-pass fixture combines noise-deformed geometry, `fwidth` coverage and input sampling; the two-pass graph adds three-tap blur and a second input. Float working output reached **1.5989501**, demonstrating that this GPU path did not clamp it to 1. U15 here means `Rgba32Float` working data; it does **not** verify AE 16-bpc boundary conversions. [gpu.log](../spike/wgsl/evidence/gpu.log) records adapter, features, dimensions, tolerances, byte equality and ranges.
- Additional compiler probe: the two WGSL modules emitted MSL 2.0 with real binding mappings and `fake_missing_bindings` disabled — [field.metal](../spike/wgsl/evidence/field.metal), [mix.metal](../spike/wgsl/evidence/mix.metal). This is **MSL generation only**, not separate Xcode/Metal-compiler acceptance; the actual six GPU pairs used WGSL → Naga IR → SPIR-V → production wgpu → Metal.
- Raw evidence: [run metadata/source/artifact hashes](../spike/wgsl/evidence/run.json), [unit log](../spike/wgsl/evidence/tests.log), [GPU log](../spike/wgsl/evidence/gpu.log), [preview](../spike/wgsl/evidence/U8-2pass.png), and six losslessly compressed `.rgba.zlib` outputs under `spike/wgsl/evidence/` (format/decoding documented in the [assessment](wgsl-feasibility.md)). Retained failures: [initial HEX vec3-default mismatch](../spike/wgsl/evidence/initial-hex-default-failure.log), resolved only in the fixture by using three decimal components; [initial sandbox adapter failure](../spike/wgsl/evidence/gpu-initial.log), followed by a successful GPU-enabled run. The underlying vec3/HEX production issue was not changed by this spike.
- Boundary: equal pixels show feasibility for these equivalent algorithms on this adapter, **not an automatic image-quality gain from changing languages**. Formal WGSL surface/diagnostic contracts, a shared production IR reflector, registry routing, persistence and AE/platform conformance remain the proposed production work in [wgsl-feasibility.md](wgsl-feasibility.md).

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

### TR-WGSL-002 — Production WGSL and host integration (0.1.0)

- Status: **PASS for the recorded bdfc native AE 2026 acceptance subset**
  under [ADR-0044](adr/0044-wgsl-and-010-release.md), qualified by
  [ADR-0045](adr/0045-010-source-undo-release-boundary.md). Source-expression
  single Undo remains a known **FAIL**, explicitly accepted for 0.1.0;
  this result does not claim that all Undo/Redo paths pass. The first scripted
  Gradient assignment failed on hidden controls and passed only after the
  UI initialization described below. Earlier candidate failures are retained.
- Final native source: `8b0fe81f5f6869d1ced67fc3cd1dbf6d366dae36`, tree
  `45fd2c3c62a008022d675bdc0376996894800d51`, based on origin/main `d4477ab`.
  Date: 2026-09-08. Host: native arm64 AE 2026 **26.3x87**, Apple M5,
  macOS **26.5.2 / 25F84**; Rust/Cargo 1.97.1, Naga/wgpu 29.0.4, Metal.
- Final default bundle: executable SHA-256
  `bdfc9f1d978cf052e01db97d69b1d61c1ad9cc073b7691a82e2cb2d90baaa141`
  (7,600,208 bytes), PiPL SHA-256
  `fe5f2d0ba4386a9cbb4b2695da5f0086f0d5d1f8a1a90ba3370085b91e07de04`.
  Administrator-authorized installation succeeded at
  `/Applications/Adobe After Effects 2026/Plug-ins/DynamicFx/DynamicFx.plugin`.
  Installed and frozen bundle identities match; before/after aerender checks
  retain the same executable hash. [Build](audits/evidence/wgsl-010-20260908/build-bdfc/README.md),
  [final host evidence](audits/evidence/wgsl-010-20260908/host-bdfc/README.md).
- Final CPU suites: `cargo +stable test --locked --offline` and the same
  with `--features editor`, **220 passed each**; six line-ending regressions
  are included, not additional. Exact commands/logs and retained Cargo.lock
  are in the build evidence. The original 205-test production WGSL suite
  remains in the [earlier default-suite log](audits/evidence/wgsl-010-20260908/build/tests-default.log).
- Final host procedure: serial `ae_acceptance.py` setup/capture/resources,
  keyframes, save/reopen/readback and queue modes; actual CUA Language and
  Source edits; actual Full/Half/Quarter menu changes; close AE and render
  the saved queue with the installed host's independent `aerender`.
  `check_acceptance.py` over the completed JSON records and PSD exports
  passed **558/558 checks**. This total includes export checks; it is not
  558 separate GUI tests. Original [checker result](audits/evidence/wgsl-010-20260908/host-bdfc/010c-check.json)
  and [operator record](audits/evidence/wgsl-010-20260908/host-bdfc/010c-ui-actions.json)
  accompany the exact commands and individual readbacks in the host evidence.
- Final scripted checks cover GLSL/WGSL 8/16/32-bpc, single/multiple passes,
  HDR, temporal requests, expected invalid-input diagnostics, Layer/Gradient/Path
  None/assigned resources, keyframes and save/reopen. Independent aerender
  exited **0** and produced **24 PSD frames**: 12 queue items × two frames,
  GLSL/WGSL × UV/multipass × Full/Half/Quarter, **8-bpc**. Physical sizes are
  321×239, 161×120 and 81×60. These exports do not grant 16/32-bpc aerender
  coverage; the interactive depth checks and export scope stay separate.
- Actual GUI first publication: a fresh WGSL source under GLSL first reports
  E17; selecting WGSL initializes gain 0.25, color approximately
  `[0.25, 0.60000002384186, 1, 1]`, alpha `0.40000000596046`, and angle
  `12.3456697463989`. One Cmd+Z restores GLSL and pristine values; one Redo
  restores WGSL/defaults. Existing gain keys 0.25 at 0 s and 0.75 at 1 s
  remain unchanged through a separate Language Undo/Redo sequence, with
  unchanged source bytes and the expected render/diagnostic after each step.
- Actual macOS Source paste stores bare CR line endings. An annotated raw
  WGSL source compiles with the expected gain and green pixel `[0, 0.25, 0, 1]`;
  a two-pass envelope with escaped WGSL attributes compiles and renders
  `[0.25, 0.4, 0.6, 1]`. Annotations/defaults and complete short labels pass
  on the final artifact. This does not change the accepted Source Undo FAIL.
- Actual CUA viewport inspection passes for both languages at Full, Half and
  Quarter: all six views retain the complete canvas; Quarter is visibly
  coarser as expected. Resolution readbacks accompany these observations.
  The operator JSON transcribes CUA image/AX observations; it is not a set of
  captured screenshots. The separate PSD checks provide numeric physical-size
  evidence; `sampleImage` alone does not establish viewport downsampling.
- Gradient prerequisite: the first script assignment failed because the
  parent/child controls were hidden, after partially assigning GLSL Layer/Path.
  Readback preserved that state. Opening each language's Effect Controls and
  expanding Main/Gradient 01 exposed the two active color rows; a newly tagged
  assignment then passed the original value/pixel assertions. Retain the
  initial failure and recovery. Fully headless first-time Gradient assignment
  is not certified by this passing, UI-initialized run.
- Source-expression single Undo remains **FAIL / accepted limitation** under
  ADR-0045, recorded on 661e and not relabeled as a bdfc PASS. Keep previous
  source in a file and restore the exact text explicitly. Windows/DX12 remains
  **NOT_RUN**; no other AE years, Intel/Rosetta or cross-GPU bit identity are granted.

Earlier production GPU and retired-candidate results remain historical:

- `scripts/wgsl/compare_gpu.py`: **18 pairs byte-equal**, 8/16/32 working
  formats × Full/Half/Quarter × one/two passes, odd logical size 321×239,
  production frontend and renderer. Source unchanged during run. First
  instrument attempt omitted GLSL parameter defaults and failed; retained
  separately, fixed by supplying identical annotations to both frontends.
- Two authored examples: 40 Metal renders, 96 checks passed. These are
  working-buffer checks, not AE 16-bpc boundary or full-AA proofs.
- On the retired 91c8 candidate, administrator-authorized installation from `/private/tmp`
  succeeded; installed executable SHA-256 matched
  `91c8afdcdc6186ca2efa24ca1c06cbcbb619a659b62e77fbe69fe988f9323a22`.
  The earlier administrator shell attempt to execute the installer in
  Documents failed with `Operation not permitted (126)` and left the old
  installation intact; temporary staging resolved that installation step.
- That 91c8 run on native arm64 AE **26.3x87** completed **217 assertions, all passed**:
  GLSL/WGSL rendering at 8/16/32 bpc, multiple passes, HDR, temporal requests,
  expected invalid-input diagnostics, Layer/Gradient/Path bindings (None and
  assigned), and keyframes. The capture used `sampleImage` at requested
  resolution `[1, 1]`; it does not prove physical viewport downsampling.
- On 91c8, actual GUI Language WGSL → GLSL correctly produced E17 and input
  passthrough. One actual Cmd+Z left GLSL/E17 in place; Redo was disabled,
  with two Change Value and twelve Pass Change Name entries observed in
  history. Intervening pixel probes were read-only. This is a host failure,
  not a completed Undo/Redo acceptance.
- The retired repair was built at `f4ba578`, executable `661e89af...`: supervised
  Source/Language, no idle name writes, fresh defaults before transport words.
  Default/editor each pass 213 tests; [build evidence](audits/evidence/wgsl-010-20260908/build-661e/README.md).
  Installation completed; actual Language Undo/Redo and preserved keyframes
  passed, but manual Source editing exposed CR annotation loss and short-label
  suffix corruption. Source single Undo failed separately. The
  [661e host record](audits/evidence/wgsl-010-20260908/host-661e-authoring-failure/README.md)
  preserves all three findings; only the Source Undo defect is exempted by ADR-0045.
- [Initial host evidence](audits/evidence/wgsl-010-20260908/host-91c8-undo-failure/README.md)
  retains 217 checks, instrument failures/recovery and the GUI Undo failure.
  The passing subset does not change the retired candidate's overall **FAIL**.

### TR-REL-010 — 0.1.0 publication

- Status: **NOT_RUN** for publication; final package verification **PASS**. Current-artifact host
  acceptance is complete under TR-WGSL-002 and ADR-0045; the Source-expression
  Undo failure remains disclosed and accepted, not repaired. No final release
  publication has occurred: no push, tag or release is published.
- Planned asset: `DynamicFX-0.1.0-macos-arm64.zip`, ad-hoc signed, no
  notarization claim. Windows asset follows later under the same tag.
- The final-candidate ZIP has now been assembled at
  `scripts/out/010/release-bdfc-final/DynamicFX-0.1.0-macos-arm64.zip`,
  **3,563,347 bytes**, SHA-256
  `f2fffc1808727231db8a713cddf024cbcf0e55a7f1064beff6d8897a129dcc3d`.
  Packaging preserves the accepted bdfc executable, PiPL and signature;
  extraction verified **287 internal-manifest files**, executable permissions,
  a valid arm64 signature and unchanged executable/PiPL hashes. See
  [package evidence](audits/evidence/wgsl-010-20260908/package-bdfc/README.md).
  Source/archive privacy and governance scans passed: 1,326 files, zero matched credential/restricted findings; 15 archive checks; 108 Markdown files, 1,279 links and 45 ADRs without errors. The package record retains this precommit scan snapshot.
- Remaining: finalize host evidence and author-facing disclosures; verify the
  final source/tag identity without rebuilding/re-signing; run the
  final source/archive publication scan; record tag-to-`8b0fe81` build-input
  equivalence and exact Cargo.lock; publish regular v0.1.0, then re-download
  and verify archive, executable/PiPL hashes and signature. Old 91c8 ZIP checks
  do not verify the final release package. [Audit](audits/08-wgsl-010.md).

## TR-SIRI-WITHDRAWAL - Current distribution withdrawal

2026-09-10, baseline `effc75b`, Windows: **PASS** remaining example compilation
(6 tests), isolated Windows packaging fixture, Python syntax, document links
and `git diff --check`. **PASS** Mac sample removal, archive CRC, 289 internal
hashes, and unchanged bytes/metadata for 285 retained entries including the
signed bundle. Thirteen repository files were backed up and hash-verified;
the original authoring checkout remains preserved. **PASS** main-branch
withdrawal, release cleanup and fresh-download verification: separate/bundled
sample absent, updated Mac ZIP and checksum download match the verified
candidates, Windows asset identity and source tag unchanged. No runtime
changes or new AE acceptance are claimed.
[Evidence and initial fixture failures](audits/evidence/siri-withdrawal-20260910/README.md).

## TR-REL-011 - Windows patch and IOS27Siri sample (historical)

2026-09-09: PASS default/editor CPU suites (222 each), quality suite (98),
explicit real-DX12 compiler rejection/recovery, versioned Windows Release
build and static artifact checks. PASS AE 2026 Sample relocation, embedded
source identity, original checkpoint preservation and 300 unique 60 fps native
frames. NOT_RUN exact-new-byte host installation, other AE years and Mac 0.1.1 AE execution;
native failure injection BLOCKED. Source Undo remains a known FAIL.
PASS: initial regular v0.1.1 publication, three downloaded assets equal their frozen hashes; contained AEX, lock, source identity and Sample files match. Exact environment, commands, source/artifact
identities and evidence: [release audit](audits/09-windows-011.md).

## TR-REL-011-MAC - Same-tag macOS ARM asset backfill

2026-09-09: **PASS** native ARM release build, static bundle verification,
default/editor CPU suites (222 each), quality suite (98), and five real-Metal
smoke renders / 23 assertions. **PASS** package verification, same-release
asset publication and fresh-download identity. Source is the unchanged v0.1.1 commit
`2428cfd94b4556cc3bc5ed63f7950ceca42a6b92`, with the exact lock published in
the Windows archive. Signed binary SHA-256:
`3d2fc6e88415f512988b02036779a48dcbae7b1f4e4d1b525020e71450c8133f`
(7,602,160 bytes). Build ran on Apple M5/macOS 26.5.2 (25F84), Rust 1.97.1;
the [release audit](audits/09-windows-011.md) records command, PiPL and lock
identity and links to logs. Metal GLSL/WGSL pairs have equal pixels at 8- and
32-bit working depth; an eight-pass tagged Siri source returns finite pixels
and alpha 1. This runner uses zero-valued Point defaults, so it is not a
calibrated visual or native Sample acceptance. The first malformed smoke
fixture failed before shader compilation; its corrected run is recorded
separately.

Published `DynamicFX-0.1.1-macos-arm64.zip`: **6,934,170 bytes**, SHA-256
`e8b4b01dab9df5567a93b27ee3f30ec174bb48098319826f57a22703bb2c3cc7`.
The fresh download passed **315 assertions**, including **297 internal-file
hashes**, native arm64 signature, executable/PiPL identity and exact
source/lock mapping. Packaging/docs are independently pinned to
`4bb7196d9c72a23700e44b67c6e68eb78f7c7ab5`; the runtime tag stays unchanged.
Publication preflight passed 15 archive checks and the 1,390-file /
55,458,705-byte input scan with zero matched findings. Windows/Sample asset
IDs, sizes, digests and update times are unchanged; the extended checksum
file preserves the original 267 bytes as its prefix. These are package and
publication assertions, not additional AE tests.
[Public evidence](audits/evidence/macos-011-20260909/README.md).

New-byte macOS AE installation, rendering, Undo/Redo, save/reopen, aerender
and IOS27Siri Sample acceptance are **NOT_RUN**, outside this user's backfill
scope. Prior 0.1.0 host results do not cover 0.1.1 bytes; Source single Undo
remains **FAIL / disclosed limitation**. Existing Windows/Sample results
above retain their original scope. [ADR-0049](adr/0047-011-macos-backfill.md),
[release audit](audits/09-windows-011.md).


## TR-OPEN-001 - project-open idle callback and sequence lifetime

2026-09-09, Windows 11, AE 2026 26.3x87, RTX 5080 DX12, Rust 1.97.1 MSVC.
Baseline `17e12a7` plus the working-tree changes on `codex/fix-project-open-lock`.
This is local repair acceptance, not a release. [Audit](audits/10-project-open-lock.md).

- FAIL retained: installed `68171a98...` hangs in project open. The affected
  byte-identical copy opens in 10.079 s with DynamicFX isolated.
- FAIL retained: the first repair `28d006fb...` passes 228 default/editor tests
  but native opening times out at 120 s. Private-symbol stacks and raw memory
  show CompletelyGeneral entering a mutex over serialized `01 00 DFXS` data;
  the attempted lock changes the first version byte to `02`. No competing
  DynamicFX thread owns the mutex. No original project bytes were modified.
- FAIL then PASS: adding the live-instance wrapper initially found three test
  compilation errors (old Mutex constructor / missing test-only try_lock).
  The corrected default and editor `cargo test --offline --lib` runs each pass
  230 tests. `cargo build --offline --release` passes.
- PASS: final candidate SHA-256
  `c91db8c0435ccf90e0fac33f5473825c29bbd959cfeef8cc5fcebe80092b0d0a`
  is installed only at AE 2026 `Support Files/Plug-ins/DynamicFx/DynamicFx.aex`.
  PE exports include DynamicFxMain, EffectMain and PluginDataEntryFunction2;
  PiPL routes through DynamicFxMain with local build generation 1.
- PASS: the saved recovery copy opens in 10.604 s, 278 items, 75 comps,
  184 footages, zero missing footage. All 426 DynamicFX properties, source,
  keys and expressions, plus the recorded Main/05 structure, match baseline.
  Runtime log confirms one pass / five parameters, no default writes, and
  successful DX12 pipeline creation. Main at 25 s finishes a native frame.
- FAIL then PASS: first direct original open reports AE's "layer has no source"
  error, but the original is fully loaded and clean. Read-only continuation
  uses recoveryId `vcz2im`, then a guarded close/reopen finishes in 4.420 s with
  `openError:null`, 278 items and `dirty:false`. Do not erase the first error.
- PASS: original AEP stays byte-identical, SHA-256
  `608eccc22a6a6e2defbca6af05fb4add9b825d043e9c269f4c53dc3f18de887c`.
  Native UI displays Main; PNG capture is dark because of the existing
  high-bit-depth capture path, so it is not color-quality acceptance.
- NOT_RUN: other AE years, macOS build/native acceptance, long-duration soak,
  and a full sequence-lifecycle rewrite of after-effects-rs. The entry guard
  specifically protects the plugin's early CompletelyGeneral idle calls.
- NOT_RUN: two project expressions referencing the transient display name
  `Refraction px` remain unmodified. The user stopped Computer Use before that
  project repair; no further AE input was issued. Original was not saved.

Exact commands, artifact identities and retained external evidence hashes:
[evidence manifest](audits/evidence/project-open-lock-20260909/manifest.json).

### TR-OPEN-COMMIT-001 — project-open repair source checkpoint

- Date: 2026-09-10; Windows; Rust/Cargo 1.97.1 MSVC; `17e12a7` plus the
  repair being committed on `codex/fix-project-open-lock`.
- `PASS`: all 33 source inputs match the frozen TR-OPEN-001 manifest; the
  frozen AEX remains `c91db8c0...` by SHA-256.
- Initial `cargo test --offline --lib` reused a 220-test executable that
  omitted the new reentry/live-header cases. That result does not validate
  the current repair. Source/build timestamps were refreshed without byte
  changes to make Cargo recompile the crate; the first output is retained.
- `PASS`: freshly compiled `cargo test --offline --lib` and the same command
  with `--features editor` each execute 230 tests, zero failures.
- `NOT_RUN`: new release build, installation or native AE tests in this
  commit turn. Existing exact-source/artifact acceptance remains TR-OPEN-001.
- Raw logs, equivalence record and governance checks:
  [commit evidence](audits/evidence/project-open-commit-20260910/README.md).

## TR-REL-011-REFRESH-001 - existing Windows asset replacement

2026-09-09, explicit user authorization to replace the existing GitHub v0.1.1
Windows package, version unchanged. PASS: 33 frozen source inputs match,
candidate equals the installed c91db8c0 AEX, PE architecture/exports pass,
270 unchanged ZIP members compare equal, and both newly downloaded release
files match staging and GitHub asset digests. The contained AEX, installation
hash and build identity agree. Only Windows ZIP/AEX checksum lines changed;
Mac and Sample asset metadata and the existing source tag are unchanged.
Release-note content matches after GitHub CRLF normalization. No runtime test
rerun or new installation occurred: exact-byte host/CPU acceptance remains
TR-OPEN-001. [Evidence](audits/evidence/release-011-refresh-20260909/README.md).

### TR-HOST-SHAPE-SCOPE-002 — coverage-only scope revision

- Date: 2026-09-10; source baseline `ff89564`; Windows; documentation and
  fixture-scope update only. The user withdrew the additional automatic
  contour input and retained original host visible coverage/alpha.
- Current required scenarios: HS-01..HS-12 and HS-16. HS-16 covers coverage opt-in and
  partial-alpha fidelity; former contour-specific HS-13..HS-15 definitions
  are retained under `withdrawn_cases`, not treated as passed or failed.
- Existing mask `hint:path` stays in the compatibility matrix. Raw vector
  acquisition remains diagnostic evidence, not production coverage acceptance.
- `NOT_RUN`: production coverage, its transport/invalidation, portable FFX,
  and all revised feature acceptance rows. This scope edit runs no native
  renders and changes no plugin binary or AE project.
- Current scope and verification: [scope record](audits/evidence/host-coverage-scope-20260910/README.md).
- `PASS`: issue title/body readback matches the prepared coverage-only text;
  13 active scenarios and 3 withdrawn definitions validated; production source
  matches `ff89564`; governance checks pass (132 Markdown files, 1677 links,
  zero errors); `git diff --check` passes. An initial fixture-edit script used
  the wrong key and stopped before writing; recovery is recorded in the evidence.
- Earlier preparation/native entries below retain their original scope and
  status; later results and this scope revision must be read with them.

### TR-HOST-SHAPE-NATIVE-004 — original alpha and raster-block transport

- Date: 2026-09-10; Windows AE 2026 26.3x87 / AE-MCP 0.10.7, 16-bpc;
  production `ff89564` / AEX `c91db8c0...` unchanged. Independently installed
  diagnostic 0.0.5 `5da92f37...` appends only its own self-source parameter;
  nine local tests and the locked native build pass.
- `PASS` source observation: `sampleImage(..., false)` returns `[0,1,0]` on
  shape adjustment and ordinary shape controls. Mask-only returns `[1,1,1]`:
  this source excludes masks. Temporary sampling controls were removed.
- `FAIL` native direct-source candidates: legacy PF checkout of input 0 and
  an explicitly self-bound Layer parameter return `[1,1,1]` on the adjustment
  shape. Both return `[0,1,0]` on the ordinary shape control. The appended
  parameter initially loads as None on old instances; that observation is
  distinct from the later explicitly bound result. Native Set Matte sourcing
  its own adjustment layer also returns `[1,1,1]` in the sampled control.
- `PASS` bounded feasibility: a mode-None data mask carries adaptively sampled
  raster blocks through PF PathQuery on render ThreadId(2), with no render-side
  AEGP call. A triangle and an animated/transformed curve at shuffled times
  `1`, `0`, `0.5` with CTI `0` compare to native reference alpha within one
  8-bit level across each 800x600 image. This is fixture-managed transport,
  not automatic production mask ownership, MFR or an implemented shader input.
- `PASS` invalidation observation: editing the original key changes 8509
  decoded pixels at the same requested time. `FAIL` first precision check:
  one pixel differs by 31/255 because coarse average sampling pruned tiny
  coverage. Revised small-block sampling/recombination passes that exact
  case (19 differing pixels, maximum 1/255); the edited paths were restored.
- `FAIL` partial-opacity budget: the 50% fill fixture exceeds both the initial
  and revised sampling/data budgets. The expression's disabled/fallback mask
  is not accepted as coverage. It remains disabled in the saved fixture.
- Harness recovery: initial Set Matte inspection queried a group as a leaf;
  readback confirmed the created fixture before continuing. PNG completion
  lagged one successful MCP response; the late file/log were recovered without
  replay, and capture now waits for a stable file. The first edit record's
  `before` array aliased the edited data; preserved reference PNGs and the
  untouched source fixture establish the baseline; subsequent records copy it.
- `NOT_RUN`: automatic FFX data-mask management, partial-opacity success,
  original-mask fidelity, production GPU/resource integration, MFR, cold
  aerender, other depths/hosts and the complete coverage acceptance matrix.
- The user was asked whether a plugin-managed data mask may appear in the
  host's mask list. No production dependency is accepted while that product
  decision is pending. The disposable project is saved with logging/requests
  reset; no mouse/keyboard or explicit foreground activation was used.
- [Evidence and procedure](audits/evidence/host-shape-native-20260910/original-source-v005/README.md).
- `PASS` record checks: 113 raw artifacts plus 12 source snapshots, three
  ordinary-output RGBA comparisons, production source/AEX identity, nine
  diagnostic tests; governance reports 133 Markdown files, 1684 links and
  zero errors. `git diff --check` passes.

### TR-HOST-SHAPE-001 — automatic host-shape input preparation

- Date: 2026-09-10; Windows; baseline `17e12a7` plus the existing uncommitted
  project-open repair on `codex/fix-project-open-lock`.
- Public request: [issue #9](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/9).
  Issue creation and fetched-body equality were verified through the GitHub
  connector. The local gh attempt returned HTTP 401; no auth settings changed.
- `CLAIMED_UNVERIFIED`: supplied AE 2026 26.3x87 / DynamicFx 0.1.1 / 16-bpc /
  RTX 5080 DX12 observation reports ordinary alpha `[1,1,1]` and correct final
  triangle clipping. No new native reproduction ran in this preparation.
- `NOT_RUN`: PF shape enumeration, native AEGP extraction, coverage checkout,
  frame-exact transport and all HS-01..HS-12 acceptance scenarios.
- `NOT_RUN`: new resource implementation/build, installation, FFX delivery,
  AE 2025/2026 acceptance, other host years and macOS for this feature.
- Prepared: [ADR-0049](adr/0049-automatic-host-shape-input.md),
  [probe shader](../spike/host-outline/input-alpha-probe.glsl),
  [fixture matrix](../spike/host-outline/cases.json) and
  [native probe specification](../spike/host-outline/README.md).
- Preparation-only checks and raw outputs:
  [evidence](audits/evidence/host-outline-prep-20260910/README.md).
  These checks cannot establish host-shape rendering support.
- `PASS` for preparation checks: bundled Python `scripts/check_governance.py`
  reports zero errors (126 Markdown files, 1630 local links); fixture JSON
  has 12 unique scenarios and independent triangle classification gives
  `[0,1,0]`; 31 runtime/build file hashes equal the pre-edit baseline.

### TR-HOST-SHAPE-PROBE-001 — coverage and contour diagnostic build

- Date: 2026-09-10; Windows; production baseline `ff89564`.
- Approved product scope: primary coverage for refraction plus independently
  requested contour sampling in the same feature batch. Recorded in
  [issue #9](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/9#issuecomment-5620243801)
  and read back through the GitHub connector.
- `PASS`: isolated diagnostic crate tests (5), native development DLL build,
  artifact identity and fixture consistency checks. This extends the earlier
  preparation record; it does not turn any native feature row into PASS.
- The first compile failed on four probe integration details (required PiPL
  support URL, nonzero instance size, PF vertex accessor and SDK time-mode
  type). Corrected locally before the passing build. Neither installed Rust
  toolchain has rustfmt; formatting automation was unavailable.
- `NOT_RUN`: installation, native path/coverage reads, transport, invalidation,
  all HS-01..HS-15 host cases, production shader resources and portable FFX.
  AE-MCP health was unreachable in this session.
- [Diagnostic procedure](../spike/host-outline/README.md) and
  [build evidence](audits/evidence/host-shape-probe-20260910/README.md).

### TR-HOST-SHAPE-NATIVE-001 — first MCP-only native acquisition checks

- Date: 2026-09-10; Windows AE 2026 26.3x87; AE-MCP 0.10.7; 16-bpc project,
  PNG readback at 8-bit. Installed diagnostic 0.0.1 `cf867972...`; production
  AEX `c91db8c0...` and source `ff89564` preserved.
- `PASS` reproduction: interior encoded ordinary-input alpha `[1,1,1]`,
  transparent shape reference alpha `[0,1,0]`, outside-shape background intact.
- `PASS` observations: PF enumeration yields 0 paths for shape-only and 1
  for mask-only/shape-plus-mask. Animated mask vertices/handles follow requested
  times `0`, `1`, `0.5` with CTI fixed at `0`. This is mask-control evidence,
  not automatic animated shape transport.
- `PASS` native metadata: MCP AEGP traversal reaches `ADBE Vector Shape`,
  type `mask`. MCP marks its value `unsupported`; vertices were not exported.
- `FAIL` initial setup transport: 45-second timeout; later readback confirmed
  completion without replay. Empty Source from a label mismatch was corrected
  before image acceptance. Full failure/recovery chain is in the evidence.
- `PASS` development only: background diagnostic 0.0.2 build and 7 tests.
  `NOT_RUN`: its installation/host acceptance, AEGP vertices, coverage checkout,
  production resources, complete HS-01..HS-15 and other host years.
- No foreground action was used. The user controls AE closure/reopen for
  the next diagnostic reload. [Evidence](audits/evidence/host-shape-native-20260910/README.md).

### TR-HOST-SHAPE-INSTALL-002 — background diagnostic replacement

- Date: 2026-09-10 22:47:51 +08:00; Windows, AE 2026 installation.
- `PASS`: after the user confirmed closure, the installer verified no
  AfterFX/aerender process, backed up diagnostic 0.0.1, replaced only the
  diagnostic with frozen 0.0.2, and verified SHA-256 `8d896b68...`.
- `PASS`: the production AEX remained `c91db8c0...`. Full hashes and the
  version-specific destination are in [install-v002.json](audits/evidence/host-shape-native-20260910/install-v002.json).
- Command: `scripts/out/host-shape-native-20260910/install-v002.ps1`.
- `NOT_RUN`: loading 0.0.2 in AE, background request execution, AEGP vertices
  and coverage checkout. The user will reopen AE; regression remains MCP-only.

### TR-HOST-SHAPE-NATIVE-002 — background vectors and coverage option rejection

- Date: 2026-09-10; Windows AE 2026 26.3x87, AE-MCP 0.10.7, 16-bpc fixture;
  installed diagnostic 0.0.2 `8d896b68...`, production `c91db8c0...` unchanged.
- `PASS`: MCP opened the saved disposable fixture from an empty project;
  native setup and readiness logs identify the newly loaded 0.0.2 process.
- `PASS` raw acquisition: shape vertices, relative tangent offsets and group
  transform streams match JSX post-expression samples at requested times
  `1`, `0`, `0.5`, with CTI remaining `0`. Numeric comparison passed 3/3.
  This does not establish transformed contours or render-thread transport.
- `FAIL` shape-coverage candidates: RenderSuite5 upstream and RenderSuite4
  upstream with plain=false return alpha `[1,1,1]` in an 800x600 16-bit world;
  expected host coverage is `[0,1,0]`. The checkout calls themselves succeed.
- `FAIL` invalid option combination: RenderSuite4 upstream with plain=true
  triggers modal 5027, `Plain flag mismatch. Upgrade to RenderSuite5 call`.
  The driver first times out after 30 seconds; after user dismissal, native
  logging records AE error 3. No retry or foreground action was performed.
- Harness failures retained: exact parameter-count guard rejected AE's
  appended Compositing Options before any write; fixture expression initially
  called `points()` on `value`, then was corrected to `thisProperty` before
  comparison. A local GBK print error did not lose the saved MCP result.
- Commands: `request-native.py` with unique evidence labels, comp 1 modes
  2/3/4/5, comp 76 mode 2 at `1/0/0.5`; `check-background.py` passed.
- `PASS` development only: 0.0.3 builds, 8 tests pass, frozen Windows x64
  artifact `2a79f154...`. Mode 5 refuses before entering host suites; modes
  6/7 use RenderSuite5 with layer/downstream options. Installation and host
  acceptance are `NOT_RUN`; shader resources and complete HS-01..15 remain unrun.
- Probe requests were reset to zero and the disposable project saved.
  [Evidence and exact next action](audits/evidence/host-shape-native-20260910/background-v002/README.md).

### TR-HOST-SHAPE-NATIVE-003 — layer options and auxiliary coverage

- Date: 2026-09-10; Windows AE 2026 26.3x87 / AE-MCP 0.10.7; 16-bpc
  disposable fixture. The user authorized autonomous AE close/start without
  mouse control. Startup used `Start-Process -WindowStyle Hidden`; shutdown
  used MCP save plus scheduled `app.quit()`. Process exit preceded replacement.
- `PASS`: diagnostic 0.0.3 `2a79f154...`, then 0.0.4 `9c61010d...`, installed
  in the version-specific AE 2026 directory with backup/hash checks. Both
  loaded; production AEX `c91db8c0...` and production source remained unchanged.
- `FAIL` adjustment coverage candidates: RenderSuite5 `NewFromLayer` and
  `NewFromDownstreamOfEffect` both return alpha `[1,1,1]`. The same layer
  constructor on an ordinary shape control returns the expected `[0,1,0]`.
  The checkout logic works, but these routes do not isolate adjustment coverage.
- `PASS` absence observation: render-side PF ChannelSuite reports zero
  auxiliary channels and `Coverage` absent on shape-only adjustment, mask-only
  adjustment, shape plus mask adjustment, and ordinary shape control. Four
  requests render successfully; this auxiliary source is unavailable here.
- `PASS` pixel regression: ordinary-input shader output matches the first-run
  PNG exactly; ordinary-shape control with probe matches the transparent
  reference exactly. These PNG comparisons use 8-bit RGBA, not deep precision.
- Commands: `request-native.py` modes 6/7; `capture-native.py` on four auxiliary
  fixtures and ordinary input; `check-layer-auxiliary.py`; locked 0.0.4 build
  and eight local tests. The fixture is saved with serials/logging reset to zero.
- No mouse/keyboard action or explicit window-activation call. OS foreground
  ownership was not measured. [Evidence](audits/evidence/host-shape-native-20260910/coverage-v003-v004/README.md).
- `NOT_RUN`: production automatic coverage/contour resources, full HS-01..15,
  legal frame-exact contour transport, custom rasterization and other hosts.


### TR-HOST-COVERAGE-MANAGED-001 — managed data mask and diagnostic memory repair

- Date: 2026-09-10/11; Windows AE 2026 26.3x87 / AE-MCP 0.10.7; local
  `ff89564` plus isolated probe changes; default 16-bpc disposable fixture.
- User decision: accepts an automatically managed mode-None data mask visible
  in the mask list. No extra contour API or helper layer/precomp is approved.
- `FAIL` partial-alpha optimization: v3's opacity-bound shortcut still exceeds
  32768 queries. The v4 exact-partial sampler with 262144 queries times out at
  30 seconds; later readback shows evaluation completed, but no image or exact
  completion-time acceptance exists. The expression is disabled afterward.
- `FAIL` diagnostic 0.0.6/0.0.7: the wrapper's absent-expression memory handling
  produces `Struct` and recurring tracked-memory-ID dialogs `23::33` reported
  by the user. The exact owned AE process is stopped to end the loop.
- `PASS` local repair: diagnostic 0.0.8/0.0.9 each pass 13 tests, including no
  memory callbacks for a null expression allocation. Raw bounded UTF-16 reads,
  balanced non-null allocation cleanup and failed-layer quarantine replace the
  unsafe wrapper path. 0.0.8 reaches mask renaming, which separately fails with
  `Parameter`; that error is logged once and does not keep retrying.
- `PASS` native 0.0.9 `f0bd26c4...`: automatic creation on renamed shape/group,
  two instances sharing one carrier, duplicated-comp isolation, retaining the
  carrier while one owner remains, removing only the carrier when the last
  owner opts out, and explicit re-enable. The ordinary user mask stays unchanged.
- `PASS` native Undo/Redo: Undo removes the newly generated carrier without an
  idle recreation; corrected Redo restores it. The first Redo harness uses the
  wrong command ID 17 and fails its assertion; that failure is retained. The
  documented command 2035 passes on this host.
- `PASS` native acquisition: render-side PF PathQuery returns 3648 data vertices.
  Decoded coverage compared with reference alpha over 480000 pixels has 49
  differing values, maximum 1/255. The reopened/purged frame has the same result
  and identical output RGBA. This is an 8-bit PNG comparison from a 16-bpc
  fixture, not deep-precision or performance acceptance.
- `PASS` saved project reload and cache-purged acquisition: 20 items and both
  managed carriers survive reload. The first capture driver times out waiting
  for a PNG after MCP returns; the original request later produces a PNG and
  native render log, recovered without submitting a second render. The delay
  remains recorded and is not an acceptable latency claim.
- Commands: locked offline native `cargo test`/`cargo build`; versioned guarded
  installers; MCP argument/result pairs; `check-managed-step.py`;
  `capture-native.py v009-reopened-cold-render 174 0 --purge`;
  `verify-coverage-blocks.py` comparison helpers.
- `PASS` preservation: production source and AEX `c91db8c0...` remain unchanged;
  AE is open on a saved 20-item disposable fixture with all 12 diagnostic
  logging/serial controls reset and the expensive opacity expression disabled.
- `NOT_RUN`: production shader resource/ABI, FFX recreation, aerender/MFR,
  original-mask visible alpha, bounded partial-opacity fidelity/performance,
  full HS acceptance and other hosts. No new commit or push.
- [Audit and frozen evidence](audits/evidence/host-shape-native-20260910/managed-carrier/README.md).

### TR-PUBLIC-TRANSFER-001 — public-history source transfer

Baseline: public main `18501ac` plus the nine-file repair diff of `ff89564`
and the independent diagnostic 0.0.9. Windows, 2026-09-11. No new AE run.
Commands and actual results: [checks](audits/evidence/public-transfer-20260911/checks.json).
[Audit and publication boundary](audits/evidence/public-transfer-20260911/README.md).
The original missing-lock attempt is retained alongside the rerun logs.

### TR-HOST-COVERAGE-REGRESSION-002 — AE 26.5 resumed acceptance

Overall **FAIL**, 2026-09-20, Windows / AE 2026 26.5x89, branch baseline
`922845c`. Production AEX `c91db8c0...`, diagnostic 0.0.9 `f0bd26c4...`.
No production or diagnostic source changed in this run. A copy of the frozen
test project was used through MCP; no mouse, keyboard or activation command.

- PASS: default/editor unit suites 228 each, diagnostic 13, governance.
- PASS: 800x600 cold interactive PF coverage: 3648 vertices, 49/480000 pixels
  differ, maximum 1/255. The two-image capture completed after 37.656 seconds.
- PASS subset: small triangle/rectangle/ellipse, group transform, 128x96 half
  opacity in 8/16/32 bpc, corrected curved path in 16/32 bpc, and shuffled
  animation times 0.75/0/0.25 while CTI is 1.5. These compare expression-path
  readback with reference PNG alpha, not render-thread PF reads.
- FAIL: add mask and subtract-mask hole each reach 255/255 error (2268 and
  704 differing pixels). 8-bpc curved edge reaches 16/255 (9 pixels).
- FAIL: 800x600 half opacity exceeds the 32768-query budget in 5.203 seconds;
  the failed expression is disabled afterward.
- FAIL: last-owner cleanup leaves both masks after an extended 8-second wait.
  The subsequent Undo state also fails; deletion/recreation was not established,
  so this does not isolate a separate Undo defect. Sharing and preservation of
  the ordinary user mask pass. Save/reopen preserves 52 items, masks and owners.
- PASS subset: supervised aerender exits 0 with three PSD frames in 25 seconds;
  all differ from interactive RGBA8 by at most 1. An earlier valid aerender
  process supplies one PF coverage record, also within 1/255. The supervised
  run has no new PF record and reuses cache. MFR was requested at 50% CPU,
  but the diagnostic does not advertise threaded rendering: concurrency remains
  BLOCKED, not PASS. Comparisons are 8-bit exports, not deep-pixel proof.
- BLOCKED: production resource/ABI and material FFX do not exist. Full shader,
  temporal, expanded-origin/ROI and true MFR acceptance cannot be run against
  the diagnostic. These are not waived or counted as successful tests.
- Harness failures are retained: placeholder collision, invalid small native
  fixture coordinates, initial tangent assignment, TIFF template failures,
  first missing AEP and console encoding. Corrected runs have separate evidence.
- Final state: saved 55-item disposable AEP, 16 bpc, all 22 diagnostic logging
  and request controls reset. AE remains open. No commit, push or main update.

[Audit](audits/evidence/host-coverage-regression-20260920/README.md),
[summary](audits/evidence/host-coverage-regression-20260920/raw/summary.json),
[artifact manifest](audits/evidence/host-coverage-regression-20260920/manifest.json).

### TR-HOST-COVERAGE-ENGINEERING-003 — idle wake and scale-safe diagnostic

2026-09-20, Windows / AE 2026 26.5x89, production source unchanged from
`922845c`; isolated diagnostic 0.0.10 working tree, AEX `64a14201...`.
- PASS diagnosis: the same disabled-owner fixture retains its data mask after
  passive waiting; one read-only native MCP request wakes idle and removes it.
- PASS: 16 locked/offline diagnostic tests and build. Added stop-before-host-call,
  no retry after wake failure, baseline coordinate preservation, and small/empty/
  extreme world bounds checks. First build's suite-version type error is retained.
- PASS native: plugin-owned periodic wake fixes sharing, partial-owner disable,
  last-owner cleanup, re-enable, Undo and Redo with ordinary user mask intact.
  The acceptance run uses only JSX MCP calls, without the native wake workaround.
- PASS native PF: 128x96 yields 818 vertices / 4 differing pixels; 800x600 yields
  3648 / 49. Both maximum errors are 1/255 against fresh reference PNG alpha.
- PASS shutdown: saved fixture, scheduled quit and confirmed exited process;
  startup remains Hidden. Only the diagnostic AEX was replaced; production
  remains `c91db8c0...`. No mouse/keyboard or activation command.
- Coverage sampler is unchanged. Original-mask, 8-bpc curve and full-frame
  partial-alpha failures remain unresolved; FFX and true MFR remain blocked.
  User requires full scope before delivery. No commit or push.
[Audit](audits/evidence/host-coverage-engineering-20260920/README.md).

### Coverage acceptance priority — user decision, 2026-09-20

Image fidelity is the prerequisite for preview-speed optimization. Establish
coverage and final-output correctness against native AE references at each
supported bit depth, including masks/holes, partial alpha, edges and animation.
The existing 8-bit-export comparisons are diagnostic evidence, not a replacement
for native-depth acceptance. Do not relax tolerances or silently lower sampling
quality, resolution or update frequency to obtain a performance PASS. Any
optimization must rerun the same correctness cases and preserve their results.
This requirement records no additional test run and clears no existing failure.

### TR-HOST-COVERAGE-FIDELITY-004 — exact sampling and mask API boundary

2026-09-20, Windows / AE 2026 26.5x89, public baseline `922845c` plus isolated
probe changes. Current diagnostic 0.0.14 `6734a384...`; production AEX remains
`c91db8c0...`. Overall delivery gate remains FAIL; no commit or push.

- PASS diagnosis: per-pixel 8-bpc curve and half-opacity oracles match native
  PNG alpha at all 12288 pixels. The old region-average skip loses small edges.
- FAIL old transport: direct/fractional-width mask coordinates quantize alpha;
  values such as 1e-10 disappear. PASS new diagnostic encoding: float32 words
  encoded as integer/half-integer symbols survive host coordinate noise when
  decoded with bounded rounding. The 8-bpc curve's complete PF PathQuery read
  matches the direct oracle bit-for-bit. Small float constants also round-trip.
- PASS native-depth comparisons: 128x96 curve and half-opacity data match
  AEGP upstream reference worlds bit-for-bit at both 16 and 32 bpc, with
  reference world origins accounted for. These use expression-path readback;
  no new production shader binding is claimed.
- PASS performance experiment without precision loss: a full-source cache
  warm-up before per-pixel queries reduces the cold 128x96 half-opacity
  expression from 29.888 s to 1.463 s. Cold curve/half cases at 8/16/32 bpc
  preserve the same values. This is one controlled diagnostic comparison,
  not a general preview-rate guarantee.
- PASS full-frame correctness: 800x600, 50% fill, 480000 decoded pixels,
  3746 vertices, zero difference from the native 16-bit reference. Cold
  expression evaluation takes 91.529 s; interactive performance is not accepted.
- PASS mask subset: PF_MaskWorldWithPath on white 16-bit worlds gives the
  correct add/subtract matte for feather=0 and opacity=1. Combining it with
  the unmasked native source matches both native 16-bit masked references
  exactly over 12288 pixels each.
- FAIL mask expansion: +8 px, 834 differing pixels, maximum error 32768 in
  native 16-bit alpha. The SDK raster route does not apply expansion.
- FAIL alternative shortcut: directly rasterizing source Bezier geometry and
  scaling by reference opacity differs by up to one native 16-bit level;
  float conversion experiments also differ. This route is not adopted.
- FAIL alternative source: sampleImage(postEffect=true) on adjustment cases
  returns background alpha [1,1,1] regardless of original mask/shape coverage.
- PASS local checks: 17 native tests/build; exact-codec tests include isolated
  thin alpha, fractional alpha, float32 bit preservation through noisy path
  coordinates, rectangle merging and invalid/bounded input rejection.
- The initial combined direct-oracle call times out at 40 s; both original
  outputs arrive later and a subsequent read proves recovery. No duplicate
  sampling call was sent. Old build/test failures and conservative geometry
  alternatives remain retained rather than reclassified as passing.
- No production resource, complete mask-stack semantics, FFX, full transform/
  modifier matrix or true MFR acceptance. The no-helper-object product boundary
  is now an explicit pending user decision; no proxy implementation is assumed.

[Audit](audits/evidence/host-coverage-fidelity-20260920/README.md),
[summary](audits/evidence/host-coverage-fidelity-20260920/raw/summary.json),
[manifest](audits/evidence/host-coverage-fidelity-20260920/manifest.json).

### TR-HOST-COVERAGE-DIRECT-20260920 — native source stages and return bridge

- Windows AE 2026 26.5x89; public `922845c` plus the frozen diagnostic source.
  Installed diagnostic 0.0.23 SHA-256 `dcffcf890dfea0177c7ecde5bdda05874d80046eaa1a104b67c485752abd715a`;
  production AEX remains `c91db8c0...`. Earlier 0.0.15–0.0.22 runs retain their
  own installation records. SDK files are local/ignored, not evidence assets.
- `PASS` source-item acquisition for footage, precomp and solid controls;
  generated shape layers have no source item. This is main-thread acquisition,
  not a production render adapter.
- `PASS` StreamSuite7 acquisition and stage read/write using SDK 26.5 headers.
  Default self parameters already use SOURCE (0). Explicit SOURCE, ONLY_MASKS
  (-2) and combined layer/stage writes do not fix self checkout.
- `FAIL` self coverage at 8/16/32 bpc: both layer params equal the background
  input, differing from original masked coverage at 140000 of 480000 canvas
  pixels in each case. The experiment reproduces failure, not acceptance.
- `PASS` cross-layer source and masks stages: two parameter checkouts × three
  depths × two stages, 480000 canvas pixels each, zero native-word mismatches.
  Fixture: triangle, requested fill opacity 37.125%, subtract mask opacity
  68.75%, feather [11.25, 7.75], expansion 4.25. These are exact comparisons
  with AE's own raster; no claim of unquantized shape-property evaluation.
- `PASS` return bridge: original ONLY_MASKS -> invisible reader -> ALL_EFFECTS
  (-1) -> original effect's layer parameter. Six depth/parameter comparisons
  each match all 480000 native words. Reader video is disabled. Positive
  effect-stage 1 is refused with error 3 in this fixture even though the limit
  query returns -1; the ALL_EFFECTS sentinel succeeds. Keep that failure.
- `FAIL` legacy plain rendering: NewFromLayer using options suites 1 and 2
  with RenderSuite4 plain=true returns error 3; no source pixels obtained.
  Quiet error handling avoids a repeated modal loop. Do not retry those pairs.
- Local checks: 19 SDK-enabled diagnostic tests and locked build pass;
  the SDK-absent 0.0.21 build passes 19 tests including explicit refusal.
  `python spike/host-outline/verify-source-stage.py
  docs/audits/evidence/host-coverage-direct-source-20260920` independently
  rechecks 18 exact comparisons and retains the self failure.
- `NOT_RUN`: automatic reader ownership, duplicate/Undo lifecycle, FFX,
  production resource ABI, transformed/3D/modifier cases, MFR concurrency,
  old-host stage control and the complete release gate. User approval for
  automatic internal layers changes scope, not these verification statuses.

[Audit](audits/evidence/host-coverage-direct-source-20260920/README.md),
[manifest](audits/evidence/host-coverage-direct-source-20260920/manifest.json).

### TR-HOST-COVERAGE-READER-20260920 — automatic native reader

- Windows AE 2026 26.5x89; public `922845c` plus frozen diagnostics.
  Pixel/lifecycle suite: 0.0.25 `fb46db4a...`; current-build pixel smoke:
  0.0.26 `7f6918d4...`; owner-deletion correction: 0.0.28 `bc0265d2...`.
  Production DynamicFx.aex remains `c91db8c0...`; production code is unchanged.
- `PASS` 10 lifecycle checks: initial creation, exact native float32 alpha,
  two instances sharing, first opt-out releasing its binding, last opt-out
  removing the reader and unused solid item, cleanup Undo/Redo, re-enable,
  duplicate-owner separation and owner renaming. Snapshot readback proves
  bindings; duplicate-owner pixels before the first idle tick are NOT_RUN.
- `PASS` 30 native frame comparisons, 480000 canvas pixels each, zero word
  mismatches: 9 cold shuffled animation/depth pairs at requested 1, 0, 0.5 s
  with CTI 1.6 s; 6 2D/3D transform/depth pairs; 9 nontrivial repeater,
  merge-subtract and stroke/depth pairs; 3 identity-repeater controls; 3
  0.0.26 smoke pairs. The identity repeater defaults to one copy and is not
  counted as modifier coverage. Each later modifier changes reference pixels.
- `PASS` saved project and stable reader count on reopen; all frame cases
  above execute after reopening. The 0.0.26 smoke also follows process restart.
- `FAIL` 0.0.24 first cleanup left its solid item. 0.0.25 fixes counting by
  resolving each layer's source item handle and then its item ID. The failing
  run and successful rerun both remain in the evidence.
- `FAIL` 0.0.27 owner deletion: AE reverts the reader selector to itself;
  the foreign-reference guard mistakenly treated that as an external user.
  `PASS` 0.0.28: deleting the original owner removes its reader and source;
  one Undo restores the orphan without immediately deleting it again; the
  second Undo restores the owner and binding. 22 local diagnostic tests pass.
  A subsequent 32-bpc stroke/merge/repeater frame also matches all 480000
  reference words on the installed 0.0.28 artifact after the Undo sequence.
- AE 2025.6.6x4: `BLOCKED` StreamSuite7 acquisition (`kSPSuiteNotFoundError`).
  A normal property-selected preset changes unrelated values; a reduced
  parameter preset exported by AE 26.5 is refused as a newer file. These are
  rejected compatibility experiments. User-approved minimum is now AE 26.5+
  for this new feature only. The temporary AE 2025 diagnostic was removed and
  its original production AEX hash remains unchanged.
- Independent verification: `python spike/host-outline/verify-reader.py
  docs/audits/evidence/host-coverage-reader-20260920` checks the compressed raw
  alpha arrays, nonempty references, changed modifier pixels and lifecycle
  summary. Earlier direct-source verifier still passes.
- `NOT_RUN` / implementation remaining: production shader resource and UI
  integration, immediate-before-idle duplicate readiness, foreign-consumer
  protection in host tests, adopted/modified-helper policy, expanded canvas,
  ROI/downsample/PAR, full FFX reuse, aerender and true MFR concurrency. The
  diagnostic is fixture-guarded by comp prefix `HS_reader_`; it is not shipped
  automatic coverage. No new commit or push.

[Reader audit](audits/evidence/host-coverage-reader-20260920/README.md),
[manifest](audits/evidence/host-coverage-reader-20260920/manifest.json).


### TR-COVERAGE-INTEGRATION-001 — production resource foundation

- Baseline: public `922845c` plus the frozen production source; Windows,
  Rust 1.97.1, 2026-09-23, `codex/host-coverage-probe`.
- `PASS`: `cargo test --offline --locked --lib` and the same command with
  `--features editor`: 237 tests each. Coverage-specific checks include both
  frontends, no uniform words, isolated pool capacity, aliases, read-only graph
  rules, E7 temporal refusal, kind-tag persistence, unchanged old parameter order,
  readiness errors, origin/stride/padding, all U15 values and exact float words.
- `PASS`: `cargo +1.97.1 build --release --locked --offline
  --target x86_64-pc-windows-msvc`.
  Candidate SHA-256 `8f9fe9fa6889b1d290e47483d545e13aef1f97ad650491c2f1cc6e7ee141e286`.
- First topology tests failed because the historical-order helper included the
  new pool and expected no appended fields. The correction keeps the old order
  comparison exact and separately asserts the two appended hidden parameters.
- `NOT_RUN`: production AE execution and every production FFX/MFR/ROI gate.
  This candidate was not installed; the native diagnostic remains separate.
  Native ownership/helper hookup does not yet publish ready state, so ordinary
  coverage declarations intentionally remain E60 rather than rendering fake data.
- No commit, push or deployment. `rustfmt` is unavailable in the pinned local
  toolchain; formatting was not claimed as a passed check.
- [Summary](audits/evidence/coverage-integration-20260923/raw/summary.json),
  [default tests](audits/evidence/coverage-integration-20260923/raw/tests-final.log),
  [editor tests](audits/evidence/coverage-integration-20260923/raw/tests-editor-final.log),
  [build](audits/evidence/coverage-integration-20260923/raw/build-release-final.log),
  [first failure](audits/evidence/coverage-integration-20260923/raw/tests-first.log),
  [manifest](audits/evidence/coverage-integration-20260923/manifest.json).
- Audit: [production integration](audits/11-host-coverage-integration.md).

### TR-COVERAGE-STAGE-002 — production SDK stage adapter

- Baseline: public `922845c` plus TR-COVERAGE-INTEGRATION-001 and the frozen
  adapter overlay; Windows, Rust 1.97.1, 2026-09-23.
- `PASS`: default and editor `cargo +1.97.1 test --offline --locked --lib`
  with `DYNAMICFX_AESDK_265_ROOT` set to the ignored SDK 26.5 root: 239 each.
  The initial default invocation omitted `--locked` to add the already-cached
  `cc` build dependency to the local ignored lockfile; no network was used.
- `PASS`: the default library tests without the SDK variable: 239. Coverage
  capability now requires both the compiled bridge and the host suite; a build
  without the bridge cannot advertise this feature.
- `PASS`: 22 SDK-header callback tests in
  `spike/host-outline/stage-check`, using `cargo +1.97.1 test --offline
  --manifest-path spike/host-outline/stage-check/Cargo.toml --target-dir
  scripts/out/coverage-stage-20260923/check-target`. The tests include the exact
  production C++ adapter and mock the officially declared suite callbacks.
  Read-only/idempotent calls, atomic layer/stage writes, wrong readback,
  acquisition/read/write/disposal failures and balanced handle lifetimes pass.
- `PASS`: SDK-enabled `cargo +1.97.1 build --release --locked --offline
  --target x86_64-pc-windows-msvc`. Candidate SHA-256
  `b9e4db45517fad8dbf1d85aade4f3cc3c0309783f36f3bd6a85ae7e1738dd6cd`.
- `NOT_RUN`: this candidate in AE. The manager does not yet call the binding
  API, and compiler dead-code warnings accurately reflect that unconnected API.
  No install, commit, push or ready-state publication. Mock callbacks are not
  host evidence or proof that production coverage works.
- [Evidence and commands](audits/evidence/coverage-stage-20260923/README.md),
  [manifest](audits/evidence/coverage-stage-20260923/manifest.json).

### TR-COVERAGE-OWNER-003 — production first frame and initial ownership

- Baseline: public `922845c` plus resource/stage foundations and the frozen
  ADR-0051 owner/reader overlay. Windows, Rust 1.97.1, 2026-09-23.
- `PASS`: SDK-enabled default/editor and SDK-absent default library suites,
  241 tests each. Native reader component: 3 pixel/checkout tests. Both Release
  builds pass. Earlier compile failures (PiPL builder/trait signatures and
  wrapper method names) are retained in the evidence.
- Host: AE 2026 **26.5x89**, accessed only through MCP. The empty, unmodified
  project was verified before scheduled quit; old production AEX was backed up.
  Installed only under AE 2026 `Support Files/Plug-ins/DynamicFx`:
  main `e72ac9379c24ffa01314e339cd66587ea820250ccc76fbe9bf43bb6aa1cd7874`,
  reader `da16133dff6cafe4057f088c89ac9e535c37511d3fa033d353f23ddc23e66546`.
  Diagnostic `bc0265d2...` is unchanged and only records native frame words.
- `PASS`: automatic hidden reader creation and ready/binding readback on both
  an ordinary shape and an adjustment shape. The first setup call timed out;
  MCP later recovered and readback proved creation completed. It was not
  replayed. The initial inspection script hit a no-value group property;
  correcting the read-only script allowed successful inspection.
- `PASS`: 6 native pixel comparisons, ordinary/adjustment × 8/16/32 bpc.
  Each compares **480000 canvas alpha words** against an ordinary AE shape.
  Every comparison has zero differences. Shader output alpha comes from
  `hint:coverage` through the production graph; it is not a direct probe-only
  reader check. Fixtures include triangle edges, 37.125% fill, subtract mask,
  68.75% mask opacity, anisotropic feather and 4.25px expansion. References
  have nonzero partial alpha and a hole, independently checked by the verifier.
- `PASS`: 8 ownership cases: creation, shared reader, first opt-out, last
  opt-out/source cleanup, cleanup Undo, cleanup Redo, re-enable and duplicate
  original with independent links **after idle**. No immediate-copy claim.
- `PASS`: reopen the saved baseline and render the adjustment frame in the
  same JSX call, before a separate idle wait: 480000 float32 alpha words equal
  the reference. This is same-process reopen, not cold-process/aerender proof.
- `PASS`: disable DynamicFx in the adjustment fixture as a negative control:
  all 140000 checked-out words become opaque background alpha, differing from
  the reference at all 140000 positions. Restore the effect and save afterwards.
  This confirms the passing shader test is distinguishable from pass-through.
  Two preceding mixed-log captures are not accepted; the final control isolates
  diagnostic output to the target comp and the verifier requires exactly one frame.
- `PASS`: `python spike/host-outline/verify-production-reader.py
  docs/audits/evidence/coverage-owner-20260923` independently recomputes all
  6 comparisons, reopen comparison and 8 lifecycle assertions from raw evidence.
- `NOT_RUN`: copy/FFX-before-idle authorization, full helper tamper/foreign
  reference protection, original deletion, empty alpha, large/expanded sources,
  animation/3D/modifier production matrix, ROI/downsample/PAR, aerender and
  actual concurrent MFR, final glass material and packaging. No push/release.
- [Evidence](audits/evidence/coverage-owner-20260923/README.md),
  [manifest](audits/evidence/coverage-owner-20260923/manifest.json),
  [audit](audits/11-host-coverage-integration.md).

### TR-COVERAGE-READINESS-004 — copied/FFX state, cache and offline authorization

- Windows, AE 2026 26.5x89, Rust 1.97.1, 2026-09-23. Public `922845c` plus
  the production overlays and ADR-0052..0055. Final installed main SHA-256
  `96dcff77f3b07c1e59cad1785120dcac4d7e3a8dabdcc798ac5ef2ba286a679c`;
  native reader remains `da16133d...`, diagnostic remains `bc0265d2...`.
- `FAIL` reproduced on `e72ac937...`: duplicate an adjustment layer, change
  its triangle and render in the same JSX call. Copied ready=1 still points to
  the original reader; 42331 of 480000 float32 alpha words differ. The first
  resetup-only certificate attempt `9e922d84...` fails identically: AE can
  construct the render copy directly from original flattened data.
- `PASS`: process-wide revocation refuses the stale certificate with E60.
  An observed-absence cache cleanup fixes automatic recovery when a reopened
  test project reuses a deleted layer ID. The earlier protected-but-unrecovered
  `6d718619...` result is preserved separately.
- `PASS` on the final artifact: warm original frame, duplicate and change shape
  within the same bounds, then render without purging caches. The 140000
  checked-out words are the expected opaque pass-through and the log reports
  E60, rather than the old coverage. After idle, binding points to the new
  original and all 480000 alpha words equal its reference.
- `PASS`: FFX applied to a fresh target and to an existing DynamicFx instance
  (effect count stays one). Both immediate renders refuse the old state with
  E60; both automatically rebound frames match all 480000 reference words.
- `FAIL` first offline attempt on `152fb695...`: aerender returns 0 but the
  prepared frame is E60/pass-through, differing at 87500 words. Normal resetup
  occurs before render-only resetup. PF_IsRenderEngine fixes that distinction.
  A following run used old disk cache and emitted no render callbacks; it is
  not accepted. Effect cache generations are advanced and effective readiness
  is mixed into the host frame GUID through GuidMixInPtr.
- `PASS`: final independent aerender process, MFR OFF, one prepared 32-bpc
  frame: return 0, zero E60 entries, all 480000 native words equal reference.
- `PASS` save-time validation: a copy saved in the same JSX call is already
  validated by the time AE writes the file (saved certificate and reader/owner
  readback agree); its independent frame is exact. The harness initially
  expected E60 and printed FAIL; that premise is rejected, not relabeled as an
  unverified-data rejection. The raw result and assessment are retained.
- `PASS` negative control: remove that copy's reader and save. Independent
  rendering keeps E60 and opaque pass-through (140000 words); saved/copy state
  cannot authorize a missing reader. Exit status alone is not a success gate.
- `PASS`: final ordinary/adjustment × 8/16/32-bpc regression, 6 pairs × 480000
  words, zero differences. Default SDK, editor SDK and default no-SDK suites
  each pass 249 tests; SDK Release builds successfully.
- The initial FFX harness assumed copyToComp inserted at index 1 and hit a
  locked reader; readback located the actual new shape ID and execution resumed
  there. A localized TIFF template failed; the existing ASCII Photoshop template
  was used. These harness failures and the missing-project render attempt remain.
- `PASS`: `python spike/host-outline/verify-coverage-readiness.py
  docs/audits/evidence/coverage-readiness-20260923` independently checks native
  pixels and E60 records. Hash/publication-pattern and governance checks pass.
- `NOT_RUN`: the remaining full production animation/3D/modifier, empty-alpha,
  ROI/downsample/PAR/expanded-source, helper-tamper and genuine concurrent MFR
  matrix, final glass material/packaging and broader host regressions.
- [Evidence](audits/evidence/coverage-readiness-20260923/README.md),
  [manifest](audits/evidence/coverage-readiness-20260923/manifest.json).

### TR-COVERAGE-FIDELITY-005 — production geometry and expanded coverage

- Windows AE 2026 26.5x89, Rust 1.97.1, 2026-09-23. Public `922845c` plus
  production overlays. Installed main
  `07a79e31c1b881fa48d0b7ed8ae621ddaea74e1ba45a5ac28fb9083452eec1ba`,
  reader `0a1f7c4c11d501ecd148fcfb2a62688ff31e5c1009130ae437640e8a9d2c75cf`,
  probe 0.0.29 `ce965bf891e166f0be6fe38a8107853b2782fe94a66c3c413cdda0d5d9e4c559`.
- `PASS`: 78 matrix pairs, repeated on repaired artifacts: curved path/opacity
  at shuffled 1/0/0.5/0.36 seconds with CTI 1.6; 2D/3D; transformed repeater,
  merge hole, stroke; Half/Quarter and PAR 1.333333; both hosts/all depths.
  Every native alpha word agrees. Modifier references change measurably;
  Half/Quarter output dimensions are verified.
- `PASS`: 24 native pairs for negative group scale/rotation/skew, transformed
  parents and expression-driven curves at 1/0.24 seconds.
- `FAIL`: ordinary expanded reference triggers the supplied GUID dialog.
  ADR-0056 moves one GuidMixInPtr call to entry for every SmartPreRender;
  non-coverage graphs mix a fixed constant. Main suites each pass 249 tests.
- Diagnostic correction adds IExpandBuffer, origin-aware forwarding and a
  1048576-pixel budget (22 tests/build pass). This alone did not cure the GUID
  assertion. A reference shader red-versus-alpha mistake is separately retained.
- `FAIL`: real outside-comp clipping loses 3548 U8 or 3554 U15/F32 pixels.
  The test shifts exterior sampling into the visible frame. Reader repair
  unions base/source extents and request-clips results, with IExpandBuffer.
- `PASS`: all 12 corrected unshifted/shifted host/depth pairs match exactly;
  reader tests pass 4 and Release passes.
- `PASS`: 15 ROI sampleImage comparisons equal the independent reference.
  A footprint average is not asserted equal to an arbitrary single full pixel.
- `PASS`: empty ordinary output and unchanged adjustment background. Initial
  absolute blue=255 expectation fails at 32 bpc; the disabled-layer background
  also exports blue=25 with zero RGBA difference. Both results are retained.
- Independent verifier recomputes 114 native pairs, 15 ROIs and retained clipping
  failures. Original 39 ordinary/reference PNG alpha comparisons also match.
- User dismissed the GUID dialog. MCP quit later timed out; only the exact
  launched process on saved full.aep without a dirty marker was terminated
  under lifecycle authorization. Backups precede replacements; launches hidden.
- `NOT_RUN`: helper tamper/foreign references, true concurrent MFR, remaining
  compatibility/material/packaging. User orders shader authoring after complete
  regression. No commit, push or main update.
- [Evidence](audits/evidence/coverage-full-20260923/README.md),
  [manifest](audits/evidence/coverage-full-20260923/manifest.json).

### TR-COVERAGE-LIFECYCLE-MFR-006 — helper validation and shared dispatch

- Windows AE 2026 26.5x89, Rust 1.97.1, 2026-09-23, `922845c` plus overlay.
- `FAIL`: main `07a79e31...` keeps a valid certificate after a 35-pixel helper
  move and idle. Transform/time/source validation repairs that ownership gap.
- `PASS`: main `314e4fe1...`, 18 changed/restored helper cases: position,
  anchor, scale, rotation, opacity, expression, keyframes, separated dimensions,
  start/stretch/in/out, effect disable, marker, mask, parent, adjustment and
  visibility. Conflict state 3 preserves the same helper; restoration revalidates.
- `PASS`: shared-dispatch main `206ce4e2...` passes 9 lifecycle cases: foreign
  selector preservation, reused reader, shared-source preservation, fresh opt-in,
  modified opt-out preservation/recovery, orphan cleanup and two-step Undo.
  A harness dereferenced the deleted owner after the successful deletion; the
  failed call is preserved and fresh readback continues without repeating it.
- ADR-0057 changes both effects to shared-reference dispatch. Default/editor/
  no-SDK suites pass 250 each; reader passes 4. Release builds pass.
- `FAIL`: the initial concurrent diagnostic wrote main/reader spans to one file;
  interleaved begin lines prevent balanced-span acceptance, although all native
  frame digests agree. Separate files plus buffered line writes repair the trace.
- `PASS`: final main `a17d714b55d77f1eb920fb0f356fb6f9b69b0b801022ea6dc56b6d2be0d8a846`,
  reader `d251a3608d2bd1931aa2ea6d72a3f7727b8d82bfd6e4bb141cc6ece904e7be8b`,
  ordinary/adjustment × 8/16/32 bpc, 48 frames each: all 288 frame digests are
  bit-identical between independent serial and MFR runs. Output hashes exclude
  row padding but cover all native ARGB words. Every sequence has 48 distinct
  frame hashes, zero E60 entries and balanced spans. MFR peaks are 9–12 overlapping
  callbacks on 12–14 threads; serial peaks are 1. Probe effects are removed.
- Raw local commands/results: `scripts/out/coverage-lifecycle-20260923/`:
  `run.py tamper`, `foreign.py` plus recorded continuation, `mfr-run.py` (OFF/ON
  for every kind/depth, `MFR_RUN_TAG=-v2`), `mfr-summary.json`, installation hashes.
- Current-artifact fidelity/readiness/resource regression and material remain
  in progress. No commit, push, main update or release.

### TR-COVERAGE-COMPLETE-007 — cancellation and usable Liquid Glass

- Windows AE 2026 26.5x89, Rust 1.97.1, DX12, 2026-09-23. Final installed
  main `6d188d2251fdecf89f18611fd429cbc4cd9b8c056dc3ad8f25dc35f74eca6a97`,
  reader `29695b72fc8a26222a51319c3e42f86696b6d15e78efb02d968bc264ff173be6`.
- `PASS`: independent verifier recomputes 174 native-alpha comparisons, 15 ROIs,
  3 empty-depth controls and 3 copy/FFX recovery images. Added 54 cases cover
  open strokes, trim, offset, round corners, disjoint contours, animated mask
  opacity/feather/expansion and a transformed 3D camera. Six pairs use WGSL;
  both frontends explicitly reject coverage plus `prev` with E7.
- `PASS`: ADR-0058 frame-owned checkout IDs and scoped release. Three fault
  tests cover cancellation, failed checkin and unwind, including empty successful
  input. SDK default/editor and no-SDK suites each pass 254; reader passes 4.
- `PASS`: repeated 288-frame serial/MFR matrix after checkout repair. All native
  ARGB digests match both serial and preceding correct output; peaks 10–12
  concurrent callbacks, balanced spans, zero E60. Eighteen additional native
  baseline/expanded comparisons and 15 ROIs pass on that implementation.
- `PASS`: actual background preview interruption generates two external-input
  cancellations; 105 surrounding cache-served PNGs match purged re-renders
  exactly. Foreground remains unchanged. Earlier Ctrl+C/WM_CLOSE trials are
  retained as `UNVERIFIED_CANCEL`, not counted as host-cancellation proof.
- `PASS`: existing GLSL/WGSL features pass 330 numeric checks, and 217 checks
  repeat after checkout repair. Hidden gradient children initially reject both
  direct and expression writes. Idle child-visibility synchronization repairs
  direct writes without opening a foreground panel. Save/reopen and keys pass.
- `PASS`: 18 helper mutations, 9 lifecycle/foreign-selector/shared-source/Undo
  cases, plus four final parent/matte-reference cases. Modified objects and
  known external references are preserved; unreferenced valid helpers clean up.
- `PASS`: Liquid Glass GPU Full/Half/Quarter × 8/16/32; Amount zero has zero
  float error; 256000 encoded field pixels have identical bytes through 8/32
  intermediate targets. Eighteen actual AE frames have the expected physical
  sizes and animate. Partial alpha is applied once (measured error <0.0001),
  outside/hole pixels remain exact, and FFX binds to renamed arbitrary Beziers.
  Applying the supplied script twice leaves one effect. FFX credential/path
  scans find nothing. Temporary test objects are removed from the saved example.
- `PASS`: exact final installed pair, without the diagnostic AEX, independently
  renders the 1440×900 example with zero channel differences from the accepted
  formal output. The final rebuild's executable/data/unwind/resource/relocation
  sections equal the preceding verified pair; attribution/debug metadata changed.
- The host's 32-bpc `saveFrameToPng` is dark even with glass disabled. Native
  sample values and formal queue output agree; shader brightness is unchanged.
  Display evidence is decoded from formal PSD output, not that faulty preview.
- Package ZIP CRC, both AEX hashes, sample members and separate dependency
  notices are checked before publication. AEX/SDK/AEP/intermediate data stay local.
- [Completion evidence](audits/evidence/coverage-completion-20260923/README.md),
  [manifest](audits/evidence/coverage-completion-20260923/manifest.json),
  [material usage](liquid-glass.md). Existing Source Undo limitation remains;
  this does not add acceptance for other AE years or macOS.
