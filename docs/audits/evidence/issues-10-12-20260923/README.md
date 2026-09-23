# Issues #10–#12 evidence — 2026-09-23

Baseline 112a6c7 plus parameter fixes. Final main and reader identities are in
artifact-verification.json and coverage-verification.json. Three library suites
pass 257 tests each. AE 26.5x89 numeric cases, display-unit readbacks, keys/reopen
and 27 unchanged glass frames are recorded separately. The initial source-only
scripted UI unit readback stays visible; the normal supervised callback then
clears it correctly. Foreground ownership changed during one viewer-selection
check and was reported; later tests omit that operation.

Four local project-copy opens did not reproduce #12. The source file has changed
since the historical report; its current hash is preserved. User confirms the
actual report is on another machine with no more details. No #12 fix, merge or
release is claimed. Private production AEP/source and intermediate PSD/f32
files remain local. Local home paths in text are normalized to <user-home>.
The collector records versions/hashes only, with no project or account data.
Harness paths require adjustment on another checkout.
