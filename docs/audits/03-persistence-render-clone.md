# Audit 03: Persistence and Render Clone

- Milestone: M3 — Persistence and Render Clone
- Audit state: Complete (M3 exited 2026-08-12: TR-M3-001 PASS, all seven roadmap exit criteria evidenced)
- Baseline branch: `codex/stabilize-programmatic-flow`
- Related roadmap: [../ROADMAP.md](../ROADMAP.md)
- Related test matrix: [../TEST_MATRIX.md](../TEST_MATRIX.md)
- Related ADRs: [ADR index](../adr/README.md) — entry requires three Accepted ADRs per [ADR-0009](../adr/0009-staged-format-adr-acceptance.md): StateToken layout (undo/redo + project-dirty semantics, stable diagnostic-code registry), sequence schema v1 (codec/limits/checksum inside ADR-0012's 8 MiB budget), and hash algorithm/canonical serialization/domain separation

## Outcome

**M3 is complete; exited 2026-08-12.** Projects now survive their own lifecycle: a saved project reopens in a fresh AE process and renders the keyframed shader with no Compile click (snapshot path, pixel-exact); **aerender renders the shader** (closing the TR-M1-004 pass-through limitation) with the render clone performing no AEGP calls; a corrupted snapshot fails closed with `SnapshotCorrupt` and the committed expression recovers automatically; duplicated instances hold fully independent parameter state; a deliberately torn StateToken loses to the checksummed snapshot and is corrected by the observer; the token stream carries real diagnostic codes in its Invalid state (word 70 = E17 observed live); and dirty semantics hold — saving leaves the project clean through continued idle republication. All seven roadmap exit criteria have evidence in TR-M3-001 (artifact `82CEA1AA…`, 68 unit tests, nine exact pixel probes across four AE sessions plus aerender).

Implementation delivered against the entry ADRs: `identity.rs` (BLAKE3 canonical hashing, `dfx:token:v1` golden-pinned), `diagnostics.rs` (the u16 append-only registry, unit-guarded), `persistence.rs` (the `DFXS` schema-v1 codec with CRC-32, every-byte-flip rejection tested), the ADR-0015 token layout with Active/Invalid/Uninitialized/Corrupt decoding, snapshot-seeded slot inheritance on restore, and snapshot-based render-clone resolution.

## Visible evidence

Curated under [evidence/m3-persistence/ae2025/](evidence/m3-persistence/ae2025/): twelve scenario logs, seven exact-probe PNGs (reopen, restored-UI, corrupt-recovery, per-duplicate isolation pair, torn-token, post-undo), the aerender PSD (shader output, exact), three plugin logs (`definition rebuilt from snapshot`, `token/snapshot fingerprint mismatch; snapshot wins`, `Invalid(17)` publication), and checks.txt. Complete result record: TR-M3-001 in [../TEST_MATRIX.md](../TEST_MATRIX.md).

## Baseline

- M2 exit state: artifact `3D11B511…` verified on AE 2025 (see [02-keyframed-params.md](02-keyframed-params.md)).

## Code paths

None changed for M3 yet.

## Contracts fixed or changed

None yet. The three M3-entry ADRs must be Accepted before any persistent byte is written (ADR-0009 rule 2).

## Commands and exact host steps

None run for M3 yet.

## Observed evidence

- TR-M3-001 StateToken/sequence schema v1 round-trip and corruption: `NOT_RUN`

## Findings and failures

| Severity | Finding | Evidence | Disposition |
|---|---|---|---|
| Medium (fixed) | Reopened projects restored the token but not the slot UI: stream renames do not persist, and the idle publication was gated on token *change*, which a reopen never triggers | run 1 (`C91C1A77…`, archived): `slot1=[Float 01]` after reopen | Fixed by a one-read staleness probe (compare one slot's stream name; republish on mismatch); run 3 restores `gain` |
| Low (measured host fact) | `AEGP_SetStreamValue` is always undoable: each real token publication occupies exactly one undo entry, so undoing past a source edit can take an extra Ctrl+Z press (the observer rewrites the token within a tick if only its entry was undone) | m3h3: two presses to traverse; plugin log shows the re-publication cycle | Accepted and documented (ADR-0015 anticipated "where the SDK offers the choice" — here it does not); publication frequency is already minimal (one write per state change). Revisit only if real-project undo UX degrades measurably |
| Info | The corrupted-project first frame already rendered correctly: idle observation recovered the expression before the first `saveFrameToPng` | m3e log/PNG | Recovery is faster than the scenario's pessimistic expectation; m3f remains the criterion |

## Known limitations

Inherited: shader output exists only in the GUI session; stale StateToken values in saved projects are ignored by design until the real layout lands.

## Residual risks

- Undo/redo and project-dirty interaction of programmatic publication is contract-assigned here and still unverified (M0 residual, spike gave indicative-only data).
- The 8 MiB snapshot budget (ADR-0012) constrains the codec design before it is written.

## Decision changes

None.

## Next exact action

Enter the M4 gate (ADR-0009 — decisions first): draft the three M4-entry ADRs as Proposed for user review — the full multi-pass source envelope grammar and escaping (inside ADR-0012's reserved `@dynamicfx` prefix), the intermediate format policy, and the ExecutionPlan resource-aliasing rules. No M4 code before acceptance.

## Reproduction

Follow the `CLAUDE.md` reading order; confirm TR-M3-001 is `NOT_RUN` and `flatten()` still returns zero bytes.
