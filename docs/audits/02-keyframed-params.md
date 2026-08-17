# Audit 02: Keyframed Parameters

- Milestone: M2 — Keyframed Parameters
- Audit state: Complete (M2 exited 2026-08-12: all six roadmap exit criteria have evidence — TR-M2-001/002/003)
- Baseline branch: `codex/stabilize-programmatic-flow`
- Related roadmap: [../ROADMAP.md](../ROADMAP.md)
- Related test matrix: [../TEST_MATRIX.md](../TEST_MATRIX.md)
- Related ADRs: [ADR index](../adr/README.md) — ADR-0013 fixes the ParamId/pool contracts M2 implements; no M2-entry ADR is required (ADR-0009)

## Outcome

**M2 is complete; exited 2026-08-12.** All six roadmap exit criteria hold with live evidence: keyframed values render different exact pixels at different times (TR-M2-001); defaults render before any stream commit (TR-M2-002); declaration-order changes preserve values by ParamId (TR-M2-001); rename-with-alias keeps keyframes and a type change reallocates (TR-M2-002 live + unit rules); pool overflow rejects the whole definition atomically with a host demonstration (TR-M2-003); save/reopen is explicitly not claimed (M3). Slice 3 (TR-M2-003, artifact `3D11B511…`) pinned the value encodings of every v1 kind in one five-band fixture frame — int passes integers, bool is `int + hint:bool` at 0/1 (std140 has no host-shareable bool; this fixes ADR-0011's "bool as i32" surface form, caught by the pinned unit fixture before any host run), color passes 0..1 RGB straight, point normalizes pixels by the render extent, angle passes degrees — and demonstrated the 49-float overflow rejecting with a visible diagnostic, a zeroed token, and byte-exact pass-through.

**Keyframed parameters work on the host, numerically exact.** On AE 2025 (artifact `BB9B17F0…`), a scripted `float gain` shader bound to Float slot 0 with its ParamId applied as the visible label by the idle observer through AEGP (unbound slots kept default names and hid — no UI callback involved on the scripted path); keyframes 0.0→0.8 interpolated in rendered pixels at exactly 0 / 102 / 204; and after a source edit that declared a new `extra` parameter *before* `gain`, gain kept its slot and both keyframes while extra took the next free slot — stable-ID slot reuse proven live, with the t=0.4 frame still exact. TR-M2-001 is `PASS` ([TEST_MATRIX.md](../TEST_MATRIX.md)); the "keyframed values at defined times" and "label/order preserves values by ParamId" exit criteria have live evidence.

Slice 2 (same day, artifact `8D0E5E04…`, TR-M2-002 `PASS`): the `@param` annotation grammar exists and is fixed by parser tests per ADR-0013 — `label:"…" min: max: default: alias: hint:angle/color`, malformed entries fail the definition closed with line numbers, stale annotations for nonexistent members are ignored. Live on AE 2025: the annotation labeled the slot "Master Level", the idle observer wrote the 0.5 default into the fresh binding via AEGP, and the untouched-stream frame rendered gray 127 — the **defaults-before-committed-streams criterion holds**. Renaming `level`→`volume` with `alias:level default:0.9` kept the slot and both keyframes, applied the new label, and did **not** apply the new default (plugin log: `1 defaults written` on first bind, `0 defaults written` on the inherited re-bind) — **rename-with-alias now has live evidence**.

Known v1 boundaries recorded with the exit: color/point annotation *defaults* are rejected as scalar-only (fail closed; their AEGP value plumbing is future work — values themselves bind and encode correctly); color working-space semantics beyond "AE RGB passes 0..1 straight in an unmanaged 8-bpc project" belong to M5's format work.

The implemented slice:

- `binding::build_with_reuse` implements ADR-0013 §2 slot inheritance: exact current-ID match first, then single-generation alias match; inheritance requires unchanged slot-kind requirements (a kind change reallocates); unmatched declarations fill ascending free slots around inherited holes; capacity validates over the complete plan atomically. Six new unit tests cover reorder stability, rename-with/without-alias, kind-change reallocation, hole filling, vec4 atomic pairing, and reuse overflow (44 total green).
- The observation path now seeds lowering with the previous plan, so editing the shader source keeps keyframed slots for stable IDs.
- Slot UI configuration from the `BindingPlan` is implemented in the host shell: bound slots take their ParamId as the label (vec4 alpha companions get an " A" suffix) and become visible; unbound slots restore default names and hide via AEGP DynamicStream. Name and visibility tokens retry independently (prototype lesson), and nothing touches the tree before a definition exists (the addProperty guard).

## Visible evidence

Curated under [evidence/m2-keyframed/ae2025/](evidence/m2-keyframed/ae2025/): scenario logs m2a-d, interpolation frames `m2b_t0/t04/t08.png` (0/102/204 exact), post-edit frame `m2d_t04.png` (102 exact with gain's keyframes surviving the re-bind), [checks.txt](evidence/m2-keyframed/ae2025/checks.txt), and the plugin log showing `compiled: 1 pass, 1 params → idle slot ui applied: 1 bound → token`, then `2 params → 2 bound → new token` after the edit.

Rename-with-alias on the host still waits on the annotation grammar (aliases have no GLSL surface syntax yet); the rule is unit-tested.

## Baseline

- M1 exit state: artifact `BDDB51F1…` verified on AE 2025 (see [01-first-frame.md](01-first-frame.md)).
- This slice's host run: artifact `BB9B17F010024F7CDF10CE6C5A2D32D3051FB1BCBC38A146F8476BEA65DC4F56` (8,199,680 B; commit `86b7c7a` + idle slot-UI publication), toolchain 1.97.1, AE 2025 25.6.6 zh_CN, Windows 11 10.0.26200; hash chain verified at install.

## Code paths

- `src/binding.rs` — `build_with_reuse` (fresh allocation is now reuse against an empty plan); `SlotRef` gained `Hash`.
- `src/definition/effect.rs` — `lower_raw_single_pass` takes the previous plan.
- `src/lib.rs` — observation seeds reuse from `Local`'s compiled binding; `configure_slots`/`apply_visibility`/`set_slot_hidden` configure slot names and DynamicStream visibility from UI callbacks; `Local` tracks `configured_token`/`visibility_token`.
- `src/host/params.rs` — `default_slot_name` (single naming source) and `stream_index_of`.
- `src/host/idle.rs` — `apply_slot_ui`: when the token moves, the idle observer applies slot names + Hidden flags via AEGP (`DynamicStream::set_stream_name`/`set_dynamic_stream_flag`) before publishing the token, so the scripted path needs no UI callback; slice 2 adds fresh-binding scalar default writes (`set_one_d`).
- Slice 2: `src/frontend/annotation.rs` — the `@param` parser (grammar fixed by its tests); `src/frontend/glsl.rs` — annotation merge (hint retyping, arity/range validation, aliases, UI meta); `src/definition/param.rs` — `ParamUiMeta`; `src/binding.rs` — `ParamBinding.inherited`; `src/lib.rs` — `SlotConfig`/`slot_configs`, UI-path range/default metadata.

## Contracts fixed or changed

None. M2 implements ADR-0013 §2/§7/§8 (aliases, atomic validation, fixture-pinned value encodings).

## Commands and exact host steps

All on 2026-08-12: `cargo test --all` (44, then 50 with the annotation suite); `cargo build --release` (zero warnings); elevated `scripts/install.bat 2025` per artifact (hosts verified closed; installed hash = build hash); `pwsh scripts/m2/run_m2.ps1 -Year 2025` (scenarios m2a-d), later `-Scenarios e,f,g,q` for the annotation run; `-Checks` after each (six numeric probes total, all exact).

## Observed evidence

- TR-M2-001 stable ParamId and BindingPlan unit/integration tests: `PASS` (float kind + reuse core)
- TR-M2-002 @param annotations, defaults, live alias rename: `PASS` (records in [TEST_MATRIX.md](../TEST_MATRIX.md); int/bool/color/point fixtures still pending)

## Findings and failures

None yet.

## Known limitations

Inherited from M1: UI not configured from bindings; value encodings unpinned; keyframed reads unverified on any host.

## Residual risks

- AE stream-value reads at arbitrary render times (keyframe interpolation) are unmeasured through the new topology.
- Slot-reuse UX (hidden slots, renames) may surface host quirks the prototype's DynamicStream experience only partially covered.

## Decision changes

None.

## Next exact action

Begin the M3 entry gate: draft the three M3-entry ADRs required by ADR-0009 as Proposed for user review — StateToken layout (with undo/redo and project-dirty semantics and the stable diagnostic-code registry), sequence schema v1 (codec, limits inside ADR-0012's 8 MiB budget, checksum), and the hash algorithm / canonical serialization / domain separation. M3 implementation must not start before they are Accepted.

## Reproduction

Follow the `CLAUDE.md` reading order; confirm TR-M2-001 is `NOT_RUN` and the M1 evidence set exists.
