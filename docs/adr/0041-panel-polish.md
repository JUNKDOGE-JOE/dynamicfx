# ADR-0041: Panel polish — Setup group, hidden empty groups, live pass names

- Status: Accepted (2026-08-26, explicit user approval with the [TR-0041-001](../TEST_MATRIX.md#tr-0041-001--panel-polish-and-the-final-006-artifact-re-verification) re-verification evidence)
- Date: 2026-08-26
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md)
- Related implementation: `src/host/params.rs`, `src/host/idle.rs`, `src/lib.rs` (slot-UI application)
- Related tests/audits: [TR-0040-001](../TEST_MATRIX.md#tr-0040-001--grouped-topology-host-legs) (the state being polished), re-verification row to be added

## Context

The ADR-0040 grouped panel shipped its host pass with three rough edges: the head controls (`Language`, `Source`, `Compile`, `Status`, `Details`) sit ungrouped above `Main`; the twelve `Pass N` groups are visible even when empty (a single-pass shader shows twelve idle collapsed headers — the §6 accepted fallback, measured 2026-08-26); and group display names are the static `Pass 1`…`Pass 12` although the envelope already names every pass. A user asked for the head controls to be grouped; the project owner folded all three into the 0.0.6 batch before release, accepting the full host re-verification that a topology change forces.

Mechanism facts already measured: wrapping id-stable parameters in topics does not disturb saved projects (TR-GRP-001, TR-0040-001 — twice); topics are flat inert rows to scripting/AEGP; slot renames through `PF_UpdateParamUI` work for v1 kinds; slot hiding through the AEGP DynStream `HIDDEN` flag works; `AEGP_SetStreamName` hangs the host (2026-08-16) and stays forbidden.

## Decision

1. **One `Setup` topic wraps every head control** — `Language`, `Source`, `Compile`, `Status`, `StateToken` (hidden), `Details`, `PlanToken` (hidden) — declared **expanded** (no `START_COLLAPSED`): the Status line is the diagnostics surface and the Compile button is the highest-frequency control; collapsing them by default would hide the instrument panel. New markers `SetupStart`/`SetupEnd` join the golden id table; every head stream-index constant shifts by one and is re-pinned; the harness re-pin sweep runs again.
2. **Empty groups hide.** The idle observer (and the UpdateParamUI path beside it) applies the same DynStream `HIDDEN` flag the slots use to `PassGroupStart(g)` rows whose bank holds no bound slot in the instance's own artifact, and to a gradient sub-group's row when that gradient is unbound; an instance with no artifact hides all twelve. `Main` and `Setup` never hide. If the host refuses the flag on a group stream, the failure is logged once and the group stays visible — the fallback is exactly the state TR-0040-001 verified.
3. **Pass groups display their envelope names.** The same name-application path that renames bound slots renames `PassGroupStart(g)` to the artifact's pass-`g` name (PF's 31-char cap, `Pass N` fallback when unnamed, absent, or the host ignores the rename). Per-instance, like slot labels. `AEGP_SetStreamName` remains forbidden.
4. Items 2 and 3 are presentation behaviors with verified-state fallbacks; item 1 is the only persistent-topology change, safe under the ADR-0040 id contract.

## Alternatives considered

- **Collapsed `Setup` group** — rejected: hides Status/Compile behind a click; the group's value is visual tidiness, not reclaiming rows.
- **Ship 0.0.6 as verified and defer polish to 0.0.7** — offered; the owner chose inclusion, paying one full re-verification of the batch on the final artifact.
- **Per-instance group renaming via AEGP** — forbidden by the measured hang.

## Consequences

### Benefits

- The panel reads as `Setup` → `Main` → only the pass groups that exist, under their real names — the shader's structure with no idle furniture.
- Single-pass shaders lose twelve dead headers; uncompiled instances show a minimal panel.

### Costs and risks

- Every property index below row 1 shifts by one more: harness re-pin again, golden pins rewritten again, and the released-index freeze is re-confirmed dead (id contract carries it).
- The whole 0.0.6 host pass repeats on the new artifact — the previous batch artifact's results ([TR-0039-001](../TEST_MATRIX.md#tr-0039-001--canvas-expansion-host-legs)/[TR-0040-001](../TEST_MATRIX.md#tr-0040-001--grouped-topology-host-legs)) remain in the record but do not gate the release; the re-run does.
- Two host behaviors are unverified until the re-run (group-row hide, group-row rename); both degrade to the verified fallback rather than failing.

## Revisit conditions

- The host refusing DynStream `HIDDEN` or `PF_UpdateParamUI` renames on group rows on any supported year — records the fallback as permanent for that year.
- Field demand for collapsing `Setup` by default or for hiding `Main`.

## Verification obligations

- Unit: golden id table + order pins updated (Setup markers, shifted constants); allocator/canvas tests stay green; the full suite green with zero release-build warnings.
- Host, AE 2025 + 2026, on the FINAL 0.0.6 artifact (fresh hash, installed by the elevated procedure): the complete TR-0039-001 + TR-0040-001 leg set repeated; panel visual — `Setup` expanded on top, empty pass groups hidden (or the logged fallback recorded), a multi-pass envelope showing real pass names; reopen legs 13/13; M2/M3 batteries; measurement spot-check.
- Evidence under a new TR row per the evidence policy; release notes gain the polish items.
