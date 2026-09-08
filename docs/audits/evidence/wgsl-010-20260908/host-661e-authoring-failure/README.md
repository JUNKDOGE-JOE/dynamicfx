# Native AE evidence — retired 661e authoring candidate

**Overall result: FAIL for this candidate.** Actual GUI Language Undo/Redo
and the measured gain-keyframe subset passed. Manual Source authoring exposed
bare-CR annotation parsing and short-name termination defects; both remain
release blockers for these bytes. Source-expression single-Undo also failed.
The user subsequently accepted that specific Source Undo limitation for
0.1.0, with disclosure and a separate repair, under
[ADR-0045](../../../../adr/0045-010-source-undo-release-boundary.md). That
exception does not turn its result into PASS and does not exempt the CR or
name defects. This candidate must not be published.

- Host: native arm64 After Effects **26.3x87**, build 87, macOS
  **26.5.2/64**, read back in [setup](010b-setup.json).
- DynamicFX 0.1.0 executable SHA-256:
  `661e89af6aa6c9cb595732146b01a42a4dad99fd1d6a19929723622a8f2a7371`.
- Source commit: `f4ba578552e02c2a82170d2d116cf0400d8319a6`;
  [build identity and CPU results](../build-661e/README.md).
- UI action transcription recorded at **2026-09-08T05:38:06.133654Z**.
  [010b-ui-actions.json](010b-ui-actions.json) identifies the tested candidate.
  Curation does not attest a currently installed copy or repeat an AE action.

## Passing Language and keyframe subset

The [setup script](010b-setup.jsx) and [exact fixtures](010b-setup.fixtures.json)
created the temporary paired GLSL/WGSL project. The [ready readback](010b-ready.json)
records AE version, Full resolution and the fixture states; merely being
present in that state list is not a complete render acceptance.

[keyframes](010b-keyframes.json) records two keys for each gain fixture:
0.25 at time 0 and 0.75 at time 1. Rendered values are respectively
`[0.25,0,0,1]` and `[0.75,0,0,1]` for both GLSL and WGSL. Its
[script](010b-keyframes.jsx) is retained. Fixed pixel-probe expressions were
[prepared before UI operations](010b-ui-readback-prepare.jsx); subsequent
readback scripts sampled them without writing AE properties.

The actual Language popup was switched from WGSL to GLSL with the Source
expression unchanged. The three readbacks show:

| UI action | Readback | Result |
|---|---|---|
| Select GLSL | [language-glsl](010b-ui-language-glsl.json) | Language=1, E17, input passthrough |
| Exactly one Cmd+Z | [language-undo](010b-ui-language-undo.json) | Language=2, E0, WGSL output `[0.25,0,0,1]` |
| Exactly one Redo | [language-redo](010b-ui-language-redo.json) | Language=1, E17, input passthrough |

All three retain identical Source text and two gain keys, 0.25/0.75. Another
Undo restored WGSL before the separate Source-edit leg. This is a **Language
Undo/Redo PASS**, not a Source Undo PASS or a full 0.1.0 host-matrix PASS.

## CR annotations and short-name termination failures

The operator pasted the green WGSL shader through the actual Source
expression editor. [source-inspect](010b-ui-source-inspect.json) preserves
the resulting expression with **bare CR line endings** and the parameter
name **`gaint 01`**. The source declares `label:"Gain"`, range 0..2 and
default 0.25. The annotated label was not applied: the parser treated the
CR-delimited annotation as part of the first comment line, leaving the
uniform name `gain`; the short-name write also retained old tail bytes.
The parser and name-copy diagnoses were confirmed during the subsequent
source repair, not by a claim that this JSON exposes the internal SDK calls.

The first label-based [readback failed](010b-ui-source-green.json) with
`Named property not ready: Gain`; its [script](010b-ui-source-green.jsx) is
retained as failing evidence. The next readback used the **observed** stable
match name from the inspected row, producing
[source-green-id](010b-ui-source-green-id.json): green `[0,0.25,0,1]`, the
same two keys and `actualParamName: "gaint 01"`. This is an explicit lookup
adjustment, not a silent retry or proof that the label was correct.

These gain streams already had keys. Their retained 0.25/0.75 values do not
prove fresh-default initialization; this leg did not independently read back
all annotation defaults or hints. A new artifact must verify the CR parser
and terminated names in actual AE before those defects can be closed.

## Source single-Undo failure and accepted limitation

After the manual Source edit, **exactly one Cmd+Z** was sent. The
[source-undo-id readback](010b-ui-source-undo-id.json) still contains the
identical CR Source expression, token `5671141488700293`, green output and
two gain keys. The operator's Edit-menu observation records Undo Change
Value, Redo disabled, and history starting with Change Value then Edit
Expression. The actual expression edit did not undo in one action.

This **FAIL** is retained. ADR-0045 accepts it only as a disclosed 0.1.0
limitation, with Source Undo repaired separately. It does not promise that a
fixed number of additional Undo presses will recover the edit. Authors must
retain the previous source and explicitly restore that source when needed.
The [archive result](010b-archive-failed-661e.json) records saving this failed
project locally; the AEP is deliberately not copied here.

`010b-ui-actions.json` is an **operator transcription of CUA actions and AX
observations**, not a saved screenshot, video, or untouched accessibility
dump. References to a screenshot inside that transcription describe the
operator's tool observation; no screenshot file is provided by this archive.

## Curation and integrity

26 necessary records are retained: 15 JSON files and 11 setup/keyframe/UI JSX
scripts. The repeated ready/archive JSX wrappers, bridge responses, request
envelopes, AEP files, unrelated runs and screenshots are excluded. Every
absolute checkout path is replaced with `<REPO>`; shader text, CR/LF escape
sequences, values, diagnostic results and action observations are preserved.
The JSX files are inspectable transcripts and require an explicit local path
substitution before execution.

[curation.json](curation.json) records original and curated SHA-256 hashes
for each file. The replacement was checked to be reversible to the original
bytes, and all retained JSON files parse. [SHA256SUMS](SHA256SUMS) covers all
files in this directory except itself. From this directory:

```sh
shasum -a 256 -c SHA256SUMS
```

The parent manifest and main verification documents are maintained
separately. No executable was rebuilt, installed, tested or published while
creating this historical evidence set. This run does not establish the
replacement candidate's correctness, full depth/resource/reduced-resolution
matrix, save/reopen, independent aerender, Windows or other AE versions.
