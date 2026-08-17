# ADR-0030: Layer input parameters (`hint:layer`)

- Status: Accepted
- Date: 2026-08-15
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md) §4.2, §7.1, §9
- Related decisions: [ADR-0013](0013-paramid-grammar-and-pools.md) (pool growth), [ADR-0018](0018-envelope-grammar-v1.md) (graph grammar), [ADR-0011](0011-shader-abi-v1-core.md) (binding budget), [ADR-0025](0025-windowed-resimulation.md) (temporal law)
- Related implementation: `src/binding.rs`, `src/host/params.rs`, `src/frontend/grammar.rs`, `src/frontend/glsl.rs`, `src/plan.rs`, `src/lib.rs`
- Related tests/audits: TR-0030-001 (to be recorded); public issue [#1](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/1)

## Context

A shader can currently read three things: the effect's own layer (`input`), pass intermediates, and `prev` (the previous frame's output). It cannot read **another layer in the comp**. That blocks the entire class of effects that need a second image — displacement maps, external mattes, LUT strips, and one layer driving another's look — and it is the most-requested gap on the public tracker.

The host capability exists and is reachable from the crate in use: `PF_Param_LAYER` (`ae::LayerDef`), `PreRenderCallbacks::checkout_layer`, and `SmartRenderCallbacks::checkout_layer_pixels` are all present in `after-effects` 0.4. DynamicFX already runs the SmartFX entry points these require (M5 added `PreRender`/`SmartRender` for float color).

Three things are genuinely unfixed and must be decided before code exists, because each becomes persistent or user-visible on first release:

1. **How a layer enters the shader.** `@param` annotations today name members of the `FxUniforms` block. A layer is a *texture binding*, not a block member, so the existing annotation path has nothing to attach to.
2. **What `uv` means** when the referenced layer differs in size, position, scale, or rotation from the effect's layer.
3. **What happens when a layer input meets temporal re-simulation**, where [ADR-0025](0025-windowed-resimulation.md) re-runs up to `W` frames for every request.

## Decision

1. **A `hint:layer` annotation declares a graph resource, not a uniform member.** The name becomes usable exactly where an intermediate is usable — as a pass input in the `@graph` block — and is sourced from an AE Layer parameter instead of a pass output:

   ```glsl
   // @param depth_map label:"Depth Map" hint:layer
   ```
   ```
   @graph
   pass main: input, depth_map -> output
   @end
   ```

   In the pass body it is an ordinary texture binding, assigned by the same rule that already governs extra inputs (ADR-0011): first input at binding 0, sampler 1, `FxUniforms` 2, remaining inputs from binding 3 in graph-declaration order. A layer name occupies no `FxUniforms` space and no float budget.

2. **New `PoolKind::Layer`, capacity 4, appended to `V1_POOLS`.** Growth is append-only per [ADR-0013](0013-paramid-grammar-and-pools.md) §5: every released parameter index keeps its kind and position, and the new slots follow the existing 104 plus the ADR-0028 `Details` button. Capacity 4 is deliberate — each bound layer costs one full layer render plus one upload per frame, so a large pool would sell a cost the runtime cannot hide. Growth beyond 4 is an ordinary append and needs no superseding ADR.

3. **Layer names are read-only graph resources.** They may appear as pass inputs any number of times; they may never be a pass name and may never be written. Violations are `E6` with the offending 1-based line, matching the `prev` rules already in [ADR-0018](0018-envelope-grammar-v1.md).

4. **`uv` is comp space.** A referenced layer is sampled as it is composited — its own transform applied, transparent black outside its bounds — so `v_uv` addresses the same point in every texture a pass reads, including `input` and intermediates. This is the only choice under which a displacement map, a matte, and the effect's own layer can be sampled with one coordinate; the alternative (each layer in its own pixel space) makes the common case require per-layer correction math that the shader has no inputs to compute.

5. **`None` binds a transparent-black texture, never an error.** A shader that reads an unassigned layer input sees all zeros and still renders. Adding a layer parameter therefore never breaks an existing render, and an author can ship a graph whose second input is optional.

