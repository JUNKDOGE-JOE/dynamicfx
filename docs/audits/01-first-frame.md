# Audit 01: New-architecture First Frame

- Milestone: M1 — New-architecture First Frame
- Audit state: Complete (M1 exited 2026-08-12: first frame verified on AE 2025)
- Baseline branch: `codex/stabilize-programmatic-flow`
- Audit date: 2026-08-12 (created at milestone entry)
- Related roadmap: [../ROADMAP.md](../ROADMAP.md)
- Related test matrix: [../TEST_MATRIX.md](../TEST_MATRIX.md)
- Related ADRs: [ADR index](../adr/README.md) — entry gates 0010-0014 all Accepted

## Outcome

**M1 is complete; exited 2026-08-12.** The new architecture rendered its first verified frame on After Effects 2025: a scripted (no-UCP) gradient-shader expression was observed by the §5.3 idle observer, classified (ADR-0012), parsed and ABI-validated by the GLSL frontend (ADR-0011), lowered to a one-pass `EffectDefinition` with a fresh `BindingPlan` (ADR-0013), compiled to SPIR-V, transported to the render clone through the session StateToken + process registry, and rendered on DirectX 12 (ADR-0014) — with all three gradient probes numerically exact (UV origin, texel centers, and row order confirmed on first contact). Invalid source produced a visible diagnostic, a zeroed token, and a byte-exact pass-through frame. TR-M1-001..004 are all `PASS` on artifact `BDDB51F1…`; complete records live in [TEST_MATRIX.md](../TEST_MATRIX.md).

The aerender leg measured the designed pre-M3 behavior: a fresh aerender process has no session registry, so the render clone fails closed to pass-through; shader output under aerender arrives with M3 persistence.

The rewrite that produced this frame:

- host-agnostic domain modules: ADR-0010 Language registry, ADR-0012 source classifier and size limits, ADR-0013 ParamId grammar + v1 pool table + fresh `BindingPlan` (`src/frontend/`, `src/definition/`, `src/binding.rs`);
- the `LanguageFrontend` trait with the GLSL frontend (`src/frontend/glsl.rs`): naga `glsl-in` parse/validate, ABI v1 interface checks (three-member `FxUniforms` head at offsets 0/8/12, reserved-binding rejection for set 0 bindings 3-15 and sets ≥ 1), reflection of user members into ADR-0013 declarations;
- minimal `EffectDefinition`/`RenderGraph`/`PassDefinition` with raw-source → one-pass-graph lowering (`src/definition/effect.rs`);
- the ADR-0013 AE parameter topology (`src/host/params.rs`): 5 head parameters + 104 pool slots derived from `binding::V1_POOLS`, with the declaration order under unit test as the persistent index contract;
- a rewritten `src/lib.rs` shell: UserChangedParam/UpdateParamsUi observe the committed expression via AEGP, classify (ADR-0012 fail-closed), run the frontend, lower to a definition, and mirror status text; the Compile button forces re-observation; `flatten()` persists zero bytes (ADR-0009: nothing persists before the M3 schema ADR) and `unflatten()` discards prototype payloads (ADR-0004).

- the GPU path: `src/render.rs` rewritten to ABI v1 (three-member builtin head at offsets 0/8/12, frontend-reflected user layout, SPIR-V emission) with the ADR-0014 backend policy (DX12 only; `DYNAMICFX_BACKEND` diagnostic override; adapter identity logged into evidence);
- session transport: 51-bit session token → hidden StateToken stream; UI commits publish via ParamDef, the idle observer mirrors via `AEGP_SetStreamValue` for scripted writes, render clones resolve through the process registry and fail closed on a miss;
- the idle observer (`src/host/idle.rs`, ported from the prototype bridge): per-instance `PF_Cmd_COMPLETELY_GENERAL` observation plus token sync;
- the prototype transport is gone (SourceChannel, legacy SourceData, flattened v1-v3, sidecar, hash registry, `compile.rs`, annotation `params.rs`).

38 unit tests pass; the release build is warning-free; `rust-toolchain.toml` pins 1.97.1 (ADR-0014 §1).

## Visible evidence

