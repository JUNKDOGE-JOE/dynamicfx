# ADR-0003: RenderGraph is the core execution model

- Status: Accepted
- Date: 2026-08-08
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md)

## Context

Multi-pass effects, intermediate textures, mipmaps, and temporal feedback change persistence, identities, resource lifetime, scheduling, image formats, SmartRender, and MFR eligibility. Adding them after freezing a single-fragment definition would require a second runtime or disruptive rewrite.

## Decision

`EffectDefinition` contains a first-class `RenderGraph` from Phase 1. A raw single-pass shader lowers to a graph with one fragment pass. Single-pass and multi-pass use the same validator, scheduler, execution plan, cache domains, parameter model, and executor.

The graph domain includes pass definitions, resource definitions, edges, one final output, execution class, and explicit history resources. Ordinary same-frame edges form a DAG; temporal cycles are represented only through `HistoryResource` semantics.

## Alternatives considered

- Ship a single-pass runtime and add graphs later: rejected because it would freeze the wrong persistent and cache models.
- Make multi-pass a separate effect/plugin: rejected because parameters, language frontends, and authoring would fragment.
- Restrict DynamicFX permanently to single-pass: rejected by product requirements.

## Consequences

### Benefits

- One runtime covers simple and advanced effects.
- Per-pass pipeline caching and resource lifetime are explicit from the start.
- Temporal eligibility and reset behavior can be modeled honestly.

### Costs and risks

- M1 requires minimal graph types even for the first visible frame.
- Source envelope, graph validation, resource scheduling, and diagnostics add early design cost.
- Temporal graphs cannot claim arbitrary frame order or MFR until proven.

## Revisit conditions

The graph may be extended by ADR, but replacing it with a single-pass-only core would contradict the product requirement and needs explicit user approval.

## Verification obligations

- Raw GLSL one-pass graph.
- Two-pass separable blur.
- Cycle/missing input/multiple writer/read-before-write rejection.
- History reset matrix and bounded resources.
- Per-pass and graph identity/cache tests.
