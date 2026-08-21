# TR-BIND-002 harness evidence — two instances of one source, different BindingPlans (ADR-0038)

Harness: [`scripts/bind/tr_bind_002.py`](../../../../scripts/bind/tr_bind_002.py), driven through the ae-mcp panel `/exec` channel of a warm After Effects session (`scripts/bind/aemcp.py`; the token is read from the user's `~/.ae-mcp/auth-token` and never recorded).

What one run does (per compile order, on fresh source text so the fingerprints differ between orders):

1. New 8-bpc project; comp `bind` 320×240; four hidden full-frame solids `red`/`green`/`blue`/`yellow` (layer inputs) and two grey solids `A`/`B` carrying the instances.
2. **A (migrated plan):** `addProperty("DynamicFx")`, commit **v1** (`p1 p2 p3` floats, `a1` angle, `texB` layer), wait for the token, then commit **v2** which inserts `p0`, `a0`, `texA` *in front* → A keeps `p1..p3` in `Float 01..03`, `p0` lands in `Float 04`; `a1` stays `Angle 01`, `a0` takes `Angle 02`; `texB` stays `Layer 01`, `texA` takes `Layer 02`.
3. **B (fresh plan):** `addProperty` + the same v2 text → declaration order (`p0..p3` → `Float 01..04`, `a0 a1` → `Angle 01..02`, `texA texB` → `Layer 01..02`).
4. Each instance gets its own values through its own slot table: A `p0..p3 = 0.1 0.2 0.3 0.4`, `a0 a1 = 36° 72°`, `texA=red texB=green`; B `0.5 0.6 0.7 0.8`, `180° 252°`, `texA=blue texB=yellow`.
5. After three seconds of idle ticks the slot names of both instances are read back, then each instance is rendered alone twice (`app.purge(ALL_CACHES)` + `saveFrameToPng(0.5)`). The shader paints four quadrants: TL `(p0,p1,p2)`, TR `(p3, a0/360, a1/360)`, BL `texA`, BR `texB` — so one PNG proves every float, both angles and both layer inputs resolved through that instance's own plan. Tolerance 3/255.
6. Both compile orders: A then B, B then A. `--reopen` adds a save/close/reopen pass in the same session (no Compile pressed) and an `aerender` pass (fresh process, cold registry; `bind` renders A alone, its duplicate `bind2` renders B alone).

## Pre-fix build — `FAIL` (defect reproduced)

- Date/time: 2026-08-21 18:19:46 local. OS Windows 11 Pro 10.0.26200. Host After Effects 2026 **26.3x87**.
- Installed artifact: `DynamicFx.aex` SHA-256 `24E963FB19E735252A5D21CFBBF48864A597D38A7A38D461F6B3B9A34F3D22F2` (the TR-CACHE-001 fix build from `cfccd5d`, installed at `…\Adobe After Effects 2026\Support Files\Plug-ins\DynamicFx\`). Its registry code is byte-for-byte the released 0.0.4's: the only change between `BFE1AB9F…` (0.0.4) and this build is the `Command::SmartRender` checkout arm (`cfccd5d`), which this leg does not exercise — so this is the 0.0.4 defect.
- Command: `python scripts/bind/tr_bind_002.py` (working tree `6386e87` + uncommitted harness). Report and the eight PNGs: [`prefix-24E963FB/`](prefix-24E963FB/).
- Observed — **whichever instance compiles last owns the shared entry; the other is read through the wrong plan**:
  - Order **A then B**: B's fresh compile evicted A's entry. A's slot names were renamed to B's labels by the idle observer (`Float 01..04` became `p0 p1 p2 p3`, `Angle 01/02` became `a0/a1`), and A rendered `TL=(51,76,102)` instead of `(26,51,76)` (its streams read one slot off), `TR=(25,51,25)` instead of `(102,26,51)` (angles swapped into the float), and **BL/BR swapped** (`green`/`red` instead of `red`/`green`) — the layer-input wiring followed the foreign plan too. B was correct.
  - Order **B then A**: symmetric — A's migrated compile evicted B's entry; B's slider names were renamed to A's table (`p1 p2 p3 p0 / a1 a0`), and B rendered `TL=(204,127,153)` instead of `(128,153,178)`, `TR=(178,178,127)` instead of `(204,128,178)`, BL/BR swapped (`yellow`/`blue`). A was correct.
  - Log deltas per order: `resolved from process registry` +2, `idle slot ui applied` +4, `pipelines built` +2; no `missed registry` lines.
- Verdict line: `[VERDICT] FAIL — see MISMATCH / CHANGED lines above`.

### Pre-fix build — reopen and `aerender` legs (`FAIL`)

`python scripts/bind/tr_bind_002.py --orders AB --reopen`, 2026-08-21 18:30 local, same host/artifact. Report, PNGs and the `aerender` PSDs: [`prefix-24E963FB/reopen-aerender/`](prefix-24E963FB/reopen-aerender/).

- **Same-session reopen** (save → close → `app.open`, no Compile): A stayed corrupted — its sliders still carried B's labels and it rendered through B's table (`TL=(51,76,102)`, BL/BR swapped); B correct. Log: `resolved from process registry` +2, `rebuilt from snapshot` +0.
- **`aerender`** (fresh process, cold registry; comp `bind` = A alone, duplicate comp `bind2` = B alone, output module fell back to the `Photoshop` template): the first clone to render rebuilt from **its** snapshot (`rebuilt from snapshot` +1, A's migrated plan) and **B then hit that entry by fingerprint** (`resolved from process registry` +1) — B rendered through A's table (`TL=(204,127,153)` for `(128,153,178)`, layers swapped yellow/blue). A correct. The cold-registry path has the same defect as the warm one: the second instance adopts the first's plan.

## Fix build, run 1 — `6E4E80A6…` (registry keyed per plan, snapshot-carried identity only): partial

- 2026-08-21 19:49 local, AE 2026 26.3x87, Windows 11 Pro 10.0.26200; artifact `dynamicfx.dll` 8,562,176 B, SHA-256 `6E4E80A689E99D57D0A8B32591BCF3F5FBA278152FF337F907C92B4602B61499`, installed on AE 2026 and AE 2025 (hashes verified after install). `cargo test` 147 passed on the same tree. Report, PNGs, PSDs: [`fix-6E4E80A6-run1/`](fix-6E4E80A6-run1/).
- **Fixed:** slot names stayed correct for both instances in both compile orders (the idle observer no longer applies a foreign plan; `without this instance's artifact` 0). **Same-session reopen: PASS** for names and all eight quadrant probes (`resolved from process registry` +2 — the reopened clones resolved their own entries through their snapshots). **`aerender`: PASS** (`rebuilt from snapshot` +2 — each clone rebuilt from its **own** snapshot instead of the second adopting the first's entry as on the pre-fix build).
- **Still failing in the warm session:** in every warm leg the instance that compiled **first** rendered through the other's plan (order A→B: A wrong; B→A: B wrong; reopen-src: A wrong), and the new log line named the cause — `definition resolved by latest entry for source; clone carries no plan` twice per leg (`resolved from process registry` +0). The render clones of a freshly added instance carry **no snapshot**: After Effects keeps serving the flattened copy it took at `addProperty` time, the compile happens in an idle observation AE does not see as a sequence-data change, and the already-declared `SUPPORTS_GET_FLATTENED_SEQUENCE_DATA` did not make it ask again. A snapshot-carried plan identity therefore reaches clones only after save/reopen or in a fresh process — exactly the two legs that passed. Outcome: ADR-0038 §7, the `PlanToken` stream (the plan identity transported beside the StateToken, written by the UI callback and the idle mirror, read first by the resolver).

## Fix build, run 2 — `FF1197D9…` (plan-token transport added): `PASS`

- 2026-08-21 19:58 local, AE 2026 26.3x87, Windows 11 Pro 10.0.26200; artifact `dynamicfx.dll` 8,564,736 B, SHA-256 `FF1197D9DB09CCE81D8E2CE26AC4FD3FA752FEA8705109FBCA8D9725670A0344`, installed on AE 2026 and AE 2025 (hashes verified after the elevated copy). Working tree: `6386e87` + the uncommitted ADR-0038 implementation (`cargo test` 147 passed, no warnings). Command: `python scripts/bind/tr_bind_002.py --reopen`. Report, PNGs, PSDs: [`fix-FF1197D9-run2/`](fix-FF1197D9-run2/).
- **Every assertion held**: slot names matched each instance's own plan in both compile orders and after reopen; all 32 warm-leg quadrant probes (A and B, two rounds, three legs) plus the 8 reopen probes and the 8 `aerender` probes read each instance's **own** floats, angles and **layer inputs** (tolerance 3/255); the strict log lines (`resolved by latest entry`, `adopting latest entry`, `not this plan`, `missed registry`) stayed at zero in every leg.
- Log deltas per warm leg: `resolved from process registry` +2 (each instance's clone resolved its own entry through the transported plan word — the line run 1 could not produce), `idle slot ui applied` +3, `pipelines built` +2, `rebuilt from snapshot` +0. Reopen: `resolved from process registry` +2. `aerender`: `rebuilt from snapshot` +2 (cold registry; each clone rebuilt its own snapshot), `resolved from process registry` +0.
- Verdict line: `[VERDICT] PASS — every instance resolved its own values and layer wiring in every leg`.

## Fix build on AE 2025 — `FF1197D9…`: `PASS`

- 2026-08-21 20:19 local, AE 2025 25.6.6x4, same machine; same artifact installed on AE 2025 (hash verified); `python scripts/bind/tr_bind_002.py --reopen --year 2025`. Report, PNGs, PSDs: [`fix-FF1197D9-ae2025/`](fix-FF1197D9-ae2025/).
- Same outcome as on AE 2026: 0 mismatches, slot names stable in both orders and after reopen, strict counters at zero in every leg; reopen `resolved from process registry` +2, `aerender` `rebuilt from snapshot` +2.

## M2/M3 batteries on the fix build

Both full batteries ran on the fix build on both years right after the harness ([TR-0038-001](../../../TEST_MATRIX.md#tr-0038-001--m2m3-batteries-on-the-adr-0038-fix-build)): AE 2026 20:07–20:12, AE 2025 20:13–20:19; every step exit 0, every numeric probe `exit=0`, no timeouts; `m3h4`/`m3h5` report the project clean after save, so the new hidden `PlanToken` stream does not dirty projects. Curated runner output: [`battery-2026.log`](battery-2026.log), [`battery-2025.log`](battery-2025.log). The runners must start After Effects themselves (they pass their output folder through `DFX_M3_OUT`/`DFX_M2_OUT`); a first attempt with AE already open timed out on `m3a` and was restarted with AE closed.
