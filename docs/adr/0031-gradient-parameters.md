# ADR-0031: Gradient parameters (`hint:gradient`)

- Status: Accepted (§2 superseded by [ADR-0032](0032-gradients-are-graph-resources.md); §3, §6 and §7 superseded by [ADR-0033](0033-gradient-stops-are-ordinary-parameters.md) — stops are ordinary parameters, not arbitrary data. §1, §4 and §5 stand)
- Date: 2026-08-15
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md) §4.2, §7.1
- Related decisions: [ADR-0013](0013-paramid-grammar-and-pools.md) (pool growth), [ADR-0016](0016-sequence-schema-v1.md) (persistence), [ADR-0026](0026-color-parameter-default-annotation.md) (color defaults), [ADR-0011](0011-shader-abi-v1-core.md) (binding budget), [ADR-0030](0030-layer-input-parameters.md) (the other texture-shaped parameter)
- Related implementation: `src/binding.rs`, `src/host/params.rs`, `src/gradient.rs` (new), `src/lib.rs`
- Related tests/audits: TR-0031-001 (to be recorded); public issue [#2](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/2)

## Context

Multi-stop color ramps are the most common thing DynamicFX shaders hand-build. The shipped thermal example spends six color parameters and a 14-line `palette()` function on one ramp, and every stop position is a magic number recompiled into the source rather than a control the user can drag. An author who wants to retune a palette edits GLSL; a user who wants to retune it cannot.

The host capability is reachable from `after-effects` 0.4: `PF_Param_ARBITRARY_DATA` (`ae::ArbitraryDef`), the full arbitrary-data callback set via `ArbParamsExtra::dispatch` (new/dispose/copy/compare/flatten/unflatten/**interpolate**), custom-UI registration (`register_ui`, `CustomUIInfo`), the `Command::Event` path with `EventExtra`, and the Drawbot vector-drawing suites. `dispatch` is generic over a `Serialize + DeserializeOwned` type, so this change adds `serde` as a direct dependency.

What must be decided before code exists, because each becomes persistent on first release: the on-disk value format, the stop limit, the interpolation space, how the value reaches the shader, and what the editor guarantees.

## Decision

1. **New `PoolKind::Gradient`, capacity 4, appended to `V1_POOLS`.** Append-only growth per [ADR-0013](0013-paramid-grammar-and-pools.md) §5 — every released index keeps its kind and position. Capacity 4 matches the practical number of distinct ramps in one effect; growth is an ordinary append needing no superseding ADR.

2. **A gradient reaches the shader as a 1D LUT texture, not as uniform-block data.** `hint:gradient` declares a graph-visible texture binding assigned by the existing extra-input rule (ADR-0011), exactly as [ADR-0030](0030-layer-input-parameters.md) does for layers. It occupies no `FxUniforms` space and no float budget:

   ```glsl
   // @param heat_ramp label:"Heat Ramp" hint:gradient
   ```
   ```glsl
   vec3 c = texture(sampler2D(u_heat_ramp, u_s), vec2(t, 0.5)).rgb;
   ```

   Unlike a layer input, a gradient is **not** a graph resource: it is not nameable in `@graph`, because it has no producer and no extent. It binds to every pass that declares it.

3. **Persistent value format, v1.** An ordered list of at most **8 stops**, each `{ position: f32 in [0,1], rgba: [f32; 4] }`, serialized by `serde` through the arbitrary-data flatten/unflatten callbacks. Positions are stored sorted and de-duplicated on write; a decoded value that is unsorted, out of range, empty, or over 8 stops **fails closed** with `E54 GradientMalformed` (runtime/transport family, appended per [ADR-0015](0015-statetoken-and-diagnostics.md) §4) rather than being repaired by guesswork. The 8-stop cap is a persistent contract: raising it later is an append-compatible format change, lowering it is not.

4. **Interpolation is linear in straight (non-premultiplied) sRGB**, both between stops when baking the LUT and between keyframes in the arbitrary-data `interpolate` callback. This matches what the existing color parameters already deliver to shaders (`0..1` RGB straight from the AE color picker, ADR-0026) so a gradient and a color control agree. Perceptual spaces are a later opt-in annotation, not a v1 default that would silently disagree with `hint:color`.

5. **The LUT is 256×1, `Rgba32Float`, rebuilt only when the gradient value changes.** 256 samples is beyond visible banding for a ramp at 8-bpc and costs 4 KB. Float storage keeps 32-bpc projects honest — a `Rgba8Unorm` LUT would quantize a ramp the rest of the pipeline is careful not to (ADR-0021). The LUT is a per-instance cached GPU resource under the M7 budget, rebuilt on value change rather than per frame.

6. **Gradients are keyframeable.** The `interpolate` callback is implemented, so a gradient animates across keyframes. Two gradients with different stop *counts* interpolate by resampling both to the union of their positions before mixing — never by pairing stops by index, which would make a stop appear to jump when a keyframe adds one.

7. **The editor is a custom-UI control drawn with Drawbot** in the Effect Controls panel: a gradient bar with draggable stops; click empty bar space to add a stop, drag to move, double-click a stop to open the native AE color picker, Delete to remove the selected stop. The editor is a *convenience over the value*, never the authority: every edit writes the value in Decision 3, and a project whose custom UI fails to draw still renders from the stored value.

## Alternatives considered

- **Packing stops into `FxUniforms` as `vec4` members.** Rejected: 8 stops is 8 colors plus 8 positions ≈ 12 float slots per gradient out of a 48-float pool, it forces the shader to hand-write the same interpolation loop this ADR exists to delete, and it cannot be keyframed as one unit.
- **A fixed stop count (e.g. always 8, unused stops transparent).** Rejected: it makes the common two-stop ramp carry six meaningless controls and makes "how many stops does this gradient have" unanswerable from the value.
- **Interpolating keyframes by stop index.** Rejected: adding a stop to one keyframe would visibly teleport unrelated stops. Resampling to the union of positions is slightly more work and is the only behavior that looks like what the user drew.
- **`Rgba8Unorm` LUT.** Rejected: it would be the only place in the pipeline that quantizes color, contradicting ADR-0021's precision policy for the sake of 12 KB.
- **Perceptual (Oklab/CIELAB) interpolation as the v1 default.** Rejected: it would make `hint:gradient` and `hint:color` disagree about what "halfway between these two colors" means. Worth adding later as an explicit opt-in.
- **Repairing malformed persisted values** (clamping, sorting, truncating). Rejected by the repository's fail-closed policy: silent repair hides corruption and makes the format's guarantees untestable.

## Consequences

### Benefits

- Retires hand-written `palette()` functions and their magic constants; the shipped thermal example collapses six color parameters plus its ramp code into one control.
- Palettes become animatable as a unit, which multi-color-parameter ramps never were.
- A ramp becomes a *user* control rather than an author control — retuning no longer means editing GLSL.
- Reuses ADR-0030's texture-binding rule, so the two new parameter kinds present one consistent story instead of two.

### Costs and risks

- This is the project's first custom UI. Event handling, hit testing, and Drawbot drawing are fiddly, host-specific, and hard to verify without a running AE — the highest-uncertainty surface in the current backlog, and the reason this ADR sequences after ADR-0030.
- `serde` becomes a direct dependency.
- The 8-stop cap and the sRGB interpolation space are persistent contracts; both are the kind of early choice ADR-0015 warns about, where sloppiness burns a format forever.
- A custom UI that misbehaves on a host year DynamicFX cannot test (AE 2023/2024) is invisible until someone reports it.

## Revisit conditions

- Host evidence that the arbitrary-data `interpolate` callback is not invoked for keyframed values on a target AE year would force a superseding ADR on Decision 6.
- Authors demonstrably needing more than 8 stops is an append-compatible format revision, recorded as a new ADR because the cap is persistent.
- Measured banding at 32-bpc traceable to 256 LUT samples would justify revisiting Decision 5.

## Verification obligations

- Rust unit tests: value round-trips through serialize/deserialize byte-exactly; malformed values (unsorted, out-of-range, empty, 9 stops) each yield `E54` and are never repaired; LUT baking matches hand-computed samples at stop boundaries and midpoints; keyframe interpolation between gradients with different stop counts resamples to the union of positions and is stable at t=0 and t=1; pool growth leaves every pre-existing declaration index unchanged.
- Host legs on Windows AE 2025 **and** AE 2026, each recorded separately: the editor draws, and add/drag/double-click-recolor/delete each change the rendered ramp; a keyframed gradient animates and matches the unit-tested interpolation at sampled times; save/reopen restores stops exactly; a duplicated instance does not share the value; aerender reproduces the ramp; a project saved with a gradient opens in a build without one and fails closed rather than crashing.
- Regression: the full M1-M7 batteries stay green on the same artifact, since this change touches the parameter topology and the render path.
