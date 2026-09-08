# ADR-0047: 0.1.1 macOS ARM asset backfill

- Status: Accepted
- Date: 2026-09-09
- Authorization: the user explicitly requested compiling the Windows-side 0.1.1 source for macOS ARM, adding it to the existing release, and updating the release description and README.
- Qualifies: [ADR-0046](0046-011-windows-patch-release.md) for this additional platform asset only.
- Related: [ADR-0043](0043-apple-silicon-host-protocol.md), [ADR-0044](0044-wgsl-and-010-release.md), [ADR-0045](0045-010-source-undo-release-boundary.md).

## Decision

Build native `aarch64-apple-darwin` from the unchanged `v0.1.1` source commit
`2428cfd94b4556cc3bc5ed63f7950ceca42a6b92` with the exact dependency lock
published in the Windows package. Add `DynamicFX-0.1.1-macos-arm64.zip` to
the existing regular release. Do not move the tag, relabel the 0.1.0 binary,
replace the existing Windows/Sample archives, or import the preserved local
Siri work. Packaging and documentation changes may follow on main; record
their identity separately from the tagged native build inputs.

This request covers native compilation, CPU suites, real-Metal checks, bundle
and package verification, and publication with fresh-download checks. It does
not include installation into AE, restarting AE, native host rendering or new
Sample acceptance. Mark those 0.1.1 Mac results **NOT_RUN** in README, release
notes and the audit. Neither the previous 0.1.0 Mac host pass nor a Metal
fixture proves AE acceptance of these new bytes. This narrowly qualifies the
platform verification boundary for this asset; the full host protocol remains
the requirement for a later claim of native host acceptance.

Ship an ARM-only, ad-hoc-signed bundle, exact lock, notices and checksums.
Keep the verified bundle unchanged during packaging: no rebuild or re-sign
after freezing its identity. Verify extracted executable/PiPL, signature,
internal manifest and source/lock mapping; after upload, download again and
verify these identities and the unchanged Windows/Sample archive hashes.
The checksum asset may be extended for the added archive; retain the previous
checksum record as historical evidence. No Developer ID notarization, Intel
or Rosetta claim is added.

## Consequences

Source-expression single Undo remains a known failure under ADR-0045; this
backfill does not repair it or change source authority, Shader ABI, persistent
topology or runtime code. The optional gradient editor stays disabled. Follow
the existing year-specific AE installation instructions and never use shared
MediaCore. The [0.1.1 audit](../audits/09-windows-011.md) records the exact
passing subset, artifacts and publication result; unrun host checks remain
visible after publication.
