# ADR-0032: Gradients are graph resources (supersedes ADR-0031 §2)

- Status: Accepted
- Date: 2026-08-15
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md) §4.2, §7.1
- Related decisions: supersedes [ADR-0031](0031-gradient-parameters.md) §2 only; builds on [ADR-0030](0030-layer-input-parameters.md) §1, [ADR-0011](0011-shader-abi-v1-core.md) §3 (binding budget), [ADR-0018](0018-envelope-grammar-v1.md) (graph grammar)
- Related implementation: `src/frontend/grammar.rs`, `src/plan.rs`, `src/lib.rs`
- Related tests/audits: TR-0031-001 (to be recorded); public issue [#2](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/2)

## Context

[ADR-0031](0031-gradient-parameters.md) §2 states two things that cannot both hold. It says a gradient's texture binding is "assigned by the existing extra-input rule (ADR-0011), exactly as [ADR-0030](0030-layer-input-parameters.md) does for layers", and it says a gradient "is not nameable in `@graph`".

The extra-input rule *is* graph order: [ADR-0030](0030-layer-input-parameters.md) §1 assigns the first input to binding 0, sampler 1, `FxUniforms` 2, and remaining inputs from binding 3 **in graph-declaration order**. A resource absent from `@graph` has no position in that order, so §2 named a rule that cannot produce an answer for it. The defect was authored into the ADR and found while implementing ADR-0030, before any gradient code existed.

ADR-0030 is now implemented, and its machinery settles the question. The scheduler and the execution plan both identify externally-fed resources by a single structural property — **read by some pass, written by none** — rather than by a name list. `input`, `prev`, and layer inputs all satisfy it. A gradient satisfies it too.

## Decision

**A gradient is a graph resource, named in `@graph` exactly like a layer input:**

```glsl
// @param heat_ramp label:"Heat Ramp" hint:gradient
```
```
@graph
pass main: input, heat_ramp -> output
@end
```

Its binding follows the one extra-input rule already in force, with no gradient-specific ordering. The rules ADR-0030 §3 fixes for layer names apply unchanged: a gradient name may be read by any number of passes, may never name a pass, and may never be written; violations are `E6` with the offending line.

Every other decision in ADR-0031 stands as Accepted — the 8-stop persistent format, `E54` fail-closed validation, straight-sRGB interpolation, the 256×1 `Rgba32Float` LUT, keyframeability with union resampling, and the Drawbot editor are untouched.

## Alternatives considered

- **Bind gradients after all graph inputs, in `@param` declaration order** (the rule ADR-0031 §2 implied but did not state). Rejected: it gives the project two binding rules instead of one, and it makes a shader's `binding = N` numbers depend on the order of comment lines — a reader could not determine a texture's binding without counting annotations elsewhere in the file.
- **Keep gradients out of `@graph` and assign bindings implicitly by first use in the body.** Rejected: binding assignment would then depend on parsing the shader body, which the grammar layer deliberately does not do.
- **Leave ADR-0031 §2 as written and resolve the contradiction in code.** Rejected by the repository's ADR policy: a decision that cannot be executed as written is not a decision, and quietly picking one of its two halves during implementation is exactly the silent divergence the policy exists to prevent.

## Consequences

### Benefits

- One binding rule covers `input`, `prev`, layer inputs, and gradients; there is no per-kind special case to learn, document, or test.
- The implementation is already built: the "read but never written" rule added for ADR-0030 admits gradients with no scheduler, plan, or grammar change.
- A shader's texture bindings are readable from the `@graph` block alone, without scanning annotations.

### Costs and risks

- The author types the gradient's name once more, in the pass line.
- A gradient has no extent of its own, so it is the first graph resource whose size is fixed by the runtime (256×1) rather than by the frame. Nothing in the grammar expresses that, so a reader could wrongly expect `uv.y` to be meaningful when sampling one; ADR-0031's `vec2(t, 0.5)` idiom must stay in the user documentation.
- ADR-0031 is now a partially superseded record, which readers must follow through this ADR.

## Revisit conditions

Evidence that graph-declared gradients collide with a future resource kind that genuinely has no graph position (a scalar-only arbitrary parameter, say) would justify revisiting the single-rule stance — but that kind would not be a texture binding at all, so it is unlikely to reach this rule.

## Verification obligations

- Rust unit tests: a gradient named in `@graph` compiles and receives the binding its graph position implies; a gradient used as a pass output or pass name is `E6` with the line; a graph mixing a layer input and a gradient assigns both bindings in graph order.
- The host obligations in [ADR-0031](0031-gradient-parameters.md) are unchanged and still apply.
