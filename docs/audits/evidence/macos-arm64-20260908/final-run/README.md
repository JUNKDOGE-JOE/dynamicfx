# Corrected native artifact — final host run

This directory records the fresh AE 2026 run of signed arm64 binary
`e239ae457ab33e1bc9466d5ab0bff8b81c08861d072eb5e667b5cd8bc6ae8b0b`.
The [installed bundle record](install-ae2026-geometry.json) matches the prepared
artifact and records the previous bundle's backup. Earlier `b3bd67cf…` results
and failures remain in [first-run](../first-run/README.md).

## Recorded results

| Check | Result and evidence |
|---|---|
| Reopen existing six-fixture project, publication/state readback | PASS: [open result](final-open.json), [executed JSX](final-open.jsx) |
| One/three pass, 8/16/32-bpc, HDR, temporal random-order samples, invalid-source passthrough | PASS: [22 checks](final-checks.json), [capture](final-capture.json), [JSX](final-capture.jsx) |
| Keyframes at 0 and 1 second | PASS: values 0.25 and 0.75, two keys: [write/read result](final-keyframes.json) |
| Save/reopen persistence | PASS: [reopen](final-reopen.json), [22 repeated numeric checks](final-reopened-checks.json), [two keyed values retained](final-keys-read.json) |
| Siri Full/Half/Quarter Composition preview | PASS as a [manual observation](manual-observations.md), supported by physical canvas logs; export PNGs are a separate result |
| Native Details dialog | PASS as a [manual observation](manual-observations.md): compiled 1 pass / 12 params / E0, dialog dismissed |
| Fresh independent aerender invocation | PASS: tool completion exit 0 in [run record](run-record.json); [raw log](final-aerender.log) reports all six queue items completed |
| Full/Half/Quarter render output, two scenes | PASS: [39 checks](final-export-checks.json) over all 12 retained PSD files |
| Clean final demo saved | PASS: [save/readback](final-demo-save.json) reports Active token, Full resolution, 32-bpc and local `Siri-Glow-macOS.aep` path |
| Normal cold-open of the saved demo | PASS: [readback](final-cold-open.json) reports AE 26.3x87, the saved demo, Active token, 32-bpc, Full resolution and 1280 × 720 after a normal launch without `--env` |

The fresh analytic source is authored by
[final-queue-inspect-source.jsx](final-queue-inspect-source.jsx), and
[final-queue-source.jsx](final-queue-source.jsx) refuses to enqueue it before
its State Token is Active. Creation/inspection results can show token 0 while
publication is pending; the later queue and demo-save results record the
published state. The status label may still say `Status: idle`; acceptance
uses the State Token and actual renders, as in the existing harness.

The queue requests a 1/25-second span. AE records approximately
0.04000651041667 seconds and emits two frames for each item; both actual frames
are retained and format/alpha-checked. The ramp is checked analytically on
both frames. Siri's four-edge, center, GPU-reference and scale comparisons use
frame 25; frame 26 receives format and alpha checks. This does not claim every
Siri frame has been compared to the GPU reference.

## Physical dimensions and pixel comparisons

| Scene | Full | Half | Quarter |
|---|---|---|---|
| Analytic three-pass ramp | 321 × 239 | 161 × 120 | 81 × 60 |
| Siri glow | 1280 × 720 | 640 × 360 | 320 × 180 |

The six ramp frames have maximum normalized RGB error no greater than
0.001960785 against the analytic UV fixture. The Full Siri frame differs from
the same-time production-GPU float reference by at most 0.001960784. The
mean differences between reduced-resolution Siri and box-reduced Full are
0.00070545 at Half and 0.00090893 at Quarter; the respective 99th percentiles
are 0.00490197 and 0.01593138. These pass the fixture's stated limits, not a
universal antialiasing guarantee. Exact values, tolerances, output hashes and
reference hash are in [final-export-checks.json](final-export-checks.json).

The checker is [scripts/macos/check_exports.py](../../../../../scripts/macos/check_exports.py).
The three `final_siri_*.png` files are conversions of frame-25 PSD outputs by
that checker. They are not Composition-viewer screenshots.

## Log boundary and attribution

[dynamicfx-final.log](dynamicfx-final.log) preserves the full captured suffix
starting with `[1788815341] registered with AEGP at global setup`. This follows
the successful installed-record mtime of 1788815230.526 and coincides with the
final-open JSX/result mtimes at 1788815341.047/1788815341.277. It includes
snapshot reconstruction, the Apple M5 Metal adapter, and actual
1280 × 720 / 640 × 360 / 320 × 180 canvas resolutions.
[Excerpt metadata](dynamicfx-excerpt.json) retains original line numbers,
boundary reasoning, and selected evidence lines; the manifest retains the
complete source snapshot's hash and the suffix hash.

The plugin logger has no PID. Its second setup/snapshot/Metal block at
1788815533 is time-correlated with the separately observed aerender launch
and the aerender log; it does not independently prove process identity.
No verbose `logical=321x239` lines were recorded. The 321 × 239 pipeline line,
render-log dimensions and actual PSD pixel checks support the odd-size result.
The process exit code comes from the root agent's tool completion, not from a
fabricated line in the raw render log; exact command and tool identifiers are
retained in [run-record.json](run-record.json).

The suffix also retains the final normal cold-open at 1788815822, including
snapshot reconstruction, Metal initialization and the compiled shader status.
The [cold-open readback](final-cold-open.json) independently records the saved
demo's Active token and render settings. The root agent reports that this
launch restored ordinary operation without the opt-in verbose environment.

## Retention

[file-manifest.json](file-manifest.json) records copied paths, byte lengths,
SHA-256 and source mtimes, plus the log excerpt's source range. All copied bytes
were rehashed after curation. The JSX/result files are retained without
redundant `.bridge.json` envelopes. AEP projects and plugin binaries remain in
the local workspace; this directory carries their relevant results and
artifact identities. GUI screenshots remain in the conversation's CUA tool
evidence because their surrounding desktop included unrelated content.
