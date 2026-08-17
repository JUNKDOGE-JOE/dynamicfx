# Audit 05: 16/32-bpc Image Quality

- Milestone: M5 — 16/32-bpc Image Quality
- Audit state: Complete — exited 2026-08-13 (TR-M5-001 PASS; [ADR-0022](../adr/0022-16bpc-working-format-f32.md) Accepted)
- Baseline branch: `codex/stabilize-programmatic-flow`
- Related roadmap: [../ROADMAP.md](../ROADMAP.md)
- Related test matrix: [../TEST_MATRIX.md](../TEST_MATRIX.md) — TR-M5-001
- Related ADRs: [ADR-0021](../adr/0021-precision-alpha-color-policy.md) (Accepted), [ADR-0022](../adr/0022-16bpc-working-format-f32.md) (Proposed: 16-bpc working format correction forced by live wgpu evidence)

## Outcome

Deep color is real and bit-honest on AE 2025: 16-bpc renders land exactly (10291/32768-grade precision through a three-pass chain with two physical intermediates), 32-bpc over-white 2.0 and negative −0.5 survive end to end including through chain inverts (2.0 → −1.0 → 2.0), the 16-bpc boundary clamp behaves per ADR-0022, the same shaders run unmodified at all three depths, AE's effect-boundary alpha is **measured straight** at every depth, and an unmanaged 32-bpc project carries HDR values untouched. All 22 gated numeric probes PASS ([checks.txt](evidence/m5-pixel-formats/ae2025/checks.txt)).

Getting there consumed one working format and surfaced four host laws, all recorded below; the fixes are in the verified artifact `D9E91637…`.

## Visible evidence

- [m5_chain16_00000.psd](evidence/m5-pixel-formats/ae2025/m5_chain16_00000.psd) — the 16-bpc three-pass chain render (file itself 8-bit; the OM `Depth` key is scripting-read-only on this host).
- [m5_hdr32_preview.png](evidence/m5-pixel-formats/ae2025/m5_hdr32_preview.png) — the 32-bpc HDR band scene (8-bit preview; over-white/negative bands clip in the preview by design — the numeric truth is the probe log).
- The primary numeric evidence is in-memory: sampleImage expression probes read back through scripting at float64, [m5all.log](evidence/m5-pixel-formats/ae2025/m5all.log).

## Baseline

- Entry: M4 exit artifact `F0AFAE74…`; per-hop 8-bit quantization was the recorded M4→M5 handoff — now closed (the 16-bpc staircase pair stays distinct through the chain).
- Verified artifact: `dynamicfx.dll` 8,432,128 B, SHA-256 `D9E916378D44EC33DF7E242C3D92CBE22568467F617E1A4B3ED0FD80E993E4A7`, Rust 1.97.1, install-path hash verified.
- Host: AE 2025 v25.6.6 zh_CN, Windows 11 Pro 10.0.26200, RTX 5080 / Dx12 / 32.0.15.9621.

## Code paths

