# Native macOS WGSL 0.1.0 host acceptance — bdfc

The tested macOS host subset passed on **After Effects 2026 26.3x87**, native
arm64, on Apple M5 / macOS 26.5.2. This acceptance retains the explicitly
accepted **Source expression single-Undo failure** in
[ADR-0045](../../../../adr/0045-010-source-undo-release-boundary.md).
It is not a claim that every Undo path passes, nor evidence of release upload.

The installed executable is
`bdfc9f1d978cf052e01db97d69b1d61c1ad9cc073b7691a82e2cb2d90baaa141`,
built from native source `8b0fe81f5f6869d1ced67fc3cd1dbf6d366dae36`.
See [build evidence](../build-bdfc/README.md) for the locked build and the
220 passing tests in each default/editor configuration.
[Installation](install-ae2026.json) records arm64, version 0.1.0 and the
ad-hoc signature; this does not claim Developer ID signing or notarization.
The [before](010c-aerender-identity-before.json) and
[after](010c-aerender-identity-after.json) identities match the installed
executable. Curation did not install, rebuild, sign, or operate AE.

## Results and evidence boundaries

| Check | Result and primary records |
| --- | --- |
| Render and host matrix | **558/558** assertions in the original [checker output](010c-check.json), reproduced on the distributed copies in [curated-checks.json](curated-checks.json). GLSL/WGSL rendering at 8/16/32 bpc, multipass, HDR, random-time temporal access, invalid inputs, Layer/Gradient/Path resources, keyframes, and save/reopen are included. |
| Physical downsampling and separate renderer | 12 queued jobs, two frames each: GLSL/WGSL × UV/three-pass double-invert × Full/Half/Quarter. All **24 original PSDs** are retained. The checker verifies every exported pixel, dimensions, opaque alpha, analytic UV and language parity. Physical dimensions are 321×239, 161×120 and 81×60. These 8-bpc exports are the physical downsampling evidence; full-resolution `sampleImage` readings alone would not establish it. |
| Fresh defaults through real Language UI Undo/Redo | [Before](010c-ui-fresh-before.json), [WGSL](010c-ui-fresh-wgsl.json), [Undo](010c-ui-fresh-undo.json), [Redo](010c-ui-fresh-redo.json) retain source, language, status, token, plan, parameter values and pixels. Scalar 0.25, color `[0.25, 0.6, 1]`, alpha 0.4 and angle 12.34567 initialize and restore through one Undo and one Redo. This is the tested float/default case, not a guarantee about untested ranges or transaction paths. |
| Existing gain keyframes through Language Undo/Redo | [GLSL](010c-ui-language-glsl.json), [Undo](010c-ui-language-undo.json), [Redo](010c-ui-language-redo.json) preserve two keys at 0.25/0.75 and restore the matching active/error state and pixels. |
| Actual expression-editor CR source | [Single-pass source](010c-ui-source-green.json) and [two-pass envelope](010c-ui-source-envelope.json) preserve the bare-CR committed text and successful compilation/rendering. Annotation/default and envelope behavior are covered. These are actual UI-pasted sources, not only LF-generated script fixtures. |
| Parameter names | Fresh readbacks contain the exact names `Fresh Gain`, `Fresh Color`, `Fresh Color A` and `Fresh Angle`. The actual UI observation also records the shorter name without the old trailing text. The source read helper accepts `Gain` or the setup name `Float 01`; its numeric result alone does **not** prove the displayed name is exactly `Gain`. |
| GUI Full/Half/Quarter | Six [viewport state records](010c-viewport-glsl-open.json) and the [operator transcription](010c-ui-actions.json) record both languages and all three resolutions. The operator observed the whole canvas without cropping; expected coarser quarter-resolution pixels remain visible. Screenshots were viewed in CUA tool output and are not distributed as local image files. |
| Source single Undo | **Known FAIL, accepted 0.1.0 limitation under ADR-0045.** This candidate does not recertify that path. Prior failure evidence remains in [host-661e-authoring-failure](../host-661e-authoring-failure/README.md). |

