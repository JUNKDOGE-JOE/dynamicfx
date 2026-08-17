# ADR-0035: Path parameters (`hint:path`)

- Status: Accepted
- Date: 2026-08-15
- Owners: DynamicFX project
- Related decisions: extends [ADR-0030](0030-layer-input-parameters.md) §1 (graph resources fed by a parameter) and [ADR-0032](0032-gradients-are-graph-resources.md) (one extra-input binding rule); constrained by [ADR-0011](0011-shader-abi-v1-core.md) §3 (set 0, one uniform block, texture bindings from 3)
- Related implementation: `src/binding.rs`, `src/host/params.rs`, `src/frontend/annotation.rs`, `src/frontend/grammar.rs`, `src/lib.rs`
- Related tests/audits: TR-0035-001 (to be recorded)

## Context

A mask is the one piece of AE authoring a shader still cannot see. Layers arrived with [ADR-0030](0030-layer-input-parameters.md) and gradients with [ADR-0031](0031-gradient-parameters.md), but the shapes a user draws directly on the layer — the thing they reach for to say *here* — remain invisible to the runtime.

The host side is fully available in the crate: `PathQuerySuite` enumerates the source layer's paths and checks one out at a time, and `PathDataSuite` walks it (`path_num_segments`, `path_vertex_info` returning `PF_PathVertex { x, y, tan_in_x, tan_in_y, tan_out_x, tan_out_y }`, plus arc-length preparation and evaluation). `PF_Param_PATH` (`PathDef`) is the selector the user picks a mask with.

What is genuinely undecided is the **ABI**: a path is a vertex sequence, not an image, and Shader ABI v1 has one uniform block and a texture-binding space. Nothing in the ABI currently carries variable-length vertex data.

## Decision

1. **New `PoolKind::Path`, capacity 2**, appended after the ADR-0034 growth. Two because each bound path costs a per-frame checkout and walk, and because a shader wanting more than two masks is better served by a layer input.

2. **`hint:path` declares a graph resource, exactly as `hint:layer` and `hint:gradient` do** — named in `@graph`, bound by the one extra-input rule, read-only, never a pass name or output. No new declaration mechanism.

3. **A path reaches the shader as an `N x 1` `Rgba32Float` texture of vertices**, where texel `i` carries `(x, y, tan_out_x, tan_out_y)` for vertex `i`, and a second row carries `(tan_in_x, tan_in_y, 0, 1)` — so the texture is `N x 2` and a shader that only wants positions reads row 0 and ignores row 1.

   All four coordinates are **normalized to the frame**, the same convention Point 2D already uses, so a path vertex and a `hint:point` parameter mean the same thing in the same shader.

4. **The vertex count is the texture width**, read with `textureSize(u_path, 0).x`. No companion uniform, no count parameter, and therefore no way for the count and the data to disagree — which is the failure mode a separate count invites.

5. **An unassigned selector, a deleted mask, or a path with no segments binds a 1x2 all-zero texture**, never an error. A shader reading an unset path sees one degenerate vertex and still renders, matching ADR-0030 §5's rule for layers.

6. **Beziers are delivered, not flattened.** The tangents are what make a mask a curve; sampling it into a polyline in the host would throw away precision the shader may want and would force a sampling-density decision that belongs to the shader, not the runtime. `PathDataSuite`'s arc-length helpers stay unused in this version.

7. **A path input in a temporal graph is refused**, with the same `E7` diagnostic and the same reasoning as [ADR-0030](0030-layer-input-parameters.md) §6: windowed re-simulation would need the path checked out at every iterated frame, a cost never measured.

## Alternatives considered

- **Rasterize the mask into a coverage texture.** Rejected for this version: it needs a scanline rasterizer with anti-aliasing rules the project would then own forever, and it discards the geometry — a shader wanting an outline, a distance field, or motion along the curve could not recover it. Worth revisiting as an *additional* `hint:path:mask` once there is demand.
- **Pass vertices in a storage buffer.** Rejected: ABI v1 fixes set 0 to one uniform block plus texture bindings ([ADR-0011](0011-shader-abi-v1-core.md) §3). Adding a storage buffer is an ABI version bump for one feature.
- **Pack vertices into the `FxUniforms` block as a fixed array.** Rejected: it burns the float budget for a worst case that is usually unused, and it caps vertices at a number chosen now.
- **A separate `Vertices` count parameter.** Rejected by Decision 4's reasoning: two sources for one fact, and the failure is silent when they diverge.
- **Flatten Beziers to a polyline at a fixed density.** Rejected by Decision 6: the density is the shader's business.

## Consequences

### Benefits

- Masks — the most direct thing a user can draw — become shader input, with no new binding or declaration mechanism.
- Reuses the ADR-0030/0032 external-resource path end to end: same grammar rule, same binding rule, same texture upload.
- Curve data survives, so outlines, distance fields and travel-along-path are all expressible by the shader rather than pre-decided by the runtime.
- The count cannot disagree with the data.

### Costs and risks

- A per-frame checkout and vertex walk per bound path; the count is unbounded by AE, so a pathological mask is a real cost the user can create. Per-render span logging must cover it.
- `Rgba32Float` sampling requires `FLOAT32_FILTERABLE`, which `Depth::required_features` only guarantees for the deep working formats. At 8-bpc the path texture must therefore be fetched with `texelFetch`, not `texture()` — a documentation obligation, and the reason positions are normalized rather than left in pixels.
- Two more declared parameters, and a new pool kind to carry through persistence.
- Shaders must handle a degenerate 1-vertex path (Decision 5) or they will divide by zero on an unset selector.

## Revisit conditions

- Demand for a rasterized mask is an additive `hint:path:mask` variant, not a change to this decision.
- A measured path-checkout cost low enough to afford `W` checkouts per frame would justify lifting the temporal refusal, as it would for ADR-0030.
- Evidence that the two-row layout is awkward in practice would justify a superseding ABI, since the row meaning is a persistent contract the moment a shader reads it.

## Verification obligations

- Rust unit tests: `hint:path` names a graph resource that no pass writes and that cannot name a pass or be written (`E6` with line); a path in a graph that reads `prev` yields `E7`; the vertex-to-texel encoding round-trips a known vertex list, including tangents and frame normalization; an empty path produces the documented 1x2 zero texture; pool growth leaves every pre-existing declaration index unchanged.
- Host legs on Windows AE 2025 **and** AE 2026, recorded separately: a mask drawn on the layer appears in the selector; a shader reading vertex 0 marks the right place; editing the mask changes the render on the next frame; an animated mask updates per frame; `textureSize` reports the vertex count AE reports; an unassigned selector renders rather than failing; save/reopen and aerender reproduce; the M1-M7 batteries stay green.
