# ADR-0023: Temporal history v1 — `prev` input, continuity and reset semantics, MFR-compatible

- Status: Accepted; the continuity/state model (§4-9) is superseded by [ADR-0025](0025-windowed-resimulation.md) on TR-M6-001 run-1 evidence — the `prev` grammar surface (§1-3) stands
- Date: 2026-08-13 (revised twice at user direction — MFR-doc review, single-frame simplification, competitor grounding — then Accepted with explicit user approval)
- Owners: DynamicFX project
- Related decisions: [ADR-0009](0009-staged-format-adr-acceptance.md) (M6-entry staging), [ADR-0018](0018-envelope-grammar-v1.md) (grammar this ADR appends to), [ADR-0011](0011-shader-abi-v1-core.md) (`u_frame` builtin), [ADR-0015](0015-statetoken-and-diagnostics.md) (E6 diagnostic domain), [ADR-0024](0024-history-format-policy.md) (storage companion)
- References (official SDK guide): "Multi-Frame Rendering in AE" and "Compute Cache API" (ae-plugins.docsforadobe.dev, effect-details)
- Related tests/audits: TR-M6-001 (to be defined); `docs/audits/06-temporal-feedback.md`

## Context

M6 makes feedback effects possible: shaders that read what the effect produced on the previous frame. Two host realities constrain the design.

**Random access.** AE can request any frame at any time (scrub, render start points), while feedback is inherently sequential — the contract must define exactly when history is trustworthy and what every discontinuity means.

**Multi-Frame Rendering.** Since AE 2022, frames render concurrently on multiple threads. Per the official SDK guide: an effect that does not set `PF_OutFlag2_SUPPORTS_THREADED_RENDERING` forces render threads through it one at a time **and AE shows a warning icon in the Effect Controls window** — the whole plugin, including the overwhelmingly common non-temporal shaders, would be branded "not optimized for Multi-Frame Rendering" and serialized. The guide further establishes: with the flag set, `sequence_data` is const at render time (accessed via `PF_EffectSequenceDataSuite1`); the escape hatch `PF_OutFlag2_MUTABLE_RENDER_SEQUENCE_DATA_SLOWER` gives each render thread **its own unshared, unsynchronized copy** of sequence data — which makes it structurally useless for cross-frame evolving state; render-adjacent selectors (`SEQUENCE_SETUP/RESETUP/SETDOWN`, `SMART_PRE_RENDER`, `RENDER`, `SMART_RENDER`) may arrive on multiple threads concurrently; no frame-ordering guarantees are documented; and Adobe's recommended mechanism for shared computed state under MFR is the Compute Cache (`AEGP_ComputeCacheSuite1` with `AEGP_HashSuite1` keys, single/multi checkout).

## Decision

### Surface (unchanged from the first draft)

1. **History v1: one slot, the previous final output.** A graph pass reads history by listing the reserved input name **`prev`** in its `@graph` manifest input list. `prev` is the effect's own final output from the previous frame — not per-pass state. Multiple passes may read it; it can never be written, never appears as an intermediate or output name, and is rejected in raw (non-envelope) sources. All violations are line-numbered E6 diagnostics (registry append-only per ADR-0015).
2. **Grammar append, not change.** One reserved input name added to the ADR-0018 grammar; every existing source stays valid. Bindings follow ADR-0018 §5 unchanged (`prev` occupies its declared input position `i`, binding `2+i`).
3. **Scope note — most "feedback-looking" effects need no temporal state at all.** Trails, echo stacks, advection, and ray-marched smears are expressible as in-frame shader loops (plain GLSL `for` with an accumulator), which are stateless, MFR-parallel, and scrub-perfect — available in DynamicFX today with no new machinery. A static study of a shipping competitor (also Rust/DX12) confirms this is the market approach: its feedback facility is a per-frame loop-carry construct holding **no cross-frame state whatsoever**, and precisely because of that its PIPL declares threaded rendering, smart render and float-color support with no mutable-sequence escape hatch. *[Product identity and the decoded flag values are redacted here for publication — see [ADR-0036](0036-single-repository-record.md); the unredacted text is in the archived private record.]* `prev` exists only for what in-frame loops cannot do: **true accumulation across frames**. Documentation must steer users to in-frame loops first.

### MFR compatibility (replaces the first draft's "serialized rendering prerequisite")

4. **The plugin declares `PF_OutFlag2_SUPPORTS_THREADED_RENDERING`.** Non-temporal effects render fully concurrently — no warning icon, no whole-plugin serialization. Consequences the official checklist imposes, adopted as implementation obligations:
   - no unguarded globals: the process registry, pipeline caches, and GPU device access are already mutex/OnceLock-guarded; the diagnostic log writer gains a lock;
   - no sequence-data writes at render: **temporal state does not live in sequence data at all** (see 5) — the const rule is satisfied structurally, and `MUTABLE_RENDER_SEQUENCE_DATA_SLOWER` is not needed (its per-thread unshared copies cannot carry cross-frame state anyway);
   - all per-instance mutation stays behind the existing instance mutex; flatten reads under the same mutex;
   - the smart-render ROI window hand-off is re-plumbed as a call parameter (the current single-field hand-off would race between concurrent frames of one instance);
   - no lock is held across host suite calls or layer checkouts.
