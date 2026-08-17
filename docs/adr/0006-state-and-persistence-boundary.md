# ADR-0006: State and persistence boundary

- Status: Accepted
- Date: 2026-08-08
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md)

## Context

AE UI and render work may use different project/sequence copies even when MFR is disabled. Render callbacks must not call AEGP. The unreleased prototype transport is intentionally discarded, so the rewrite needs a clean state boundary.

## Decision

Use a newly designed hidden primitive `StateToken` for atomic UI/render revision/status handoff and sequence schema v1 for the exact persisted snapshot. The snapshot contains LanguageId, committed source envelope, canonical EffectDefinition, BindingPlan, full identities, state, commit status, lengths, limits, and integrity data.

The token is a hint/revision, not a complete shader identity. Render clones combine token, validated snapshot, and process registry. They rebuild missing artifacts from the exact snapshot. Corrupt or unsupported data fails closed with diagnostic and input pass-through.

Compile transaction/generation is session-local and is never persisted. GPU modules, pipelines, transient resources, history pixels, and editor state are never persisted.

## Alternatives considered

- Reuse prototype SourceChannel/v3 flatten: rejected by the breaking-rewrite decision.
- Render-side AEGP source reads: rejected for thread/host safety.
- Make hidden arbitrary data the sole authority: not selected without AE 2023-2026 transport evidence; avoid duplicate payload authorities.

## Consequences

### Benefits

- Save/reopen, render clone, and registry reconstruction share one explicit schema.
- Session concurrency cannot leak into project state.
- GPU resources remain rebuildable and device-local.

### Costs and risks

- StateToken layout and binary schema become persistent contracts.
- Payload size/validation and corruption tests are mandatory.
- Source authority versus snapshot precedence must be enforced exactly.

## Revisit conditions

AE host testing may remove a redundant DefinitionData parameter or alter the primitive token layout before release, but any accepted persistent schema change requires a superseding ADR.

## Verification obligations

- UI/render clone and aerender without AEGP.
- Save/reopen and registry miss rebuild.
- Corrupt, truncated, oversized, unknown-version payloads.
- Exact source change invalidates stale snapshot eligibility.
