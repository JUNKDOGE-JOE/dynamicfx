# Managed coverage carrier follow-up

The user accepted an automatically managed mode-None mask visible in the mask
list. Production coverage is still **not implemented**. This follow-up records
failed partial-alpha optimizations, the first native ownership attempt, and a
diagnostic memory-handle defect encountered during that attempt.

## Baseline

- Local source checkpoint: `ff89564`, branch `codex/fix-project-open-lock`.
- Windows; AE 2026 26.3x87; AE-MCP 0.10.7; disposable 800x600 fixture, normally
  16 bpc. The alpha-value probe also switches to 8 and 32 bpc, then restores 16.
- Production source is unchanged. Production AEX remains `c91db8c0...`.
- Diagnostic 0.0.9 `f0bd26c4...` is currently installed. Earlier 0.0.6: `795b7c6f...`; 0.0.7: `fc98a3c4...`; 0.0.8: `90248eeb...`.
  Full identities and destinations are in the copied installation records.
- Exact commands, MCP argument/result pairs and logs are frozen beside this
  audit by `freeze-managed-evidence.py`; binaries remain in the ignored run
  directory at the identities above.

## Observed failures

The first partial-alpha optimization derives an upper bound from shape fill
and stroke opacity. A nominal 50% fill does not produce exactly 0.5 in this
fixture: the mask-path readback of its sampled alpha is 0.501953125 at 16 bpc.
The bound-based sampler still exceeds 32768 queries. The 8/16/32-bpc probe
records eight opacity settings. Its values are transported through mask
vertices, so those numbers also include mask-coordinate quantization; they
are not a full-precision characterization of AE's rasterizer.

Increasing the exact partial-alpha sampling budget to 262144 queries causes
an `ae_exec` timeout after 30 seconds. A later host-status query reports the
bridge recovered, and readback finds the expression enabled with no expression
error. The expression is then disabled. Its exact completion time and native
pixel comparison are unknown; this is not a successful performance result.

Diagnostic 0.0.6 adds parameter 7, `Manage coverage carrier`, and a guarded
main-thread idle manager. `HS_auto_base` (comp 174, layer 177) contains an
ordinary mode-None user mask with four vertices and one opted-in diagnostic.
The first scan returns `Struct`; no carrier is created. Diagnostic 0.0.7
attempts to interpret that failure as an empty expression, but is also wrong:
the user reports recurring tracked-memory-ID dialogs, `23::33`, which cannot
be dismissed permanently. The idle callback keeps revisiting the bad wrapper
path. An idle-control request first times out and later completes after modal
interaction; the first timeout remains failed/uncertain evidence.

Source inspection finds that the wrapper's `expression_string` constructs a
`MemHandle` without checking an absent expression's null allocation, then tries
to lock and free it. Treating the returned error as empty does not prevent the
invalid cleanup or its dialog. The recurring diagnostic process is stopped by
its verified PID/start time after the user reports the loop. Only the disposable
test project is involved; the original fixture checkpoint is preserved.

## Repair and current verification

Diagnostic 0.0.8 reads the expression through the raw SDK with a null check
before memory operations, bounds UTF-16 reads, and balances lock/unlock/free
only for an actual allocation. A failed layer operation is quarantined until
an explicit disable/re-enable. Diagnostic host errors are captured quietly and
logged with their operation; this is not a production error contract.

- `PASS`: 13 local tests and a locked offline build, including the regression
  that an absent expression never reaches memory callbacks.
- `PASS`: installation hash checks preserve the production AEX.
- `PASS`: native ordinary-mask scanning no longer fails on its empty expression.
  Diagnostic 0.0.8 logs a separate `rename Parameter` error once and stops
  retrying. Renaming a mask through this stream is rejected by this host.
- AE 0.0.8 initially has no MCP listener because a default crash-recovery prompt
  is waiting. The user clears it; readback then confirms an empty, clean project
  before reopening the fixture. The connection refusal dispatches no host action.

Diagnostic 0.0.9 leaves the generated mask's default AE name and identifies it
by its internal expression. It passes 13 tests and native automatic creation,
a user-renamed carrier, two owners sharing one mask, duplicated-comp isolation,
last-owner cleanup and re-enable. The ordinary user mask remains unchanged.
Undo removes only the generated carrier, and idle does not recreate it. The
first Redo assertion fails because the driver uses command 17. After checking
the [Adobe SDK discussion](https://community.adobe.com/questions-529/force-an-ae-undo-redo-from-a-plugin-70731),
command 2035 restores the carrier and passes all state checks. The incorrect
attempt is a harness failure, not evidence of broken product Redo.

Two native frames, before and after project reopen/cache purge, return 3648
coverage vertices through PF PathQuery. Comparison with independent reference
alpha covers 480000 pixels: 49 differ, maximum error 1/255. Output RGBA is
byte-identical between the frames. After purge the driver first fails its
10-second PNG wait even though MCP has returned successfully; the PNG and
native log arrive later. This original request is recovered without another
render. It proves acquisition here, not acceptable cold-frame latency or deep
floating-point alpha fidelity.

FFX recreation, aerender/MFR, production integration, arbitrary partial opacity,
original masks and other hosts remain unrun or unresolved. The 20-item fixture
is saved and remains open with all 12 diagnostic logging/serial controls reset
and the expensive opacity expression disabled.

The user also authorized autonomous AE closure/startup without mouse control.
All project actions used MCP; starts used Hidden window style. Two exact owned
AE PIDs were stopped: one after MCP quit left an empty, clean project but a live
process; the other to end the reported recurring modal loop. The complete
close/restart chain is retained. No mouse, keyboard, or window-activation call
was used; OS focus was not measured.

The issue-body follow-up was rejected by automatic approval review because the
full existing body included internal diagnostic information and export
authorization was insufficient. No alternate publication route was attempted.
The approval of the managed mask is recorded locally in ADR-0049 and status.

## Next action

Resolve bounded partial-alpha acquisition and original-mask coverage before
production integration. Basic managed ownership is now demonstrated; FFX
recreation, full cold-render timing and other host acceptance still require
independent evidence. Do not integrate the production ABI from the opaque
triangle result alone.
