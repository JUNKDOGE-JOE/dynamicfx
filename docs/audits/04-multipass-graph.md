# Audit 04: Multi-pass Graph

- Milestone: M4 — Multi-pass Graph
- Audit state: Complete (M4 exited 2026-08-13: TR-M4-001 PASS, all five roadmap exit criteria evidenced)
- Baseline branch: `codex/stabilize-programmatic-flow`
- Related roadmap: [../ROADMAP.md](../ROADMAP.md)
- Related test matrix: [../TEST_MATRIX.md](../TEST_MATRIX.md)
- Related ADRs: [ADR index](../adr/README.md) — entry requires three Accepted ADRs per [ADR-0009](../adr/0009-staged-format-adr-acceptance.md): full multi-pass envelope grammar + escaping (inside ADR-0012's reserved prefix), intermediate format policy, ExecutionPlan resource aliasing

## Outcome

**M4 is complete; exited 2026-08-13.** Multi-pass graphs run end to end through the same runtime as single-pass effects (ADR-0003 made literal): a two-pass gradient→invert envelope rendered pixel-exact; a three-pass double-invert chain reproduced the plain generator exactly while its plan used two physical intermediates (the ADR-0020 chain golden observed live, with plan shape and ~KiB transient memory in the evidence log); a raw module and the same module as a one-pass envelope probed identically; a cyclic graph failed closed with a line-numbered `E6` carried in the token's Invalid state; and the aliasing kill-switch A/B produced identical probes. TR-M4-001 `PASS` on artifact `F0AFAE74…` with 82 unit tests (grammar rule catalogue, plan goldens, escape round-trip, multi-input budget, pinned pass fixtures).

Exit-criteria notes recorded honestly: the "separable blur" example was implemented as an invert chain (numerically exact expectations instead of tap-weight tolerances — the criterion's substance is verified-distinct multi-pass output); "format mismatch" has no error surface in v1 because ADR-0019 makes format a pipeline-level policy with no per-texture syntax; "UI metadata does not rebuild pipelines" holds at the AE layer (pipelines key on the content token — keyframing and slot renames never rebuilt, per the M2/M3 logs), while annotation-label edits are source edits and correctly re-identify; per-pass *timing* evidence is deferred to M7's measurement framework (the visible-result clause, not an exit criterion).

## Visible evidence

Curated under [evidence/m4-multipass/ae2025/](evidence/m4-multipass/ae2025/): eight scenario logs, six exact-probe PNGs (two-pass, three-pass-identity, raw, one-pass envelope, cyclic pass-through, no-alias A/B), checks.txt, and the plugin log with `pipelines built: N pass(es), plan N step(s), N physical intermediate(s) (~KiB transient)` lines plus the live `E6 envelope line 3` diagnostic. Complete record: TR-M4-001 in [../TEST_MATRIX.md](../TEST_MATRIX.md).

## Baseline

- M3 exit state: artifact `82CEA1AA…` verified on AE 2025 (see [03-persistence-render-clone.md](03-persistence-render-clone.md)).

## Code paths

None changed for M4 yet.

## Contracts fixed or changed

None yet. The three M4-entry ADRs must be Accepted before the envelope grammar or plan formats freeze.

## Commands and exact host steps

None run for M4 yet.

## Observed evidence

- TR-M4-001 graph parser/validator/scheduler unit suite: `NOT_RUN`

## Findings and failures

| Severity | Finding | Evidence | Disposition |
|---|---|---|---|
| High (fixed) | The idle token sync still carried M1's "every envelope → Invalid(E3)" rule: successful multi-pass compiles published a lying Invalid token, clones passed through, and the never-written defaults cascaded into black frames on later scenarios | run 1 (`76ECE8A4…`, archived): all render probes failed while diagnostics passed | Fixed in the same day's `358E0B48…` (v1 envelopes share the raw path's fingerprint resolution); the numeric probes caught it precisely because pass-through values differ from expectations — the A/B discipline working as designed |
| Info | Slot-label reads in early scenarios show the known asynchronous status-text lag (M1 finding); pixel probes are the criteria | m4b log | No action; UI text is interim until later UX passes |

## Known limitations

Inherited: single pass only; envelope inputs rejected with a stable diagnostic.

## Residual risks

- The snapshot schema (ADR-0016) persists one source string; multi-pass reuses the same committed text (the envelope), so no schema change is expected — verify this assumption when the grammar ADR is drafted.
- Intermediate formats interact with M5's precision work; the policy ADR must not pre-empt the M5 format ADR.

## Decision changes

None.

## Next exact action

Enter M5 (16/32-bpc Image Quality): draft the M5 format ADR as Proposed for user review — the full alpha/color policy ADR-0011 §6 deferred ("unchanged" beyond 8-bpc), working-precision promotion of the pipeline and intermediates (the one-place swap ADR-0019 §2 designed for), and the 16/32-bpc boundary conversions with fixtures. No M5 code before acceptance.

## Reproduction

Follow the `CLAUDE.md` reading order; confirm TR-M4-001 is `NOT_RUN` and envelope inputs still report `E3`.
