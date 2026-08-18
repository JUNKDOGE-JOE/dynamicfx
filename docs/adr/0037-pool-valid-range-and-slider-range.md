# ADR-0037: Pool slider valid ranges are wide and fixed at `PARAMS_SETUP`; `@param min:/max:` is the slider range

- Status: Accepted
- Date: 2026-08-19
- Deciders: user (approved the fix shape — wide registered range, `min:/max:` as slider range — 2026-08-19, after the defect assessment) + assistant session
- Owners: DynamicFX project
- Related decisions: refines [ADR-0013](0013-paramid-grammar-and-pools.md) (fixed pools whose UI is configured per definition) and the value-encoding contract pinned by TR-M2-003 ("sliders pass raw"); same shape as [ADR-0028](0028-details-button-and-slider-precision.md) (a `PARAMS_SETUP` slider-definition fix after a user report)
- Related implementation: `src/host/params.rs` (pool declarations), `src/lib.rs` (`configure_slots`, `read_bound_values`)
- Related tests/audits: [TR-0037-001](../TEST_MATRIX.md#tr-0037-001--pool-valid-range-float1-negative-int10); public issue [#5](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/5)

## Context

A Float pool slot is declared at `PF_Cmd_PARAMS_SETUP` with slider **and** valid range `0..1`, an Integer slot with `0..10`. When a definition binds a parameter, `configure_slots` writes the shader's `@param min:/max:` into the slider *and* valid fields of the slot's `PF_ParamDef` and calls `PF_UpdateParamUI`.

Measured on After Effects 2026 (26.3) with the 0.0.3 artifact, by pixel readback (public issue #5, first written up 2026-08-15): a Float slot declared `min:2 max:200` and set to 40 in After Effects reaches the shader as exactly `1.0`; every value `<= 1` arrives exactly; angle parameters, whose `PF_AngleDef` carries no valid range, arrive intact. The After Effects side holds the right value and the right range — `setValue(0.3)` on that slot is rejected by the host as outside `2..200`.

The plug-in's own path is not the clamp: `read_bound_values` reads `fs_d.value` raw and the std140 packing writes it raw (`src/lib.rs`, `src/render.rs`). The host is, and the SDK says why. The header comment on `PF_UpdateParamUI` (`AE_EffectSuites.h`, `PF_ParamUtilsSuite3`):

> The ONLY fields that can be changed in this way are: … `PF_ParamDefUnion`: `slider_min`, `slider_max`, `precision`, `display_flags` of any slider type

`valid_min` / `valid_max` are not on the list: they are fixed by the definition registered at `PARAMS_SETUP`, and After Effects clamps the evaluated stream value to that range when it hands the parameter to the render. The pool cannot know a shader's range at `PARAMS_SETUP` time, so registering `0..1` there put a permanent ceiling of 1.0 (and, by the same mechanism, a floor of 0.0 for floats and a ceiling of 10 for integers) under every parameter — silently, with the UI showing the intended value.

Why the M2 evidence did not catch it: TR-M2-002/003 exercised `min:0 max:2` at 0.5, an integer at 3 and an angle at 90 — every probe lay inside the registered range. The README's own example (`speed min:0 max:4`) and the shipped `examples/thermal.glsl` (`glow` default 1.2, `heat`/`speed` up to 2/4) are affected; thermal's default palette has never rendered as intended on a host.

## Decision

1. **The Float and Integer pools register a wide valid range at `PARAMS_SETUP`.** Float: `-1 000 000 000 ..= 1 000 000 000` (exactly representable in the `f32` the SDK stores); Integer: `-1 000 000 000 ..= 1 000 000 000` (fits `i32`). The registered *slider* ranges stay `0..1` and `0..10` — the display range of an unbound or un-annotated slot. Every other slider (Source, Status, StateToken, gradient `Stops`/`Pos`/`Alpha`) keeps its purpose-fixed range: none of them is annotation-driven and each range is the value's true domain.

2. **`@param min:/max:` is the slider range.** `configure_slots` continues to write both the slider and the valid fields through `PF_UpdateParamUI`; the SDK guarantees only the slider fields take. The valid-field write is a **measured no-op**: on AE 2025 and AE 2026 (TR-0037-001), after a binding declared `min:2 max:200`, `setValue(0.3)` on that slot was *accepted*, not rejected — so the host uses the wide `PARAMS_SETUP` range for scripting/typing as well as for rendering, and `PF_UpdateParamUI`'s valid fields are ignored on both paths. (This differs from the 0.0.3 observation in issue #5, where the same `setValue` was rejected as out of `2..200`; there the narrow `0..1` was registered at `PARAMS_SETUP` and *that* is what validated typing. The lesson is the same either way: the render clamp and the typing bound both come from the registered range, never from the `PF_UpdateParamUI` write.) The valid write is kept only so the stored definition is internally consistent; nothing relies on it. A binding that declares no range (or only one side) gets the display default back as its slider range and the wide registered range as its valid range, so a slot never hands a previous parameter's range to the next one.

3. **The runtime never clamps a parameter value.** `read_bound_values` keeps passing the evaluated stream value raw; there is no clamp to the declared `min:/max:` on the render side. What After Effects shows is what the shader receives — a runtime clamp would reintroduce exactly the silent UI-vs-render disagreement this ADR removes. A shader that needs a hard limit writes `clamp()`.

4. **The `1..8` gradient count, `0..1` stop position and `0..1` stop alpha rows are unaffected** by (1)–(3): they are not pool slots and their ranges are the value's meaning, not a display default.

Existing projects change no meaning: every value saved under the old ranges lies inside the new ones. No parameter index, kind, or persistent field changes; no PIPL bump is required.

## Alternatives considered

- **Clamp on the render side to the declared range.** Rejected by (3): it makes the shader receive something other than what the row shows whenever the declaration is narrowed under an existing keyframe or an expression overshoots — the same class of defect, one layer down.
- **Normalise every slider to `0..1` and remap in the shader** (the workaround shaders have been using). Rejected: it gives up the readable range the fixed-pool + `@param` design exists to provide, and it hides the real limit inside `label:` text.
- **A narrower "sensible" valid range (`±1 000 000`, as After Effects' own Slider Control uses).** Not chosen: nothing in the host benefits from the narrower figure, and a seed, frame count or pixel dimension can legitimately exceed a million. `±10⁹` is exact in `f32`, fits `i32`, and is still a finite typing bound.

## Consequences

### Benefits

- Float parameters above 1 and below 0, and integers above 10, reach the shader as shown; the README's parameter table ("value the shader receives: as shown") becomes true, and the shipped examples' upper slider ranges start doing something.
- Shaders written with the `0..1` workaround keep working unchanged.
- No topology, persistence or ABI change; old projects reopen with their values intact.

### Costs and risks

- A value outside the declared `min:/max:` is no longer impossible: an expression, or a declaration narrowed under an existing keyframe, can deliver one to the shader. This is documented, and it is the correct reading of "as shown".
- The alpha companion of a `vec4` colour is a Float slot too, so an alpha above 1 can now be typed; the shader receives it as such.
- Whether After Effects' UI constrains typing to the declared range after this change is not something the runtime can guarantee — it is measured, not designed.

## Revisit conditions

- Host evidence that a wide `valid_*` range breaks a host behaviour the narrow one did not (keyframe interpolation, graph editor, expression evaluation, scripting) is grounds for a superseding ADR choosing a narrower bound.
- If a future SDK version lets `PF_UpdateParamUI` (or another suite) change `valid_*` per instance, the declared range could become a real per-shader hard limit; that would be a new ADR, not an edit here.

## Verification obligations

- Unit: the registered pool ranges are named constants pinned by tests (wide, symmetric, `f32`-exact, `i32`-fitting, display defaults inside them) and `PARAMS_SETUP` uses them; `configure_slots` derives every slider/valid write from the same constants (a host call, reviewed rather than unit-tested).
- Host, TR-0037-001, Windows AE 2025 **and** AE 2026 recorded separately, by pixel readback with `app.purge` before every render: a Float slot `min:2 max:200` at 40 arrives as 40; a Float slot `min:-1 max:1` at −0.5 arrives as −0.5; an Integer slot `min:0 max:100` at 50 arrives as 50; `examples/thermal.glsl` renders its default palette (first host sighting); the M2 parameter battery and the M3 persistence battery stay green on the new artifact (projects saved under the old ranges reopen unchanged).
