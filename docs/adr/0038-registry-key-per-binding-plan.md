# ADR-0038: The process registry is keyed per binding plan, not per source alone

- Status: Accepted
- Date: 2026-08-21 (drafted as `Proposed` with two mechanisms; mechanism B selected the same day)
- Deciders: DynamicFX project owner selected **mechanism B** (the recommended option) on 2026-08-21; the assistant session drafted the options and the refinement below
- Owners: DynamicFX project
- Related decisions: fixes a violation of [ADR-0005](0005-stable-parameter-ids.md) (per-instance stable ParamIds) and [ADR-0007](0007-identity-and-cache-boundaries.md) (layered identities and cache boundaries); relies on [ADR-0013](0013-paramid-grammar-and-pools.md) (the per-instance `BindingPlan`) and [ADR-0016](0016-sequence-schema-v1.md) (which already persists that plan per instance); appends the `dfx:plan:v1` domain to [ADR-0017](0017-hash-domains.md)'s tag list (ADR-0017 §2 permits append-only growth through an ADR — this one). [ADR-0015](0015-statetoken-and-diagnostics.md)'s StateToken is **unchanged**.
- Related implementation: `src/lib.rs` (`registry`/`registry_get_with_origin`/`registry_insert`/`registry_contains_source`/`registry_latest`, `Local::own_plan`/`own_plan_ids`/`plan_lineage`/`last_good`/`source_absent`/`self_authored`, `follows_stream`, `resolve_transported_definition`/`resolve_active`, `resolve_from_snapshot`, `publish_token_param`/`plan_word`, `flatten`, the `Command::CompletelyGeneral` arm and `GeneralReply`), `src/host/idle.rs` (`sync_state_token`, `decide_token`, `slot_ui_out_of_date`, `apply_slot_ui`), `src/host/params.rs` (`ParamKey::PlanToken`, `plan_token_stream_index`), `src/identity.rs` (`plan_identity`), `src/binding.rs` (`BindingPlan::mapping`), `src/persistence.rs` (`to_previous_plan`, the persistent pool-kind codes)
- Related tests/audits: [TR-BIND-002](../TEST_MATRIX.md#tr-bind-002--copied-instance-corrupts-slot-mapping-field-defect); public issue [#6](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/6); field evidence [`docs/audits/evidence/field-20260819-copy-instance/`](../audits/evidence/field-20260819-copy-instance/README.md) and [`field-20260821-prism-sample/`](../audits/evidence/field-20260821-prism-sample/README.md) (Finding B)

## Context

The session process registry maps a **source fingerprint** to one compiled artifact:

```
registry: TokenMap<u64, Arc<CompiledEffect>>          // key = session_token(language, source), a 51-bit BLAKE3 truncation
```

`registry_insert` refuses only a *different source* under the same 51-bit key (a truncation collision); for the **same source it replaces** the entry — the last instance to compile owns it. `resolve_transported_definition`, on a `TokenState::Active(fp)` whose `fp` differs from the instance's current token, calls `registry_get(fp)` and adopts whatever `CompiledEffect` it finds — **including that effect's `definition.binding`**, the per-instance `BindingPlan`.

The stored `CompiledEffect` carries instance-specific state, not just source-shared state:

- `definition.binding` is the per-instance `BindingPlan` from `binding::build_with_reuse`. Two instances of one source legitimately differ here: an instance edited in place over several commits keeps old ParamIds in their first slots and appends new ones (a *migrated* plan, slot order ≠ declaration order), while a fresh instance gets declaration-order slots.
- `externals` — which AE parameter index feeds each graph layer/path resource — is derived **from `definition.binding`** (`evaluate_committed_source`: `let slot = def.binding.bindings.get(index)?.slots.first()?;` → the AE param index). So the layer-input wiring is plan-derived too, not only the float/angle stream map.

The registry has a third consumer besides render clones: the **idle observer** (`src/host/idle.rs`). After giving an instance its `PF_Cmd_COMPLETELY_GENERAL` observation it calls `registry_get(fp)` by source alone to republish slot labels, Hidden flags and fresh-binding defaults through AEGP (`apply_slot_ui`) and to probe staleness (`slot_ui_out_of_date`). That is how the *original* instance's stream names were renamed to the copy's labels in the field log (`idle slot ui applied` on the original right after the copy compiled).

Consequences observed in the field (TR-BIND-002, issue #6, AE 2025 2026-08-19; flicker re-confirmed on AE 2026 2026-08-21): copy/paste an instance whose plan is migrated, compile the copy, and the copy's fresh plan replaces the shared entry; the **original** instance then reads its AE streams and configures its slot UI through the copy's declaration-order table — 16 of 18 float slots and both angles permute, four values clamp to other controls' slider ranges — with `Status: compiled` and no diagnostic. Both instances flicker while they alternately own the entry. Even when two instances carry **identical** fresh plans (the simple 12-param `prism` sample), no value corrupts but the shared entry still churns as they alternate ownership — the visible flicker with no data loss.

This contradicts ADR-0005 (each instance's ParamIds are stable and its own) and ADR-0007 (identities are layered so incompatible artifacts are not reused). The per-instance plan authority **already exists**: ADR-0016 persists each instance's plan in its sequence snapshot, and `resolve_from_snapshot` already rebuilds an instance's definition from `snapshot.to_previous_plan()`. The registry is the one place that treats a per-instance artifact as source-shared.

The fix changes a cache key / identity domain, so ADR policy requires this ADR before implementation.

## Decision

**Invariant.** An instance's own binding plan governs that instance's stream reads (`read_bound_values`), slot UI (`slot_configs`), and layer-input wiring (`externals`). A second instance of the same source must never alter the first instance's parameter mapping, and the registry must never serve one instance a compiled artifact built for a plan other than its own.

**Mechanism B is accepted**: the registry is keyed by `(source fingerprint, plan identity)`. The refinement below is part of the decision; it closes the two gaps the `Proposed` draft left open (the idle observer, and render clones whose own plan is older than the entry they need).

### 1. Plan identity — `dfx:plan:v1`

A plan's identity is a BLAKE3-256 digest through `identity::Canonical` with the domain tag `dfx:plan:v1`, over the plan's bindings **in order**: the ParamId bytes, the slot count (u32), then per slot the **persistent pool-kind code** from ADR-0016's snapshot encoding (`Float 0 … Path 9`, append-only) as u32 and the slot index as u32. The first eight digest bytes, little-endian, form the `u64` plan id. The `inherited` flag is **excluded**: it is compile-transient default-writing state, not part of the mapping, and two instances whose id→slot tables agree must share one entry whether they inherited or not. The id is session-local and is **never persisted** (ADR-0016's schema is untouched); it gets a pinned golden vector like the other domains so an accidental encoding change is caught, and may change only through an ADR amendment.

### 2. Registry shape and operations

Per source fingerprint `fp` the registry holds a bucket: `entries: plan_id → Arc<CompiledEffect>` and `aliases: past plan_id → plan_id`.

- `insert(fp, plan_id, effect, lineage)`: refuse (fail closed, as today) when the bucket already holds a **different source** under this 51-bit `fp`; refuse when the same `plan_id` is present with a **different** mapping (digest collision — mappings compare by ParamId and slots, never by `inherited`); when the same `plan_id` is present with an **equal** mapping, **keep the existing `Arc`** — nothing is evicted, so two instances with identical plans never churn the entry (this is what removes the identical-plan flicker); otherwise insert. Afterwards record `aliases[l] = plan_id` for every `l` in `lineage` with `l ≠ plan_id`. **An alias asked to point at a second, different target becomes ambiguous and never resolves again**: a duplicate and its original share every plan from before the copy, and when they reach the same source by different edit routes their plans differ — neither may claim the other's stale clones.
- `get(fp, plan_id)`: the direct entry, else an unambiguous alias target, else none. **Direct always wins over alias.** Alias hits log their own line (`definition resolved from process registry via lineage`).
- `contains_source(fp)`: the bucket is non-empty.
- `latest(fp)`: the entry of the **most recent successful publication** in the bucket, a keep-existing republication included — "the instance that compiled most recently" is the best available guess for a clone that cannot name its plan (§4).

### 3. Lineage

Each `Local` keeps a session-local, deduplicated list of the plan identities it has compiled **with** (the `previous` seed) and **to** (the result). It is never persisted. Every publication passes this lineage so that every plan the instance ever held this session aliases to its current entry. A render clone of that instance always carries a plan from that lineage — the snapshot it was flattened with, or an entry it resolved earlier — so a clone whose token moved on (the UI instance committed new source) still lands on **its own instance's** entry even when it is several edits stale.

Each `Local` also remembers the **last definition it compiled successfully this session** (`last_good`: language, fingerprint, source and plan — snapshot-shaped, session-local). It is the reuse seed whenever the live definition is gone, so a failed compile no longer demotes a migrated plan to declaration order on the next success (an ADR-0013 hole that per-plan keying would otherwise turn into a clone/UI mismatch), and it is what `flatten` emits when there is no live definition (§4). The seed order everywhere — reuse, resolution, flatten — is: live definition, then `last_good`, then the restored snapshot.

### 4. Resolution

The StateToken stream is transport for clones; it lags an instance's **own** compile by up to one idle tick (only `UserChangedParam` and the idle mirror write it, main-thread-render and `UpdateParamsUi` compiles do not), and `render` off the main thread resolves whatever `Local` it holds — the UI instance's included. So, first: **a `Local` whose definition came from its own observation (`self_authored`) does not follow the stream at all**; observation is authoritative over transport (architecture §5.1) and settles any disagreement on the next callback or tick. Every other `Local`, on `TokenState::Active(fp)` with `fp ≠ local.token`:

1. The candidate plan ids, in order: the transported **plan word** (§7) when non-zero, then its live definition's plan, then `last_good`'s, then the restored snapshot's.
2. `get(fp, candidate)` for each → adopt: `definition resolved from process registry` for a direct hit, `… via lineage` for an alias hit.
3. Miss, and the snapshot's fingerprint **equals** `fp` → rebuild from the snapshot (`resolve_from_snapshot`: it is the current source and its plan is this instance's; the rebuild inserts under that plan).
4. Miss, and the snapshot names **another** source: with **no bucket** for `fp` the registry is cold (fresh process, or a torn token/snapshot pair) and the checksummed snapshot wins exactly as ADR-0015 §2 says; with a bucket present the token's source is real and published, so the stale snapshot does **not** win — the clone adopts `latest(fp)` (`registry knows this source; stale snapshot does not win; adopting latest entry`). Rebuilding the old source here would recompile it on every frame until AE re-flattens the clone.
5. Miss with no usable snapshot → `latest(fp)`, logged (`definition resolved by latest entry for source; clone carries no plan`, or `registry has this source but not this plan; adopting latest entry` when the clone did carry one); with no entry at all → pass through (`token missed registry with no snapshot; passing through`), the pre-existing cold-miss behaviour. A plan-bearing miss is **not** answered by clearing the definition: that only made the clone plan-less for the very next frame and took the same fallback one frame later.

`flatten` emits the live definition's snapshot; else `last_good`; else the restored snapshot; else an empty payload — the two fallbacks only for an instance whose last observation neither found the source block absent (`source_absent`) nor failed to compile (its status is `Ok`: it has simply not observed yet this session, or it is a render clone). A cleared, non-source or non-compiling expression still persists as empty, as today: the stream may read `Active` for that text until the idle mirror catches up, and a persisted snapshot would outvote the broken expression on reopen. Clones flattened from a reopened-but-not-yet-compiled instance therefore carry the instance's plan, which is what makes the `latest` fallback rare rather than routine. On an `Uninitialized` word a clone that holds a snapshot keeps a definition it already rebuilt from it (the torn-pair rule applied consistently across the calls of one frame); a cleared source reaches clones through their next re-flatten.

These session-local fields live in the `Local` the host holds; After Effects disposes and rebuilds that `Local` from its flattened payload on every save (`SequenceFlatten`/`SequenceResetup`), so the lineage and `last_good` restart from the saved plan after each save. A clone that last rendered before a save and whose instance has since moved on may therefore miss its alias and take the logged `latest` fallback. `SmartPreRender`/`SmartRender` call the resolver unconditionally (a no-op when the token matches), so a stale clone never stages external textures by an old wiring and then renders with a new one.

### 5. The idle observer uses the instance's own artifact

`PF_Cmd_COMPLETELY_GENERAL` is issued by the idle observer itself through `AEGP_EffectCallGeneric`, synchronously on the main thread. The arm now leaves this instance's own `(token, Arc<CompiledEffect>)` in a main-thread thread-local reply (the same hand-off pattern `SMART_WINDOW`/`SMART_LAYERS` already use between PreRender and SmartRender), and the idle observer takes it right after the call returns. `apply_slot_ui` and `slot_ui_out_of_date` operate on **that** artifact — never on a registry lookup — and ignore a reply whose token does not match the fingerprint the observer computed from the streams (an expression changed between the two reads; the next tick settles it). The token decision uses the reply first: a matching token with an artifact → `Active(fp)`; a reply carrying a failure code → `Invalid(code)`, the instance's **own** diagnostic, no longer masked by another instance's success on the same text; no reply at all (the call failed, or the expression changed between the reads) → today's per-source logic (`contains_source(fp)` → failure map → pending E53). A token published without this instance's artifact is logged.

### 6. Unchanged

`StateToken` encoding and semantics (ADR-0015), sequence schema v1 (ADR-0016), `CompiledEffect`, the render/PreRender/SmartRender path, the PIPL, and every released parameter index. The topology grows by exactly one hidden parameter at the tail (§7), the append-only growth ADR-0013 §5 provides for. Existing projects change no meaning; the compile transaction stays session-local.

### 7. Plan token transport

The first host run of the fix (AE 2026, 2026-08-21, `fix-6E4E80A6-run1`) showed that in a warm session render clones carry **no snapshot at all**: After Effects takes an instance's flattened copy when the effect is added and keeps serving it to render clones — the compile happens inside an idle observation that AE does not treat as a sequence-data change, and `PF_OutFlag2_SUPPORTS_GET_FLATTENED_SEQUENCE_DATA`, which this plug-in already declares, did not make it ask again. Every such clone logged `resolved by latest entry for source; clone carries no plan`; the instance that compiled first then rendered through the other's plan. Snapshot-carried identity therefore reaches clones only after a save/reopen or in a fresh process (where the same run passed), never in the live session where the defect is reported. The old source-keyed registry had hidden this because it never needed the clone's plan.

The plan identity is therefore **transported the way the source fingerprint already is**: a second hidden, non-time-varying Float parameter, `PlanToken`, declared after every gradient stop (the last parameter; `host::params::plan_token_stream_index()`), carries the `dfx:plan:v1` identity of the instance's published artifact — the identity is truncated to 51 bits with zero mapped to one, exactly like the token fingerprint, so it is an exact f64 integer; `0` means "no plan word". It is written by the same two paths that write the StateToken — `publish_token_param` in UI-callback contexts and the idle observer's AEGP mirror — and, like the token, only when it differs, so a scan never dirties the project. The resolver reads it beside the token and tries it first (§4.1). Projects saved before this build restore the default `0` and take the §4 fallbacks; older builds ignore the extra stream. `plan_lineage`, `last_good` and the snapshot fallbacks stay as the safety net for a clone whose plan word has not been mirrored yet (up to one idle tick after a scripted compile) and for projects saved by earlier builds.

## Alternatives considered

- **Mechanism A — the registry value carries no per-instance state.** Split `CompiledEffect` into a source-shared artifact (declarations, passes, `ExecutionPlan`, `window`, `source`) keyed by `fp` alone, with each `Local` overlaying its own plan and deriving its own `externals`, `slot_configs` and stream map. The cleaner conceptual model and the only route to sharing GPU pipelines between same-source instances with different plans — but it touches the identity model and the render path far more widely (struct split, `externals` threading, `slot_configs`/`read_bound_values` re-plumbing) for a memory optimisation nobody has measured. **Deferred**, not rejected: the revisit condition below names it.
- **Do nothing / document the workaround only** (add a fresh DynamicFx and paste the expression; remove+re-add to repair). Rejected: TR-BIND-002 is silent data corruption on a released build; a workaround is not a fix, and ADR-0005/0007 are violated.
- **Fix it in shaders** (normalise so a permuted plan still looks acceptable). Rejected outright: impossible in general and contrary to the fixed-pool + stable-ParamId design.
- **Keep last-writer-wins but re-apply the reader's own plan after every resolve.** Rejected: it keeps the shared entry thrashing (the flicker) and re-introduces a race every time two instances render concurrently.
- **Carry the plan identity in the StateToken.** Rejected: the 51-bit payload has no room and ADR-0015's released encoding would change; splitting the word would weaken the source fingerprint. A second word beside it (§7) keeps ADR-0015 intact.
- **Make After Effects re-flatten the instance after a compile** so clones carry the snapshot. No API does this: the plug-in already declares `SUPPORTS_GET_FLATTENED_SEQUENCE_DATA` and AE still served the copy it took at `addProperty` time; a compile is not a parameter change AE can see.
- **Add an instance id to the snapshot** so clones can name their instance. Rejected for now: a sequence-schema change (ADR-0016 v2) to solve a session-local problem that the snapshot's own plan plus the session lineage already solve.

## Consequences

### Benefits
- The field corruption (TR-BIND-002) and the flicker both close: an instance's mapping is immune to another instance of the same source, and identical plans share one entry without eviction.
- The per-instance plan authority that ADR-0016 already persists becomes the single source of truth for stream reads, slot UI, and layer wiring; the idle observer stops being a second path that could disagree.
- No persistent-format or schema change; no re-release migration.

### Costs and risks
- The registry holds one artifact per distinct `(source, plan)` plus a small alias table — bounded by the number of distinct plans actually present in a project.
- One more hidden parameter (`PlanToken`, the last index) in every project saved by this build onward; it costs a word per instance and is invisible in the Effect Controls. Between a scripted compile and the next idle tick a clone may still take a §4 fallback; the harness measures that window as zero `latest` adoptions.
- The plan identity must be computed identically at insert and at resolve — pinned by a unit test that round-trips a plan through `persistence::Snapshot::from_state` → `to_previous_plan`.
- The `latest` fallback (§4.4–4.5) is the one place a foreign plan can still be adopted, and it is always logged. It remains reachable for a clone flattened from an instance that has neither compiled this session nor restored a snapshot (a fresh instance before its first compile) while another instance already published the same text, and for a stale clone whose lineage alias became ambiguous. Both are transient — AE re-flattens clones from the instance's next snapshot — and the harness treats any such log line as a failure so the host run shows whether they occur at all.
- `flatten` now persists `last_good` / the restored snapshot for an instance without a live definition that has not observed yet this session (a reopened instance before its first compile; render clones), and an empty payload after a no-source or failed observation as before. The M3 battery (save/reopen/corrupt/recover/torn/undo) is the regression net for this.
- The M2/M3 batteries and the new two-instance battery must be re-run on the host before this is considered shipped.

## Revisit conditions
- If pipeline/GPU-artifact memory for many same-source instances is measured to matter, implement mechanism A (or add pipeline de-duplication under B keyed by `PipelineKey`, which is already artifact/device-based per ADR-0007).
- If the plan-less fallback is ever observed to adopt a foreign plan on the host, the `flatten` follow-up above becomes mandatory.
- If a future feature makes two instances legitimately need to *share* a mutable plan, this ADR is superseded.

## Verification obligations
Before this is marked shipped, `TEST_MATRIX` must carry a harness leg (currently the `NOT_RUN` half of TR-BIND-002), run on the host:
1. Instance **A** with a *migrated* plan: compile with params `p1..p3`, then insert `p0` first in the block and re-commit → A keeps `p1..p3` in F01..F03 and `p0` in F04.
2. Instance **B** fresh (`addProperty` + the same final expression) → declaration-order slots.
3. Render both with per-slot **distinguishable** values (floats, both angles, and at least one **layer input** bound to a different layer per instance so the `externals` wiring is exercised, not only the float map); assert each instance reads **its own** values and samples **its own** layer; assert A's stream names do not change when B compiles (the idle-observer half).
4. Repeat for **both compile orders** (A then B, B then A).
5. Save the two-plan project, reopen it in the same session and in a fresh process (`aerender`), render both again without pressing Compile; the log lines `resolved by latest entry`, `adopting latest entry`, `not this plan` and `missed registry` must stay at zero across every leg, while `via lineage` and `rebuilt from snapshot` are reported.
6. Unit tests: the plan-identity golden vector and snapshot round trip (through the wire format); two entries under one fingerprint; keep-existing on an equal mapping; alias resolution with direct-over-alias precedence and alias ambiguity; `latest` following the last publication; the collision refusals; `flatten` with and without a live definition / `source_absent` / a failed compile; the reuse seed surviving a failed compile; the stream-following decision and the token decision as pure functions; the parameter topology test pinning `PlanToken` as the last index with every released index unmoved.
Record `FAIL` on the pre-fix build first (the defect), then green on the fix build, plus the M2 and M3 batteries on AE 2025 and AE 2026.
