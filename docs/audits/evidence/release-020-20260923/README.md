# Windows 0.2.0 acceptance — 2026-09-23

The user deferred issue #12 and authorized release of completed #9–#11.
Final main and reader identities are frozen in binary-identities.json. Both
are installed in Windows AE 26.5x89; all 27 frame pairs equal the accepted
issue-fix baseline. SDK default/editor/no-SDK tests pass 257 each; reader four.
Metadata/cache generation and diagnostic-path remapping are the only release
binary changes. Source runtime behavior remains the accepted issue batch.

The first comparison used edited workspace geometry and failed; that project
and its initial frames remain preserved locally. Final tests use a frozen
fixture, with no edits to the user's workspace. Initial binaries containing
home paths remain local; the final pair excludes them. Build with locked Rust
1.97.1, SDK 26.5 and RUSTFLAGS remapping USERPROFILE to /user-home and checkout
to /dynamicfx. Text paths are normalized where needed.

SDK/AEX/PDB/private AEP/PSD and intermediate f32 files are not source payload.
The exact release ZIP, checksums and source identity are published as release
assets; fresh-download SHA-256/CRC results are retained in the local release
output and checked against those public assets. The reader's internal version
remains 0.1.0. No macOS 0.2.0 asset or affected-host #12 fix is claimed.
