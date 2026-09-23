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

## Current - IOS27Siri withdrawn for refinement

The separately requested automatic host-coverage feature and Liquid Glass
example have completed Windows AE 26.5x89 acceptance. The development candidate
contains paired native components, shader, FFX and application script. See
[TR-COVERAGE-COMPLETE-007](TEST_MATRIX.md#tr-coverage-complete-007--cancellation-and-usable-liquid-glass).
Feature-branch review is next; main integration, public release and other-host
acceptance are separate. This does not republish the withdrawn Siri sample.

Repository and maintained release distribution have been withdrawn and checked.
Local authoring files and plugin binaries remain preserved. Re-publication is
pending the user's request after refinement.
[ADR-0048](adr/0048-ios27siri-withdrawal.md).

## Historical completion - 0.1.1 macOS ARM backfill

The user separately authorized adding an Apple Silicon archive built from
the unchanged v0.1.1 tag and published dependency lock. Completed: exact-source
build and CPU/Metal/static checks → preserve the signed bundle while packaging
→ upload to the existing release → fresh-download verification and disclosure
updates. The download passed 315 assertions, including 297 internal-file
hashes and signature/source/lock identities. Windows/Sample archives and the
tag remain unchanged.
[ADR-0049](adr/0047-011-macos-backfill.md) records this narrow scope. New-byte
AE execution and Mac Sample acceptance remain NOT_RUN; native host testing
and Source Undo repair are separate work. The former local Siri follow-up is
stopped and its working files remain preserved.

## Released - 0.1.1 Windows patch and current Sample

User-authorized 2026-09-09 publication is complete: runtime repairs, portable
IOS27Siri Sample and downloaded hashes verified. Source Undo and host coverage
limitations remain disclosed. [ADR-0046](adr/0046-011-windows-patch-release.md).
Further visual refinement and Undo redesign remain separate work.

## Historical post-M7 follow-up — 0.1.0 then Siri

User-authorized on 2026-09-08; [ADR-0044](adr/0044-wgsl-and-010-release.md).
Order: production WGSL integration → current-artifact AE 2026 macOS tests →
0.1.0 publication and asset re-download verification → local iOS 27 Siri
shader and real AE visual acceptance. Windows 0.1.0 build/host verification
and artifact upload may follow later; they do not block the Mac release.
The shader follow-up must deliver the observed bubble/refraction/light-band
look, source, an AE project and inspected native renders, not only research.

Production WGSL and the frozen macOS candidate have completed AE 2026 host
acceptance, including independent Full/Half/Quarter renders. Publication is
the next boundary. Source-expression single Undo remains a disclosed failure
explicitly accepted for 0.1.0 under [ADR-0045](adr/0045-010-source-undo-release-boundary.md);
its state-publication repair is separate follow-up work.

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

8. **0.0.6 batch — canvas expansion + per-pass parameter groups (ACTIVE, approved 2026-08-26).** Two user-requested features entered together after the TR-GRP-001 spike measured that AE re-matches saved parameter streams by param id (`ID_MATCH`), not declaration index. (a) [ADR-0039](adr/0039-canvas-expansion.md) — canvas expansion: a shader-declared expansion `@param` is the boundary when present; undeclared sources use the layer frame ∪ upstream extent, making Grow Bounds work out of the box; closes [issue #8](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/8) / [TR-BOUNDS-001](TEST_MATRIX.md#tr-bounds-001--shader-canvas-is-the-layer-frame-field-observation). (b) [ADR-0040](adr/0040-parameter-groups-and-id-identity.md) — per-pass groups over partitioned banks with `Main` on top; param id becomes the persistent stream identity. Implementation order canvas → groups; one combined host pass (AE 2025 + 2026) exits the batch, then package and release 0.0.6. Custom-UI controls (the removed gradient editor) remain queued behind this batch. Exit criteria: the ADR-0039 and ADR-0040 verification obligations, plus M1–M7 batteries green on the batch artifact on both years. **Shipped 2026-08-26 as `v0.0.6`** ([TR-REL-006](TEST_MATRIX.md#tr-rel-006--006-release-verification)); issue #8 closed.

9. **Gradient-editor batch — custom-UI presentation (ACTIVE, [ADR-0042](adr/0042-gradient-editor-presentation-contract.md) Accepted 2026-08-28).** The custom-controls track left the queue after [TR-CUI-001](TEST_MATRIX.md#tr-cui-001--custom-ui-crash-bisection-spike) Rounds 2–4 pinned the 2026-08-15 crash to one missing `PF_REGISTER_UI` call (declared `PF_PUI_CONTROL` without it is a WER-invisible host kill; the registered arm is measured healthy). ADR-0042 governs the return: pure presentation over the ADR-0033 value rows, per-gradient inert canvas parameters, one build-time custom-UI atom with a boundary unit test, `current_frame`-only geometry, edits only through ordinary parameter commits — staged: read-only preview on measured mechanisms first; interactive editing and any `PF_AppColorPickerDialog` use each gated on their own probe leg. Exit: the ADR-0042 verification obligations (TR-0042-001 on **AE 2025 and AE 2026** — every custom-UI datum to date is 2025-only), batteries green on the editor artifact. Parallel, non-blocking (user direction 2026-08-28): the upstream `after-effects` report — the `CONTROL`-without-`register_ui` guardrail gap and the pipl 0.1.1 code-page corruption. **Outcome (2026-08-29): the verification obligations were met in full — TR-0042-001 `PASS` on both years (interactive legs, reopen, M2 12/12 + M3 4/4 + aerender per year, bidirectional flavor interchange, and a real-project A/B pixel-identity check) — and the editor was then SHELVED by user decision: no release, the `editor` feature defaults off (the shipping surface stays 0.0.6), the capability and its evidence stay in-repo (implementation method: [docs/gradient-editor.md](gradient-editor.md)). The batch is closed without a version.**

**Host matrix:** AE 2024 provisioning is deliberately deferred; releases stay pre-releases under [ADR-0027](adr/0027-0.0.1-prerelease-scope.md). [ADR-0014](adr/0014-windows-host-protocol.md) §7's four-year matrix is **not** superseded and remains the 1.0 gate.

**Next exact action:** See [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md). It is intentionally not duplicated here.

## 2026-09-08 user-directed batch

After the completed Windows 0.0.6/editor work, activate Apple Silicon macOS
AE 2026 verification, image-quality correction and authoring fixtures, and
WGSL feasibility research in parallel. [ADR-0043](adr/0043-apple-silicon-host-protocol.md)
implements the already-planned macOS phase. Exit requires recorded ARM bundle
and native host evidence, numeric quality comparisons and a running WGSL
IR/GPU proof with a scoped production-adaptation assessment. The Windows
four-year release gate and shelved gradient editor remain as recorded.

This batch completed its recorded AE 2026 subset in [TR-MAC-001](TEST_MATRIX.md#tr-mac-001--native-apple-silicon-ae-2026), image-quality fixtures in TR-QUALITY-001, and WGSL research in TR-WGSL-001. The user then requested an independent, reference-backed iOS 27 Siri motion/shader study; it does not imply production WGSL integration or a new release.

The follow-up iOS 27 Siri study was completed as research: observed official preview states, proposed material decomposition and a one-pass starting design. Its files are withdrawn under [ADR-0048](adr/0048-ios27siri-withdrawal.md); further refinement remains separate work.


## 2026-09-10 user-directed host-shape preparation

[Issue #9](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/9) enters post-M7
preparation at the user's request. Retain ordinary background `input` and
prepare automatic host coverage for portable adjustment-layer FFX.
The user's latest 2026-09-10 direction withdraws the previously requested
additional contour resource. Only the original host layer's visible
coverage/alpha is required; existing `hint:path` remains unchanged.
[ADR-0049](adr/0049-automatic-host-shape-input.md) keeps transport/ABI Proposed.

Order: prove frame-exact host acquisition and invalidation, decide the
resource ABI, integrate, then execute HS-01..HS-12 and HS-16 from the
[fixture matrix](../spike/host-outline/cases.json). Exiting preparation means
the issue, source/SDK findings and reproducible acceptance specification are
recorded. Exiting implementation requires native evidence for the required
shapes, animation, transforms, canvas, portability and compatibility; unsupported
modifiers are explicit. HS-16 checks coverage opt-in and alpha fidelity;
the former contour-specific HS-13..HS-15 definitions are retained as withdrawn
history in the fixture file. The current project-open repair remains preserved.
This batch adds no release, installation or production-project scope.

Subsequent user authorization covers independent diagnostic installation and
background AE lifecycle for feasibility testing. The tested upstream/layer/
downstream receipts do not isolate adjustment coverage, and auxiliary Coverage
is absent in the current fixtures. Raw animated contours are readable only in
the tested main-thread context. The acquisition/transport gate therefore remains
open; production integration must not advance from these observations alone.
[Native evidence](audits/evidence/host-shape-native-20260910/coverage-v003-v004/README.md).

The follow-up original-source experiment finds pre-effect expression alpha and
a fixture-managed mode-None mask carrier for sampled raster blocks. Direct PF
checkout still supplies the adjustment background. The carrier passes a small
opaque/animated subset, but fails the partial-opacity budget and excludes
original masks. The user now accepts an automatically managed mode-None entry
in the mask list. The isolated 0.0.9 probe now validates basic ownership, duplication, cleanup,
Undo/Redo and saved/cache-purged data retrieval. FFX recreation, original masks,
partial-alpha cost and production integration remain unresolved; the acquisition
and transport gate is still open.
[Current evidence](audits/evidence/host-shape-native-20260910/original-source-v005/README.md).

## 2026-09-20 coverage regression gate

Acceptance remains FAIL on AE 26.5x89: original masks/holes, 8-bpc curved
edges, full-frame partial opacity and managed cleanup. Runtime FFX, canvas/ROI,
temporal use and true MFR concurrency remain blocked by implementation gaps.
The user has suspended further pushes until tests pass.
[Current regression](audits/evidence/host-coverage-regression-20260920/README.md).

The 0.0.10 engineering correction clears the observed idle cleanup/Undo failure
and fixed-coordinate small-frame diagnostic error. Acquisition failures remain.
The user explicitly requires the original full scope before delivery; do not
use a reduced first-release scope as an exit from this gate.
[Correction evidence](audits/evidence/host-coverage-engineering-20260920/README.md).

The user's clarified priority is image fidelity first, then preview speed within
the proven fidelity contract. Benchmark and optimize against the same native
reference cases; do not trade away edges, partial alpha, masks or animation to
meet a preview target. The full original delivery gate remains required.

## 2026-09-20 fidelity baseline and acquisition boundary

Exact per-pixel sampling and float32 word transport are verified for recorded
cases, including the full 800x600 half-opacity fixture. Cache warm-up is adopted
only in the standalone exact diagnostic after unchanged-pixel checks. Full-frame
cost remains 91.529 s, and native mask rasterization omits expansion. A decision
on automatically managed reference objects is pending; do not assume permission
to alter the original no-helper-object workflow. Full delivery gate remains FAIL.
[Fidelity audit](audits/evidence/host-coverage-fidelity-20260920/README.md).

## 2026-09-20 native reader path approved

The user has accepted an automatically maintained internal reading layer after
the native return bridge passed at 8/16/32 bpc. This resolves the pending
project-structure choice above. Continue in order: ownership/lifecycle and
requested-time validation -> resource/coordinate ABI -> runtime integration ->
full FFX and original host/render acceptance. No reduced delivery scope or push.
[Direct-source audit](audits/evidence/host-coverage-direct-source-20260920/README.md).

The subsequent user decision sets **AE 26.5+ for this new feature only**.
AE 2025.6.6 returned `kSPSuiteNotFoundError` for StreamSuite7 and rejected the
newer-host preset. Do not pursue an undocumented compatibility workaround or
change existing-feature support. Native automatic-reader lifecycle and frame
tests now precede production shader integration; they are not release acceptance.

The guarded reader now passes automatic binding/sharing, unused-source cleanup,
Undo/Redo, duplicate-owner/rename checks and the recorded native pixel matrix.
Owner-layer deletion and two-step Undo pass on diagnostic 0.0.28. The active
gate is production resource/coordinate/readiness integration, followed by FFX,
ROI/downsample and actual MFR acceptance.
[Reader audit](audits/evidence/host-coverage-reader-20260920/README.md).

## 2026-09-23 production integration

The user has requested formal integration. ADR-0050 fixes `hint:coverage`, a
dedicated hidden slot, native-alpha canvas encoding and E59/E60 readiness gates.
The resource/reflection/persistence/encoding layer is now in production source.
Next connect native ownership/helper rendering and first-frame readiness, then
run production host, FFX and MFR/ROI acceptance. No deployment or push yet.
[Integration audit](audits/11-host-coverage-integration.md).

The production SDK stage adapter is compiled and callback-fault-tested. Native
ownership and the helper frame path remain the active integration work; no
first production AE frame or lifecycle/render acceptance is inferred from
these local tests. The full remaining gate is listed in the integration audit.

The subsequent ADR-0051 integration passes its first production AE 26.5 frames:
6 native-word comparisons at 8/16/32 bpc, 8 initial lifecycle cases and one
same-process reopen frame. Native ownership/helper hookup is complete for this
fixture. Next is copy/FFX-before-idle authorization, then the full production
coordinate, animation, ownership, aerender/MFR and glass-material acceptance.
[Production first-frame evidence](audits/evidence/coverage-owner-20260923/README.md).

Copy/FFX-before-idle authorization now passes on final `96dcff77...`, including
warm-cache copy, automatic recovery, new/existing preset targets, prepared
independent rendering and missing-reader refusal. The 6 native-depth first-frame
comparisons still pass. Continue the full production animation/coordinate and
ownership/MFR matrix before glass-material delivery or any push.
[Readiness evidence](audits/evidence/coverage-readiness-20260923/README.md).

The user's explicit delivery order is complete regression, then liquid-glass
shader authoring and native visual/preset acceptance. The material must follow
the original layer automatically; no manual geometry or layer binding is added.

Production fidelity now passes 114 native comparisons and 15 ROI comparisons,
including the repeated 78-case base matrix, parent/negative-scale/expression
cases and repaired outside-comp sampling. Helper tamper, actual concurrent MFR
and remaining release gates precede liquid-glass shader authoring.
[Fidelity evidence](audits/evidence/coverage-full-20260923/README.md).
