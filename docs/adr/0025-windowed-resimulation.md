# ADR-0025: Temporal v2 — windowed re-simulation

- Status: Accepted
- Date: 2026-08-13 (proposed from TR-M6-001 run-1 evidence; Accepted with explicit user approval)
- Owners: DynamicFX project
- Supersedes: the continuity/state model of [ADR-0023](0023-temporal-seek-reset.md) (its `prev` grammar surface stands) and the persistent-storage model of [ADR-0024](0024-history-format-policy.md) (its format/aliasing rules stand)
- Related tests/audits: TR-M6-001; `docs/audits/06-temporal-feedback.md`

## Context — what the fixtures measured

The ADR-0023 session-chain model (`prev` = last committed render, continue iff `F == last+1`) passed every interactive probe bit-exactly, then died on the render path, for three independently fatal host reasons, all captured in the M6 evidence:

1. **Order**: with `SUPPORTS_THREADED_RENDERING`, the render queue dispatched frames essentially unordered (measured arrival: 0, 8, 1, 9, 2, 13, 15, 11, 3, …) — nearly every frame reset.
2. **Clones**: with the flag removed (A/B probe), AE still fed TWO sequence-data clones alternating frames — per-instance state fragments even when each stream is locally ordered.
3. **Frame numbering**: render contexts report different frame numbers than interactive evaluation (doubled values measured in RQ/aerender logs) — even a perfectly ordered stream would fail the `last+1` test.

No amount of waiting, locking, or instance-identity plumbing fixes all three. The competitor study (ADR-0023 §3) is corroborated: shipping products avoid cross-frame session state entirely.

## Decision

1. **`prev` means windowed re-simulation.** For a temporal effect at frame F, the runtime computes the output **from scratch, within the frame**: starting from transparent black, it iterates the whole graph `n(F) = min(F+1, W)` times, with the ABI builtins advancing per iteration (`u_frame` = F−n+1 … F, `u_time` stepped by the frame duration); `prev` in iteration *i* samples iteration *i−1*'s final output (black for the first). The last iteration's output is the frame.
2. **Every frame is self-contained.** No cross-frame state exists anywhere — no store, no continuity rule, no resets, no instance identity. Consequently: MFR order/clones/numbering are all irrelevant; preview, render queue, and aerender are bit-identical by construction; **scrubbing shows the true value of any frame**; determinism is total.
3. **W comes from the source**, like everything else in DynamicFX: the annotation `// @window <n>` anywhere in the committed text (default **16**, cap **64**; a malformed or out-of-range value is a ParamRejected diagnostic, never a clamp). W is part of the compiled definition — changing it recompiles and re-caches normally.
4. **v1 input semantics: the current frame's input is used for every iteration.** Historical-input windows (sampling the layer at F−k per iteration, the full Echo shape, via `AUTOMATIC_WIDE_TIME_INPUT` + multi-checkout) are the recorded follow-up slice — the plumbing is additive and does not change this ADR's surface. Generator-style feedback (the dominant art case) does not read the input at all and is exact today.
5. **Storage** (amends ADR-0024): the basis/result pair becomes a **per-render transient ping-pong** at the working format — allocated, iterated, read back, dropped. ADR-0024's format rule (working format, full precision at 16/32-bpc), aliasing exclusion, and never-persisted rule all stand; its "per-instance persistent pair" does not (nothing persists at all).
6. **`SUPPORTS_THREADED_RENDERING` stays declared.** Temporal renders are now embarrassingly parallel like everything else; no warning icon, no serialization, no special casing.

## Costs and honesty

- Per-frame compute multiplies by n(F) ≤ W (W=16 default: sixteen plan executions per frame; bounded, user-controlled, M7 measures). Intermediate iterations skip CPU readback.
- Feedback horizon is finite: effects converge to a W-frame window (ramp-in over the first W frames, then steady state). True unbounded accumulation (long-horizon simulation) is out of v1 — the Compute Cache successor path recorded in ADR-0023 remains the designated future for that, now with the M6 measurements as its motivation.
- v1 filter-style temporal sees the current input in every iteration (motion trails smear the *current* image; historical-input follow-up restores true motion echo).

## Verification obligations

- Unit: window annotation parse (default/cap/rejection); n(F) math incl. the F+1 clamp; the ping-pong loop's swap/readback discipline.
- TR-M6-001 (re-targeted): accumulator value = `min(F+1, W) × step` exactly, at 8 and 32-bpc, **read in shuffled frame order** (order-independence is the point); RQ and aerender sequences equal the same formula frame-by-frame; recompile with a different `@window` changes the plateau accordingly; no MFR warning icon.
