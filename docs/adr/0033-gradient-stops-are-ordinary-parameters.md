# ADR-0033: Gradient stops are ordinary parameters (supersedes ADR-0031 §3, §6, §7)

- Status: Accepted
- Date: 2026-08-15
- Owners: DynamicFX project
- Related decisions: supersedes [ADR-0031](0031-gradient-parameters.md) §3 (persistent value format), §6 (keyframe interpolation) and §7 (editor role); [ADR-0032](0032-gradients-are-graph-resources.md) (graph binding) and ADR-0031 §1/§2/§4/§5 stand unchanged; builds on [ADR-0013](0013-paramid-grammar-and-pools.md) (fixed pools, stable IDs), [ADR-0026](0026-color-parameter-default-annotation.md) (colour encoding)
- Related implementation: `src/host/params.rs`, `src/gradient.rs`, `src/host/gradient_ui.rs`, `src/lib.rs`
- Related tests/audits: [TR-0031-001](../TEST_MATRIX.md#tr-0031-001--gradient-parameters)

## Context

[ADR-0031](0031-gradient-parameters.md) §3 made a gradient one **arbitrary-data** parameter: a `Vec` of stops, serialized through the flatten/unflatten callbacks, with a hand-written keyframe `interpolate`, and a custom-UI editor as the only way to change it. That was implemented and it **crashes After Effects the moment the row is expanded** (reported from interactive use, 2026-08-15). Two host cycles of bisection narrowed it no further than "before the draw path's first log line", and a third showed the custom-UI event never arriving at all.

The decisive evidence came from a shipping third-party gradient effect the user supplied for comparison (`bfxMapRamp` 1.1.0.1). Decoding its PiPL ruled out the global out-flags — its `out_flags2` is identical to ours and both set `CustomUI` — and its binary shows the same suites this implementation uses (`PF Effect Custom UI Suite`, `AEFX_AcquireDrawbotSuites`, the three Drawbot suites). So neither the flags nor the API choice explained anything.

Dumping its **parameter structure** from ExtendScript did:

```
[13] Ramp Preview                                   <- custom-UI canvas, one parameter
[14] Position  [15] Color  [16] Alpha  [17] Blending  <- stop 1
[18] Position  [19] Color  [20] Alpha  [21] Blending  <- stop 2
...                                                   32 stop groups, 128 parameters
```

It stores the gradient as **ordinary After Effects parameters** — four per stop, in a fixed pool — and uses custom UI only to draw a preview and to edit those parameters. There is no arbitrary data anywhere in it.

That is the same shape ADR-0013 already chose for shader parameters: a fixed pool with stable identity. This project applied that model one level up and then abandoned it inside the gradient, for no reason recorded at the time.

## Decision

1. **A gradient's stops are ordinary AE parameters, not arbitrary data.** Each stop is a `Position` float, a `Color`, and an `Alpha` float. The arbitrary-data parameter kind, its serde value, its seven lifecycle callbacks, and `E54`'s role as a persisted-blob validator are all withdrawn.

2. **Fixed capacity: 2 gradients × 8 stops.** Each gradient additionally declares a `Stops` integer (how many of its eight are live) and a `Preview` parameter that owns the editor's drawing area. Total growth is 2 × (8 × 3 + 2) = **52 parameters**, appended after the ADR-0030/0031 growth pools and therefore after `Details` — every released index stays where it is (ADR-0013 §5).

   Two gradients rather than ADR-0031's four: at 26 parameters each the cost is now visible in the topology, and two distinct ramps in one effect is the realistic case. Nothing is released yet, so this is a choice, not a shrink.

3. **Keyframing is AE's, per parameter.** ADR-0031 §6's union-resampling interpolation is withdrawn. Stops animate independently, which is what an AE user expects of ordinary rows and what the reference effect does.

4. **Persistence is AE's.** Save/reopen, copy/paste, duplicate, and undo all come from the parameters themselves. ADR-0031 §3's format, its 8-stop cap as a *wire* contract, and its fail-closed decode disappear with the blob.

5. **`E54 GradientMalformed` is retained but re-scoped** to reading the live parameters: a `Stops` count outside `1..=8`, or positions that are not non-decreasing across the live stops. The reaction is unchanged — refuse and bind transparent black rather than repair — but it now guards a read, not a decode.

6. **The editor is demoted to a convenience and is no longer load-bearing.** With stops as ordinary rows, a user can set every value without any custom UI. The preview/editor may therefore be shipped disabled, fixed later, or dropped entirely without making the feature unusable — which is precisely the property ADR-0031 §7 lacked, and the reason a crash there could hold the whole feature hostage.

ADR-0031 §1 (pool kind), §2 as corrected by [ADR-0032](0032-gradients-are-graph-resources.md) (graph-resource binding), §4 (straight-sRGB interpolation) and §5 (256×1 LUT) are unaffected: the LUT is now baked from the live parameters instead of from a decoded blob.

## Alternatives considered

- **Keep arbitrary data and fix the crash.** Rejected. The crash is not yet understood after three host cycles, and even fully fixed the design keeps a hand-rolled format, hand-rolled keyframe interpolation, and a single point of failure whose loss makes the parameter uneditable. The evidence that a simpler shape ships and works is stronger than the sunk implementation.
- **Arbitrary data with a POD (`[Stop; 8]` + count) value.** Rejected as a half-measure: it removes the `Vec` but keeps the callbacks, the format, the custom interpolation, and the editor-or-nothing dependency.
- **One gradient slot instead of two.** Rejected: a heat ramp plus a tint ramp is an ordinary request, and 26 parameters is affordable.
- **32 stops, matching the reference.** Rejected: 32 × 3 × 2 gradients is 192 parameters for a generic shader runtime that also carries 118 of its own. Eight stops covers the ramps this project's own examples need, and the pool can grow by append.

## Consequences

### Benefits

- The host-crash risk stops being existential: the editor becomes optional decoration over parameters that already work.
- Save/reopen, undo, copy/paste, duplication, and keyframing are inherited from AE rather than implemented and tested here.
- `serde`, `bincode`, the arbitrary-data dispatch, and the seven callbacks all leave the codebase.
- Stop values become visible and scriptable as ordinary properties — the same automation surface every other DynamicFX parameter has.
- The design now matches ADR-0013's fixed-pool model instead of contradicting it.

### Costs and risks

- The Effect Controls panel gains a long list of rows per gradient. The reference effect has the same shape, so this is normal for the control, but it is visually heavier than one arbitrary row.
- 52 more declared parameters, permanently.
- Work already done under ADR-0031 §3/§6/§7 is discarded: the serde value, the callback dispatch, the union-resampling interpolation, and part of the editor.
- Per-stop keyframes can be animated into a non-monotone order; `E54` catches it at read time rather than making it unrepresentable.

## Revisit conditions

- Evidence that 8 stops is too few for real ramps is an append-compatible growth, recorded as a new ADR because capacity is a persistent contract.
- If the custom-UI crash is later understood and the editor becomes reliable, that changes nothing here — this ADR is about where the *value* lives, not about whether an editor exists.

## Verification obligations

- Rust unit tests: the live-parameter read reproduces the same LUT as the equivalent literal gradient; a `Stops` count outside `1..=8` and non-decreasing-position violations each yield `E54` and never repair; pool growth leaves every pre-existing declaration index unchanged, including `Details` at 109 and the ADR-0030 Layer slots.
- Host legs on Windows AE 2025 **and** AE 2026, recorded separately: the stop rows appear and edit; a gradient renders from them; changing a stop changes the render; keyframing a stop animates; save/reopen restores the values; a duplicated instance does not share them; **expanding every gradient row does not destabilise the host** — the specific failure this ADR exists to make impossible.

## Erratum — 2026-08-15

The Context section above says of the reference effect: *"There is no arbitrary data anywhere in it."* **That is false, and the disassembly it cites proves the opposite.** `bfxMapRamp 1.1.0.1`'s PARAMS_SETUP declares its ramp canvas as `param_type = 11` — `PF_Param_ARBITRARY_DATA` — with `ui_flags = 0x82` (`CONTROL | DONT_ERASE_CONTROL`) over a 200x80 control area. The reference stores its *stops* in ordinary parameters and uses one arbitrary parameter as the editor's canvas.

The Decision is unaffected: stops-as-ordinary-parameters is the part that was read correctly, and it is the part this ADR turns into a rule. Only the supporting claim was wrong, and it is corrected here rather than edited above, so the reasoning as it was actually made stays legible.

## Outcome of Decision 6 — 2026-08-15

Decision 6 demoted the editor to a convenience that could be *"shipped disabled, fixed later, or dropped entirely."* It was dropped, the same day, by the project owner's instruction. Reproducing the reference's declaration byte for byte still crashed After Effects on expand, with zero editor log lines written — so the fault is not in the declaration, and no further host cycles were spent on it.

What left the codebase: `src/host/gradient_ui.rs`, the `Event` and `ArbitraryCallback` command arms, the `gradient::Canvas` value, the `serde` dependency, and the `%TEMP%` level switch. What stayed: the whole ADR-0033 value model. `Pool(Gradient, g)` survives as an inert, permanently invisible float — it is the binding anchor `hint:gradient` resolves to, and it holds its declaration index so the append-only topology contract still holds for every parameter after it.

Two presentation changes follow from having no selector: the `Stops` count now drives how many stop groups are on screen (previously one selected group, selectable only from the editor that no longer exists), and its default drops from 8 to 2 so an untouched gradient is 7 rows rather than 25. Stop defaults become black at 0.0 and white at 1.0, with the six spare stops parked at 1.0 white — monotone, so never `E54`, and raising the count never changes the rendered ramp.
