# Audit 06: Temporal Feedback

- Milestone: M6 — Temporal Feedback
- Audit state: Complete — exited 2026-08-13 (TR-M6-001 PASS; ADRs 0023/0024/0025 Accepted)
- Baseline branch: `codex/stabilize-programmatic-flow`
- Related roadmap: [../ROADMAP.md](../ROADMAP.md)
- Related test matrix: [../TEST_MATRIX.md](../TEST_MATRIX.md) — TR-M6-001
- Related ADRs: [0023](../adr/0023-temporal-seek-reset.md) (`prev` surface; state model superseded), [0024](../adr/0024-history-format-policy.md) (format rules; storage model superseded), [0025](../adr/0025-windowed-resimulation.md) (the shipped semantics)

## Outcome

Temporal feedback works and is **provably order-independent**: `prev` in a graph makes each frame re-simulate `min(F+1, W)` iterations from black (ADR-0025), so value(F) is a pure function of the frame — verified exact at 8 and 32-bpc under deliberately shuffled interactive reads, across the **measured-out-of-order MFR render queue (25/25 frames exact)**, and in a **fresh aerender process (25/25 exact)**. `@window` rides the source like everything else; recompiling with a different W moves the plateau. The plugin declares `SUPPORTS_THREADED_RENDERING` — no Effect Controls warning icon ([screenshot](evidence/m6-temporal/ae2025/m6_no_mfr_warning_effect_controls.png)) — and non-temporal shaders render fully concurrently.

The milestone consumed one design generation to get here, and that journey is the evidence trail below.

## Visible evidence

- [checks.txt](evidence/m6-temporal/ae2025/checks.txt) — the full numeric gate (fails=0).
- Sample PSDs: rq/ar frames 0, 15, 24 (ramp start, plateau edge, steady state; render queue and aerender byte-agree).
- [m6_no_mfr_warning_effect_controls.png](evidence/m6-temporal/ae2025/m6_no_mfr_warning_effect_controls.png).

## Baseline

- Verified artifact: `dynamicfx.dll` 8,438,272 B, SHA-256 `C7023854…`, Rust 1.97.1, install-hash verified. Host: AE 2025 v25.6.6 zh_CN, Win11 10.0.26200, RTX 5080/Dx12/32.0.15.9621, MFR at defaults.
- Entry state: M5 exit artifact `D9E91637…`.

## Code paths

- `src/frontend/grammar.rs`: `prev` reserved input (ADR-0023 surface — readable anywhere, never writable/nameable, always "available" to scheduling); `Envelope.uses_prev`.
- `src/frontend/annotation.rs`: `// @window <n>` (default 16, cap 64, reject-not-clamp).
- `src/plan.rs`: `TexSlot::History` outside the aliasing pool.
- `src/render.rs`: `TemporalTextures` ping-pong, `create_history_texture` (zero-init = the initial state), `execute_plan(readback: bool)` so intermediate iterations skip the CPU round-trip.
- `src/lib.rs`: `CompiledEffect.window`; the per-frame iteration loop (builtins advance per iteration: `u_frame` = F−n+1…F, `u_time` stepped by the frame duration); `CompiledEffect.source` = the exact committed text for snapshots (defect fix, below); thread-safety hardening for the MFR flag (locked log writer, thread-local ROI-window hand-off replacing the racy field).
- `build.rs`: `SupportsThreadedRendering`; subversion 3.
- Harness: `scripts/m6/` (scheduleTask fixture, shuffled-order probes, RQ + aerender legs, numeric gate).

## Contracts fixed or changed

Three ADRs in one milestone — an honest record of design-by-measurement:

1. **ADR-0023** (session chain: continue iff `F == last+1`, else reset) — its `prev` grammar surface shipped intact; its state model was **refuted by TR-M6-001 run 1**.
2. **ADR-0024** (persistent basis/result pair) — format/aliasing/never-persisted rules shipped intact; the persistent pair became a per-render transient.
3. **ADR-0025** (windowed re-simulation) — the shipped semantics, drafted from the run-1 measurements, Accepted with user approval.

## Commands and exact host steps

`pwsh scripts/m6/run_m6.ps1 -Year 2025` → screenshot → `-QuitAE` → `-Aerender` → `-Checks`. Full record in TR-M6-001.

## Observed evidence

TR-M6-001 PASS: 22 shuffled interactive probes exact (both depths, plateau/backwards/repeats), `@window 8` recompile follows W, RQ 25/25, aerender 25/25, 96 windowed renders logged, no warning icon.

## Findings and failures

All archived (superseded runs under `scripts/out/m6/2025/`), all same-day:

1. **Session-chain death by measurement (run 1)** — interactive probes passed bit-exactly, then: the MFR render queue dispatched frames essentially unordered (0, 8, 1, 9, 2, 13, …) so nearly every frame reset; the no-flag A/B run showed TWO sequence clones alternating frames (state fragments even per-ordered-stream) plus different frame numbering in render contexts. Three independent kill mechanisms → ADR-0025. The 0023 fixture design did its job: the reset counter turned a design assumption into a measured refutation in one run.
2. **Snapshot stored only the first pass body** (M4-latent): `flatten` persisted `passes[0].source` instead of the committed text, so envelope sources reopened as raw single-pass effects — the accumulator sampled its *input* instead of `prev` and every aerender frame read exactly one step. Caught by the aerender leg's value math; fixed to `CompiledEffect.source` (ADR-0016's "exact source", finally literally). M3's reopen fixtures predate envelopes; M4/M5 never reopened one — the M6 aerender leg was the first to cross snapshot × envelope.
3. Host facts recorded: backwards interactive reads serve AE's frame cache (the historical value returns without invoking the effect — correct and cheap); `aerender` reports healthy `time_step` (1024/25600) contrary to the zero-guard concern, but resolves the effect **per frame** via fresh clones (19 rebuilds / 19 renders — snapshot-resolution cost noted for M7); the crate's `current_frame()` returns 0 when `time_step == 0` (guarded fallback needed if such a context appears).

## Known limitations

- v1 windows sample the **current** input every iteration (ADR-0025 §4): motion trails smear the current image; historical-input windows (`AUTOMATIC_WIDE_TIME_INPUT` + multi-checkout) are the recorded follow-up.
- Per-frame cost multiplies by n ≤ W; aerender additionally re-resolves the definition per frame. Both are M7 measurement targets.
- Per-render diagnostic logging (render-enter + one temporal line) is active in the verified artifact; M7 revisits.
- True unbounded accumulation is out of v1 (Compute Cache successor path recorded in ADR-0023/0025).

## Residual risks

- The windowed law is verified on AE 2025; other years inherit per-year rows.
- W=64 at 4K/32-bpc is a heavy frame (64 plan executions); no guardrail beyond the cap until M7 measures.

## Decision changes

ADR-0025 Accepted (supersedes 0023's state model and 0024's storage model, both marked); no other Accepted decision touched.

## Next exact action

Enter M7 (Performance, SmartRender, and MFR): define the measurement plan first — per-pass/per-frame timing evidence, ROI-aware smart rendering, render caching, aerender per-frame re-resolution, and the M5/M6 deferred items (idle-invalidation nudge, log-level policy).

## Reproduction

Follow the `CLAUDE.md` reading order; install the artifact per ADR-0014; run the four driver stages; expect `CHECKS_RESULT fails=0`.
