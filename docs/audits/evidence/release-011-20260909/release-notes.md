# DynamicFX 0.1.1

Windows GPU compiler failure handling and an editable IOS27Siri sample.

## Fixes

- Capture DX12/FXC pipeline rejection as diagnostic E58, including Internal
  and OutOfMemory errors, before the uncaptured-error handler can panic and
  poison the instance lock.
- Discard stale pipelines before rebuilding. A failed replacement cannot
  leave an old pipeline installed for the new source or working depth.

## IOS27Siri sample

Includes an AE 2026 project, background assets, editable eight-pass GLSL and
a five-second native 60 fps preview. Four analytic colored sheets retain
independent phase offsets, with smooth glass breathing, refraction, reflected
color and Transmission/Sheen controls. It evaluates time continuously; no
sampled animation table is used. This is an independent visual approximation,
not Apple's shader or a claim of perfect fidelity.

[Browse the sample](https://github.com/JUNKDOGE-JOE/dynamicfx/tree/v0.1.1/examples/IOS27Siri).

## Downloads and verification

- `DynamicFX-0.1.1-windows-x64.zip`: Windows x64 AEX, installation instructions,
  locked dependencies and third-party notices.
- `IOS27Siri-0.1.1.zip`: portable editable sample and preview.
- `SHA256SUMS.txt`: checksums for both archives and the contained AEX.

Default/editor suites pass 222 tests each; the quality suite passes 98 plus
the explicit real-DX12 compiler-failure recovery regression. The sample was
relocated and rendered in AE 2026 26.3x87, with 300 unique native frames and
matching master-frame bytes before/after relocation.

The runtime repair passed AE 2026 valid rendering, source replacement and
save/reopen before this version metadata bump. Production runtime code is
identical to that installed repair; the newly versioned AEX was built and
statically verified, but was not reinstalled during packaging. Native
bad-shader injection, other AE years and macOS 0.1.1 are not covered. The
previous verified macOS ARM build remains available under v0.1.0.

**Known limitation:** editing Source may not be undone with one Undo, and
Redo may become unavailable. Keep previous shader text and restore it
explicitly. This patch does not fix Source-expression Undo. The optional
gradient editor remains disabled. Close AE before installing and use its
version-specific plug-ins directory, never shared MediaCore.

[Detailed evidence and limits](https://github.com/JUNKDOGE-JOE/dynamicfx/blob/v0.1.1/docs/audits/09-windows-011.md).
