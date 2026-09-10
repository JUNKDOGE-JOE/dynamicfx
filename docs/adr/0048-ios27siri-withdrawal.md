# ADR-0048: Temporarily withdraw the IOS27Siri animation sample

- Status: Accepted
- Date: 2026-09-10
- Authorization: the user requested withdrawing the iOS 27 Siri animation from the repository to refine it before publishing again.
- Supersedes: only the sample-shipping decisions in [ADR-0046](0046-011-windows-patch-release.md) and [ADR-0047](0047-011-macos-backfill.md); their runtime, source-tag and host-verification boundaries remain in force.

## Decision

Remove the IOS27Siri project, media, reference/ribbon shaders, shader guide,
dedicated quality runner and research assets from the current repository tree.
Remove its promotional entries and stop producing a separate sample archive.
The older, independent `siri-glow.glsl` example remains available.

Withdraw `IOS27Siri-0.1.1.zip` from the existing release. The macOS archive
also contains the sample, so refresh that archive by removing the same sample
files and updating its documentation and checksum manifest. Preserve every
signed bundle file, dependency, source-build record and installation script
byte for byte; do not rebuild or re-sign. Keep the Windows archive unchanged.
Update the release description and checksums to match the resulting downloads.

Back up the downloaded assets and removed repository files before withdrawal.
Preserve the original local authoring checkout and its unfinished work. A later
publication requires the user's request after refinement.

## History and verification

Keep the existing source tag and historical release/audit evidence. This is a
withdrawal from the current tree and maintained downloads, not a rewrite of
published Git history; old commits, tag archives and external copies remain.
Historical validation records do not mean the sample is currently shipped.

Check remaining example compilation, packaging without the withdrawn files,
documentation links, archive CRCs and every retained Mac file. Re-download the
changed assets and verify their checksums, internal manifest and unchanged
plugin identities. No new AE execution or visual acceptance is implied.
