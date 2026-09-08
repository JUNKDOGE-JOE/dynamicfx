# Native AE evidence — retired 91c8 candidate

**Overall result: FAIL for the 0.1.0 release gate.** The recorded numerical
battery passed **217/217 checks**, but one real GUI Undo did not undo a
Language change and Redo was disabled afterward. This candidate must not be
published. These are historical observations of the candidate before the
SUPERVISE/idle publication repair, not evidence for a later executable.

- Host readback: After Effects **26.3x87**, build 87; macOS **26.5.2/64**.
- Candidate: native arm64 DynamicFX 0.1.0, executable SHA-256
  `91c8afdcdc6186ca2efa24ca1c06cbcbb619a659b62e77fbe69fe988f9323a22`.
- Build identity: [the recorded candidate build](../build/build-record.json).
  This curation does not rebuild, install, or independently attest a currently
  installed copy. The UI action record identifies the tested executable.
- UI action transcription recorded at **2026-09-08T05:11:04.966046Z**.

## What passed

[010-initial-checks.json](010-initial-checks.json) is the original offline
checker result: 217 checks, zero failed. The five input records are setup,
capture, corrected resources-none, corrected resources-assigned and keyframes.
They cover all 18 fixture instances, default Language=GLSL, production
GLSL/WGSL sampling at 8/16/32 bpc, true three-pass parity, HDR/clamping,
random-time temporal samples, invalid-source diagnostics/pass-through,
Layer/Gradient/Path none and assigned states, and two rendered gain keyframes.

The capture used **Full** (`resolutionSetting: [1,1]`). This result does not
establish physical Half/Quarter output, a cold reopen, independent aerender,
MFR, Windows/DX12, or successful GUI Undo/Redo. Saving the temporary project
inside a scenario is not a cold-reopen test. No AEP or image capture is
included in this subset.

[curated-checks.json](curated-checks.json) is a newly rerun **offline** check
against the path-redacted copies. It also passes 217/217 and records their
curated hashes. No AE action was repeated to produce it. From repository root:

```sh
python3 scripts/wgsl/check_acceptance.py \
  docs/audits/evidence/wgsl-010-20260908/host-91c8-undo-failure/010-setup.json \
  docs/audits/evidence/wgsl-010-20260908/host-91c8-undo-failure/010-capture.json \
  docs/audits/evidence/wgsl-010-20260908/host-91c8-undo-failure/010-resources-none-fixed.json \
  docs/audits/evidence/wgsl-010-20260908/host-91c8-undo-failure/010-resources-assigned-fixed.json \
  docs/audits/evidence/wgsl-010-20260908/host-91c8-undo-failure/010-keyframes.json
```

## Initial resource failures and recovery

Both [resources-none](010-resources-none.json) and
[resources-assign](010-resources-assign.json) initially failed with
`Named property not ready: Ramp Stops`; their exact scenario JSX is retained.
The assignment failure occurred after the GLSL Layer/Path selectors had
changed. It was not a side-effect-free failure.

[010-restore-none.json](010-restore-none.json) records the partial state:
GLSL selectors were Layer=3, Path=1 while WGSL remained 0/0. The explicit
recovery restored both languages to 0/0. The corrected scenarios accept
either the shader label or the known setup label of the fixture's one
gradient, reject ambiguous matches, and do not guess a numeric row index.
The `*-fixed.jsx`/JSON records preserve that change and subsequent successful
none/assigned measurements. Later success does not erase these first failures.

## GUI Undo failure

The starting WGSL gain rendered `[0.25,0,0,1]`. Fixed pixel-probe expressions
were prepared before the UI operation. The operator then selected **GLSL**
using the actual Language menu, leaving the WGSL source unchanged. The
[read-only result](010-ui-language-glsl.json) reports Language=1, StateToken=70
(`Invalid(E17)`), input pass-through, and unchanged gain keys 0.25/0.75.

After **exactly one Cmd+Z**, the [next read-only result](010-ui-undo-failure.json)
still reports GLSL/E17/pass-through. The
[CUA action record](010-ui-undo-actions.json) transcribes the observed Edit
menu: Redo disabled; history began with two Change Value entries and twelve
Pass group Change Name entries. These facts are consistent with background
publication contaminating Undo; the JSON samples alone do not identify the
particular SDK calls responsible.

`010-ui-undo-actions.json` is explicitly a **CUA action/accessibility
observation transcription**, not a raw screenshot, video, or untouched AX
dump. The readback scripts between the switch and Undo did not change AE
properties. Their full wrappers and the earlier probe preparation are
included so that boundary is inspectable.

## Curation and integrity

Only necessary `010-*.json` and `010-*.jsx` records were copied from the local
run. Bridge responses, request envelopes, AEP files, unrelated preinstall
state, screenshots and private machine logs are excluded. Every occurrence
of the absolute repository directory was replaced with `<REPO>`; numbers,
shader contents, diagnostics and observations were preserved. Curated JSX
is an inspectable transcript and needs an explicit local path substitution
before use as a script.

[curation.json](curation.json) records original and curated SHA-256 values
for all 33 copied files and identifies which files changed only by path
redaction. The original checker intentionally retains the original input
hashes; compare those with `original_sha256`, not the redacted bytes.
`curated-checks.json` instead hashes the distributed copies. The local
[SHA256SUMS](SHA256SUMS) covers every file in this directory except itself.
The parent evidence manifest is deliberately unchanged by this curation.
