# Audit 00: Architecture Contract

- Milestone: M0 — Architecture Contract
- Audit state: Complete (M0 exited 2026-08-12)
- Baseline branch: `codex/stabilize-programmatic-flow`
- Audit date: 2026-08-08; updated 2026-08-10 (baseline commit, ADR-0009 staging); updated 2026-08-12 (transport spike results; ADR-0012/0013 drafted)
- Related roadmap: [../ROADMAP.md](../ROADMAP.md)
- Related test matrix: [../TEST_MATRIX.md](../TEST_MATRIX.md)
- Related ADRs: [ADR index](../adr/README.md)

## Outcome

**M0 is complete; exited 2026-08-12.** The product direction, target architecture, session-handoff protocol, milestone sequence, evidence policy, and fourteen Accepted ADRs are documented and committed. All five M0-blocking format ADRs are Accepted: 0010/0011/0014 on 2026-08-11; [0012](../adr/0012-source-envelope-marker-and-limits.md) (envelope `@dynamicfx` marker, fail-closed versioning, 4 MiB source cap, 8 MiB snapshot budget) and [0013](../adr/0013-paramid-grammar-and-pools.md) (ParamId grammar/aliases, 104-slot pool table, append-only growth, DefinitionData dropped) on 2026-08-12 — both drafted from the transport-spike data and Accepted with explicit user approval. The transport feasibility spike (TR-M0-002..007) ran on **AE 2025 with all six scenarios PASS**. `ARCHITECTURE.md` is synced to the accepted contracts. AE 2026 re-verification remains a tracked follow-up outside the M0 gate. Runtime rewrite work has not started — that is M1, which begins with the new AE parameter topology per ADR-0013.

### Transport spike outcome (AE 2025, 25.6.6x4)

- Expressions carry committed source byte-exact to at least 16 MB (TR-M0-002) and survive save/reopen across ASCII/punctuation/CRLF/Unicode variants (TR-M0-003).
- Sequence flatten (the schema-v1 carrier) round-trips a 16 MB checksummed payload intact (TR-M0-004); the arb-parameter *value* write path is ineffective and a 33 MB project crashed AE on overlapping open — payloads are viable but must be bounded.
- A Popup's menu is fixed at PARAMS_SETUP; runtime mutation has no host-visible effect (TR-M0-006).
- A 1 MB expression evaluates identically in GUI and aerender (alpha 96 == 96) (TR-M0-007).
- Cross-cutting: scripted parameter/expression writes do not reach the plugin as committed changes, confirming the idle-observer design (ARCHITECTURE §5.3).

These directly inform ADR-0012 (large envelope is feasible but should be size-capped) and ADR-0013 (parameter pools cannot rely on runtime-mutable Popup menus; the hidden DefinitionData arb parameter should be dropped in favor of the sequence snapshot).

## Visible evidence

| Evidence | Path | What it proves | What it does not prove |
|---|---|---|---|
| Approved target architecture | [../ARCHITECTURE.md](../ARCHITECTURE.md) | Selected product/runtime design, language and graph boundaries | Runtime feasibility or AE host success |
| Competitor research | not published — see [ADR-0036](../adr/0036-single-repository-record.md) | Static evidence and adopt/defer/reject inputs (the boundaries it produced are in ARCHITECTURE.md and the ADRs) | Any third-party runtime performance or security claim |
| Repository instructions | [../../CLAUDE.md](../../CLAUDE.md) | Required handoff and evidence workflow | That future sessions followed it |
| Current status | [../IMPLEMENTATION_STATUS.md](../IMPLEMENTATION_STATUS.md) | Runtime rewrite has not started and names one next action | Target code correctness |
| Roadmap | [../ROADMAP.md](../ROADMAP.md) | M0-M7 order and exit criteria | Any milestone completion |
| Test matrix | [../TEST_MATRIX.md](../TEST_MATRIX.md) | Target results are all NOT_RUN | AE compatibility |
| Accepted ADRs | [../adr/README.md](../adr/README.md) | Fourteen approved decisions are immutable without superseding ADR | M3/M4/M6-staged format details remain open by design |

## Baseline

- Branch at audit initialization: `codex/stabilize-programmatic-flow`
- Runtime source: pre-rewrite prototype
- Documentation: uncommitted changes at initialization; inspect current `git status`
- OS: Windows 11 development environment
- Target hosts: Windows AE 2023/2024/2025/2026
- Target AEX artifact: not built
- Target GPU/backend evidence: not collected

