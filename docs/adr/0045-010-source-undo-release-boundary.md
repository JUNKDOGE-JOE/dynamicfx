# ADR-0045: 0.1.0 source-expression Undo limitation and release acceptance

- Status: Accepted
- Date: 2026-09-08
- Owners: DynamicFX project
- Authorization: the user explicitly chose to release 0.1.0 with the source-expression single-Undo limitation disclosed, then repair it separately.
- Qualifies: the 0.1.0 release acceptance obligations in [ADR-0044](0044-wgsl-and-010-release.md); its production WGSL, host subset, packaging and publication decisions remain in force.
- Related decisions: [ADR-0001](0001-expression-authority-and-open-runtime.md), [ADR-0015](0015-statetoken-and-diagnostics.md), [ADR-0043](0043-apple-silicon-host-protocol.md)
- Related tests/audit: [TR-M3-001](../TEST_MATRIX.md#tr-m3-001--persistence-and-render-clone), [TR-WGSL-002](../TEST_MATRIX.md#tr-wgsl-002--production-wgsl-and-host-integration-010), [TR-REL-010](../TEST_MATRIX.md#tr-rel-010--010-publication), [Audit 08](../audits/08-wgsl-010.md)

## Context

On macOS AE 2026 26.3x87, candidate executable
`661e89af6aa6c9cb595732146b01a42a4dad99fd1d6a19929723622a8f2a7371`
([build identity](../audits/evidence/wgsl-010-20260908/build-661e/README.md))
passed the real GUI Language Undo/Redo procedure: changing WGSL to GLSL
produced E17 and input passthrough, one Cmd+Z restored WGSL and shader output,
and one Redo restored GLSL/E17. The source stayed unchanged; the gain stream
retained two keys, 0.25 at time 0 and 0.75 at time 1. This is a Language and
keyframe result, not evidence that source edits have the same Undo behavior.

A separate manual Source expression edit, still ending in `` `...`;0 ``,
compiled and changed the image. One Cmd+Z did not restore the previous
expression or image, and the host showed Redo disabled. The actual Source
and stream readbacks, plus the `010b-ui-actions.json` operator record, retain
this failure. On this observed path the expression's numeric result remained
zero and AE did not deliver the supervised `UserChangedParam` path; the idle
fallback published internal transport words through undoable stream writes.
Those writes occupied independent history entries. Supervision fixed the
tested Language gesture but did not make this expression edit atomic.

This is consistent with the historical TR-M3-001 finding that an AEGP token
write occupies an Undo entry. That older test needed two Undo presses to
recover its particular invalid-source case; it neither proves single-action
source Undo nor promises that repeated Undo will recover every current case.
ADR-0015's intended publication semantics therefore remain an unfulfilled
target on this source-edit path, rather than a measured PASS.

The same 661e host run found two separate authoring defects: macOS expression
editor CR line endings lost annotation interpretation, and writing a shorter
parameter label left trailing bytes from its former name. These defects are
not covered by the user's accepted Undo limitation.

## Decision

1. For **0.1.0 only**, the observed source-expression single-Undo failure is
   an explicitly accepted release limitation. It does not block publication
   once the remaining current-artifact gates pass. Retain its **FAIL** result
   and evidence; never relabel it PASS or summarize all Undo/Redo as passing.
2. Before publication, the release notes, README and shader authoring skill
   must plainly disclose that a source expression edit may not be restored
   by one Undo and Redo may be unavailable because of internal publication
   writes. Tell authors to retain the previous source and explicitly replace
   the Source expression with that saved source to restore it, then allow
   recompilation. Do not promise an exact count of Undo presses as a remedy.
3. The macOS CR-line-ending and short-name-termination defects must still be
   fixed and verified in real AE on the final artifact. Language Undo/Redo,
   preserved parameter values/keyframes, and the other ADR-0044 host and
   release checks remain required. The 661e subset does not transfer a PASS
   to newly built bytes without recorded verification or an explicit,
   narrowly justified equivalence argument.
4. Repair source-edit Undo in a separate host-publication architecture task.
   This release exception does not change `Language + Source.expression`
   authority, StateToken/PlanToken meaning, sequence schema v1, parameter
   identities, persistence, or the expression transport contract. It neither
   adds nor requires an editor, service or other authoring dependency.
5. Preserve ADR-0044 as an immutable record. This ADR qualifies its release
   acceptance only for the named 0.1.0 source-edit limitation; it does not
   replace ADR-0015's long-term goal of publication that leaves user Undo
   semantics intact. No general exemption for later releases is granted.

## Alternatives considered

- Hold 0.1.0 until source publication is redesigned: the user explicitly
  chose the disclosed limitation and a separate repair instead.
- Treat the passing Language gesture as proof of source Undo: contradicted
  by the separate real GUI expression-edit failure.
- Require an editor or change source authority to avoid the host path:
  outside this decision and the accepted ordinary-property authoring model.

## Consequences

0.1.0 can ship its verified WGSL/macOS work with a concrete authoring
limitation. Authors must keep a recoverable copy of source when editing;
explicit source restoration is the documented recovery route. The project
retains the publication defect as follow-up work, and the current candidate
still needs the two authoring repairs and final host/release verification.
Acceptance of this ADR is neither a published release nor a completed AE
acceptance. Windows/DX12 and other untested host subsets remain NOT_RUN.

## Revisit conditions

Before removing the warning or claiming source Undo/Redo PASS, a separately
reviewed host-publication design must demonstrate that one actual source
edit, one GUI Undo and one GUI Redo restore the corresponding expression,
parameter bindings/values/keyframes and rendered output after idle settles.
Any proposed authority or persistent-format change requires its own ADR.

## Verification obligations

- Preserve the 661e Language/keyframe success and source single-Undo failure
  separately in TR-WGSL-002 and Audit 08, with exact artifact identity,
  readbacks, GUI procedure and the operator-record evidence limitations.
- On the final installed artifact, verify manual macOS CR source editing,
  annotation/default interpretation and complete short labels; retain the
  fixed-source recovery readback and rendered output. Do not replace the
  original failed evidence with the repaired run.
- Verify the remaining ADR-0044 host gates, and show the same known-limitation
  wording in all three author-facing surfaces before packaging/publication.
- Record the source-Undo exception explicitly when evaluating TR-REL-010;
  all other source/tag, executable, package, signature and downloaded-asset
  identity checks still apply. Actual iOS 27 Siri implementation follows
  completed release publication, as already decided in ADR-0044.
