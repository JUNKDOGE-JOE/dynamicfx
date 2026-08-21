# DynamicFX implementation roadmap

> This file is the only authority for milestone order and exit criteria.  
> Current reality and the exact next action live in [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md). Detailed design lives in [ARCHITECTURE.md](ARCHITECTURE.md).

## Milestone rules

- A milestone exits only when every exit criterion has evidence in [TEST_MATRIX.md](TEST_MATRIX.md) and its audit.
- A visible AE result is required from M1 onward.
- A later milestone may not redefine an Accepted core decision silently.
- Failed criteria remain visible; they are not replaced by prose stating that a milestone is complete.
- Every milestone has one canonical audit document.
- A milestone that first implements or persists a staged contract may not begin until its entry ADRs are Accepted ([ADR-0009](adr/0009-staged-format-adr-acceptance.md)).

## M0 — Architecture Contract

**State:** COMPLETE — exited 2026-08-12 (ADRs 0010-0014 Accepted; transport spike TR-M0-002..007 PASS on AE 2025; AE 2026 re-verify tracked as non-blocking follow-up)

**Goal:** Convert approved product choices into implementable, versioned contracts before changing persistent AE topology or project data.

**Entry:** P1-P11 approved and target architecture documented.

**Scope:**

- repository handoff and evidence rules;
- Accepted product ADRs;
- staged format-ADR acceptance plan ([ADR-0009](adr/0009-staged-format-adr-acceptance.md));
- M0-blocking format ADRs 0010-0014: Language IDs; Shader ABI v1 core; envelope version marker; ParamId grammar and initial pool capacities; Windows AE 2023-2026 build/install/test protocol including wgpu backend policy and harness requirements;
- transport feasibility spike (TR-M0-002..TR-M0-007): expression capacity, long-expression save/reopen, arbitrary-data size, undo/project-dirty behavior, Popup menu mutation, aerender parity;
- remaining format contracts staged to M3/M4/M6 entry.

**Exit criteria:**

- the M0-blocking format ADRs 0010-0014 are Accepted;
- the transport spike has complete result records on at least one target AE year in [TEST_MATRIX.md](TEST_MATRIX.md), or M0 exit is recorded as `BLOCKED` naming the missing host;
- no unresolved contradiction exists between architecture and ADRs;
- the new parameter topology and other M1-visible surfaces are frozen for implementation; contracts staged to M3/M4/M6 remain explicitly session-local until their entry ADRs are Accepted;
- target tests remain `NOT_RUN`; no implementation success is implied;
- M0 audit identifies the exact first code change for M1.

**Visible result:** No new AE pixels required; the result is a complete, reviewable implementation contract.

**Audit:** [audits/00-architecture-contract.md](audits/00-architecture-contract.md)

**Next exact action:** See [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md). It is intentionally not duplicated here.

## M1 — New-architecture First Frame

**State:** COMPLETE — exited 2026-08-12 (TR-M1-001..004 all PASS on AE 2025, artifact `BDDB51F1…`; aerender measured fail-closed pass-through pending M3 persistence; 2026 `NOT_RUN`, 2023/2024 `BLOCKED`)

**Goal:** Render the first visible frame through the new Language/EffectDefinition/RenderGraph path.

**Entry:** M0 exit criteria satisfied.

**Scope:**

- new unreleased AE parameter topology;
- Language Popup defaulting to GLSL;
- `LanguageFrontend` registry with GLSL frontend;
- raw GLSL lowered to a one-pass RenderGraph;
- minimal Shader ABI v1;
- 8-bpc input/output;
- Status and structured compile diagnostics;
- single-call `addProperty("DynamicFx")`;
- no legacy transport or migration code;
- automated Windows host harness (JSX + aerender + numeric image comparison) producing raw evidence artifacts as a first-class deliverable;
- `PixelFormatAdapter` boundary present from the first frame; 8-bpc is the first implementation, not a structural assumption.

**Exit criteria:**

- new `DynamicFx` can be added exactly once without property-tree errors;
- committed GLSL expression compiles through the frontend and graph path;
- a visible, non-pass-through 8-bpc frame is produced;
- invalid source produces stable diagnostic and input pass-through;
- Rust tests and at least one Windows AE target have complete evidence;
- the automated harness runs the M1 host scenarios and stores the raw evidence referenced by the audit;
- other AE years remain explicitly `NOT_RUN` until tested.

**Visible result:** A screenshot/output frame rendered by the new one-pass graph.

**Audit:** `docs/audits/01-first-frame.md`

## M2 — Keyframed Parameters

