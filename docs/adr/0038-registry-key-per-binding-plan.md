# ADR-0038: The process registry is keyed per binding plan, not per source alone

- Status: Proposed
- Date: 2026-08-21
- Deciders: **pending** — the user chooses between mechanism B (recommended) and mechanism A at Acceptance; assistant session drafted the options
- Owners: DynamicFX project
- Related decisions: fixes a violation of [ADR-0005](0005-stable-parameter-ids.md) (per-instance stable ParamIds) and [ADR-0007](0007-identity-and-cache-boundaries.md) (layered identities and cache boundaries); relies on [ADR-0013](0013-paramid-grammar-and-pools.md) (the per-instance `BindingPlan`) and [ADR-0016](0016-sequence-schema-v1.md) (which already persists that plan per instance)
- Related implementation: `src/lib.rs` (`registry`/`registry_get`/`registry_insert`/`registry_contains`, `session_token`, `resolve_transported_definition`, `resolve_from_snapshot`, the `externals` derivation in `evaluate_committed_source`, `slot_configs`, `read_bound_values`), `src/binding.rs` (`BindingPlan`)
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

Consequences observed in the field (TR-BIND-002, issue #6, AE 2025 2026-08-19; flicker re-confirmed on AE 2026 2026-08-21): copy/paste an instance whose plan is migrated, compile the copy, and the copy's fresh plan replaces the shared entry; the **original** instance then reads its AE streams and configures its slot UI through the copy's declaration-order table — 16 of 18 float slots and both angles permute, four values clamp to other controls' slider ranges — with `Status: compiled` and no diagnostic. Both instances flicker while they alternately own the entry. Even when two instances carry **identical** fresh plans (the simple 12-param `prism` sample), no value corrupts but the shared entry still churns as they alternate ownership — the visible flicker with no data loss.

This contradicts ADR-0005 (each instance's ParamIds are stable and its own) and ADR-0007 (identities are layered so incompatible artifacts are not reused). The per-instance plan authority **already exists**: ADR-0016 persists each instance's plan in its sequence snapshot, and `resolve_from_snapshot` already rebuilds an instance's definition from `snapshot.to_previous_plan()`. The registry is the one place that treats a per-instance artifact as source-shared.

The fix changes a cache key / identity domain, so ADR policy requires this ADR before implementation.

## Decision

**Invariant (not in dispute).** An instance's own binding plan governs that instance's stream reads (`read_bound_values`), slot UI (`slot_configs`), and layer-input wiring (`externals`). A second instance of the same source must never alter the first instance's parameter mapping, and the registry must never serve one instance a compiled artifact built for a plan other than its own.

**Two mechanisms deliver the invariant. This ADR is `Proposed` until the user selects one; that choice is the binding decision.**

### Mechanism B — key the registry by `(source fingerprint, plan identity)` *(recommended)*

- Compute a stable **plan identity** at compile time: a hash over `BindingPlan.bindings` (each `id → slots`, in order). Key the registry by `(fp, plan_id)`.
- `registry_insert` / `registry_get` / `registry_contains` take the compound key. Two instances with different plans occupy different entries; identical plans share one (the `prism` case — flicker gone because neither evicts the other).
- The `StateToken` written to the AE stream is **unchanged** (it still carries only `fp`; ADR-0015/0016 schemas untouched). `resolve_transported_definition` disambiguates using the plan the instance already has: `local.compiled`'s plan when present, otherwise the plan reconstructed from `local.snapshot` (`to_previous_plan()`), which every render clone and reopened/pasted instance carries.
- `registry_insert` keeps its fail-closed check, now on the compound key: refuse if a *different* `(source, plan)` would collide under one hashed key.
- `CompiledEffect` is unchanged; the render path is unchanged.

Rationale for recommending B: because `externals` (layer-input wiring) is plan-derived, two different plans are genuinely **two different compiled artifacts**, so caching them under distinct keys is the accurate model, not a workaround. The change is contained to the registry key and the resolve lookup — low risk on a released-contract surface. Cost is a handful of extra small artifacts per source (bounded by the number of distinct plans actually present in a project).

### Mechanism A — the registry value carries no per-instance state

- Split `CompiledEffect` into a **source-shared** artifact (parsed declarations/`params`, `passes`, `plan::ExecutionPlan`, `window`, `source`) held by the registry keyed by `fp` alone, and **per-instance** data (`BindingPlan` and the `externals` derived from it) held by each `Local` from its own compile or its snapshot.
- At resolve, a registry hit supplies the shared artifact; the instance overlays its own plan and derives its own `externals`, `slot_configs`, and stream map.
- `StateToken` unchanged.

Rationale against A *for now*: it is the cleaner conceptual model (the registry becomes a pure source-shared cache and GPU pipelines are shared across instances of one source), but it touches the identity model and the render/PreRender path (`externals` threading, the `CompiledEffect` split, `slot_configs`/`read_bound_values` re-plumbing) far more widely. Its only correctness-neutral extra benefit over B is de-duplicating GPU pipelines between same-source instances with different plans — a memory optimization not worth the churn on a released-contract fix. A remains the right target if pipeline de-duplication is later shown to matter.

Neither mechanism changes any persistent field, parameter index, `StateToken`/sequence schema, or PIPL. Existing projects change no meaning; the compile transaction stays session-local.

## Alternatives considered

- **Do nothing / document the workaround only** (add a fresh DynamicFx and paste the expression; remove+re-add to repair). Rejected: TR-BIND-002 is silent data corruption on a released build; a workaround is not a fix, and ADR-0005/0007 are violated.
- **Fix it in shaders** (normalise so a permuted plan still looks acceptable). Rejected outright: impossible in general and contrary to the fixed-pool + stable-ParamId design.
- **Mechanism A vs B**: captured above. B is recommended; A is the deferred cleaner target.
- **Make the registry last-writer-wins but re-apply the reader's own plan after every resolve.** Rejected: it keeps the shared entry thrashing (the flicker) and re-introduces a race every time two instances render concurrently.

## Consequences

### Benefits
- The field corruption (TR-BIND-002) and the flicker both close: an instance's mapping is immune to another instance of the same source.
- The per-instance plan authority that ADR-0016 already persists becomes the single source of truth for stream reads, slot UI, and layer wiring.
- No persistent-format or schema change; no re-release migration.

### Costs and risks
- **B:** the registry holds one artifact per distinct `(source, plan)` — a small, bounded increase; and the resolve path must reconstruct plan identity from the snapshot when the instance has no live `compiled` (render clones already load the snapshot, so this is a compute, not a new I/O). Risk: the plan-identity hash must be computed identically at insert and at resolve — pinned by a unit test round-tripping a plan through the snapshot.
- **A:** wider surface (struct split, `externals` and slot config threading through resolve and the render path) → higher regression risk on the released-contract render path; more to verify on the host.
- Either way, the M2/M3 batteries and a new copy/duplicate battery must be re-run on the host before this is considered shipped.

## Revisit conditions
- If pipeline/GPU-artifact memory for many same-source instances is measured to matter, revisit A (or add pipeline de-duplication under B keyed by `PipelineKey`, which is already artifact/device-based per ADR-0007).
- If a future feature makes two instances legitimately need to *share* a mutable plan, this ADR is superseded.

## Verification obligations
Before this is marked shipped, `TEST_MATRIX` must carry a harness leg (currently the `NOT_RUN` half of TR-BIND-002), run on the host:
1. Instance **A** with a *migrated* plan: compile with params `p1..p3`, then insert `p0` first in the block and re-commit → A keeps `p1..p3` in F01..F03 and `p0` in F04.
2. Instance **B** fresh (copy/paste or `addProperty` + the same expression) → declaration-order slots.
3. Render both with per-slot **distinguishable** values (floats, both angles, and at least one **layer input** so the `externals` wiring is exercised, not only the float map); assert each instance reads **its own** values and samples **its own** layer.
4. Repeat for **both compile orders** (A then B, B then A).
5. Add the plan-identity round-trip unit test (mechanism-specific) next to the binding tests.
Record `FAIL` on the 0.0.4 artifact first (the defect), then green on the fix build, plus the M2 and M3 batteries on AE 2025 and AE 2026.
