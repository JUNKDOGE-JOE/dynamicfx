# ADR-0028: Details button and float-slider precision

- Status: Accepted; the frozen-declaration-index rationale for the append position is superseded by [ADR-0040](0040-parameter-groups-and-id-identity.md)'s param-id identity ([TR-GRP-001](../TEST_MATRIX.md#tr-grp-001--parameter-stream-matching-across-layout-change-spike) measured id matching) — the button itself and the precision decision stand
- Date: 2026-08-14
- Deciders: user (design specified from first-user feedback) + assistant session

## Context

First 0.0.1 user feedback:

1. Float parameters drag in integer steps. Root cause measured: pool
   `FloatSliderDef` setup never set display precision, and the zeroed PF
   field means integer stepping (the hidden StateToken slider sets
   `Precision::Integer` explicitly — same field, deliberate there).
2. Error messages truncate. The Status row carries its text in the PF
   parameter NAME, which caps at 31 characters (ADR-0015 keeps the code up
   front, so codes never truncate — but long compiler messages do).

The user specified the fix for (2): keep Status short, add a button that
pops a dialog with the complete text.

## Decision

1. **Precision** (display-only, no contract change): pool Float sliders set
   `Precision::Hundredths`.
2. **Details button** (topology append per ADR-0013 §5): a new `Details`
   Button parameter appended AFTER all pool slots — declaration index 110
   (stream 111 behind the implicit input layer). Every 0.0.1 index is
   unchanged; buttons persist no value, so the sequence schema (ADR-0016)
   is untouched. Clicking shows a task-modal info dialog (Win32
   `MessageBoxW`, no new dependency) containing the full status text and
   the diagnostic code. PIPL subversion bumps 4 → 5.
3. Visibility: the button follows the pool-slot policy exposure rules only
   in that it always exists; it stays visible in all states (idle,
   compiled, failed) so the full text is always reachable.

## Consequences

- Old projects open unchanged: AE appends new trailing parameters with
  defaults; the topology contract tests now pin 110 entries with `Details`
  last.
- The Status name may stay terse; every character of a compiler error is
  reachable via one click.
- Dialog code is host-layer only (`host::show_info_dialog`); domain layers
  stay host-agnostic.

## Verification

- Unit: declaration-order contract tests (110 entries, Details last, head
  stream indexes unchanged).
- Host: instance with a failing shader shows the truncated Status; the
  Details click shows the complete message (manual click — modal dialogs
  are not scriptable); m1-m3 suites green on the new artifact, including
  reopening projects saved under the 109-entry topology.
