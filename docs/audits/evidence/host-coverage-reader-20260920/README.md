# Automatic native reader diagnostic

The original-alpha acquisition is now automatic in the guarded diagnostic:
opting in creates an invisible, shy, locked reader and binds it without user
layer selection. Recorded lifecycle and native-depth frame tests pass. This is
not yet the production shader input or a completed liquid-glass feature.

## Baselines

Public `922845c` plus the diagnostic. AE 2026 26.5x89 on Windows. Production
DynamicFx.aex remains `c91db8c0...`; its source is unchanged. The [0.0.25 install](raw/native-reader-20260920/install-v025.json)
identifies the 10-case lifecycle and broad frame matrix. [0.0.26 source](source/lib.rs)
adds the opt-in preset export experiment and corrects a stale initialization
banner; default-build frame/ownership logic is unchanged from 0.0.25.
Its three native-depth smoke cases pass after process restart.

[0.0.28 installation](raw/native-reader-20260920/install-v028.json) and
[source](source-v028/reader.rs) identify the owner-deletion correction. The
[test log](raw/native-reader-20260920/v028-test.log) records 22 local passes.
The old hard-coded `probe=0.0.17` initialization text is not an artifact identity;
installation hashes and the version-specific operations identify those runs.

All operations used MCP and native callbacks, with hidden AE launches. A
sandbox-launched AE process stalled before bridge initialization; it was
terminated by its checked PID after all fixtures had been saved, then normal
hidden startup restored MCP. No mouse or foreground automation was used.

## Lifecycle

[Ten cases](raw/native-reader-20260920/lifecycle-summary.json) cover creation,
native float32 alpha, sharing, opt-out, cleanup, Undo/Redo, re-enable, duplicate
owners and renaming. Recognition uses layer IDs, an internal role and an owned
source marker. Only `HS_reader_` compositions are automatically maintained in
this diagnostic. That fixture guard is not a production naming requirement.

The first [0.0.24 cleanup](raw/native-reader-20260920/v024-failed/lifecycle-summary.json)
left an unused solid item. Resolving the source item handle and then comparing
item IDs fixes it; the [corrected cleanup](raw/native-reader-20260920/disable-last.native.log.gz)
records one reference and removes both owned objects. Other references are
checked before removal. The foreign-consumer preservation branch still needs
a dedicated host test.

[Owner deletion](raw/native-reader-20260920/orphan-summary.json) adds three
0.0.28 checks. The [failed 0.0.27 attempt](raw/native-reader-20260920/v027-failed/orphan-summary.json)
shows AE restoring the deleted-layer selector to MYSELF. Cleanup now excludes
that owned internal reference and requires previously observed ownership with
the original owner absent. It preserves unknown or modified objects and does
not immediately repeat cleanup after Undo. Undoing the original deletion
restores the prior owner/reader pair. Orphans first encountered without prior
ownership history are intentionally preserved in this diagnostic.
The [post-Undo 0.0.28 frame](raw/native-reader-20260920/v028-frame-summary.json)
also matches all 480000 native float32 words in the stroke/merge/repeater case.

## Native pixel tests

[Frame matrix](raw/native-reader-20260920/frame-summary.json): 30 pairs, each
compared as 480000 native alpha words on an 800x600 canvas. 8/16-bit values are
compared as integers; 32-bit values as float bit patterns. All differences are
zero. The cases are:

- animation sampled cold in order 1, 0, 0.5 seconds, with CTI fixed at 1.6;
- nonidentity 2D anchor/position/scale/rotation and 3D orientation;
- three transformed repeater copies with varying opacity;
- merge subtraction using an ellipse, followed by the repeater;
- 12.5-pixel, 77%-opacity stroke with the preceding operators;
- a three-depth repeat on the 0.0.26 default build after restart.

The initial repeater defaults to one copy and zero offset. Those three records
are identity controls, not evidence of a nontrivial modifier. The subsequent
configured operators change 75065, 6820 and 47804 reference pixels respectively
at 32 bpc. Nonzero alpha and changed content are checked so two empty or
unchanged outputs cannot falsely accept a feature.

Run `python spike/host-outline/verify-reader.py
docs/audits/evidence/host-coverage-reader-20260920` to independently recheck the
compressed logs. This compares pixels before production GPU/resource binding;
it does not prove the final glass shader, final compositing, ROI or MFR.

## Host-version decision

AE 2025.6.6x4 [refuses StreamSuite7](raw/reader-compat-20260920/stage-2025.native.log.gz)
with SDK error `kSPSuiteNotFoundError` (`S!Fd`). The
[ordinary preset experiment](raw/reader-compat-20260920/apply-param-preset-2025.json)
overwrites unrelated parameter values. A reduced-topology build creates an
AE-authored two-parameter seed, but [AE 2025 rejects that newer-host preset](raw/reader-compat-20260920/apply-minimal-seed-2025.json).
No binary preset patching or private API workaround was adopted. The background
export uses the host's `Layer.savePreset(File)` method, also reported in this
[scripting-guide issue](https://github.com/docsforadobe/after-effects-scripting-guide/issues/43);
the actual run, not that report, is the evidence here.

The user subsequently approved **AE 26.5+ for this new feature**. Existing
features keep their previous compatibility; feature/depth/3D/mask/modifier/FFX
scope is unchanged. The temporary AE 2025 diagnostic was
[removed](raw/reader-compat-20260920/restored-2025.json) and its production plugin
was preserved. The reduced `preset-seed` build is not a shipping configuration
and must never open an existing full-diagnostic authoring project.

## Remaining work

Define the opt-in shader resource, alpha channel and coordinate contract, then
integrate the verified acquisition path into the production runtime. Validate
readiness before the first render after copy/FFX, foreign references and modified
helpers, cropped/expanded canvases, ROI/downsampling/PAR, native empty coverage,
FFX and aerender, and actual MFR concurrency. Layer construction and stage writes
must remain on the main thread; render callbacks only consume declared inputs.

The full delivery gate remains open. No new commit, push, merge or release.
The [manifest](manifest.json) preserves original/normalized/stored hashes;
local path prefixes and MCP artifact IDs are redacted. SDK archives/headers,
AEP/AEX/FFX binaries and user assets are excluded from the published record.
