# ADR-0001: Expression authority and open runtime

- Status: Accepted
- Date: 2026-08-08
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md)

## Context

DynamicFX's core purpose is to read shader source from an After Effects expression and render it. The runtime must remain directly controllable by users, JSX, ae-mcp, and future tools without requiring a proprietary panel or service.

## Decision

The non-time-varying `Language` parameter plus the committed `Source.expression` are the authoritative user intent. The compiled `EffectDefinition` and persisted snapshot are derived execution state; they may reconstruct render state but must not override a newly observed authoritative input.

DynamicFX remains an open shader runtime exposed through ordinary AE properties. A future editor is optional and writes through the same boundary.

## Alternatives considered

- Make sequence data the sole authority: rejected because shader state would become an opaque private store.
- Make an external package or panel authoritative: rejected because project rendering would depend on external tooling or files.
- Introduce a required private IPC channel: rejected because it weakens automation and project portability.

## Consequences

### Benefits

- JSX, ae-mcp, and users share one control path.
- Projects do not require an editor, account, network, or local service to render.
- Shader intent remains inspectable in AE.

### Costs and risks

- Source and persisted execution snapshots can coexist, so precedence and stale-state rules require tests.
- Expression transport needs a versioned, size-limited source envelope for multi-pass graphs.

## Revisit conditions

Only evidence that AE cannot reliably persist or expose the required expression payload across the Windows 2023-2026 host matrix justifies reconsideration, and any replacement must retain an ordinary-property automation path.

## Verification obligations

- Expression-only writes in all four Windows AE targets.
- Save/reopen and render-clone reconstruction without editor involvement.
- Invalid/new source cannot revive a stale persisted definition.
