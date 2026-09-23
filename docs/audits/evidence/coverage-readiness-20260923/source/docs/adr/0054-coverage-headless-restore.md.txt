# ADR-0054: Preserve prepared coverage in headless render hosts

- Status: Accepted
- Date: 2026-09-23
- Extends [ADR-0053](0053-coverage-certificate-revocation.md); supersedes the
  read-only-flag-only classification in [ADR-0052](0052-coverage-instance-readiness.md)
  decision 5. Transport bytes and UI copy revocation are unchanged.

## Evidence and decision

Independent AE 26.5 aerender first sends normal resetup with the project flag
off, then creates read-only render copies. Candidate `152fb695...` consequently
revokes a valid saved certificate and produces E60/pass-through. The renderer
exits successfully but differs at 87500 alpha words; exit status is not proof
of a correct frame.

The SDK's PF App suite exposes `PF_IsRenderEngine`, documented to include
render-engine, no-UI and watch-folder operation. During SequenceResetup, accept
a provisional, checksummed saved certificate when either the read-only project
flag or this official host-mode query is true. A query failure defaults to the
conservative UI classification. No process-name heuristic or private API is used.

Interactive UI resetup still revokes the old certificate process-wide. Fresh
instances and saved certificate 0 remain unauthorized in every host mode; the
query cannot mint a certificate or bypass owner validation. It is used only
during sequence setup, never as render-side AEGP or an idle wait.

Repeat independent prepared-project rendering, immediate UI copy/FFX rejection
and post-idle recovery. Save-before-idle and the wider offline/MFR matrix remain
explicit acceptance cases rather than inferred support.
