# DynamicFX repository instructions

This file is the mandatory entry point for every new development session. The repository, not chat history or model memory, is the project record.

## Required reading order

Before proposing or changing implementation:

1. Read this file completely.
2. Read [docs/IMPLEMENTATION_STATUS.md](docs/IMPLEMENTATION_STATUS.md) for the only authoritative current state and exact next action.
3. Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the approved target design.
4. Read the relevant Accepted ADRs listed in [docs/adr/README.md](docs/adr/README.md).
5. Read the audit for the current milestone from [docs/audits/README.md](docs/audits/README.md).
6. Read [docs/TEST_MATRIX.md](docs/TEST_MATRIX.md); never infer test status from prose elsewhere.
7. Read [docs/ROADMAP.md](docs/ROADMAP.md) for milestone scope and exit criteria.
8. Inspect `git status`, recent commits, and the diff before acting.

If these sources disagree, stop implementation and reconcile them in this order:

1. Accepted ADRs for decisions;
2. `ARCHITECTURE.md` for target design;
3. `IMPLEMENTATION_STATUS.md` for current reality;
4. `TEST_MATRIX.md` for verification truth;
5. `ROADMAP.md` for planned sequence.

`docs/CONCEPT.md` and `docs/SHADER.md` are prototype snapshots, not target contracts.

## Approved non-negotiable decisions

Do not reopen these decisions during ordinary implementation. A change requires a new ADR that explicitly supersedes the relevant Accepted ADR.

- DynamicFX is an open shader runtime controlled through ordinary After Effects properties.
- The brand is `DynamicFX`; the AE effect and match name remain `DynamicFx`.
- The unreleased prototype is rewritten in place. Do not create `DynamicFx2` and do not preserve prototype parameter indexes or wire formats.
- The selected `Language` popup plus committed `Source.expression` are the source authority.
- `Language` is non-time-varying, defaults to GLSL, and selects an extensible `LanguageFrontend` by stable numeric ID.
- Multi-pass `RenderGraph` is core in Phase 1. Single-pass is a one-pass graph, not a separate runtime.
- Shader parameters use a fixed AE pool, stable `ParamId`s, atomic `BindingPlan` publication, and normal keyframed AE streams.
- Render-side code never calls AEGP. UI/render clones use a newly designed `StateToken` plus sequence schema v1.
- Do not retain prototype `SourceChannel`, flattened v1-v3 migration, legacy `SourceData`, or the sidecar.
- Compile transaction/generation is session-local and must never be persisted.
- `PipelineKey` is based on per-pass artifact and pipeline/device state; it is not derived directly from `DefinitionHash`.
- Multi-pass identities remain separate: module, artifact, graph, definition, pipeline, execution plan, and frame resource.
- Image correctness comes before performance: 8/16/32-bpc and alpha/color behavior precede SmartRender and MFR.
- Target hosts are Windows After Effects 2023, 2024, 2025, and 2026. Apple Silicon macOS follows only after Windows is stable.
- Core rendering does not depend on an editor, WebSocket, cloud service, account, store, licensing layer, or telemetry.
- A local effect package may be added later; an editor is optional and deferred.
- This public repository is the single project record — code and governance corpus together (ADR-0036). There is no private half; the archived `dynamicfx-dev` is history, never a write target.

Full consequences and rationale are in [docs/adr/README.md](docs/adr/README.md).

## Development sequence

Develop by vertical, visible milestones rather than broad refactors with no AE output:

- M0 Architecture Contract
- M1 New-architecture First Frame
- M2 Keyframed Parameters
- M3 Persistence and Render Clone
- M4 Multi-pass Graph
- M5 16/32-bpc Image Quality
- M6 Temporal Feedback
- M7 Performance, SmartRender, and MFR

The exact current milestone and next action live only in [docs/IMPLEMENTATION_STATUS.md](docs/IMPLEMENTATION_STATUS.md). Exit criteria live only in [docs/ROADMAP.md](docs/ROADMAP.md).

Do not silently pull work from a later milestone into the current one unless it is required to preserve an approved core boundary. If that changes scope or order, update the roadmap and record the reason before implementation.

## Required session workflow

### At session start

1. Follow the required reading order.
2. Confirm the working tree and current branch.
3. Restate the current milestone and exact next action from `IMPLEMENTATION_STATUS.md`.
4. Verify that the planned change fits the current milestone and Accepted ADRs.
5. Mark the active work in `IMPLEMENTATION_STATUS.md` before broad implementation begins.

### During implementation

- Keep host-specific AE code outside the definition and graph domain layers.
- Do not add a new persistent field, parameter index, Language ID, graph grammar rule, Shader ABI rule, hash domain, or history semantic without the appropriate ADR.
- Write tests with the contract, not after the implementation is considered finished.
- On discovering an out-of-scope defect, record it as a blocker/follow-up; do not expand the current milestone silently.
- Never convert a failure into pass-through without a stable diagnostic code and test expectation.