| Evidence | Path | What it proves |
|---|---|---|
| Gradient first frame | [evidence/m1-first-frame/ae2025/m1c_gui.png](evidence/m1-first-frame/ae2025/m1c_gui.png) + [checks.txt](evidence/m1-first-frame/ae2025/checks.txt) | Non-pass-through 8-bpc frame through the new graph path; three probes within tolerance (two exact) |
| Pass-through frame | [m1d_invalid.png](evidence/m1-first-frame/ae2025/m1d_invalid.png) | Invalid source fails closed byte-exact |
| Scenario logs | [m1a](evidence/m1-first-frame/ae2025/m1a.log)/[m1b](evidence/m1-first-frame/ae2025/m1b.log)/[m1c](evidence/m1-first-frame/ae2025/m1c.log)/[m1d](evidence/m1-first-frame/ae2025/m1d.log)/[m1e](evidence/m1-first-frame/ae2025/m1e.log) | Topology, popup rejection, save/reopen, token lifecycle, diagnostic text |
| Plugin log | [dynamicfx_plugin.log](evidence/m1-first-frame/ae2025/dynamicfx_plugin.log) | Idle observation → token publish → registry resolve → DX12 adapter identity → pipeline build; invalid → token 0 |
| aerender leg | [m1_ar_00000.psd](evidence/m1-first-frame/ae2025/m1_ar_00000.psd) + [aerender_m1.txt](evidence/m1-first-frame/ae2025/aerender_m1.txt) | Fresh render process fails closed to pass-through (pre-M3 design behavior), exit 0, no crash |

## Baseline

- Branch: `codex/stabilize-programmatic-flow` (working tree uncommitted at run time; parent commit `de401a2`)
- Entry contracts: ADRs 0010-0014, all Accepted before any M1 code
- Toolchain: 1.97.1 pinned in `rust-toolchain.toml`; target `x86_64-pc-windows-msvc`
- Artifact: `DynamicFx.aex` 8,160,256 bytes, SHA-256 `BDDB51F1A349ED3ED96F4A18587C687A2986BE40759F4361DC6734999ECC58D4`, installed via elevated `scripts/install.bat 2025`; installed hash verified equal to the build (a superseded same-day artifact `6E8F2345…` and its run are archived in `scripts/out/m1/2025_run1_6e8f2345/`)
- OS: Windows 11 Pro 10.0.26200
- Host: After Effects 2025 (25.6.6, zh_CN) + aerender 25.6.6; GPU adapter "NVIDIA GeForce RTX 5080", backend Dx12, driver 32.0.15.9621
- Other hosts: AE 2026 `NOT_RUN`; AE 2023/2024 `BLOCKED` (not installed; release gate per ADR-0014 §7)

## Code paths

Target-architecture modules:

- `src/frontend/mod.rs` — ADR-0010 `LanguageId` registry and popup mapping; `LanguageFrontend` trait, `PassModule`, `FrontendError` classes, `frontend_for()` registry lookup;
- `src/frontend/envelope.rs` — ADR-0012 `classify()` (BOM/whitespace scan, `@dynamicfx` prefix, fail-closed marker parsing), `MAX_COMMITTED_SOURCE_BYTES`/`MAX_SNAPSHOT_BYTES`;
- `src/frontend/glsl.rs` — GLSL frontend: naga parse/validate, ABI v1 head/binding checks, user-parameter reflection;
- `src/definition/param.rs` — ADR-0013 `ParamId` grammar with reserved names, `ShaderParamType` → pool-slot mapping (vec4 = Color+Float pair), shared ID/alias namespace validation;
- `src/definition/effect.rs` — minimal `EffectDefinition`/`RenderGraph`/`PassDefinition`, `lower_raw_single_pass()`;
- `src/binding.rs` — `V1_POOLS` single configuration source (104 slots), `build_fresh()` with atomic pool-overflow rejection;
- `src/host/params.rs` — `ParamKey`, `declaration_order()` (the persistent index contract, unit-tested), `setup()` deriving `PARAMS_SETUP` from `V1_POOLS`;
- `src/lib.rs` — rewritten host shell (observation pipeline, status mirroring, zero-byte flatten, pass-through render).

Removed: `src/idle.rs` and the prototype transport inside the old `lib.rs` (SourceChannel encode/decode, legacy arbitrary parameter, flattened v1-v3 codec, hash registry, sidecar). Retained under `#[allow(dead_code)]` for the GPU step to absorb: `src/compile.rs`, `src/render.rs`, `src/params.rs` (annotation parser; rewritten per ADR-0013 at M2). `src/source.rs` (backtick extraction) stays active.

## Contracts fixed or changed

None. M1 implements Accepted contracts; M3/M4/M6-staged contracts stay session-local and non-persistent during M1.

## Commands and exact host steps

All on 2026-08-12:

```text
cargo test --all                                  # 38 passed; 0 failed
cargo build --release                             # zero warnings
scripts\install.bat 2025                          # elevated; hosts verified closed first
pwsh scripts/m1/run_m1.ps1 -Year 2025             # warm AE; scenarios m1a..m1e + quit
pwsh scripts/m1/run_m1.ps1 -Year 2025 -Aerender   # aerender on the staged m1_ar.aep
pwsh scripts/m1/run_m1.ps1 -Year 2025 -Checks     # numeric PNG/PSD probes
```