## Code paths

No runtime code path was changed for M0 documentation work. Current `src/` still represents the prototype and is not target contract evidence.

Documentation/governance paths introduced or revised:

- `CLAUDE.md`
- `README.md`
- `docs/ARCHITECTURE.md`
- `docs/IMPLEMENTATION_STATUS.md`
- `docs/ROADMAP.md`
- `docs/TEST_MATRIX.md`
- `docs/adr/`
- `docs/audits/`
- prototype snapshot notices in `docs/CONCEPT.md` and `docs/SHADER.md`

## Contracts fixed or changed

Accepted decisions:

1. [ADR-0001](../adr/0001-expression-authority-and-open-runtime.md): Language + Source expression authority and open runtime.
2. [ADR-0002](../adr/0002-extensible-language-frontends.md): non-time-varying Language popup, default GLSL, extensible frontends.
3. [ADR-0003](../adr/0003-render-graph-is-core.md): multi-pass RenderGraph is core; one pass is a special case.
4. [ADR-0004](../adr/0004-breaking-rewrite-and-host-matrix.md): in-place breaking rewrite; Windows AE 2023-2026.
5. [ADR-0005](../adr/0005-stable-parameter-ids.md): fixed AE pools, Stable Param IDs, atomic BindingPlan, keyframed streams.
6. [ADR-0006](../adr/0006-state-and-persistence-boundary.md): new StateToken and sequence schema v1; no session generation persistence.
7. [ADR-0007](../adr/0007-identity-and-cache-boundaries.md): module/artifact/graph/definition/pipeline/plan/frame identities remain separate.
8. [ADR-0008](../adr/0008-product-scope-and-delivery-order.md): visible vertical milestones, image correctness before optimization, editor/package later.
9. [ADR-0009](../adr/0009-staged-format-adr-acceptance.md): staged format-ADR acceptance at milestone entries; M0 transport spike; undo/dirty semantics and the diagnostic code registry assigned to the M3 StateToken ADR; wgpu backend/adapter policy assigned to the host-protocol ADR.

10. [ADR-0010](../adr/0010-stable-language-ids.md) (Accepted 2026-08-11): permanent append-only `LanguageId` registry (0 invalid, 1 GLSL default, 2 WGSL reserved); snapshot ID is restore authority over the popup stream; unknown IDs fail closed preserving source.
11. [ADR-0011](../adr/0011-shader-abi-v1-core.md) (Accepted 2026-08-11): versioned per-pass fragment ABI (v1) with fixed builtin head, reserved binding space for M4, and fixture-pinned numeric conventions.
12. [ADR-0014](../adr/0014-windows-host-protocol.md) (Accepted 2026-08-11): single-artifact identity chain, per-year install/verification protocol, DX12-only support claim, scripted-harness evidence discipline.

Still unfixed: ADR-0012/0013 are drafted as Proposed (2026-08-12) and bind nothing until Accepted; the M3/M4/M6-staged contracts (StateToken/sequence schema/hash domains; full envelope grammar, intermediate format, ExecutionPlan aliasing; temporal seek/reset and history format) remain open by design.

## Commands and exact host steps

No target runtime, AEX build, or AE host procedure was executed for M0.

The repository governance check was executed and preserved:

```text
python scripts/check_governance.py > docs/audits/00-governance-check.txt 2>&1
```

It validates local Markdown links, required handoff paths, Mermaid block structure, nine Accepted ADRs and their index, architecture/status clauses, target-host PASS discipline, `git diff --check`, and absence of runtime-source/config diffs. The check was re-run on 2026-08-10 after the ADR-0009 staging changes; the 2026-08-08 report is preserved at commit `fe3ada7`.

## Observed evidence

- `TR-M0-001`: `PASS` — [raw governance report](00-governance-check.txt) re-run 2026-08-10 records 23 Markdown files, 122 local links, 19 Mermaid blocks, 9 Accepted ADRs, 0 errors; the 2026-08-08 run is preserved at commit `fe3ada7`.
- Transport spike TR-M0-002..TR-M0-007: `NOT_RUN`.
- Prototype `cargo test --all`: 19 passed, 0 failed, recorded only as `PB-RUST-001` `PROTOTYPE_BASELINE`.
- Target rewrite Rust tests: `NOT_RUN`.
- Target release build: `NOT_RUN`.
- Windows AE 2023/2024/2025/2026: all `NOT_RUN`.
- aerender: `NOT_RUN`.

