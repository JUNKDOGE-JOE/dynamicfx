# ADR-0002: Extensible language frontends

- Status: Accepted
- Date: 2026-08-08
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md)

## Context

GLSL is the required first language, but the runtime may later support WGSL or other source languages. Hard-coding a GLSL/WGSL branch would force compiler, persistence, identity, and UI redesign whenever a language is added.

## Decision

Add a non-time-varying `Language` Popup to `DynamicFx`. GLSL is the default. Each language has a stable numeric `LanguageId` and a registered `LanguageFrontend` implementation.

Every frontend converts source text into the same neutral output: pass sources, graph declaration, parameter/resource declarations, source mapping, and diagnostics. Frontends do not access AE, create AE parameters, allocate GPU resources, or choose cache policy.

## Alternatives considered

- GLSL only forever: rejected because it closes the planned language extension boundary.
- WGSL-only rewrite: rejected because GLSL expression rendering is the primary product goal.
- Auto-detect language from source: rejected as the authoritative mechanism because detection is ambiguous and makes persistence/error behavior unstable.

## Consequences

### Benefits

- New languages do not redefine RenderGraph, parameters, persistence, or caches.
- Language selection is explicit and testable.
- Frontend version can participate in module identity.

### Costs and risks

- Language IDs become persistent protocol and cannot be reordered.
- Every frontend must meet the same ABI, source-map, diagnostic, and conformance tests.
- The Popup has finite predeclared entries under AE parameter topology constraints; extension policy must be planned.

## Revisit conditions

A superseding ADR is required if AE Popup topology makes the expected language growth impossible. Any replacement must keep an explicit, stable language identity.

## Verification obligations

- Default GLSL in AE 2023-2026.
- Stable Language ID save/reopen behavior.
- Unsupported ID and frontend-version diagnostics.
- Same neutral model invariants across each added frontend.