- `src/render.rs`: `Depth` (U8/U15/F32) → working formats Rgba8Unorm / Rgba32Float / Rgba32Float (ADR-0022); `FLOAT32_FILTERABLE` requested when offered; format-parameterized pipelines/intermediates/readback; exact converters (`u15_to_f32`/`f32_to_u15` with the AE-range clamp, bit-exact F32 lane reorder); pipeline creation inside a wgpu validation error scope (panic → diagnostic + pass-through).
- `src/lib.rs`: depth detection from world types with fail-closed mismatch; per-depth `PipelineSet` keying; full-frame GPU render + windowed write-back (ROI origins may be negative; underfilled outputs zeroed); SmartFX entry (`SmartPreRender` checkout of the full input + result-echo + full-frame `max_result_rect` + origin hand-off, `SmartRender` checkout/render/checkin); `passthrough` gained the F32 arm and a degenerate-extent fallback; per-render `render enter` diagnostic line.
- `build.rs`: `SupportsSmartRender` + `FloatColorAware` out-flags (float worlds only reach smart effects); effect subversion bump so AE re-reads the PIPL.
- Harness: `scripts/m5/` — single-script `m5all.jsx` scheduleTask state machine (immune to AE's second-`-r`-script rejection), post-compile probe arming, per-depth reads, `check_probes.py` gate, `m5x_cmprobe.jsx` host probe, driver with launch retry/settle and PSD archival.

## Contracts fixed or changed

- ADR-0021 held except its §1 16-bpc row: **wgpu refuses `Rgba16Unorm` as a render attachment** ("Format Rgba16Unorm is not renderable", live). [ADR-0022](../adr/0022-16bpc-working-format-f32.md) (Proposed) supersedes that row with Rgba32Float — exact for all U15 integers, zero per-hop quantization (stronger than promised), at ×4 memory instead of ×2 (equal to 32-bpc).
- ADR-0021 §4 measured contract: AE delivers **straight alpha** at 8/16/32-bpc on this host (probe shader sees r = 1.0 against a 50%-alpha red precomp; pre-effect buffer reads r = 1.0, a = 0.5).
- ADR-0021 §5 measured: unmanaged and sRGB-managed (classic engine) 32-bpc renders both carry 2.0 exactly — no host transform on float renders. The ACES leg is impossible as specified: the OCIO engine has no scripting surface on 2025, refuses 16-bpc from scripting, and ACES ICC names raise a modal "profile missing (83::0)" — deviation recorded, sRGB used as the managed case.
- Scope note: minimal SmartFX entry was pulled into M5 because `FLOAT_COLOR_AWARE` requires `SUPPORTS_SMART_RENDER` (AE API fact); performance-side SmartRender work stays M7 (recorded in the roadmap before implementation).

## Commands and exact host steps

`pwsh scripts/m5/run_m5.ps1 -Year 2025` (cold AE start; scenarios m5x + m5all), then `-Checks`. Host precondition: AE prefs new-project color engine flipped OCIO→classic (`colorManagementSystem 1→0`, `pcms 01→00` in `…\25.6\Adobe After Effects 25.6 设置-indep-general.txt`; backups kept). Full record in TR-M5-001.

## Observed evidence

TR-M5-001: 22/22 gated probes PASS, 2 record-only. Chain@16 exact to the last digit (0.314056396 = 10291/32768); staircase (973 vs 1075) distinct through intermediates; clamp 2.0→1.0 / −0.5→0 at 16; ±HDR exact at 32 direct and chained; 8-bpc canary exact with the predicted collapse; alpha straight ×3 depths; color pair recorded.

## Findings and failures

All were caught by the numeric probes or the diagnostic log, fixed same-day, and are visible in the archived failing runs (`scripts/out/m5/2025/*_2026*.log`):

1. **`Rgba16Unorm` is not renderable in wgpu** — first live 16-bpc render panicked in `create_render_pipeline`; the panic surfaced as a modal AE error dialog per render, which also deadlocks scripted `-r` sessions. Fix: ADR-0022 (16→Rgba32Float) + a wgpu validation error scope so any future validation failure becomes log + pass-through, never a dialog.
2. **SmartFX rect law** — AE rejects result rects larger than the request ("结果矩形不得超过… (25::237)"), yet echoing the tiny request as `max_result_rect` makes AE cache "empty" for the whole rest of the frame (later samples return permanent black without rendering). And `max_result_rect` must also CONTAIN the result: requests can start outside the layer (a padded comp render arrived as origin (−32,−24)). Fix: result = request, max = full-frame ∪ request, full-frame GPU render + offset windowed write-back (zero-filled underfill).
3. **ROI vs ABI** — sampleImage requests arrive as ~11×11 rects; rendering only the ROI would shrink `u_resolution`/uv (a 12×12 uv ramp was measured). The ABI's full-frame semantics are preserved by always rendering the full frame and windowing the write; ROI-aware scheduling is M7.
4. **Host caching quirks** — sliders evaluated before the idle compile cache their results essentially permanently in a scripted session (surviving token writes, expression re-assignment, and depth round-trips); the idle observer's hidden-token write does not invalidate frames. The fixture arms probe expressions strictly post-compile; one comp (`m5ramp`) still never re-evaluates at 16-bpc (its full matrix runs at 8-bpc; single-pass 16-bpc evidence rides the HDR comp). Product impact is nil for interactive use (UserChangedParam dirties), but **idle-published compiles not invalidating cached frames** is a real gap noted for M6/M7.
5. **Host facts recorded**: OCIO default on new projects (16-bpc refused from scripting; prefs flip required for the harness), OM `Depth` scripting-read-only (PSD artifacts 8-bit), 32-bpc PSD export fails with a modal, sampleImage integer coordinates are pixel centers, AE coalesces same-block bpc toggles.

## Known limitations

- Visible artifacts are 8-bit files; numeric truth lives in the probe log (OM depth unscriptable).
- Per-render `render enter` diagnostic logging is active in the verified artifact (useful for M6; revisit at M7).
- Every smart render currently renders the full frame regardless of ROI (correctness first; M7 owns ROI/caching performance).
- 16-bpc working memory is ×4 (ADR-0022 cost), bounded by ADR-0020 aliasing.

## Residual risks

- Alpha/color semantics are measured on AE 2025 only; other years inherit per-year rows (ADR-0021 anticipated this).
- The OCIO prefs flip is machine state, not project state: OCIO-default machines refuse scripted 16-bpc until flipped (users flip per-project in Project Settings; only scripting is blocked).
- The `m5ramp`@16 evaluation quirk is unexplained at root; it is fenced in the fixture and does not touch the render path (identical code renders `m5hdr`@16 correctly every run).

## Decision changes

ADR-0022 Accepted (user approval at the M5 report), superseding ADR-0021's 16-bpc working-format rows on live evidence.

## Next exact action

User review of the Proposed M6-entry ADRs ([0023](../adr/0023-temporal-seek-reset.md) temporal seek/reset, [0024](../adr/0024-history-format-policy.md) history format policy); on acceptance, the M6 implementation slice.

## Reproduction

Follow the `CLAUDE.md` reading order; install the artifact per ADR-0014; run `pwsh scripts/m5/run_m5.ps1 -Year 2025` then `-Checks`; expect `CHECKS_RESULT fails=0`.
