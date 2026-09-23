# Production fidelity regression and expanded coverage repair

Current main `07a79e31c1b881fa48d0b7ed8ae621ddaea74e1ba45a5ac28fb9083452eec1ba`,
reader `0a1f7c4c11d501ecd148fcfb2a62688ff31e5c1009130ae437640e8a9d2c75cf`, and
diagnostic 0.0.29 `ce965bf8...` pass the recorded fidelity subset on Windows
AE 2026 26.5x89. This is **not the complete release gate**. Helper tamper/foreign
references, real concurrent MFR and remaining compatibility/material acceptance
remain open. The user requests liquid-glass shader authoring only after complete
regression; no material completion or publication is claimed.

## Passed comparisons

- 78 native pairs: animated curves/fill opacity at shuffled times 1, 0, 0.5,
  0.36 with CTI 1.6; 2D/3D transforms; transformed three-copy repeater, merged
  hole and stroke; Half/Quarter and PAR 1.333333. Ordinary and adjustment layers
  at every depth. The whole matrix was rerun on current artifacts, under the
  `final-` filename prefix. Modifier changes affect real reference pixels.
- 24 native pairs: negative group scale, rotation/skew, transformed parent,
  expression-driven curved paths at 1 and 0.24 seconds.
- 12 expanded/unshifted pairs: actual sampler access to the original's alpha
  outside comp bounds. Translating sampling by 128 pixels brings that exterior
  into view; 3548 U8 or 3554 U15/F32 reference pixels prove the fixture tests it.
- 15 sampleImage ROI comparisons against the independent ordinary reference.
  Footprint averaging differs from a single full-frame texel at an antialiased
  edge; the two independently rendered sampleImage results agree exactly.
- Empty ordinary layers remain transparent; empty adjustment layers match the
  disabled-layer background. A hard-coded blue=255 PNG expectation failed at
  32 bpc, where the host exported blue=25. The same-host control has zero RGBA
  differences; the original failed assumption is retained, not a plugin defect.

The initial 39 ordinary/reference PNG alpha comparisons match as an additional
screen-space check. Half/Quarter outputs are actually 400×300 and 200×150.
Native-depth comparisons use recorded alpha words placed at their buffer origins,
not 8-bit PNG precision.

## Defects and corrections

The ordinary input-only expanded reference exposed the exact host assertion:
`I_MIX_GUID_DEPENDENCIES effect missing call to GuidMixInPtr during SMART_PRE_RENDER`.
The user supplied the dialog. ADR-0056 calls the GUID mix-in once at entry for
all graphs, using a fixed constant without coverage. The diagnostic separately
needed IExpandBuffer, origin-aware forwarding and a larger bounded readback
budget; those changes alone did not resolve the assertion.

The first comparison shader also sampled input red instead of alpha, producing
an invalid opaque reference. Correcting that independent harness defect made
unshifted alpha exact, then exposed a real reader defect: comp-sized output
discarded the 3548/3554 exterior pixels. The reader now declares the union of
base and source extents, clipping result to request while preserving the maximum
extent. Its native byte copy already honored origins. All exterior samples now
match. Failed logs and summaries remain in the record.

The user dismissed dialogs. A later MCP exit timed out; the exact launched AE
process was verified on saved `full.aep` with no dirty marker before termination
under the standing lifecycle authorization. Backups precede all version-specific
replacements. Launches were hidden; no foreground or mouse automation was used.

## Reproduction and limits

```text
python spike/host-outline/verify-full-coverage.py docs/audits/evidence/coverage-full-20260923
```

The verifier recomputes 114 final native pairs and the retained exterior clipping
failures, and checks the 15 ROI readbacks and empty-background control. Local
default/editor/no-SDK main suites pass 249 each, reader 4, probe 22; builds pass.
Main build uses Rust 1.97.1, the ignored SDK root in DYNAMICFX_AESDK_265_ROOT and
`cargo +1.97.1 build --release --locked --offline --target x86_64-pc-windows-msvc`.
Reader/probe manifest paths and their actual commands are retained in the logs.

Public baseline is `922845c` plus the previous production overlays and these
frozen source inputs. [Manifest](manifest.json) hashes every recorded file.
Native logs are losslessly gzipped; other logs normalize workspace/home paths
and MCP artifact/recovery IDs. SDK, AEX/AEP binaries, images and local lockfiles
are excluded. Scratch harnesses show actual test actions and must not be replayed
against authoring projects. See the [integration audit](../../11-host-coverage-integration.md).
