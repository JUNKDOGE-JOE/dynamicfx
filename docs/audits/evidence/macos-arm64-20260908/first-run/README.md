# First artifact: native AE checks and retained failures

All host data in this directory applies to signed binary **`b3bd67cf…3c480`**,
on After Effects **26.3x87, build 87**, reporting **Macintosh OS 26.5.2/64**.
It predates the later geometry-fix binary `e239ae45…`; passing first-run data
must not be relabelled as final-artifact acceptance.

The scripted first-frame/depth/HDR/temporal/invalid-source measurements passed
their [22 checks](checks.json), with [raw values](capture.json) and
[gradient](capture_gradient.png)/[multi-pass](capture_multi.png) PNGs.
The [keyframe result](keyframes.json) records two keys and values 0.25/0.75.
After reopening, [numeric checks](reopened-checks.json),
[raw capture](reopened-capture.json), and
[keyframe readback](reopened-keys.json) preserve the separate reopen evidence.
The GLSL fixtures, actual JSX and publication-state results are retained beside
these outputs.

Two harness failures remain visible:

- **Initial AppleScript call: outcome unverified.** The exact submitted
  [JSX](setup.applescript-unverified.jsx) and
  [AppleScript result](setup.apple.txt) are retained. The latter records
  `exit=-15` with empty stdout/stderr after the waiting call was interrupted;
  it is not a successful execution and does not establish why the call waited.
  Subsequent `setup.json` and host results were obtained through the separate
  bridge transport. No JSON/API cause is asserted from the waiting call alone.
- **First aerender output naming: `FAIL`.** The first
  [aerender log](aerender.log) records the missing temporary output error when
  the Photoshop sequence target omitted a frame placeholder. The zero-byte
  [`aerender.psd00000`](aerender.psd00000) is preserved. The later
  [corrected log](aerender-fixed.log) uses `aerender_fixed_[#####].psd` and
  writes two PSD frames; it does not erase that failure.

A further **fresh-project aerender** leg used a separately created 321×239
fixture. Its [setup](fresh-setup.json), [queue](fresh-queue.json),
[aerender log](aerender-fresh.log),
[numeric checks](aerender-fresh-checks.json), and actual `fresh_00000.psd` /
`fresh_00001.psd` outputs are retained. The AEPs stay in the local raw-output
directory and are deliberately not copied here.

Siri demonstration frames and sample readbacks are retained as well. The
original artifact subsequently exposed a **Half-preview geometry defect**:
Full rendered the complete frame; Half rendered only its upper-left portion.
The saved [Full PNG](siri_full_purged.png) is 1280×720 and the
[Half PNG](siri_half.png) is 640×360. These are exported frame files; the live
viewport observations and final-fix comparison belong to the parent audit.
The export helper does not independently verify the live preview resolution.
Its 32-bpc PNGs were also about 0.1 times the actual sampleImage values: at
t=1, (640,26), sampleImage and the same-source Metal runner both gave
(0.783520877,0.539768040,0.794666767), while the helper PNG encoded roughly
(0.0783,0.0540,0.0795). The 8-bpc export siri_8bpc.png matches the expected
quantized RGB (200,138,203). No compensating shader gain was added.
This export behavior is distinct from the observed live Half viewport crop.
The latter triggered the physical-dimension correction in the second artifact.

[`file-manifest.json`](file-manifest.json) records every original source path,
byte count and SHA-256. Raw files were copied byte-for-byte. Bridge-response
duplicates, AEPs and plug-in binaries are omitted. The retained first-binary
ZIP location and hash are recorded in
[`../build-identities.json`](../build-identities.json).
