# Host coverage production integration

## Outcome

Current main `07a79e31...` / reader `0a1f7c4c...` pass 114 native pairs and 15
ROI comparisons: animation/expressions, transforms/parents, modifiers,
downsampling/PAR and expanded source. Empty output controls pass. The base
78 pairs were repeated after fixing the ordinary-shader GUID assertion and
outside-comp alpha clipping. [Fidelity evidence](evidence/coverage-full-20260923/README.md).
Full acceptance still needs helper tamper/foreign references, actual concurrent
MFR and remaining release gates. Shader authoring follows complete regression.

### Readiness baseline

The subsequent readiness correction now protects immediate UI copies and FFX
application, including warm-cache copy, and preserves prepared independent
rendering. Final artifact `96dcff77...` passes post-idle pixel recovery, new and
existing FFX targets, independent aerender, missing-reader refusal and the
6-case 8/16/32-bpc regression. Local suites pass 249 each. See
[readiness evidence](evidence/coverage-readiness-20260923/README.md) and
TR-COVERAGE-READINESS-004. This completes the current readiness gate, not the
remaining full production/glass-material delivery gate.

### First-frame baseline

The first production coverage frame is now verified on AE 2026 26.5x89. The
main-thread manager creates and binds an invisible native reader automatically;
the formal shader graph consumes its alpha. Ordinary and adjustment shapes
match an ordinary AE reference in all 6 8/16/32-bpc comparisons, each 480000
native alpha words with zero differences. Eight initial ownership checks and a
same-process save/reopen frame also pass. This remains an acceptance build,
not a complete liquid-glass delivery; copy/FFX-before-idle readiness is next.
[Production evidence](evidence/coverage-owner-20260923/README.md).

## Resource and adapter foundations (preceding the first host frame)

The production source now recognizes `hint:coverage`, allocates its own hidden
Layer slot, persists the new slot kind, declares SmartFX input dependencies and
encodes native alpha onto the render canvas. Missing readiness/input is E60;
missing StreamSuite7 capability is E59. Automatic production ownership has not
yet been connected, so the default state deliberately remains pending. This is
not an enabled liquid-glass feature and no candidate has been installed.

The SDK-header stage adapter is now also in the production build. It can read
and atomically set the layer/stage pair, then verifies both halves. Host resource
cleanup errors remain errors; every owned stream/value/suite is released on
failure. Its Rust guard is main-thread-only and not Send/Sync. The manager has
not yet been connected to this API. A build without the SDK bridge now reports
coverage unsupported instead of merely trusting the host's suite availability.

## Baseline and contracts

Public `922845c` plus the working-tree integration, on Windows with Rust 1.97.1.
[ADR-0050](../adr/0050-host-coverage-resource.md) fixes the resource contract.
Existing physical parameter indexes and IDs remain unchanged; Coverage and its
state are appended. Manual Layer capacity remains four. Sequence schema stays
v1; its kind registry appends Coverage at tag 10. Source.expression remains the
shader authority. This feature's host minimum is AE 26.5+ only.

[ADR-0051](../adr/0051-native-coverage-reader-component.md) adds the separate
`DynamicFx Coverage Reader` utility with a single original-layer selector.
It copies native pixels without color conversion; no alternative shader
authority or special source-text mode was introduced. Its artifact is installed
beside the main plugin. The owner uses ID/stage/marker/structure checks, shares
one reader per original, and preserves unknown or conflicting objects.

## Implementation

- Shared annotations/graph validation and GLSL/WGSL resource lowering recognize
  `hint:coverage`. It is not a uniform member and consumes no uniform words.
- The separate capacity-one pool keeps manual layer selections independent.
  The parameter is kept hidden in both PF and AEGP presentation paths.
- SmartPreRender/SmartRender reuse the declared external-checkout accounting.
  Coverage carries its buffer origin separately from ordinary layer inputs.
- [Canvas encoding](../../src/coverage.rs) pads with zero and clips by paired
  coordinates. Native U15 alpha becomes exact float32 using 32768; native
  float32 words are replicated to RGBA without arithmetic or clamping.
- Main-thread suite availability updates unsupported-host state. It acquires
  and releases StreamSuite7 without guessing or dereferencing a table layout.
- Render uses the transported state and pixel input only. Pending state,
  absent inputs or malformed coverage encoding use explicit diagnostics and
  pass-through; background alpha is never used as coverage.

## Verification and remaining gates

Commands and results are recorded in TR-COVERAGE-INTEGRATION-001 in the
[test matrix](../TEST_MATRIX.md). Tests cover both frontends, aliases, read-only
graph restrictions, pool overflow, persistence, unchanged existing topology,
pixel origin/stride/padding, every U15 alpha value, exact float32 words and
balanced suite availability checks. Release build is compilation evidence only.
Default and editor suites each pass 237 tests. The uninstalled Windows Release
candidate is `8f9fe9fa6889b1d290e47483d545e13aef1f97ad650491c2f1cc6e7ee141e286`.
[Frozen evidence](evidence/coverage-integration-20260923/manifest.json) records
the production source and actual outputs; earlier diagnostic tests are separate.

Remaining: migrate automatic native ownership into the production host adapter,
implement its helper frame path and copy/FFX-before-idle readiness, then perform
production AE pixel, ROI/downsample/PAR, FFX, aerender and MFR acceptance. The
earlier diagnostic results are not production acceptance. No push or release.

The subsequent adapter overlay passes 239 default tests with SDK, 239 editor
tests with SDK, 239 default tests without SDK, and 22 actual-C++ callback fault
tests. SDK-enabled Release builds as `b9e4db45...`; it is uninstalled and retains
unused-API warnings until ownership is wired. See TR-COVERAGE-STAGE-002 and
[adapter evidence](evidence/coverage-stage-20260923/README.md). These checks
verify the adapter's contract and cleanup, not an AE production frame.

