# IOS27Siri withdrawal verification

Baseline: `effc75becb4829f2675086be6fdf11b009f95a57`; isolated Windows checkout,
2026-09-10. [ADR-0048](../../../adr/0048-ios27siri-withdrawal.md).

## Verified preparation

- Removed 13 sample/research/runner files after ZIP backup and SHA-256 readback.
  Original authoring checkout and its unfinished project work remain untouched.
- `cargo test --offline --lib example_tests::`: PASS, 6 passed, 214 filtered,
  zero failures. Only the two tests embedding withdrawn sources were removed;
  production runtime source is unchanged.
- Windows `scripts/release/package_windows.py` fixture: PASS, one plugin ZIP,
  valid CRC, unchanged AEX and no Siri file or sample instructions. The fixture
  reuses the downloaded Windows dependency files and lock, with a test-only
  manifest binding to this checkout's Cargo.toml. It is not a release candidate.
- Edited Python AST parsing: PASS with explicit UTF-8.
- `scripts/check_governance.py`: PASS, 115 Markdown files, 1,352 local links,
  16 Mermaid blocks and 48 Accepted ADRs before adding this evidence record.
  The final check runs again with the complete evidence corpus.
- `git diff --check`: PASS.

The first Windows packaging fixture rejected the refreshed release's
Cargo.toml hash, which does not identify this main-branch checkout. Checking
LF normalization did not resolve it. The isolated fixture was explicitly
rebound to the current input; release metadata and downloads were untouched.
Packaging then succeeded. A following AST-only check initially used Windows'
GBK default and failed to decode UTF-8; explicit UTF-8 passed. Neither failure
was a plugin/runtime regression.

## Mac archive

[Package verification](package-verification.json) records the original asset
hashes, removed paths and new archive identity. Nine bundled sample files were
removed. Only README, example instructions, INSTALL and the checksum manifest
were edited; a separate package-withdrawal record was added. All 285 other
entries retain exact bytes and ZIP permission/timestamp metadata, including
all six signed bundle files, dependencies, scripts and source identity.
CRC and all 289 internal checksum entries pass. No rebuild or re-sign occurred.

New Mac ZIP: 3,581,945 bytes, SHA-256
`8a536d03aa5960bda701dacc92e862bb26b66ec585de73dffb2b15e24ef00b12`.
Windows ZIP remains SHA-256
`96ec69db520d687e8a266c688995dfb6c4a98b061c66b4b227283a56cb783d65`.

Historical Git commits, the source tag and prior audit evidence remain.
New native AE execution is NOT_RUN; unchanged plugin bytes preserve the
previous validation scope without adding a new host claim.

## Publication

Main received withdrawal commit `82c302c`. Release cleanup was performed after
the user separately confirmed that the existing Release should also be cleaned.
[Fresh-download verification](published-verification.json) passes: only the two
plugin ZIPs and checksum asset remain, the Mac ZIP and checksums exactly match
the candidates, all 289 internal hashes pass and the signed bundle is unchanged.
Windows asset ID, size, digest and update time remain identical. The source tag
object remains `c5e73d1731c3424ea2e09d66b9968d8e307a2b2c`.
Re-publication of the animation awaits the user's request after refinement.

Publication preflight scanned 1,390 files with zero credential-pattern findings;
all 15 Mac archive checks passed. Its historical machine-path/media review
flags refer only to unchanged files. No new binary/media content is published
in the repository by this withdrawal; existing audit records retain their
original scope.
