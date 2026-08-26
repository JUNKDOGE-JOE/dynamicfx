# Host-pass evidence — the 0.0.6 batch on AE 2025 + 2026 (2026-08-26)

> **Round structure.** This directory holds three rounds against three
> artifacts, because the batch grew and a harness fault was chased mid-pass:
> **R1** = ADR-0039+0040 on `8A472BFE…` (the sections below, all green) —
> superseded when ADR-0041 joined the batch; **R2** = +ADR-0041 on
> `54B5F0AF…` (`*-r2-*` files: index map with `Setup`@1/426 rows, reopen
> 13/13, canvas PASS, live pass names `warm`/`cool`, empty-group hiding
> photographed) — its M2 battery exposed the stale-literal harness fault
> below, and a misdiagnosis briefly produced a third build (`3E732238…`,
> never fully verified, superseded same hour after a live experiment proved
> the plugin healthy); **R3 = the release-gating round** on
> **`9E438A6444394EA8…` (8,613,888 B)**, source-identical to R2's plugin:
> every leg repeated green on BOTH years (`*-r3-*` files) — reopen 13/13 ×2,
> canvas 5 legs ×2 (equivalences again 0.000/0), compile/allocator/pass-name
> legs ×2, M2 **12/12** ×2, M3 **4/4 + aerender** ×2, measurements
> (`measurements-final-*`: addProperty ≈8–9 ms, RTT deltas in noise,
> one-instance `.aep` ≈290 KB). R3 additions to the incident list: the m2h2 +
> m2b/m2d harness literals had gone stale across the two topology shifts
> (masked once by a re-pin gap) and now address bound slots **by label**;
> scripted `setValue` on a hidden UNBOUND slot fails by AE design (the
> "parent hidden" message misread as a plugin fault — the misdiagnosis
> above); and on AE 2026 the 2025-saved `baseline.aep` raises a **version
> conversion modal** on open, which blocks the panel bridge until dismissed —
> the reopen leg there uses a targeted light probe
> (`grp-reopen-r3-ae2026*`) instead of the 426-row full dump.