## Completed coverage and material acceptance

TR-COVERAGE-COMPLETE-007 closes the agreed Windows AE 26.5 feature gate and
delivers Liquid Glass source, FFX, application script and a clean local example.
The final installed main is `6d188d22...`, reader `29695b72...`; its independent
formal render is pixel-identical to the accepted image. Temporary diagnostic
plugins are backed up outside the host. No main merge or release is included.
[Completion evidence](evidence/coverage-completion-20260923/README.md).

The final record includes 174 native pairs, 288 repeated serial/MFR pairs,
real cancellation with 105 exact cache/purged comparisons, existing-resource
checks, helper mutation/foreign-reference tests, and actual material outputs.
The narrower preceding next-action lists below are historical stages, not
outstanding delivery work. The next action is review of the development branch
and local candidate; integration/release requires its own decision.

## Preceding delivery work

Current lifecycle/MFR overlay closes two additional implementation gaps:
static reader transforms/timing are validated before granting ownership, and
ADR-0057 removes exclusive Rust references to shared host allocations from the
effect dispatcher. Eighteen helper mutations and nine lifecycle cases pass.
The final main `a17d714b...` / reader `d251a360...` pass 288 exact native-frame
comparisons between independent serial and concurrent MFR runs (9–12 overlapping
callbacks). Initial invalid helper readiness and interleaved trace evidence are
retained. See TR-COVERAGE-LIFECYCLE-MFR-006. Current-byte geometry/FFX/resource
regression is now active before the requested material work.

1. The initial manager and native helper path are connected. Extend ownership
   validation to changed helper transforms/timing, unknown/foreign usage and
   original deletion, and verify each case on the production artifact.
2. Copy/FFX readiness and prepared single-frame offline rendering are now
   verified. Keep their regression cases active while extending the remaining
   ownership and render matrix; do not infer genuine concurrent MFR from one frame.
3. Verify production coverage and liquid-glass output at 8/16/32 bpc, animated
   masks/holes/modifiers, 2D/3D transforms, Full/Half/Quarter, PAR, ROI and expanded
   origins. Earlier diagnostic pixel matches are a reference, not a substitute.
4. Complete copy/rename/Undo/Redo/FFX/save/reopen, cold shuffled frames,
   aerender and real MFR acceptance; preserve ordinary Layer/Gradient/Path and
   GLSL/WGSL behavior. Package a usable glass preset and its controls, then scan
   the exact outgoing branch payload. Full acceptance remains the push gate.

## Production host verification

TR-COVERAGE-OWNER-003 records the installed main `e72ac937...` and reader
`da16133d...`. Main default/editor/no-SDK suites pass 241 tests each; reader
tests pass 3, and both Release builds pass. There are no unused stage-API
warnings in the SDK-enabled production build now that ownership calls it.
Initial compile failures and the recovered setup timeout remain recorded.

The saved disposable project contains two automatically created hidden readers
and an ordinary reference. Native readback is taken after the formal shader's
coverage-to-alpha output. The independent verifier compares the exact U8/U15/
float32 words after placing each cropped world at its recorded origin. It also
checks nonzero partial-alpha interiors, masked holes and transparent exteriors.
Eight lifecycle cases pass after idle; the same-process reopened adjustment
frame also matches all 480000 float32 words before an explicit idle wait.
Disabling DynamicFx produces opaque background alpha at all 140000 checked-out
positions, unlike the reference. This negative control rules out a silent
pass-through result masquerading as successful coverage; the effect is restored
and the project saved after the control.

The test build remains installed in AE 2026, with the former production AEX
backed up in the ignored installation evidence directory. The baseline is
restored and saved; lifecycle mutations have their own saved project copy.
There was no mouse/foreground operation, commit, push, main update or release.

## Next exact action

Run the full production coordinate/fidelity matrix against native references:
animated masks/modifiers and 2D/3D transforms at 8/16/32 bpc, followed by
Full/Half/Quarter, ROI/PAR and expanded/empty sources. Fix any mismatch before
preview-speed work, then complete ownership tamper and concurrent MFR acceptance.

## Readiness implementation and failure trail

ADR-0052 adds an instance certificate and checksummed outer AE version-2
transport around the unchanged DFXS v1 map/source. Non-coverage flatten bytes
remain version 1. Hidden CoverageState carries the exact certificate; state 1
is no longer authorization. The manager gets the instance's Arc through the
existing observation reply, revokes before rebind and publishes after validation.

The first native fix still read the old outline because AE creates render
copies from original bytes. ADR-0053 therefore revokes the certificate across
the process. Original and duplicate are revalidated separately. Cache records
for owners observed absent are removed, preventing reused layer IDs from being
mistaken for a manual reader removal. The actual resetup flag logs, initial
42331-word failure and delayed-recovery failure are retained.

ADR-0054 uses the official PF_IsRenderEngine query during resetup, since
aerender first sends normal resetup too. ADR-0055 mixes effective readiness into
AE's frame GUID only for coverage graphs. The effect cache generation advances
to default 15/editor 16 so old rendered output cannot validate a new binary.
No render callback calls AEGP or waits for idle.

On the final build, prepared independent rendering matches the reference;
missing-reader rendering explicitly remains E60/pass-through even though
aerender exits 0. Saving a fresh copy can itself run owner validation, so that
case is recorded as correct save-time validation, not an E60 negative control.
The baseline project is restored for further tests. All AEX replacements have
backups; no commit, push or main update was performed.
