# ADR-0008: Product scope and delivery order

- Status: Accepted
- Date: 2026-08-08
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md)
- Related roadmap: [../ROADMAP.md](../ROADMAP.md)

## Context

The project needs visible results quickly without letting UI, cloud, commercial infrastructure, or premature optimization delay the shader runtime. Professional AE correctness also requires high precision before performance claims.

## Decision

Deliver vertical milestones in this order: architecture contract, new first frame, keyframed parameters, persistence/render clone, real multi-pass graph, 16/32-bpc image quality, temporal feedback, then performance/SmartRender/MFR.

Image correctness precedes performance. The independent editor is deferred. A local effect package may follow the runtime; online catalog, cloud, account, store, licensing, required WebSocket, and telemetry are outside core scope. Windows AE 2023-2026 precedes Apple Silicon macOS.

Every visible milestone produces an audit with visible evidence, exact tests, limitations, risks, and reproduction steps.

## Alternatives considered

- Build an editor/content platform first: rejected due to low current runtime benefit and large unrelated scope.
- Optimize the current 8-bit prototype first: rejected because the path will be replaced and precision semantics are not correct.
- Delay multi-pass until an authoring phase: rejected by ADR-0003.
- Develop Windows/macOS simultaneously: deferred until the Windows matrix is stable.

## Consequences

### Benefits

- The user sees an AE result at every development node.
- Correctness contracts are fixed before optimization.
- Handoffs have concrete milestone/audit boundaries.

### Costs and risks

- Some attractive UI and distribution features arrive late.
- Multi-pass domain cost exists before advanced effects are visible.
- Four Windows AE years make host validation substantial.

## Revisit conditions

Ordering changes require evidence that a later item blocks the current milestone or that product priorities changed explicitly. Record the change in a superseding ADR and roadmap update before implementation.

## Verification obligations

- Every milestone exit maps to Test Matrix entries and an audit.
- No performance claim without pixel-equivalence evidence.
- No host support claim generalized across AE years.
- Editor/package removal must not prevent project render.
