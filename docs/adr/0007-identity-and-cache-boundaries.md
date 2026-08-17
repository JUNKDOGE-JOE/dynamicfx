# ADR-0007: Layered identities and cache boundaries

- Status: Accepted
- Date: 2026-08-08
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md)

## Context

Source, compiler artifacts, graph topology, UI parameter metadata, GPU pipelines, execution plans, and frame resources change for different reasons. A single source or definition hash would either reuse incompatible resources or invalidate expensive GPU state for UI-only changes.

## Decision

Use separate typed identities:

- `ModuleHash`: canonical pass source + LanguageId + frontend version + Shader ABI;
- `ArtifactHash`: ModuleHash + compiler backend/version/options;
- `GraphHash`: pass ModuleHashes + canonical topology + static resources;
- `DefinitionHash`: GraphHash + parameter schema + effect metadata;
- `PipelineKey`: ArtifactHash + pass pipeline state + target format + device generation;
- `ExecutionPlanKey`: GraphHash + resolved formats/extents/capabilities;
- `FrameResourceKey`: device generation + extent + format + usage/lifetime class.

`PipelineKey` must not be derived directly from `DefinitionHash`. Dynamic keyframed values, labels, UI ordering, time, and current frame are excluded from pipeline identity.

## Alternatives considered

- One source hash for all caches: rejected because multi-pass/device/format resources would collide semantically.
- PipelineKey based on DefinitionHash: rejected because UI metadata changes would rebuild GPU pipelines.
- Stringly typed hashes: rejected because misuse across cache layers would be easy.

## Consequences

### Benefits

- UI changes do not invalidate GPU pipelines.
- Device loss and target-format changes cannot reuse stale resources.
- Per-pass artifacts are shared across graphs where safe.

### Costs and risks

- More identity types and domain-separation rules.
- Canonical serialization must be deterministic.
- Cache diagnostics and tests must name the exact key layer.

## Revisit conditions

Algorithms and serialized digest representation may change before release through format ADRs, but the semantic separation requires a superseding ADR to collapse.

## Verification obligations

- Canonical hash stability/golden tests.
- UI label/order changes do not change PipelineKey.
- Source/compiler/device/format/topology changes invalidate only required layers.
- Collision handling fails closed where exact data is unavailable.
