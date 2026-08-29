# ADR-0042: The gradient editor returns as pure presentation — custom-UI safety contract

- Status: Accepted (2026-08-28, explicit user approval after one review round — the clarified point: "pure presentation" is a failure-containment property, the editor itself is the interactive editing surface; drafted and accepted the same day)
- Date: 2026-08-28
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md) (its "可选 Editor" / P4 is the deferred *external* authoring client; this ADR is an Effect Controls panel control and does not touch that boundary)
- Related decisions: builds on [ADR-0033](0033-gradient-stops-are-ordinary-parameters.md) (the gradient *value* lives in ordinary parameters — its revisit clause states the editor's existence does not affect it, and this ADR does not reopen it); replaces the presentation role that [ADR-0031](0031-gradient-parameters.md) §7 defined and [ADR-0033](0033-gradient-stops-are-ordinary-parameters.md) Decision 6/Outcome withdrew (§7 stays superseded; nothing here restores editor-or-nothing); mechanism from [ADR-0040](0040-parameter-groups-and-id-identity.md) (param id is stream identity; topology placement is id-safe) and [ADR-0041](0041-panel-polish.md) (slot/group hide mechanism); [ADR-0013](0013-paramid-grammar-and-pools.md) pools and [ADR-0015](0015-statetoken-and-diagnostics.md) diagnostics are untouched
- Related implementation: `build.rs` (PiPL out-flags + `repair_pipl_resource`), `src/host/params.rs` (declaration + `register_ui`), `src/lib.rs` (event arm), `spike/probe/` (the measured shapes)
- Related tests/audits: [TR-CUI-001](../TEST_MATRIX.md#tr-cui-001--custom-ui-crash-bisection-spike) Rounds 1–4 (the evidence base), [TR-0031-001](../TEST_MATRIX.md#tr-0031-001--gradient-parameters) (the 2026-08-15 record), TR-0042-001 (to be recorded by implementation)

## Context

The ADR-0031 §7 gradient editor crashed After Effects 2025 on row expand — instantly, silently, zero log lines, zero dumps — and was removed on 2026-08-16. ADR-0033 had already moved the gradient *value* into ordinary stop parameters, so the removal cost presentation only; `PF_OutFlag_CUSTOM_UI` left with it, and the shipped 0.0.6 declares no custom-UI surface at all (`build.rs` line ~101 records the guard).

TR-CUI-001 (2026-08-28, AE 2025 25.6.6x4) closed the cold case at code level:

- **The crash class is reproduced and pinned to one missing call.** A parameter declaring `PF_PUI_CONTROL` while the plug-in never calls `PF_REGISTER_UI` kills After Effects — 3/3, no gesture required, within ~8 s of apply. The identical byte-for-byte binary with the call made is healthy: canvas painted, alive at 105 s (Round 3). The fault is a null-pointer dereference inside AE's own Effect Controls paint path (`AfterFXLib!CEffects::UpdateInvalidParams`, `mov rcx,[rbx]` with `rbx=0`): AE paints the row expecting a registered custom-UI record, finds null, and dereferences it unchecked. Because the paint runs inside a kernel-dispatched window-procedure callback, Windows escalates to `C000041D STATUS_FATAL_USER_CALLBACK_EXCEPTION`, which **bypasses WER entirely** — no dump, no event-log entry — matching the 2026-08-15 "instant, silent death" record word for word (Round 4). Omitting the call is therefore not a degraded-but-safe state; it is host-fatal. (Inference boundary: `gradient_ui.rs` never entered git, so "the 2026-08-15 editor omitted the call" is reconstructed from the identical signature, not read from source.)
- **The healthy pipeline is measured end to end.** A *standard* (non-arbitrary) parameter carrying `ParamUIFlags::CONTROL`, registered via `register_ui`, drawing with Drawbot, passed Draw / Click / the drag stream (`set_send_drag` effective, `last=true` terminator, automatic redraw after the drag) / AdjustCursor on-host (Rounds 1–2). Upstream's `custom_ecw_ui` sample demonstrates the same shape drawing image and text.
- **Geometry is measured.** `screen_point` and `current_frame` share one coordinate space (error ≤ 2 px), and AE stretches the control to the panel column width — declared 200 rendered as 280. An editor must paint and hit-test from `current_frame`, never from its declared size (Round 2 Finding 3).
- **The co-presence rule is a host validation.** AE checks at `PARAMS_SETUP` and refuses the whole effect when a parameter carries `ui_width`/`ui_height`/`PF_PUI_CONTROL` without the global `CustomUI` out-flag ("no custom ui outflag, but param has ui_width or ui_height or PF_PUI_TOPIC/CONTROL flags" — TR-0031-001 first defect, recorded in `build.rs`).
- **The `CustomUI` PiPL byte is the code-page-vulnerable one.** `OutFlags::CustomUI` is bit 15 — byte `0x84` of the little-endian out-flags word, the first byte ≥ 0x80 this project ever shipped. pipl 0.1.1's RC string-literal path corrupts it to `?` (`repair_pipl_resource` exists for exactly this; upstream's own Windows custom-UI example DLLs ship dead because of it — TR-CUI-001 instrument record).

With the root cause pinned and the healthy shape host-backed, the editor track is unfrozen. Reintroducing it touches two release surfaces that require an ADR: a drawing-area parameter (panel topology) and the global `CustomUI` out-flag (PiPL). Custom-UI flags are not persistent *project* state — saved streams carry neither `ParamUIFlags` nor out-flags — so this is a presentation contract, not a value or persistence change.

## Decision

1. **The gradient editor returns as pure decoration, and must stay disposable.** The gradient value's sole authority remains the ADR-0033 ordinary parameters (`Stops` count, per-stop `Position`/`Color`/`Alpha`). Every gradient remains fully creatable, editable, keyframeable, savable and renderable with the editor absent, disabled, or failed. No feature may ever require the editor — the property whose absence let a presentation crash hold the whole control hostage in ADR-0031 §7 is now a standing invariant.

2. **One drawing-area parameter per gradient, standard kind, inert stream.** A new `ParamKey` variant (indicatively `GradientCanvas(g)`, one per gradient, `GRADIENTS` total) is declared inside its gradient's sub-topic in `Main`:
   - **Standard parameter, never `PF_Param_ARBITRARY_DATA`.** ADR-0033 withdrew the arbitrary-data protocol from gradients; the canvas does not reintroduce it. The concrete PF kind is an implementation choice informed by the measured resting-visibility difference (a Float+`CONTROL` canvas renders collapsed until expanded; a Color+`CONTROL` canvas paints on apply — probe, 2026-08-28) and is fixed at first release.
   - **The value stream is inert.** The runtime never reads or writes it; it joins no pool, owns no `ParamId`, and appears in no `BindingPlan`, no identity/hash domain, and no `DFXS` field. Declared `CANNOT_TIME_VARY`. Because the stream carries nothing, projects are interchangeable between editor-on and editor-off builds: reopening an editor-on project in an editor-off build merely discards a valueless stream.
   - **Identity per ADR-0040:** the variant's `Debug` rendering joins the frozen golden id table at first release; placement inside the sub-topic is id-safe and pinned by the order/id golden tests.
   - **Visibility follows the gradient**: hidden by the same predicate that hides the gradient sub-group when its anchor is unbound (ADR-0041 §2 mechanism).

3. **The custom-UI surface is one atom, driven by one build-time switch.** A single predicate (the editor build flag) drives, together and never separately: the PiPL `AE_Effect_Global_OutFlags` `CustomUI` bit, the runtime global out-flag, every `ParamUIFlags::CONTROL` / `ui_width` / `ui_height` declaration, and the `register_ui` call. Consequences:
   - The editor-off build declares **exactly the pre-editor topology and flags** — the shipped 0.0.6 shape, which is the permanent retreat position.
   - Half-states are unbuildable by construction and additionally rejected by AE's own `PARAMS_SETUP` validation (Context). A unit test pins the coupling (§Verification).
   - The PiPL out-flag change bumps the PiPL subversion on the release that ships it, per the established practice (`build.rs`: out-flag changes bump subversion so AE's plugin cache re-reads the PiPL).
   - The `0x84` byte rides the existing `repair_pipl_resource` path; the byte-exactness of the built resource is re-verified per artifact (§Verification).

4. **The `register_ui` invariant.** Every build that declares `PF_PUI_CONTROL` on any parameter **must** call `register_ui` (`PF_REGISTER_UI`, `CustomEventFlags::EFFECT`) in `params_setup`, before `params_setup` returns. If the call fails, `params_setup` returns the error — the effect fails to load visibly. A declared-but-unregistered state must never reach a host: it is not a degraded mode but a measured host kill (Round 3/4). The probe's fatal arm stays a spike-only instrument and is never built into a shippable artifact.

5. **Draw and hit-test from `current_frame` only.** The declared `ui_width`/`ui_height` is a layout request; AE stretches the control to the panel column width (measured 200 → 280). All painting and all hit-testing derive from the event's `current_frame`, in whose coordinate space `screen_point` arrives (≤ 2 px, measured). The geometry mapping (stop position ↔ pixel) is one pure function, unit-tested, used by both draw and hit-test so they cannot disagree.

6. **Editing goes through the host's ordinary parameter commit — staged by evidence.**
   - Every editor edit mutates only the ADR-0033 rows, through the host's normal parameter-change protocol, so undo, keyframing, expressions and persistence are inherited exactly as if the user had edited the rows directly. The editor holds no durable state of its own; transient UI state (selection, hover) is session-local and never persisted.
   - **Stage 1 — read-only preview** uses only mechanisms already measured (Draw path). It may ship alone.
   - **Stage 2 — interactive editing** (drag a stop's position/alpha, adjust `Stops`) additionally requires the one mechanism the rounds did *not* measure: committing a value to a **sibling** parameter from inside the event handler. A probe leg measures it before stage-2 implementation freezes; if the host refuses, the editor stays at stage 1 and the rows remain the editing surface — recorded, not superseded.
   - **Any modal host interaction** — specifically `PF_AppColorPickerDialog` (the old editor's double-click recolor) — is an independent host surface and is gated on its own probe leg. Until that leg passes, stop colors are edited through their ordinary Color rows.
   - Drag uses the measured protocol: `set_send_drag(true)`, terminate on `last=true`, rely on the measured automatic post-drag redraw.

7. **Failure containment.** `catch_panics` stands. A failure inside the editor's draw/event path logs and paints a visible degraded state (flat fill), never silent, never fatal, and never touches `StateToken`, the `Status` line, or the ADR-0015 diagnostic registry — those are value-pipeline surfaces and the editor is not in the value pipeline. Render-side code still never calls AEGP; the editor registers `CustomEventFlags::EFFECT` only (Effect Controls window; no comp/layer-window UI in this ADR).

## Alternatives considered

- **Reuse the invisible `Pool(Gradient, g)` anchor as the canvas** (zero new parameters). Rejected: the anchor is load-bearing binding identity, and ADR-0033's Outcome froze it "inert, permanently invisible"; hanging the decoration on it would couple the disposable surface to a non-disposable one and contradict a recorded outcome.
- **Arbitrary-data canvas (the reference effect's and 2026-08-15 editor's shape).** Rejected: ADR-0033 withdrew the arbitrary protocol from gradients; Round 1 proved arb+UI *viable*, but the standard shape is equally verified and keeps the value model byte-untouched. Nothing needs the arb protocol back.
- **One shared canvas with a gradient selector.** Rejected: reintroduces selector state the ADR-0033 Outcome deliberately removed; per-gradient canvases match the sub-group structure and the reference effect's one-canvas-per-ramp shape.
- **Always-declare the canvas and gate only registration/flags at runtime.** Rejected: leaves a `CONTROL`-capable declaration in every artifact, exactly the half-state family the atom rule exists to make unbuildable.
- **A drawn panel replacing the grouped topology outright** (ADR-0040's deferred alternative). Out of scope: this ADR ships the minimal verified surface; a full drawn panel remains ADR-0040's revisit condition and would arrive as its own ADR.
- **Leave the editor dead.** Rejected as the end-state by the project owner now that the root cause is pinned — but it remains the permanent, designed retreat position via Decisions 1 and 3.

## Consequences

### Benefits

- The most-requested presentation surface returns without re-creating the hostage geometry: every failure mode lands on "no editor", never on "no gradients".
- The crash class that cost the 2026-08-15 investigation is made unbuildable (coupling test) rather than merely remembered.
- Zero value/persistence surface moves: no migration, no `DFXS` change, no identity change, no new E-code; editor-on and editor-off builds read each other's projects.
- The upstream guardrail gap (`ParamUIFlags::CONTROL` with no register guard, silent host kill, no dump) is now documented here with the defense in our own tree, independent of when the upstream report lands.

### Costs and risks

- Two more declared parameters (~419 total) and one more `START_COLLAPSED`-family visibility rule — negligible against the ADR-0040 scale numbers, but the golden tables and harness pins move once more.
- The `CustomUI` PiPL bit re-exposes the code-page-vulnerable `0x84` byte; the repair is in place and byte-verified on the probe, but every editor artifact must re-verify it (§Verification) or AE refuses the effect with the out-flags mismatch.
- **Every custom-UI measurement to date is AE 2025 25.6.6x4 only.** AE 2026 behavior (registration, draw, stretch width, drag protocol, and the crash class itself) is unmeasured until TR-0042-001 runs there.
- The editor adds a live event surface (click/drag) to the shipping artifact; containment is Decision 7, but any future editor-crash investigation must use the TR-CUI-001 method — WER is blind to `C000041D`, evidence is taken procdump-attached, and a debugger-attached AE *survives* the fault, so repro records must state attachment.
- Two build flavors exist (editor on/off); the release ships one, and the flavor split must not fork verification: host evidence binds to the shipped flavor's artifact hash as always.

## Revisit conditions

- Host evidence that a **registered** editor build destabilizes any supported AE year: the retreat is Decision 3's switch (ship editor-off, exactly the verified 0.0.6 surface) and a recorded fallback per year — a superseding ADR only if the presentation contract itself must change.
- The stage-2 probe leg measuring that event-path sibling commits are impossible on a supported year caps the editor at stage 1 permanently for that year — recorded, no supersession.
- Any need for the editor to own value state, persist editor state, or become required for any gradient operation **conflicts with ADR-0033 and Decision 1** and requires an explicit superseding ADR, not an implementation drift.
- Upstream `after-effects` shipping a type-level guard coupling `CONTROL` to registration simplifies Decision 4's local defense; adopt without a new ADR (the invariant is unchanged, only its enforcement point).
- A full drawn-panel replacing grouped topics (ADR-0040 revisit) subsumes this ADR's canvas in its own record.

## Verification obligations

Unit (inner loop, every build):

- **The boundary test this ADR exists for:** the set of declared parameters carrying `PF_PUI_CONTROL` (or nonzero `ui_width`/`ui_height`) equals the set covered by the `params_setup` registration path, and is nonempty **iff** the runtime global out-flags and the PiPL properties both carry `CustomUI` — all four read from one predicate, so the test fails on any decoupling. Runs in both flavors: editor-off asserts the empty case.
- Golden id/order tables: editor flavor = base tables + the canvas entries, collision-free; editor-off flavor = base tables exactly.
- Geometry: the stop↔pixel mapping is pure and pinned (edges, stretch case 200→280, round-trip with hit-test).
- PiPL byte-exactness: the built resource's `eGLO` word equals the code's out-flags for the flavor (the `0x84` byte check, per the probe's decode method).

Probe legs (spike/probe, before the corresponding editor stage freezes; procdump-attached method per TR-CUI-001 Round 4):

- Event-path sibling-parameter value commit: from a Click/Drag on the canvas, write a stop `Position`, observe the committed value, an undo entry, and keyframe behavior on a keyframed stream (gates stage 2).
- `PF_AppColorPickerDialog` from the event path: modal raised and dismissed with values honored, host alive (gates any picker use; note the modal wedges the JSX bridge — drive it per the TR-CUI-001 instrument notes).
- Short control area (the old editor's 46 px class) draws and hit-tests correctly under Decision 5.
- Drag stream re-verified at the editor's shipped declared size.

Host, on the editor-flavor artifact, **AE 2025 and AE 2026 recorded separately** (TR-0042-001; every custom-UI datum to date is 2025-only):

- Apply, expand, and draw on both years: canvas paints from `current_frame` at panel widths including a dragged column width; collapsed→expanded transition delivers Draw (the gesture matrix that was unreachable in the fatal arm, now run in the registered arm).
- Stage-2 gestures (if shipped): each edit lands in the correct ADR-0033 row, produces one undo entry, honors keyframes; editor and rows never disagree (edit in one, read in the other).
- Decoration property: an editor-on project reopens in the editor-off flavor with all gradient values/keyframes intact and the canvas stream silently dropped; and vice versa.
- Regression on the editor artifact: reopen 13/13, M2 + M3 batteries, aerender leg — the topology moved, so the full re-pin discipline of TR-0040/0041 applies.
- Evidence per the evidence policy: artifact hashes, exact host builds, purge-before-render, raw logs/PNGs under `docs/audits/evidence/`, and any "did not crash" claim states whether a debugger was attached.
