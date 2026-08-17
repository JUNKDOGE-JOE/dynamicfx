# ADR-0004: In-place breaking rewrite and Windows host matrix

- Status: Accepted
- Date: 2026-08-08
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md)

## Context

The current prototype has not been released. Preserving its parameter order, hidden transport, sequence versions, sidecar, and migration code would constrain the target architecture without protecting real user projects. Creating a separate `DynamicFx2` would unnecessarily split the product.

## Decision

Rewrite the existing `DynamicFx` in place. Keep the brand and match name, but introduce a new parameter topology, StateToken, and sequence schema v1. Do not preserve or migrate prototype `SourceChannel`, flattened sequence v1-v3, legacy `SourceData`, sidecar, or parameter indexes.

The initial support matrix is Windows After Effects 2023, 2024, 2025, and 2026. Each year requires independent evidence. Continue version-specific installation under `Support Files/Plug-ins/DynamicFx`; never use shared MediaCore. Apple Silicon macOS follows only after Windows is stable.

## Alternatives considered

- Preserve all prototype compatibility: rejected because there are no released projects to protect.
- Create `DynamicFx2`: rejected because it would pollute discovery and maintenance during development.
- Support only AE 2025/2026: rejected in favor of the broader 2023-2026 Windows matrix demonstrated as realistic by the reference product.
- Build Windows/macOS simultaneously: deferred to reduce validation surface until Windows is stable.

## Consequences

### Benefits

- Persistent contracts can be designed correctly once.
- Legacy branches, hidden parameters, and debug transport can be deleted.
- One product identity remains.

### Costs and risks

- Existing developer AEP fixtures using the prototype may break and must be recreated.
- Installer, test harness, and host matrix expand to four AE years.
- Reusing the same match name makes it essential to avoid accidentally testing an old installed AEX.

## Revisit conditions

Only evidence of external prototype deployment that must be protected justifies a compatibility layer. That evidence must identify actual users/projects and migration cost.

## Verification obligations

- Artifact identity and install path recorded for every host run.
- Independent AE 2023/2024/2025/2026 load/render/save/aerender results.
- Test setup proves no stale prototype AEX is loaded.
