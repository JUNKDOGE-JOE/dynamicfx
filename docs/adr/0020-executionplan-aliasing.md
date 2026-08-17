# ADR-0020: ExecutionPlan resource aliasing (v1)

- Status: Accepted
- Date: 2026-08-13
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md) §9.2, §14
- Related decisions: [ADR-0007](0007-identity-and-cache-boundaries.md), [ADR-0018](0018-envelope-grammar-v1.md), [ADR-0019](0019-intermediate-format-policy.md)
- Related tests/audits: TR-M4-001 in [../TEST_MATRIX.md](../TEST_MATRIX.md); `docs/audits/04-multipass-graph.md`

## Context

Without aliasing, a 16-pass graph at 4K holds up to 15 intermediates ≈ 500 MB of transient textures. Architecture §9.2 makes the immutable `ExecutionPlan` the place where lifetimes and aliasing are decided once per (graph, extent, format) — never per frame. The rules must be deterministic (plans are compared and tested) and invisible (aliasing is an optimization, forbidden from changing output).

## Decision

1. **Lifetimes.** Plan construction computes, in topological order, each intermediate's lifetime `[first-writing step, last-reading step]`. Topological order itself is deterministic: ready passes are scheduled in manifest declaration order.
2. **Aliasing rule.** Two intermediates may share one physical texture iff their formats and extents match and their lifetimes do not overlap (a write step may reuse a texture whose last read completed in an earlier step — never the same step). Assignment is first-fit over physical slots in lifetime order; ties broken by manifest declaration order. The result is part of the immutable plan.
3. **Determinism contract.** The same (graph, extent, format) inputs produce the same plan, including physical assignments — plan construction uses no maps with iteration-order dependence. Golden plan tests pin this.
4. **Never aliased:** the effect input, the final output, and (from M6) history resources. Aliasing is per-plan; nothing is shared across instances or frames-in-flight (v1 renders serially; M7 revisits under MFR).
5. **Observability guard.** Aliasing must be semantics-free: a debug/env switch (`DYNAMICFX_NO_ALIAS`) disables it for A/B verification, and the test obligation below makes equal-output-with-and-without part of the contract.
6. **Standard path, not best-effort:** aliasing runs for every multi-pass plan (a chain of N passes needs exactly 2 physical intermediates); peak transient memory is recorded in the plan and logged for evidence (ADR-0014 §6 spirit).

## Alternatives considered

- No aliasing in v1 ("optimize later"): rejected; 4K chains would ship with hundreds of MB of avoidable transient footprint, and retrofitting aliasing later would perturb a frozen plan shape.
- A general graph-coloring optimum: rejected; first-fit over interval lifetimes is optimal for chains, near-optimal for real effect graphs, and far easier to make deterministic and testable.
- Aliasing across frames/instances: rejected for v1; cross-frame reuse is an M7 (and M6-history) concern with MFR implications this ADR must not preclude or prejudge.

## Consequences

### Benefits

- Chain-shaped graphs (the dominant real case) run with two intermediates regardless of length.
- Deterministic plans make "the plan changed" a reviewable diff and a testable golden.
- The kill switch turns any suspected aliasing bug into a one-variable experiment.

### Costs and risks

- First-fit is not globally optimal for adversarial DAGs — accepted; the plan logs peak memory so regressions are visible.
- The determinism contract constrains future parallel plan construction (M7 must preserve it or supersede this ADR).

## Revisit conditions

M6 history resources and M7 MFR/pooling both touch this territory by design and arrive with their own ADRs; anything that would share textures across frames or instances supersedes §4 explicitly. Measured plans where first-fit's waste is material justify a smarter allocator behind the same determinism contract.

## Verification obligations

- Rust unit tests: golden plans for a chain (2 physical intermediates), a diamond (a→b,c→d), and a worst-case all-live graph; determinism across repeated construction; the never-alias set; lifetime edge case (read and write in adjacent steps may share, same step may not).
- TR-M4-001 host leg: the two-pass blur renders identically with `DYNAMICFX_NO_ALIAS` set and unset (numeric probes both ways).
