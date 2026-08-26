# ADR-0040: Per-pass parameter groups over partitioned banks; param id replaces declaration index as stream identity

- Status: Accepted
- Date: 2026-08-26 (Proposed, redrafted after the user rejected by-kind grouping — "把每个pass的参数单独一组，最上方是总参数" — and Accepted the same day with explicit user approval)
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md)
- Related implementation: `src/host/params.rs` (`ParamKey`, `declaration_order`, `setup`), `src/binding.rs` (pool tables, allocator), `src/host/idle.rs`, harness JSX property indexes
- Related tests/audits: [TR-GRP-001](../TEST_MATRIX.md#tr-grp-001--parameter-stream-matching-across-layout-change-spike), evidence [`spike-20260826-param-group-matching/`](../audits/evidence/spike-20260826-param-group-matching/README.md)

## Context

The Effect Controls panel shows all 177 declared parameters as one flat list. The user's requirement (2026-08-26) is semantic grouping: **each pass's parameters in their own group, shared/global parameters at the top**. Merging parameters by kind was explicitly rejected as meaningless.

Two measured facts shape the mechanism:

1. **AE re-attaches saved streams by param id, not declaration index** (TR-GRP-001, `ID_MATCH`, 13/13 probes). The `after-effects` crate stamps `uu.id = murmur3(format!("{ParamKey:?}"))` on every parameter; AE derives stream matchNames from it (`Float 01` = `DynamicFx-447494404`); projects persist streams by matchName. Wrapping existing parameters in topics and inserting new ones does not break saved projects while ids hold. Topics are flat inert rows to scripting/AEGP.
2. **Topology is frozen at `PARAMS_SETUP`.** A slot's group membership is its declaration interval, fixed for the process. Per-instance or per-compile regrouping of one physical slot is impossible; AEGP cannot re-parent effect parameter streams. Therefore per-pass grouping can only come from **pre-declared group banks** that the binding allocator assigns per shader.

## Decision

1. **Stream identity is the param id.** The persistent identity of every declared parameter is `uu.id` = murmur3 of the `ParamKey` variant's `Debug` rendering, as every released project already carries. The `Debug` rendering of every shipped `ParamKey` variant is **frozen**: changing one is a persistence break, exactly as moving an index used to be. A golden unit test pins the complete id table (group markers included) and its collision-freeness. Declaration indexes stop being a persistence contract; ADR-0013 §5's append-only rule survives as the policy for pool capacity growth, and ADR-0028's frozen-index mechanism is superseded by id stability.

2. **The panel becomes: heads, then `Main`, then up to twelve pass groups.**
   - Heads stay top-level: `Language`, `Source`, `Compile`, `Status`, `StateToken` (hidden), `Details`, `PlanToken` (hidden). ([ADR-0039](0039-canvas-expansion.md) adds no head control — its round-2 model is source-declared.)
   - **`Main` is the existing pool set wrapped in one topic** (ids unchanged — the measured-safe operation): all 104 v1 slots, the 16 growth slots, and the 50 gradient-stop rows, with each gradient's rows additionally wrapped in a nested `Gradient 1`/`Gradient 2` sub-topic. Every parameter of every **existing saved project therefore reopens inside `Main`** with values, keyframes and expressions intact, no migration.
   - **Twelve pass groups (`Pass 1`…`Pass 12`)** are new partitioned banks declared after `Main`, each containing Float 8, Integer 2, Bool 2, Color 3, Point 2D 2, Angle 1 — 18 slots per group, 216 new slots plus 24 markers. Heavy resource kinds (Layer, Gradient anchors/stops, Point 3D, Path) stay `Main`-only in v1. All groups declare `START_COLLAPSED`.

3. **Assignment is automatic from pass reflection; no source annotation.** At binding time: a parameter referenced by **exactly one pass** allocates from that pass's bank (pass order = envelope order); a parameter referenced by **two or more passes**, by a pass beyond the twelfth, by a raw single-pass source, or of a `Main`-only kind allocates from `Main`. **Overflow degrades, never fails:** when a pass bank's kind capacity is exhausted, the parameter allocates from `Main` and the status line notes the spill. A `@group` override annotation is out of scope for v1 (revisit condition).

4. **Keyframe stability outranks regrouping.** `build_with_reuse`'s exact-ParamId inheritance is unchanged and runs first: a parameter that already owns a slot keeps it across source edits **even when its pass assignment changes** — its group may go visually stale rather than lose keyframes (the ADR-0005/M2 contract wins). Fresh parameters and fresh plans allocate per §3. Re-adding the effect is the documented way to re-group a drifted instance. Existing saved plans continue rendering on their existing slots unconditionally.

5. **Group display names.** Declared names are the static `Main` / `Pass N`. Renaming a group row to the shader's actual pass name via `PF_UpdateParamUI` is an implementation-verification item: adopted if the host honors it (slot renames measured working for v1 kinds), silently dropped to static names if not — the `AEGP_SetStreamName` route stays forbidden while the measured undo-group hang stands (2026-08-16).

6. **Empty and unused groups.** Unused slots stay hidden as today. Hiding an entirely-unused group's header row by the same DynStream mechanism is expected but unmeasured; the accepted fallback (user decision 2026-08-26) is visible collapsed empty headers. Verify first during implementation.

7. **Untouched:** `BindingPlan` wire format, `ParamId` grammar, sequence schema, `StateToken`, graph grammar, Shader ABI. This is panel topology plus allocator policy only.

## Alternatives considered

- **By-kind groups (Floats/Colors/…)** — rejected by the user: organizes nothing semantically.
- **Index-break regroup with migration script** — unnecessary after TR-GRP-001; not taken.
- **Runtime re-parenting (AEGP)** — impossible; effect param streams cannot re-parent.
- **Custom-UI drawn panel presenting logical groups** — the only mechanism with fully free grouping, but it is the deferred custom-controls track with a measured host-crash history (ADR-0031 §7 → ADR-0033 §6); revisit there, not here.
- **Per-pass banks including Layer/Point 3D/Path/Gradient kinds** — deferred: multiplies declared streams for kinds that are rare per pass; `Main` serves them meanwhile.
- **More/fewer than twelve groups, other bank shapes** — twelve covers the largest shipped shader (`apple-thermal`, 10 passes) with headroom; bank shape (8F/2I/2B/3C/2P/1A) is sized from the shipped corpus (2–3 params per pass typical, floats dominant). Both are tunable before Accept; growing later is an id-safe append.

## Consequences

### Benefits

- The panel reads as the shader's structure: shared controls in `Main` on top, one collapsed group per pass — the user's stated model, with zero annotation burden.
- Existing projects reopen organized (everything in `Main`) with nothing lost; no migration, no release break.
- The id contract frees every future topology decision (including ADR-0039's head placement) from index arithmetic, guarded by one golden table.

### Costs and risks

- **Scale:** declared parameters grow from 177 to ~417 (+216 bank slots, +24 markers). Costs to verify: `PARAMS_SETUP` time, project-file growth per instance, and the idle observer's per-second stream walk (~2.3×) — each gets a measured number before release; the walk has an obvious optimization (skip hidden unbound streams) if needed.
- The id table becomes load-bearing forever; an accidental `ParamKey` Debug change is a silent project-breaker until the golden test catches it. The same table also detects an upstream crate change to the hashing scheme.
- Numeric-index user scripts break once (name/matchName addressing unaffected); release notes must say so. Harness index pins move once (mechanical, re-pinned by their own runs).
- Group assignment can go visually stale under §4's stability rule (a keyframed param keeps its old group after a pass refactor) — accepted, documented, self-healed by re-adding the effect.
- Pass-bank capacity is a guess against future shaders; the §3 overflow rule makes exhaustion graceful (param lands in `Main` with a status note), and banks can grow by id-safe append later.
- TR-GRP-001 measured AE 2025 only; AE 2026 must repeat the reopen check before release. AE 2023/2024 stay unmeasured (hosts unavailable; ADR-0014 §7 gate unchanged).

## Revisit conditions

- Any host measurement of id matching failing on a supported AE year — forces back to append-only indexes and supersedes §1.
- Field demand for explicit `@group` override, per-pass heavy kinds, or >12 groups — extends §2/§3 by id-safe append.
- The `AEGP_SetStreamName` undo-group requirement being solved — reopens live group renaming beyond the §5 `PF_UpdateParamUI` route.
- The custom-controls track shipping a drawn panel — may supersede topic-based grouping entirely.

## Verification obligations

- Unit: golden id table (all ~441 declared ids incl. markers, collision-free); grouped `declaration_order()` pins; head stream-index constants re-pinned; allocator tests — single-pass→`Main`, per-pass assignment, shared-param→`Main`, overflow spill, exact-id inheritance overriding regroup.
- Host, AE 2025 **and** 2026, on the grouped production build: the TR-GRP-001 `baseline.aep` (0.0.5-saved) reopens with every value/keyframe/expression/label intact inside `Main`; a multi-pass shader (`apple-thermal`) compiles with per-pass groups populated and renders identically to 0.0.5; M2 + M3 batteries green; measured numbers for `PARAMS_SETUP` time, idle-walk cost, and per-instance project-size delta.
- Panel: collapsed-by-default groups, bound-slot reveal inside pass groups, gradient sub-groups inside `Main`, group-rename outcome (§5) and empty-group outcome (§6) recorded either way.
- Evidence recorded under a new TR row per the evidence policy before any release ships the grouped topology.
