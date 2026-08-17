# ADR-0034: Point 3D parameters (`hint:point3d`)

- Status: Accepted
- Date: 2026-08-15
- Owners: DynamicFX project
- Related decisions: closes the Point 3D reservation in [ADR-0013](0013-paramid-grammar-and-pools.md) §3; follows [ADR-0026](0026-color-parameter-default-annotation.md) (the `vec3` default it works around) and [ADR-0011](0011-shader-abi-v1-core.md) §4 (value encodings)
- Related implementation: `src/binding.rs`, `src/host/params.rs`, `src/definition/param.rs`, `src/frontend/annotation.rs`, `src/frontend/glsl.rs`, `src/lib.rs`
- Related tests/audits: TR-0034-001 (to be recorded)

## Context

[ADR-0013](0013-paramid-grammar-and-pools.md) §3 declared the v1 pools and left Point 3D out: *"Point 3D kind reserved, pending host evidence."* That evidence gap has since closed — the project now has a full Windows host harness (M1-M7 batteries, per-year evidence records) and has added two pool kinds through it.

The gap this leaves is not cosmetic. [ADR-0026](0026-color-parameter-default-annotation.md) makes a shader's `vec3` a **Color** by default, and there is no way to say otherwise. A shader therefore cannot declare a three-component *spatial* value at all: a light direction, a 3D offset, a normal, or an axis all have to be faked with three separate floats, which loses the AE crosshair widget and the single keyframe stream that makes such a value editable.

`Point3DDef` is available in the crate (`set_default`, `set_value`, `value` over `(f64, f64, f64)`), so the host side is a solved problem.

## Decision

1. **New `PoolKind::Point3D`, capacity 8**, appended after the ADR-0033 growth so every released index keeps its position (ADR-0013 §5).

2. **A `vec3` becomes a Point 3D when annotated `hint:point3d`, and stays a Color otherwise.** The default is deliberately unchanged: flipping `vec3` to spatial would silently retype every colour parameter in every existing shader, which ADR-0026's whole purpose was to make explicit. `hint:point3d` on any type other than `vec3` is rejected the way the other hints already reject mismatches.

3. **Value encoding: `x` and `y` are normalized to the frame exactly as Point 2D is; `z` is passed in pixels, unnormalized.**

   There is no third frame dimension to divide by, and inventing one — dividing by height, or by a diagonal — would be a fabricated convention the user cannot predict. Passing `z` raw is honest, and `u_resolution` is already in the ABI head if a shader wants to scale it itself. This asymmetry is documented rather than hidden.

4. **Annotation defaults stay unsupported for this kind**, as they already are for Point 2D: a default would need the AEGP ThreeD stream-value plumbing that neither kind has. The pool default is AE's own.

## Alternatives considered

- **Make `vec3` default to Point 3D and require `hint:color` for colours.** Rejected: it retypes existing shaders silently, and colour is by far the more common `vec3` in this project's own examples.
- **Normalize `z` by the frame height.** Rejected: it reads as principled but is arbitrary — nothing about a comp makes height the depth unit, and a shader author would have to guess.
- **Reuse three Float slots with a naming convention.** Rejected: that is exactly the workaround this ADR exists to remove, and it gives three independent keyframe streams where the user expects one.
- **Wait for further host evidence, as ADR-0013 did.** Rejected: the evidence gap ADR-0013 named is closed, and the crate exposes the type completely.

## Consequences

### Benefits

- Three-component spatial values become expressible with the AE widget and one keyframe stream.
- The last *reserved* kind from ADR-0013 §3 is resolved, so the pool table stops carrying an open question.
- `hint:point3d` follows the shape of `hint:angle`/`hint:bool`/`hint:color` exactly — no new annotation mechanism.

### Costs and risks

- Eight more declared parameters, permanently.
- The `z` asymmetry is a documentation burden: `x,y` normalized and `z` in pixels will surprise someone who does not read it.
- A shader that wants a normalized `z` must divide it itself, and `u_resolution` gives it no depth to divide by.

## Revisit conditions

- A measured need for normalized depth — for instance if a later ADR introduces a comp-space Z convention for layer inputs — would justify revisiting Decision 3 in a superseding ADR.
- Real shaders exceeding 8 Point 3D slots is an ordinary append.

## Verification obligations

- Rust unit tests: `hint:point3d` on a `vec3` yields a `Point3D` declaration and binds to the Point3D pool; the same hint on any other member type is rejected; an un-annotated `vec3` is still a Color (the ADR-0026 default is untouched); pool growth leaves every pre-existing declaration index unchanged, including `Details` at 109, the ADR-0030 Layer slots and the ADR-0033 stop parameters.
- Host legs on Windows AE 2025 **and** AE 2026, recorded separately: the control appears as an AE 3D point widget; moving it changes the render; `x,y` land normalized and `z` in pixels as documented; keyframing animates; save/reopen restores; the M1-M7 batteries stay green, since this changes the parameter topology.
