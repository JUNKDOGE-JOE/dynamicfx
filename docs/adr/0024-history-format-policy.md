# ADR-0024: History storage format, lifetime, and update discipline

- Status: Accepted; the persistent per-instance storage model (§2/§4) is superseded by [ADR-0025](0025-windowed-resimulation.md) (the pair becomes a per-render transient ping-pong) — the format, aliasing-exclusion, and never-persisted rules stand
- Date: 2026-08-13 (Accepted with explicit user approval; §2/§4 revised at acceptance for same-frame re-render idempotency — a basis/result texture pair with role swap, disclosed in the M6 report)
- Owners: DynamicFX project
- Related decisions: [ADR-0023](0023-temporal-seek-reset.md) (semantics companion), [ADR-0021](0021-precision-alpha-color-policy.md)/[ADR-0022](0022-16bpc-working-format-f32.md) (working formats), [ADR-0020](0020-executionplan-aliasing.md) (aliasing exclusion), [ADR-0016](0016-sequence-schema-v1.md) (persistence explicitly untouched)
- Related tests/audits: TR-M6-001; `docs/audits/06-temporal-feedback.md`

## Context

ADR-0023 defines when history is valid; this ADR defines what it physically is. The choices interact with the depth policy (a feedback texture that quantized differently from the pipeline would defeat M5's precision work), with the aliasing plan (history must survive across frames while plan intermediates are recycled within one), and with persistence (a serialized feedback texture would bloat projects and break the reset semantics).

## Decision

1. **Format follows the working format.** The history texture uses the working format of the depth it was written at (Rgba8Unorm at 8-bpc; Rgba32Float at 16/32-bpc per ADR-0022). Policy-level like ADR-0019: no per-effect or per-texture choice exists. Feedback therefore accumulates at full working precision — at 16/32-bpc, temporal accumulation never quantizes between frames.
2. **A basis/result texture pair per temporal instance**, full frame each, allocated lazily on the first temporal render and reallocated (with reset) on depth or resolution change. Two textures, not one, because AE re-renders the *same* frame routinely (parameter tweaks, cache refreshes): frame F must always render from `basis` = output(F−1), no matter how many times it renders. A single texture that commits output(F) over its own input would feed the effect its own current output on the next same-frame re-render — runaway feedback on every parameter tweak. Non-temporal effects allocate nothing.
3. **Excluded from plan aliasing.** History lives outside the ADR-0020 lifetime analysis — the plan's physical intermediates remain within-frame recyclable; history is a cross-frame resource by definition. The `DYNAMICFX_NO_ALIAS` switch does not affect it.
4. **Update discipline: render into `result`, advance by role swap.** A temporal render draws its final pass directly into the persistent `result` texture while sampling `basis`; advancing to frame F+1 swaps the two roles — no copy at all, and no CPU round-trip enters the feedback loop. Same-frame re-renders redraw `basis` → `result` idempotently without advancing. State (`last_frame`, the swap) commits only after a successful render; failure leaves the pair untouched so the next frame resets rather than reading a torn state.
5. **Never persisted.** No snapshot field, no sequence-schema change (ADR-0016 stays at v1 byte-for-byte), no sidecar. Reopen and fresh aerender processes start from the ADR-0023 initial state. Temporal state is execution state, not document state.
6. **Identity and memory.** No new hash domains; history binds as an ordinary input texture, so `PipelineKey` and layouts are unchanged. Cost is one full frame at working-format size per temporal instance (×4 bytes/px at 8-bpc, ×16 at 16/32), reported in the render log next to the ADR-0020 transient line for host evidence.

## Alternatives considered

- Fixed 8-bit history at all depths: cheap but re-introduces per-frame quantization exactly where accumulation magnifies it; rejected.
- Persisting history in the project (snapshot growth): breaks reset determinism, bloats projects by megabytes per instance, and couples document identity to execution state; rejected.
- Sharing history storage with plan intermediates: saves one texture but makes the last intermediate un-recyclable and entangles frame lifetime with plan lifetime; rejected for clarity.

## Consequences

- Temporal precision inherits M5's guarantees automatically; the M5 fixtures' bit-exactness extends to accumulation.
- Projects stay clean: temporal effects serialize exactly like non-temporal ones.
- VRAM grows by two working-format frames per temporal instance (the idempotency price) — bounded, logged, measured at M7.

## Revisit conditions

Per-pass history (ADR-0023's deferred extension) would multiply storage and deserves its own layout policy; MFR concurrency (M7) re-opens the single-texture assumption.

## Verification obligations

- Unit: reset-on-depth/resolution-change reallocation; failed-render leaves no committed history (`last_frame` semantics); history excluded from plan aliasing goldens.
- TR-M6-001 host fixtures: accumulation at 32-bpc carries exact float sums across frames (no per-frame quantization); the memory log line reports the history allocation; reopen/aerender resets per ADR-0023.
