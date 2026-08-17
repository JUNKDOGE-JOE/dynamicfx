# ADR-0009: Staged format-ADR acceptance and M0 transport spike

- Status: Accepted
- Date: 2026-08-10
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md)
- Related roadmap: [../ROADMAP.md](../ROADMAP.md)
- Related tests/audits: [../TEST_MATRIX.md](../TEST_MATRIX.md); [../audits/00-architecture-contract.md](../audits/00-architecture-contract.md)

## Context

M0 originally required all twelve format ADRs to be Accepted before any runtime code. Several of those contracts can only be validated by implementation feedback (ABI numeric conventions, envelope escaping, execution-plan aliasing), and byte-level contracts written with no implementation feedback are likely to require superseding revisions, which the deliberately heavy ADR process makes expensive.

The product is unreleased (ADR-0004): a format becomes binding only when a milestone first implements or persists it. Freezing everything up front therefore protects no released data while delaying the code that would inform the formats.

Separately, the architecture's largest physical assumption — that AE expressions reliably carry large committed source payloads across UI/render clones, save/reopen, aerender, and all four Windows AE years (the revisit condition of ADR-0001) — had no scheduled early verification, while prototype history shows transport is where host behavior bites. Undo/redo and project-dirty semantics of programmatic state publication, a known AE plugin failure area, and the stable diagnostic code registry required by repository policy were also covered by no planned ADR.

## Decision

1. Format ADRs are accepted in stages, at the entry of the milestone that first implements or persists each contract:
   - **Before M0 exit (gates M1 implementation):**
     - ADR-0010 stable Language numeric IDs;
     - ADR-0011 Shader ABI v1 core builtin set and semantics — numeric conventions are finalized against M1 fixtures; pre-release ABI version bumps remain allowed;
     - ADR-0012 source envelope version marker and reserved prefix, sufficient to distinguish raw single-pass source from a versioned envelope from the first frame; the full grammar is deferred to M4 entry;
     - ADR-0013 ParamId grammar, aliases, initial pool capacities, and the append-only pool growth policy;
     - ADR-0014 Windows AE 2023-2026 build/install/test protocol, including the supported wgpu backend/adapter policy and the automated host-harness requirements.
   - **At M3 entry:** StateToken layout including undo/redo and project-dirty semantics and the stable diagnostic/status code registry; sequence schema v1 codec, limits, and checksum; hash algorithm, canonical serialization, and hash domain separation.
   - **At M4 entry:** full multi-pass envelope grammar and escaping; intermediate format policy; ExecutionPlan resource aliasing rules.
   - **At M6 entry:** temporal seek/reset semantics; history format policy.
2. A staged contract that is not yet Accepted must not be persisted and must not be treated as stable: interim encodings are session-local, explicitly non-contractual, and must be replaceable without migration.
3. M0 additionally requires a transport feasibility spike executed on at least one target AE year (2025 recommended). The spike measures host behavior, so the prototype AEX and JSX probe scripts are acceptable instruments; scenarios and evidence rules are TR-M0-002 through TR-M0-007 in the test matrix. Its measurements feed envelope size limits (ADR-0012), Popup pool viability (ADR-0013), and the M3 StateToken/DefinitionData decisions.
4. The milestone order M0-M7 is unchanged. This ADR refines gating within ADR-0008's delivery order and does not supersede it.

## Alternatives considered

- Accept all twelve format ADRs before any code (original plan): rejected because it produces low-information contracts likely to need superseding revisions, delays the implementation that would validate them, and freezes formats that protect no released data.
- Write no format ADRs until problems appear: rejected because persistent contracts still require decision records before implementation freezes them, per repository ADR policy.
- Fold the spike into M1: rejected because spike results gate M0-blocking decisions (pool capacities, envelope limits) and a cheaper instrument (prototype AEX plus JSX probes) already exists.

## Consequences

### Benefits

- M1 implementation starts after five focused ADRs instead of twelve speculative ones.
- Later format ADRs are written with implementation evidence behind them.
- The riskiest architectural assumption gets the earliest and cheapest test.
- Undo/dirty semantics, diagnostic codes, and GPU backend policy now have owning ADRs.

### Costs and risks

- Milestones M3, M4, and M6 begin with an ADR-writing step; entry gates must be enforced by the roadmap and audits.
- Interim session-local encodings could leak into persistence if rule 2 is violated; M3 review must verify no pre-ADR bytes persist.
- The spike requires a Windows AE host; if none is available, M0 exit is recorded as `BLOCKED` with the named condition rather than silently skipped.

## Revisit conditions

Evidence that staged acceptance let an unversioned or accidental format become persistent, or that milestone entry ADRs are being skipped in practice, justifies a superseding ADR restoring earlier acceptance points.

## Verification obligations

- Each milestone audit records its entry ADRs as Accepted before gated contracts are implemented.
- Transport spike result records (TR-M0-002..TR-M0-007) exist before M0 exit, or M0 exit is `BLOCKED` with a named condition.
- M3 review verifies that no persistent bytes predate their Accepted contract.
- The governance check validates ADR-0009's presence, status, and index entry.