6. **A layer input combined with a temporal graph is refused in this feature's first release**, with the new diagnostic `E7 LayerInTemporalGraph` (source/envelope family, appended per [ADR-0015](0015-statetoken-and-diagnostics.md) §4). ("First release" here means the initial shipped scope of layer inputs — not a format version; the `v1` used elsewhere in this repository always names a persistent format contract.)

   Windowed re-simulation re-runs up to `W` frames per request. A correct layer read would have to check the referenced layer out at each iterated time, costing `W` layer renders per frame — `@window 16` with four bound layers is 64 layer renders per frame, a cost never measured. Rather than ship a silent wrong answer (sampling the requested frame's pixels for every iteration) or an unbudgeted one, the combination fails closed with a diagnostic and pass-through. Supporting it is a later ADR with its own measurement.

## Alternatives considered

- **Layer as a `sampler2D` uniform member declared in `FxUniforms`.** Rejected: std140 blocks cannot carry opaque types, and it would split texture binding across two unrelated mechanisms.
- **A dedicated `@layer` envelope block instead of `hint:layer`.** Rejected: the graph already has a resource namespace and a binding-assignment rule; a second declaration site would need its own ordering, aliasing, and validation rules for no gain.
- **Layer-space `uv` (sample the referenced layer's raw pixels, ignoring its transform).** Rejected as the default for the reason in Decision 4. It remains the right semantic for LUT strips and texture atlases, so a future `hint:layer:raw` variant may add it as an opt-in without disturbing this decision.
- **Sampling the requested frame's layer pixels for every temporal iteration.** Rejected: it silently produces a result that is neither physically meaningful nor stable across window sizes, which is exactly the class of quiet wrongness the repository's fail-closed policy exists to prevent.
- **Unbounded Layer pool.** Rejected: pool capacity is a persistent contract and each slot carries a real per-frame cost.

## Consequences

### Benefits

- Displacement, external mattes, LUT strips, and cross-layer looks become expressible with no new mechanism beyond one annotation and one graph name.
- One coordinate space across every texture in a pass keeps shaders readable and keeps `uv` meaning one thing.
- Optional inputs (`None` → zeros) mean adding a layer parameter is never a breaking change to an existing project.
- The binding rule, grammar rule, and pool-growth rule are all reuses of contracts already Accepted and tested.

### Costs and risks

- Each bound layer is a full layer render plus an upload every frame; four bound layers on a 4K comp is a real cost the user can create without warning. Per-frame span logging already exists (M7) and must cover layer checkout so the cost is measurable.
- Comp-space sampling means a referenced layer's transform is baked in; an author who wanted raw pixels has no v1 escape hatch.
- The temporal refusal is a visible limitation on the public tracker; it will read as an incomplete feature until a later ADR lifts it.
- Layer parameters cannot be keyframed to *different layers* over time (an AE property of layer params, not a DynamicFX choice) — worth documenting so it is not reported as a defect.

## Revisit conditions

- Measured evidence that comp-space checkout does not deliver the referenced layer aligned as composited on a target AE year would force a superseding ADR on `uv` semantics before release.
- A measured layer-checkout cost low enough that `W` checkouts per frame is affordable would justify a superseding ADR lifting the temporal refusal.
- Real projects hitting the 4-slot capacity are an ordinary append, not a revisit.

## Verification obligations

- Rust unit tests: grammar accepts a layer name as a pass input and rejects it as a pass output or pass name (`E6` with line); binding assignment places layer inputs in graph-declaration order alongside intermediates; a temporal graph containing a layer input yields `E7`; pool growth keeps every pre-existing declaration index unchanged (the `declaration_order()` contract test).
- Host legs on Windows AE 2025 **and** AE 2026, each recorded separately: a displacement-style graph reading a second layer renders numerically as expected; `None` renders with the input untouched rather than failing; a referenced layer with a non-identity transform samples aligned as composited; the referenced layer animating over time updates per frame; a duplicated effect instance does not share the reference; save/reopen and aerender both reproduce the result.
- Regression: the full M1-M7 batteries stay green on the same artifact, since this change touches the parameter topology, the grammar, and the render path.