Interim development runs earlier the same day (41 → 45 tests before the prototype
modules were absorbed/removed) are visible in this file's history; the final
suite on the shipped artifact is the 38-test run recorded in TR-M1-001.

## Observed evidence

- TR-M1-001 target Rust unit suite: `PASS` (38/38)
- TR-M1-002 target release AEX build: `PASS` (artifact identity above)
- TR-M1-003 Language ID/default GLSL on AE 2025: `PASS` (110 properties, popup position 1, `setValue(2)` rejected, save/reopen stable)
- TR-M1-004 raw GLSL first frame through the graph path on AE 2025: `PASS` (gradient probes exact/within tolerance; invalid source diagnostic + byte-exact pass-through; aerender leg measured fail-closed pass-through)

Complete result records: [TEST_MATRIX.md](../TEST_MATRIX.md).

## Findings and failures

| Severity | Finding | Evidence | Disposition |
|---|---|---|---|
| Medium (harness defect, fixed) | `app.project.workingSpace = "None"` stores a literal missing-profile name; AE later raises a modal "profile None missing" dialog that blocked one harness run | run-2 timeout; user-observed dialog | Scripts fixed to assign `""` (disables color management); the two run-1 `.aep` files in the archived folder still carry the bad profile — do not reuse them |
| Low | Status text lands asynchronously: observation in idle/render contexts must not touch parameters, so the text appears on the next AE UI refresh (m1c read "idle" before the refresh; m1e read the diagnostic after it) | m1c/m1e logs | Accepted for M1; the M3 StateToken/diagnostic-code work gives render-side status a first-class carrier |
| Low | PF parameter names truncate at 31 chars, so long diagnostics clip (`Status: GLSL error: Error { kin`) | m1e log | Stable short diagnostic codes are the M3 registry's contract; text is interim |
| Expected | aerender renders pass-through: a fresh render process has no session registry and the persisted StateToken value is deliberately meaningless across sessions | PSD probes | By design pre-M3 (ADR-0009 keeps staged contracts session-local); M3 persistence closes it |

## Known limitations

- Shader output exists in the GUI session only: aerender and reopened projects pass through until M3 persistence (sequence snapshot + real StateToken layout).
- Pool slots are validated and value-read at render, but the UI is not yet configured from a `BindingPlan` (labels/ranges/visibility, rename/alias stability) — M2 scope, TR-M2-001.
- Bound-value encodings (point normalization, angle units, color space, int rounding) ride the prototype conventions until M2 fixtures pin them (ADR-0013 §8).
- The 31-char PF name limit truncates long diagnostics; render-side status carriage is interim until M3.
- 16-bpc renders reuse the 8-bpc conversion path unverified by fixtures (M5 owns precision claims).
- AE 2026 (plus the M0 spike re-verification) and AE 2023/2024 remain unmeasured for every M1 row.

## Residual risks

- DX12 evidence covers one adapter (RTX 5080); other adapters/drivers may behave differently — the Unavailable diagnostic path exists but is untested on real hardware without DX12.
- AE 2023 SDK/host behavior remains unmeasured and could surprise the topology or transport (tracked M0 residual).
- The 110-property tree loaded cleanly on AE 2025, but tail-append growth across builds (ADR-0013 §5) is still unmeasured on any host.
- The idle observer walks every project item once per second; large projects' scan cost is unmeasured (M7 owns performance).

## Decision changes

None.

## Next exact action

Begin M2 (Keyframed Parameters): apply the `BindingPlan` to the AE UI (slot labels/ranges/visibility from declarations), read keyframed stream values per frame through the bound slots, implement slot reuse + alias inheritance against the previous plan, pin the value-encoding fixtures (ADR-0013 §8), and extend the harness with the keyframed-parameter scenario (TR-M2-001). Track in `audits/02-keyframed-params.md` and [IMPLEMENTATION_STATUS.md](../IMPLEMENTATION_STATUS.md).

## Reproduction

A new session can reproduce the M1 result by: `cargo test --all` (38 green) and `cargo build --release`; verifying the artifact SHA-256; installing via `scripts/install.bat 2025` with hosts closed; running the three `run_m1.ps1` passes (GUI, `-Aerender`, `-Checks`); and comparing the numeric probes against the expectations coded in the driver. The curated evidence set under `evidence/m1-first-frame/ae2025/` is the reference for what a passing run looks like.