**State:** COMPLETE — exited 2026-08-12 (TR-M2-001/002/003 all PASS on AE 2025; all six exit criteria evidenced; color/point annotation defaults recorded as a v1 scalar-only boundary)

**Goal:** Drive shader parameters from normal keyframed AE streams through stable IDs.

**Entry:** M1 completed.

**Scope:**

- ParamDefinition and Stable Param IDs;
- initial fixed pools;
- atomic BindingPlan;
- defaults versus committed streams;
- float/int/bool/color/point and selected Phase-1 parameter kinds;
- one normalized per-frame parameter read shared across passes.

**Exit criteria:**

- keyframed values produce different verified pixels at different times;
- defaults render correctly before streams are committed;
- label/order changes preserve compatible values by ParamId;
- rename/type-change behavior matches ADR rules;
- pool overflow rejects the complete definition atomically;
- save/reopen is not claimed yet unless M3 evidence exists.

**Visible result:** An animated parameter-driven shader with captured frames at defined times.

**Audit:** `docs/audits/02-keyframed-params.md`

## M3 — Persistence and Render Clone

**State:** COMPLETE — exited 2026-08-12 (TR-M3-001 PASS on AE 2025: reopen/aerender render the shader without Compile, corruption fails closed and recovers, duplicates isolate, torn tokens lose to the snapshot, undo/dirty semantics measured; one host fact recorded — AEGP token writes occupy one undo entry each)

**Goal:** Restore Language, source, graph, bindings, and render identity without render-side AEGP.

**Entry:** M2 completed; M3-entry ADRs Accepted per [ADR-0009](adr/0009-staged-format-adr-acceptance.md): StateToken layout including undo/redo and project-dirty semantics plus the stable diagnostic code registry; sequence schema v1 codec/limits/checksum; hash algorithm, canonical serialization, and domain separation.

**Scope:**

- StateToken;
- sequence schema v1;
- exact source/definition snapshot;
- UI/render project clone resolution;
- registry hit/miss rebuild;
- save/reopen, duplicate instance, corruption, and aerender.

**Exit criteria:**

- saved project reopens and renders without clicking Compile;
- render clone performs no AEGP calls;
- registry miss reconstructs the same identities and output;
- corrupt/unsupported payload fails closed with diagnostic;
- duplicate instances do not share mutable parameter or history state;
- undo/redo and project-dirty behavior of state publication matches the StateToken ADR;
- Windows AE host evidence is recorded separately by year.

**Visible result:** Close/reopen and aerender reproduce the expected frame and animation.

**Audit:** `docs/audits/03-persistence-render-clone.md`

## M4 — Multi-pass Graph

**State:** COMPLETE — exited 2026-08-13 (TR-M4-001 PASS on AE 2025: two- and three-pass chains pixel-exact, raw/envelope identity, line-numbered E6 fail-closed, no-alias A/B identical, plan shape + transient memory in evidence; blur example implemented as an exact invert chain, per-pass timing deferred to M7)

**Goal:** Execute a real multi-pass DAG using the same runtime as one-pass effects.

**Entry:** M3 completed; RenderGraph domain model already exists from M1; M4-entry ADRs Accepted per [ADR-0009](adr/0009-staged-format-adr-acceptance.md): full multi-pass envelope grammar and escaping, intermediate format policy, ExecutionPlan resource aliasing.

**Scope:**

- versioned multi-pass source envelope;
- graph parser and canonicalization;
- graph validation and topological scheduling;
- transient intermediate textures;
- per-pass Shader ABI, artifacts, pipelines, and diagnostics;
- effect-wide parameters shared across passes;
- graph and execution-plan cache identities.

**Exit criteria:**

- a two-pass separable blur produces verified output distinct from one pass;
- cycles, missing inputs, multiple writers, format mismatch, and read-before-write fail deterministically;
- graph analysis does not run every frame;
- changing UI-only metadata does not rebuild a pass pipeline;
- transient resource lifetime is inspectable and bounded.

**Visible result:** Two-pass horizontal/vertical blur with graph and per-pass timing evidence.

**Audit:** `docs/audits/04-multipass-graph.md`

## M5 — 16/32-bpc Image Quality

**State:** COMPLETE — exited 2026-08-13 (TR-M5-001 PASS on AE 2025: 16-bpc bit-exact through multi-pass chains, 32-bpc ±HDR survival, per-depth clamp, straight-alpha measured, color pair recorded; [ADR-0022](adr/0022-16bpc-working-format-f32.md) Accepted — 16-bpc rides Rgba32Float on live wgpu evidence). Scope note recorded before implementation: minimal SmartFX entry (PreRender/SmartRender) moved into M5 because AE only delivers float worlds to smart effects (`FLOAT_COLOR_AWARE` requires `SUPPORTS_SMART_RENDER`); performance-side SmartRender work (ROI scheduling, caching, MFR) remains M7.

