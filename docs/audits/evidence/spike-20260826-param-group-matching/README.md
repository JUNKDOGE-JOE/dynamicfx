# Spike evidence — AE re-matches effect parameter streams by param id (2026-08-26)

**Question ([TR-GRP-001](../../../TEST_MATRIX.md#tr-grp-001--parameter-stream-matching-across-layout-change-spike)):**
when a plug-in's declared parameter layout changes between the save and the
load of a project, does After Effects re-attach the saved streams (values,
keyframes, expressions) by **declaration index** or by **`PF_ParamDef.uu.id`**?

Everything this repository froze about panel order (ADR-0013 §5 append-only
growth, ADR-0028's "Details is index 109" rationale) assumed index matching.
That assumption had never been measured. It decides whether parameter
grouping — wrapping the pools in AE topics, which inserts GROUP_START/END
rows and shifts every later index — breaks released projects.

**Answer: `uu.id` matching. 13/13 probes survived an index shift intact.**

## Mechanism, visible before the swap

The `after-effects` crate (0.4.0) has always stamped every declared parameter
with `uu.id = murmur3(format!("{ParamKey:?}"))` (`Parameters::param_id`). The
baseline dump shows AE builds each effect parameter's **stream matchName from
that id** — `Float 01` is `DynamicFx-447494404`, ids appear in decimal, sign
included (`DynamicFx--805715378`). Projects persist streams by matchName, so
id-stable parameters keep their streams wherever they sit in the declaration.

## Procedure

1. **phase1** — AE 2025 running the released 0.0.5 artifact (`FF1197D9…`):
   scripted project `grp` (100×100 comp, solids `target`+`other`), DynamicFx
   applied, 11 distinctive values + 2 keyframes (`Float 02`: 1.25@0s, 2.5@1s)
   + 1 expression (`Float 03` = `'0.123'`) set across every parameter kind;
   saved as `baseline.aep`; full recursive property dump → `baseline.json`.
2. AE quit; the installed AEX swapped (elevated copy, user-executed; hashes
   logged) for the **spike build** `5C6C3B4D9D81A13725BAA15C54920A22D8DCBD7998850C95147C1A25255CCA06`
   (8,574,976 B) — built at repo commit `8da89d8` + [`spike.patch`](spike.patch)
   (worktree `AePlugin_Dynamicfx-spike-grp`, toolchain 1.97.1, the carried
   `Cargo.lock`): one topic **"Floats"** wrapped around the Float pool, i.e. a
   GROUP_START inserted at released declaration position 5 and a GROUP_END
   after Float 48 — floats shift +1, everything after them +2. Every
   pre-existing `ParamKey` Debug string (hence every id) unchanged.
3. **phase2** — AE 2025 relaunched on the spike build: `baseline.aep` opened,
   same dump → `after_spike.json`, closed without saving.
4. **verdict** — reproduced in full below (the raw file was later lost; see the record-integrity note).
5. The released 0.0.5 AEX restored from the hash-verified backup.

Driver: [`scripts/grp/tr_grp_001.py`](../../../../scripts/grp/tr_grp_001.py)
(phases as subcommands, ae-mcp panel `/exec` channel, port 11488).

## Result

Every probe kept its value **on the parameter of the same name and matchName**
while its flat property index shifted exactly as declared:

| Probe | Value | Flat path 0.0.5 → spike | matchName |
|---|---|---|---|
| Float 01 | 11.5 | 6 → 7 | `DynamicFx-447494404` (unchanged) |
| Float 48 | −3.25 | 53 → 54 | unchanged |
| Int 01 | 7 | 54 → 56 | unchanged |
| Bool 01 | on | 62 → 64 | unchanged |
| Color 01 | (0.1, 0.2, 0.3) | 78 → 80 | unchanged |
| Point 01 | (30, 40) | 90 → 92 | unchanged |
| Angle 01 | 45° | 102 → 104 | unchanged |
| Layer 01 | layer 2 | 111 → 113 | unchanged |
| Point 3D 01 | (10, 20, 30) | 117 → 119 | unchanged |
| Gradient 01 Stops | 5 | 127 → 129 | unchanged |
| G01 Stop 01 Pos | 0.42 | 128 → 130 | unchanged |
| Float 02 | 2 keyframes, 1.25/2.5 read back exact | 7 → 8 | unchanged |
| Float 03 | expression `'0.123'` intact | 8 → 9 | unchanged |

`[VERDICT] ID_MATCH — every probe survived the index shift; AE re-matched
streams by parameter id.`

Secondary finding: **scripting/AEGP see the topic flat.** The group start is
an inert `NO_VALUE` row (`"Floats"`, `DynamicFx--733460086`, propertyValueType
6412) at top-level path 6; the floats stay top-level siblings (paths 7…), not
children. Top-level row count 178 → 180 (+2 markers; the trailing nested
group both dumps share is AE's own 合成选项/Compositing Options). So
`stream_index_of`'s flat position arithmetic and the idle observer's flat
walk stay valid under grouping — only the numbers move, pinned by tests.

## Environment

- Windows 11 Pro 10.0.26200; **After Effects 2025 (25.6.6x4)**, launched and
  quit per phase; AE 2026 untouched throughout.
- Baseline plug-in: released 0.0.5 `DynamicFx.aex` (8,564,736 B) SHA-256
  `FF1197D9DB09CCE81D8E2CE26AC4FD3FA752FEA8705109FBCA8D9725670A0344` at
  `C:\Program Files\Adobe\Adobe After Effects 2025\Support Files\Plug-ins\DynamicFx\`
  (pre-swap hash logged, backed up, restored after phase2).
- Spike plug-in: `5C6C3B4D…` as above; swap/restore hashes in the session's
  swap log (scratchpad) and reproduced here.
- Repo `main` at `8da89d8` plus this session's documentation edits; the spike
  code itself never entered `main` (worktree diff only, [`spike.patch`](spike.patch)).

## Files

- [`baseline.aep`](baseline.aep) — the 0.0.5-saved probe project (AE 2025).
- [`baseline.json`](baseline.json) — the full recursive baseline dump (name,
  matchName, type, value, keys, expression per row).
- [`spike.patch`](spike.patch) — the exact spike-build diff over `8da89d8`.

**Record-integrity note (2026-08-26, same day, later session):** the raw
`after_spike.json` and `verdict.txt` were accidentally overwritten before
this directory was ever committed — the 0.0.6 host pass re-ran the driver's
`phase2`/`verdict` (whose output paths were hardcoded here) against the
grouped batch build, and the spike-build dumps are not recoverable. The
tables and the verdict line above transcribe the lost files verbatim (they
were written from those files the hour they existed); the 13-probe result
stands on them plus `baseline.json`. The driver now refuses to overwrite an
existing evidence file, and the host-pass rerun's own dumps live in
[`../hostpass-20260826-006/`](../hostpass-20260826-006/) under their own
names.

## Reproduction

```
python scripts/grp/tr_grp_001.py phase1   # AE 2025 + 0.0.5 running
python scripts/grp/tr_grp_001.py quit
# swap the installed AEX for the spike build (elevated), relaunch AE 2025
python scripts/grp/tr_grp_001.py phase2
python scripts/grp/tr_grp_001.py verdict
python scripts/grp/tr_grp_001.py quit
# restore the released AEX from the hash-verified backup
```