### Before ending a session

Update, in this order:

1. `docs/TEST_MATRIX.md` with commands actually run and raw evidence references;
2. the current milestone audit in `docs/audits/` with visible outcome and residual risks;
3. `docs/IMPLEMENTATION_STATUS.md` with current reality and one exact next action;
4. `docs/ROADMAP.md` only if milestone state, blockers, or ordering changed;
5. ADR index/status only if an architecture decision changed.

Then run document/link checks and `git diff --check`. Do not leave progress known only to the conversation.

## Evidence policy

Allowed statuses:

- `PASS`: the documented command or exact host procedure was run successfully and has the required evidence.
- `FAIL`: it was run and failed; preserve the failure evidence.
- `NOT_RUN`: no valid run exists for the current target/baseline.
- `BLOCKED`: execution is impossible until a named condition is resolved.
- `CLAIMED_UNVERIFIED`: prose or history claims success but the required evidence is unavailable.
- `PROTOTYPE_BASELINE`: a result applies only to the code that will be replaced, never to the target rewrite.

A target result may be marked `PASS` only when its `TEST_MATRIX` entry records, where applicable:

- exact commit or working-tree baseline;
- command or exact AE UI/script steps;
- operating system;
- AE year and full host version/build;
- plugin artifact identity and installation path;
- date/time;
- expected and observed results;
- original log, PNG/AEP, report, or other artifact path;
- related milestone audit.

The following are never proof of `PASS`:

- chat statements;
- an implementation claim in README or architecture prose;
- a script that only writes `PENDING_LOG`;
- a run from another commit without an explicit equivalence argument;
- a test on one AE year generalized to another;
- a prototype result applied to the rewrite.

Do not delete or hide failing evidence when replacing it with a later passing result. Keep an audit trail.

## ADR policy

Create an ADR before changing any of these:

- product boundary or source authority;
- parameter topology or persistent index;
- Language ID or frontend contract;
- source envelope or graph grammar;
- Shader ABI or builtins;
- ParamId/binding migration semantics;
- StateToken or sequence schema;
- identity/hash domains or cache keys;
- temporal history semantics;
- supported host matrix or installation boundary;
- the publication boundary below;
- milestone ordering when it changes product risk.

Accepted ADRs are immutable historical records. To change one:

1. create a new ADR;
2. explain the new evidence and consequences;
3. mark the old ADR `Superseded by ADR-NNNN`;
4. update architecture, status, roadmap, tests, and audits as needed.

Do not silently edit an Accepted decision to make history look consistent.

## Audit policy

Every visible milestone has one audit document. It must lead with the actual outcome, then include:

- visible evidence;
- exact baseline and code paths;
- contracts fixed by the milestone;
- commands and AE procedures run;
- observed results, including failures;
- known limitations and residual risks;
- one next exact action;
- reproduction steps.

An audit summarizes evidence; it does not replace raw artifacts or `TEST_MATRIX` entries.

## Publication boundary

Every commit here is public the moment it is pushed, and a push cannot be
taken back — deleted content stays in clones, caches, and forks. The record is
published in full **except** for what [ADR-0036](docs/adr/0036-single-repository-record.md)
withholds:

- the competitor analysis and any other reproduction of a third party's
  product internals — vendor names, decoded binary constants, extracted
  strings. Conclusions this project drew from such research are ours and stay
  published;
- credentials, tokens, or auth material of any kind;
- machine-local paths and host identifiers beyond what an evidence record
  genuinely needs.

Before any push that adds or edits documents, scan the tree for those terms
and record the result. If a redaction is needed in an Accepted ADR or audit,
mark it visibly and list it in ADR-0036 §4 — never edit the text silently.

## Commands and destructive actions

- Do not commit, push, install an AEX, overwrite an installed plugin, kill AE/aerender, or delete user files unless the user explicitly requests that action.
- Before installing, verify the target host year and that After Effects/aerender are closed.
- Never install into Adobe shared `Common/Plug-ins/7.0/MediaCore`.
- Use version-specific `Support Files/Plug-ins/DynamicFx` destinations.
- Do not claim a live result when only Rust tests or an AEX build ran.

## Definition of a successful handoff

A new session must be able to answer from repository files alone:

1. What is DynamicFX's core product goal?
2. What inputs are authoritative?
3. How are languages extended?
4. Why is multi-pass part of the core model?
5. How do keyframed parameters remain stable?
6. Why is prototype compatibility intentionally discarded?
7. What milestone is active?
8. Which exact tests passed, failed, or were not run?
9. What is currently broken or blocked?
10. What single action should happen next?
11. Which decisions require an ADR to change?

If any answer depends on chat history, the handoff documentation is incomplete and must be fixed before further implementation.