5. **Temporal state is process-global, per instance, self-synchronized:** a mutex-guarded store of `(last_frame, history texture)` keyed by the instance, GPU-resident (ADR-0024), invisible to sequence data and to persistence.
6. **Continuity rule: one frame cached, one comparison.** For a temporal render at frame F, under the instance's temporal lock: `F == last + 1` → use history, render, commit F; **anything else → reset** (defined initial state), render, commit F. Nothing more. The per-instance lock already serializes concurrent same-instance frames on MFR hosts; sequential hosts (pre-MFR AE years, MFR disabled, single-threaded aerender) hit the identical two-line rule. If MFR's near-order dispatch ever inverts same-instance arrivals, that render is a defined reset — and every reset is counted in the render log, so the real-world inversion rate becomes measured evidence rather than speculation (see Verification).
7. **Reset is the defined initial state, never an error:** transparent black, also produced on token change (edit/recompile/Compile), depth or resolution change, and fresh processes (reopen/aerender; never persisted per ADR-0024). Shaders can detect it by zero alpha or `u_frame`.
8. **Determinism statement.** Any single random-access frame is a pure function of (source, parameters, frame) — jumps reset, so scrub history never leaks into an isolated frame. Sequential evaluation from any reset point is fully deterministic. Under MFR playback, wherever dispatch preserves same-instance order (the normal case) the chain result is bit-identical to the sequential result; any inversion is a *defined, logged reset*, never an approximation and never an out-of-order commit (which would poison AE's frame cache). These claims are fixture obligations.
9. **Failure policy.** History allocation failure follows the existing GPU-unavailable path: diagnostic log + pass-through — never a silent render with missing declared history.

### Why not the Compute Cache today

Adobe's Compute Cache is the official mechanism for **shared computed state** under MFR — but its documented exemplar (Auto Color) caches *histograms*: kilobytes per frame. Temporal feedback state is a **full frame at working precision**: ~2 MB per frame at 320×240/f32 and ~132 MB per frame at 4K/f32. Memoizing per-frame states at that scale would exhaust the cache immediately, and host-side eviction under memory pressure would force whole-chain recomputation (worst-case quadratic). The Compute Cache remains the right tool for the *future* refinements it actually fits — small per-frame derived state, per-pass state descriptors, or chain-reconstruction bookkeeping — via a superseding ADR; the `prev` surface and reset semantics remain binding either way.

## Alternatives considered

- **Not setting `SUPPORTS_THREADED_RENDERING`** (first draft): serializes every instance of the plugin and shows the Effect Controls warning icon — punishes all non-temporal use to simplify the temporal corner; rejected on official-guidance review.
- **State in sequence data + `MUTABLE_RENDER_SEQUENCE_DATA_SLOWER`**: per-thread copies are documented as unshared and unsynchronized — cross-frame continuity is impossible by construction, plus it costs concurrency; rejected.
- **Compute Cache as the v1 state store**: memory math above; rejected for frame-sized state, reserved for small-state successors.
- **Bounded predecessor wait on out-of-order arrivals** (previous revision of this draft): adds a liveness/latency knob for an inversion case whose real-world frequency is unmeasured; rejected for v1 in favor of the two-line rule plus a logged reset counter — if fixtures measure a nonzero, objectionable rate, the successor ADR picks wait-or-reconstruction with data in hand.
- **Re-simulating from frame 0 on random access**: unbounded scrub cost; rejected for v1.
- **Accepting any `F > last` as continuation**: out-of-order commits would poison AE's frame cache with wrong pixels; only `last+1` commits.

## Consequences

- The plugin presents as MFR-optimized; the concurrency win applies to the dominant non-temporal case, and AE 2023 (pre-MFR-default years in the support matrix) runs the identical code path.
- Temporal instances self-serialize only against themselves, only when temporal — the dependency chain's inherent cost, paid narrowly. The entire temporal semantics is one comparison and one reset; there is nothing else to reason about.
- Scrubbing shows a visible, documented restart; sequential runs are exactly reproducible.
- The thread-safety obligations (log lock, window-parameter re-plumb) harden the whole plugin, not just temporal paths.

## Revisit conditions

Per-pass state, warmup/pre-roll, or long-gap reconstruction → the Compute Cache successor ADR (for the small-state shapes it fits). A measured, objectionable mid-playback reset rate under MFR (the fixtures' logged counter) → the successor decides wait-or-reconstruction with data in hand.

## Verification obligations

- Unit: grammar accepts `prev` as input and rejects it elsewhere with line numbers; binding positions with `prev` in every slot order; the continue/reset decision fully covered (continuation, backwards, gap, token/depth/size change); the window hand-off re-plumb has a concurrency-shaped test.
- TR-M6-001 host fixtures (AE 2025, MFR at host defaults): an accumulator shader rendered sequentially K frames equals K× exactly at 8 and 32-bpc; RAM-preview and RQ legs (MFR-concurrent) match the sequential result **and the logged reset counter reports the observed inversion rate** (the number becomes evidence); a scrub jump yields the reset value; recompile and reopen reset; an aerender leg proves fresh-process determinism; the Effect Controls panel shows **no** MFR warning icon (screenshot evidence).