The independent [UI record checker](check-ui-records.py) adds **103 passing
offline consistency assertions** in [ui-record-checks.json](ui-record-checks.json).
It checks the retained values, source identity, names where actually returned,
Undo/Redo restoration, CR/envelope behavior, installation identity and gradient
sequence. These are checks of existing records, not 103 additional host runs.
They should not be added to 558 as an independent host-test count.

## Retained gradient initialization failure

The first [assignment](010c-resources-assign.json) failed at JSX line 69:
AE rejected writing a hidden gradient child. The preceding
[None-resource capture](010c-resources-none.json) passed. Assignment had already
changed the GLSL layer/path selectors before the error; the subsequent
[inspection](010c-gradient-inspect.json) retains that partial state and confirms
the gradient colors were still unchanged. No failed operation is relabeled PASS.

The operator then opened the input layer's effect controls for both languages
and expanded Main / Gradient 01 so the dynamic child rows were initialized.
The same assignment body [succeeded after this UI step](010c-resources-assign-after-ui.json),
and the [assigned-resource capture](010c-resources-assigned.json) passed.
This is an automation precondition for the first gradient write; no native
candidate change was made in response. The records do not assert that selectors
were reset to None between the failed attempt and the successful rerun.

## Independent aerender process

The [aerender log](010c-aerender.log) records all 12 completed jobs, from
2026-09-08 14:19:12 to 14:19:13 GMT+8. The command was:

```sh
'/Applications/Adobe After Effects 2026/aerender' -project '<REPO>/scripts/out/010/ae2026/native-wgsl-010-aerender.aep' > scripts/out/010/ae2026/010c-aerender.log 2>&1
```

`<REPO>` replaces the local repository path. The raw log does not contain an
exit code. [operator-process-record.json](operator-process-record.json) records
the root operator's actual tool results: command session `20792`, initial chunk
`1f9aaf`, followed by `write_stdin` chunk `21f92b` with `exit_code: 0` and empty
output. This is a transcription of the terminal tool observation, distinct
from the raw process log. Curation did not rerun aerender.

## Reproduce the offline checks

From the repository root, with Python 3 and no AE process interaction:

```sh
evidence=docs/audits/evidence/wgsl-010-20260908/host-bdfc
PYTHONDONTWRITEBYTECODE=1 python3 scripts/wgsl/check_acceptance.py \
  "$evidence/010c-capture.json" \
  "$evidence/010c-resources-none.json" \
  "$evidence/010c-resources-assigned.json" \
  "$evidence/010c-keyframes.json" \
  "$evidence/010c-reopen.json" \
  "$evidence/010c-reopen-capture.json" \
  "$evidence/010c-reopen-keys.json" \
  "$evidence/010c-reopen-resources.json" \
  --queue "$evidence/010c-queue.json" --exports "$evidence"
PYTHONDONTWRITEBYTECODE=1 python3 "$evidence/check-ui-records.py"
```

Both commands returned exit code 0 during curation. Checker source identities
are recorded in [curation.json](curation.json). The original operator also ran
the first checker against the eight original records in that order, with the
original queue and export directory, and observed exit code 0 (chunk `e8452f`).

## Provenance and privacy

[curation.json](curation.json) maps **110 copied artifacts** to relative source
paths and records both original and distributed SHA-256 values. The 24 PSDs
total 1,343,552 bytes and are byte-for-byte unchanged. Text replaces only the
local repository path with `<REPO>` and the private temporary installation
backup path with `<INSTALL_BACKUP>`. These replacements can affect text hashes;
the original `010c-check.json` hashes therefore refer to original files, while
`curated-checks.json` hashes refer to the distributed copies.

Project files, bridge/request payloads, credentials and local screenshot files
are excluded. The retained JSX contains placeholders and is evidence of the
executed commands; it needs deliberate path substitution before reuse. UI
actions in `010c-ui-actions.json` are explicitly an operator transcription,
not screenshot files or a machine recording of keystrokes. Clipboard/AX
timeouts and the successful observed follow-up are preserved there.

This README, `operator-process-record.json`, `curation.json`, the independent
UI checker and its result, and `curated-checks.json` are generated curation
artifacts. [SHA256SUMS](SHA256SUMS) covers all files in this directory except
itself. The adjacent [frozen package evidence](../package-bdfc/README.md)
establishes packaging and extraction checks separately.