**Goal:** Preserve professional AE precision and alpha/color behavior across every graph pass.

**Entry:** M4 completed.

**Scope:**

- 8/16/32-bpc PixelFormatAdapters;
- intermediate format propagation;
- alpha/premultiplication policy;
- negative and over-white float values;
- explicit conversion passes where required;
- pixel fixtures and numeric tolerances.

**Exit criteria:**

- 16 bpc never silently passes through an 8-bit working format;
- 32 bpc preserves required negative/over-white values within documented tolerance;
- alpha edges pass fixtures without hidden premultiplication errors;
- multi-pass intermediates do not silently reduce precision;
- unsupported format/capability is explicit diagnostic plus pass-through.

**Visible result:** Side-by-side 8/16/32-bpc gradient and alpha fixture outputs.

**Audit:** `docs/audits/05-pixel-formats.md`

## M6 — Temporal Feedback

**State:** COMPLETE — exited 2026-08-13 (TR-M6-001 PASS on AE 2025: the ADR-0025 windowed re-simulation law `value(F) = min(F+1, W) × step` exact under shuffled interactive reads, the measured-out-of-order MFR render queue 25/25, and a fresh aerender process 25/25; `@window` rides the source; `SUPPORTS_THREADED_RENDERING` declared with no warning icon. ADR-0023's session-chain state model was refuted by run-1 measurements and superseded by [ADR-0025](adr/0025-windowed-resimulation.md); the M4-latent envelope-snapshot defect was found by the aerender leg and fixed)

**Goal:** Support explicit history resources with deterministic reset and sequencing rules.

**Entry:** M5 completed; M6-entry ADRs Accepted per [ADR-0009](adr/0009-staged-format-adr-acceptance.md): temporal seek/reset semantics and history format policy.

**Scope:**

- HistoryResource read/write;
- per-instance history pools;
- continuity detection;
- reset on seek/reverse/purge/resize/source/device changes;
- Stateless versus Temporal execution classes;
- initial serial temporal rendering policy.

**Exit criteria:**

- a feedback/trail graph produces visible history during continuous playback;
- every documented invalidation event resets history predictably;
- copied instances do not share history;
- memory remains bounded;
- random-access and MFR limitations are explicit and tested, not implied away.

**Visible result:** Feedback/trail render plus seek/reset demonstration.

**Audit:** `docs/audits/06-temporal-feedback.md`

## M7 — Performance, SmartRender, and MFR

**State:** COMPLETE — exited 2026-08-14 (TR-M7-001…006 PASS on AE 2025). Baseline → optimizations with before/after pairs and green M1-M6 batteries throughout: GPU resource reuse (−36…−53% p50 everywhere, temporal @16 −81%, 4K float halved), log policy (zero always-on per-render appends), ROI final-pass delivery (uv-preserving scissor, identical pixels, ~3.5× on small downstream requests), MFR stance confirmed against measured concurrency (intra-instance 1.0×, cross-instance 2.7×, host-bound wall), per-instance cache budget enforced with transient fallback, matrix covers 720p/1080p/4K × 8/32-bpc × 1/6 passes × 1/4 instances × temporal. Two inherited targets closed by measurement, not code: aerender per-frame re-resolution no longer reproduces (M6 snapshot fix); preview invalidation already correct — WYSIWYG verified adversarially (TR-M7-003)

**Goal:** Optimize only after correctness contracts are protected by tests.

**Entry:** M6 completed.

**Scope:**

- bounded pipeline/artifact/resource caches;
- transient aliasing and submit reduction;
- SmartRender/ROI;
- Stateless graph MFR eligibility;
- Temporal graph restrictions/checkpoints if implemented;
- GPU timing, CPU timing, allocation and memory metrics;
- later Apple Silicon feasibility after Windows stability.

**Exit criteria:**

- benchmark matrix covers 1080p/4K, one/many instances, one/many passes;
- memory and cache budgets are enforced;
- ROI produces equivalent pixels in covered regions;
- MFR is enabled only for graph classes proven thread-safe;
- performance claims include baseline, hardware, host, commit, and raw report.

**Visible result:** Before/after performance report with identical-image verification.

**Audit:** `docs/audits/07-performance-mfr.md`

## After M7 — release-driven sequencing

**State:** ACTIVE from 2026-08-15. The M0-M7 ladder is exhausted; no further milestone is defined. Work is sequenced by release batch instead of by milestone.

**Rule change:** a post-M7 batch has no milestone audit of its own. Evidence goes to [TEST_MATRIX.md](TEST_MATRIX.md) under a `TR-REL-NNN` row, as 0.0.1 and 0.0.2 already did. Everything else in `CLAUDE.md` is unchanged — in particular, feature work that touches a durable contract still requires an Accepted ADR before implementation.

**Ordering** (agreed with the user 2026-08-15; this list is the sequence authority, contents and current position live in [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md)):

1. **0.0.3 batch** — not-ready render marker, public `examples/`, the already-committed ADR-0028 precision line.
2. **Layer-input parameters** `hint:layer` ([public issue #1](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/1)) — requires ADR-0030 before implementation.
3. **Gradient control** `hint:gradient` ([public issue #2](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/2)) — requires its own ADR before implementation.
4. **Point 3D** `hint:point3d` — [ADR-0034](adr/0034-point3d-parameters.md). Added to this batch 2026-08-15.
5. **Paths** `hint:path` — [ADR-0035](adr/0035-path-parameters.md). Added to this batch 2026-08-15.

Items 2-5 were pulled forward into the 0.0.3 batch at the user's direction: write the controls first, then spend one host pass on all of them, rather than one host cycle per feature. The [ADR-0031](adr/0031-gradient-parameters.md) §7 custom-UI editor was **dropped** from item 3 on 2026-08-16 after it crashed the host in every configuration tried; ADR-0033 §6 made that a presentation loss rather than a feature loss, which is why it did not reopen the decision.

6. **0.0.4 — pool valid-range fix — SHIPPED 2026-08-19** ([public issue #5](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/5), [ADR-0037](adr/0037-pool-valid-range-and-slider-range.md)). Added 2026-08-19 at the user's direction ("file the issue, answer the old ones, then fix"); a released-contract defect, so it preceded the two unscheduled candidates (byte-reproducible build; growth-pool labels). [TR-0037-001](TEST_MATRIX.md#tr-0037-001--pool-valid-range-float1-negative-int10) `PASS` on AE 2025 and 2026 with the m2/m3 batteries green; released as a pre-release per [TR-REL-004](TEST_MATRIX.md#tr-rel-004--004-release-verification).

7. **Two open field defects on 0.0.4 — UNSCHEDULED, recorded 2026-08-19/21.** Both are released-contract correctness defects, so they precede the byte-reproducible-build and growth-pool-label candidates. (a) [TR-BIND-002](TEST_MATRIX.md#tr-bind-002--copied-instance-corrupts-slot-mapping-field-defect) / [issue #6](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/6) — copy/paste of an instance permutes the other instance's parameter roles through the shared process-registry entry; flicker confirmed again on AE 2026 2026-08-21. **Fix: [ADR-0038](adr/0038-registry-key-per-binding-plan.md) Accepted 2026-08-21 (mechanism B — registry keyed by `(source, plan identity)`, plan identity transported in a hidden `PlanToken` stream, idle observer uses the instance's own artifact) and implemented; the harness `scripts/bind/tr_bind_002.py` records `FAIL` on the pre-fix build and `PASS` on the fix build `ff1197d9…` on AE 2026 and AE 2025 (2026-08-21), and the M2/M3 batteries are `PASS` on both years on the same artifact (TR-0038-001). **Released as 0.0.5 on 2026-08-21 (TR-REL-005); issue #6 closed.** (b) [TR-CACHE-001](TEST_MATRIX.md#tr-cache-001--interrupted-render-poisons-the-frame-cache-field-defect) / [issue #7](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/7) — an interrupted preview poisons the cache (a frame is committed missing one DynamicFx layer) — **FIXED at `cfccd5d`, host-verified `PASS` on AE 2026 2026-08-21** (local `SmartRender` correction: propagate `InterruptCancel` instead of filling transparent black; no ADR needed). **Released in 0.0.5 (TR-REL-005); issue #7 closed.** Exact next action and sequencing live in [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md).

**Host matrix:** AE 2024 provisioning is deliberately deferred; releases stay pre-releases under [ADR-0027](adr/0027-0.0.1-prerelease-scope.md). [ADR-0014](adr/0014-windows-host-protocol.md) §7's four-year matrix is **not** superseded and remains the 1.0 gate.

**Next exact action:** See [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md). It is intentionally not duplicated here.