## Findings and failures

| Severity | Finding | Evidence | Impact | Disposition |
|---|---|---|---|---|
| ~~High~~ Resolved | Persistent target formats are not yet fixed | Format ADR list in `docs/adr/README.md` | Was: runtime implementation could freeze accidental wire/ABI contracts | M0-blocking set 0010-0014 all Accepted (0010/0011/0014 on 2026-08-11; 0012/0013 on 2026-08-12); M3/M4/M6 contracts stay session-local per ADR-0009 |
| ~~High~~ Resolved | Expression transport capacity and behavior are unmeasured | Transport spike TR-M0-002..007, all PASS on AE 2025 | Was: envelope limits/Popup/DefinitionData lacked host data | Measured 2026-08-12: 16 MB expression + 16 MB sequence round-trip; Popup menu fixed; DefinitionData value write ineffective. Feeds ADR-0012/0013 |
| Medium | 33 MB sequence-payload project crashed AE on overlapping open | TR-M0-004 finding | Very large persisted payloads may destabilize the host | ADR-0012 must cap envelope/payload size; do not rely on multi-MB payloads |
| High | No target host test exists | `docs/TEST_MATRIX.md` | No target compatibility claim is allowed | Begin only at M1 after M0 exit |
| Medium | Current runtime code and snapshot docs describe the prototype | `src/`, `docs/CONCEPT.md`, `docs/SHADER.md` | New sessions may confuse implementation with target | Snapshot warnings and status reading order are mandatory |
| Medium | M0 documentation changes were initially uncommitted | `git status` on 2026-08-08 | Another session could have missed them on another checkout | Resolved: committed at `fe3ada7` on 2026-08-10 per explicit user request |

## Known limitations

- No target source code exists.
- No target parameter index, Language numeric ID, graph grammar, ABI, StateToken layout, or sequence bytes are frozen.
- No Mermaid CLI render was performed; prior architecture diagrams received static structural checks only.
- No Windows AE target was run.

## Residual risks

- AE 2023 may expose SDK/host behavior that changes target topology or transport details.
- Multi-pass expression size and source-map behavior may constrain the envelope grammar.
- Fixed parameter pool capacity could be chosen too low without representative effects.
- Temporal feedback and aerender frame order may force stricter execution classifications.
- Undo/redo and project-dirty interaction of programmatic state publication is contract-assigned (M3 StateToken ADR) but unverified until the spike and M3 tests run.
- Staged acceptance (ADR-0009) relies on entry-gate discipline; skipping an entry ADR would freeze an accidental format.

## Decision changes

ADR-0009 (2026-08-10) stages format-ADR acceptance at milestone entries and adds the M0 transport spike. It refines gating within ADR-0008's delivery order and does not supersede any Accepted product decision.

ADR-0010, ADR-0011, and ADR-0014 were drafted and Accepted on 2026-08-11 with explicit user approval, fixing Language numeric IDs, the Shader ABI v1 core, and the Windows build/install/test protocol including the DX12-only backend policy.

ADR-0012 (source envelope marker and size limits) and ADR-0013 (ParamId grammar, parameter pools, growth policy; DefinitionData dropped) were drafted from the transport-spike measurements and Accepted on 2026-08-12 with explicit user approval. `ARCHITECTURE.md` §4/§4.2/§7.1/§13/§23 were synced to the accepted contracts in the same change.

## Next exact action

Begin the M1 first runtime slice with this exact first code change: replace the prototype `Params` topology with the ADR-0013 head parameters (Input, Language, Source, Compile, Status, StateToken) plus the 104-slot pool declarations, implemented behind the new domain layout with the ADR-0010 Language registry and ADR-0012 envelope detector as pure domain modules under unit test. Track progress in [audits/01-first-frame.md](01-first-frame.md) and [IMPLEMENTATION_STATUS.md](../IMPLEMENTATION_STATUS.md).

## Reproduction

A new session can reproduce the current audit state by:

1. following `CLAUDE.md` reading order;
2. running `git status` and inspecting documentation changes;
3. validating local Markdown links and ADR index entries;
4. confirming all target-rewrite Test Matrix rows remain `NOT_RUN`;
5. confirming `src/`, `build.rs`, and `Cargo.toml` contain no target rewrite changes;
6. running `git diff --check`;
7. not running or claiming any AE host result unless a complete result record is added.