One combined host pass over the ADR-0039 (canvas expansion) + ADR-0040
(per-pass groups, id identity) batch build, run after the unit-level slices
were accepted. Matrix rows:
[TR-0039-001](../../../TEST_MATRIX.md#tr-0039-001--canvas-expansion-host-legs) and
[TR-0040-001](../../../TEST_MATRIX.md#tr-0040-001--grouped-topology-host-legs).

## Artifact and environment

- Batch build `dynamicfx.dll` → `DynamicFx.aex`, **8,589,824 B**, SHA-256
  `8A472BFE3D51C7418AFF227137B929F656AC0CA8FDBBEB72F5E24207533925E0`, built
  once from the working tree (`main` at `8da89d8` + the accepted slices 1–3,
  uncommitted; `cargo test` 166 passed, release build warning-free) and never
  rebuilt during the pass.
- Installed by user-elevated copy into both
  `…\Adobe After Effects 2025\Support Files\Plug-ins\DynamicFx\` and the 2026
  equivalent (0.0.5 `FF1197D9…` backed up first, hashes logged before/after);
  **restored to 0.0.5 `FF1197D9…` on both years at the end, re-verified**.
- Windows 11 Pro 10.0.26200; AE 2025 (25.6.6x4) and AE 2026 (26.3x87);
  ae-mcp dev panel 0.10.2 `/exec` channel; 8-bpc comps unless stated.
- The machine was handed over by the concurrent ae-mcp gate session
  (after-effects-mcp-09) before the pass began and was not shared during it.

## Legs and results — every leg on BOTH years unless noted

### Property-index map (live)

A fresh instance dumps 424 top-level rows (423 declared + AE's 合成选项):
heads 1–5, `Details` 6, `Plan Token` 7, `Main` 8 (topics are flat inert rows
to scripting), `Float 01` 9 … `Float 48` 56, `Int 01` 57, `Bool 01` 65,
`Color 01` 81, `Point 01` 93, `Angle 01` 105, `Layer 01` 113,
`Point 3D 01` 117, `Mask 01` 125, `Gradient 01` group 127 / anchor 128 /
`Stops` 129 / first stop 130, `Main` end 183, `Pass 1` 184 with `P01 Float
01` 185, last row 423. Matches the slice-3 table exactly; the 26 shifted
harness JSX sites were re-pinned (`scripts/grp/repin_indexes.py apply`) and
every battery below ran on the re-pinned harnesses.

### ADR-0040 reopen legs — PASS

The spike's 0.0.5-saved [`baseline.aep`](../spike-20260826-param-group-matching/baseline.aep)
opened on the grouped build: **13/13 probes KEPT** on AE 2025
([`grp-reopen-ae2025.json`](grp-reopen-ae2025.json) /
[`-verdict.txt`](grp-reopen-ae2025-verdict.txt)) and **13/13 KEPT** on AE 2026
([`grp-reopen-ae2026.json`](grp-reopen-ae2026.json) /
[`-verdict.txt`](grp-reopen-ae2026-verdict.txt)) — values, the 2-keyframe
stream (1.25/2.5 exact) and the expression all followed their id-derived
matchNames into the new positions. The ADR-0040 obligation to repeat the
id-matching check on AE 2026 is met.

### ADR-0039 canvas legs — PASS, pixel-exact

Scene per [`scripts/canvas/tr_0039.py`](../../../../scripts/canvas/tr_0039.py):
1024×1024 comps, a 256×256 white solid centered, the `reach-ring` halo shader
(reach in logical px), references rendered on padded precomps. AE 2025
results in [`canvas-ae2025/report.txt`](canvas-ae2025/report.txt) (PNGs
beside it), AE 2026 in [`canvas-ae2026/report.txt`](canvas-ae2026/report.txt);
the two years' numbers are identical:

| Leg | Result |
|---|---|
| L1 undeclared + plain layer | bbox exactly the layer rect (384..640) — released clipping preserved |
| L2 undeclared + Red Giant GrowBounds 256 | halo reaches 240..784; **mean abs diff 0.000, peak 0 vs the 256-padded precomp — bit-identical** (TR-BOUNDS-001's no-op tile B becomes the positive test) |
| L3 declared `hint:canvas` 160 | **bit-identical to its padded reference** (0.000 / 0) |
| L4 declared 64 under GrowBounds 256 | bbox 326..698 (reach 58 ≤ 64) — the author's boundary crops the upstream expansion |
| L5 keyframed reach 0→200 | t=0 exactly clipped; t=1 reach 180 ≤ 200 — the canvas animates |

### ADR-0040 compile/allocator legs — PASS

- `apple-thermal.glsl` (10 passes, 22 params): compiles ("compiled: 10
  passes, 22 …", no spill suffix) and renders
  ([`grp-compile-appleth-ae2025.png`](grp-compile-appleth-ae2025.png) /
  [`…ae2026.png`](grp-compile-appleth-ae2026.png), dumps beside them). All 22
  params bind in `Main` — correct per ADR-0040 §3: the shader declares the
  same full uniform block in every pass, so every parameter is multi-pass.
- A 2-pass demo whose passes declare only their own uniforms
  ([`grp-allocator-demo-ae2025.txt`](grp-allocator-demo-ae2025.txt) /
  [`…ae2026.txt`](grp-allocator-demo-ae2026.txt)): `shared_gain` → `Main`
  row 9, `warm_tint` → `Pass 1` row 185, `cool_tint` → `Pass 2` row 205,
  labels applied inside the pass groups. Both years identical.
- Panel visual (screenshot during the pass, AE 2025): `Main` expanded with
  labeled bound rows, `Gradient 01/02` nested groups, `Pass 1`–`Pass 12`
  collapsed. Empty pass groups are **visible** on single-pass shaders — the
  accepted ADR-0040 §6 fallback (group-header hiding was not implemented in
  this batch); recorded as cosmetics, candidate follow-up.

### M2 + M3 batteries on the batch build — PASS both years

Runner-started AE per the recorded discipline (a first attempt riding a
pre-started AE timed out exactly as the memory says; outputs cleared and
re-run cleanly): M2 GUI scenarios a–d and e–j plus checks — **12/12 numeric
probes PASS** per year (`scripts/out/m2/<year>/`); M3 three GUI sessions +
aerender leg + checks — **4/4 PASS** per year, aerender exit 0
(`scripts/out/m3/<year>/`). Zero TIMEOUT/FAIL in the accepted runs.

### Scale measurements ([`measurements-*.txt`](measurements-batch-ae2025.txt))

| Metric | 0.0.5 baseline (AE 2025) | batch AE 2025 | batch AE 2026 |
|---|---|---|---|
| `addProperty` (PARAMS_SETUP proxy), median of 3 | 5 ms | 9 ms | 8 ms |
| exec round-trip delta, 3 instances vs 0 | +9.8 ms (noisy) | +3.2 ms | +2.4 ms |
| one-fresh-instance `.aep` | 170,213 B | 287,737 B | 294,311 B |

Reading: PARAMS_SETUP grows ~4 ms for +246 declared rows; idle-walk deltas
are inside ambient noise; the real cost is **≈ +117 KB of project file per
instance** (the 246 extra parameter streams) — recorded as the accepted price
of the pass banks.

## Incidents (kept per the evidence policy)

1. The spike directory's raw `after_spike.json`/`verdict.txt` were
   overwritten by a driver re-run early in this pass (hardcoded output
   paths); details and the transcribed preservation in the
   [spike README's record-integrity note](../spike-20260826-param-group-matching/README.md);
   the driver now refuses to overwrite existing evidence files.
2. A `TaskStop` on a battery-runner command killed a booting AE 2026
   mid-start; the next boot raised AE's crash-recovery dialog plus a script
   error, which blocked one M2 attempt (its outputs were discarded and the
   battery re-run cleanly). AE 2026 later crashed once while wedged in a
   quit sequence (empty untitled project; no evidence affected). Lesson
   recorded in the project memory: quit AE before stopping a runner.
3. `hint:canvas` legs surfaced three harness-side bugs (fixed in
   `tr_0039.py` during the pass: `addSolid` arity, a guarded effect-registry
   scan, `moveTo` reference invalidation) and one driver hygiene fix
   (`close` before `newProject`). None touched the plugin.

## Reproduction

Install the batch artifact on the target year (elevated, backup first), then:

```
python scripts/grp/repin_indexes.py verify
python scripts/canvas/tr_0039.py all --out <evidence>/canvas-<year>
python scripts/grp/tr_grp_001.py phase2   # only against a fresh evidence dir
pwsh scripts/m2/run_m2.ps1 -Year <year>   # then -Scenarios e,f,g,h,h2,i,j,q; then -Checks
pwsh scripts/m3/run_m3.ps1 -Year <year>   # then -Aerender; then -Checks
```

AE must be closed before each battery (the runners start it themselves), and
the installed artifact hash must be verified before attributing results.
