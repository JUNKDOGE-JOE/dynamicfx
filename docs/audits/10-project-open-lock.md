# Project-open callback repair

PASS for the reported AE 2026 project-open hang: the installed local repair
opens the recovery copy in 10.604 s and the unchanged original on repeat in
4.420 s. The user stopped desktop operation after successful native rendering;
two separate display-name expression references are still unmodified.

## Evidence and baseline

[TR-OPEN-001](../TEST_MATRIX.md#tr-open-001---project-open-idle-callback-and-sequence-lifetime)
records the full failure-to-pass chain. The
[manifest](evidence/project-open-lock-20260909/manifest.json) identifies source
inputs, both candidate binaries, raw stacks, tests and native results by hash.
Baseline is `17e12a7` on `codex/fix-project-open-lock`, Windows 11 / AE 26.3x87.
The final installed AEX is `c91db8c0...`; this is not a published release.

## Cause and code paths

During BEE_LoadProject the AEGP idle hook can issue CompletelyGeneral before
SequenceResetup. after-effects-rs 0.4.0 casts sequence_data to the Rust instance
before invoking its callback. Here the handle still held the flattened
`01 00 DFXS` snapshot. Mutex acquisition interpreted the version as a locked
mutex, changed its first byte to `02`, and waited forever. The PDB-matched
stack and raw allocation distinguish this from real lock contention.

`src/host/entry.rs` introduces an in-memory tag at a fixed offset, validates
the allocation size and tag before entering the upstream dispatcher for
CompletelyGeneral, and defers until sequence initialization. `build.rs` routes
PiPL and registration to DynamicFxMain and advances the local build generation.
`scripts/macos.py` checks that same entry point; macOS itself is NOT_RUN.
`src/host/idle.rs` skips token/UI work when the call supplies no initialized
reply. No Rust instance reference or mutex access is made to flattened data.

`src/lib.rs` also releases the instance mutex around host source reads and UI
publication. `src/host/callback.rs` prevents recursive host work for the same
instance; UI completion only publishes bookkeeping to the matching definition.
The reentry and header tests preserve snapshot data and unknown-schema behavior.

## Contracts

No parameter index, source authority, binding rule, StateToken, persistent
schema, shader ABI or rendering algorithm changes. The live tag is never
flattened; version 1 plus DFXS persistence remains unchanged. No registry or
vendored dependency was edited. Source Undo remains a separate known issue.

## Procedure and findings

Run `cargo test --offline --lib`, the same command with `--features editor`,
and `cargo build --offline --release`; final runs pass 230 / 230 tests and build.
Freeze AEX/PDB, stop only the verified hung AE PID, verify no AE/aerender remains,
install to the AE 2026-specific folder, restart, and use MCP to open the saved
copy. Read back effect values/keys/expressions and Main links, render Main at
25 s, then inspect the unchanged original and perform one guarded reopen.

The first lock-only candidate's 120 s timeout and the subsequent test compile
failure are retained. The first direct original open also reported "layer has
no source" after loading all items. Read-only recovery confirmed clean state;
the repeated original open returned no error. Its cause is not established.
Do not describe these first attempts as passing.

The project has two expressions in comp 3231 / layer 3260 pointing at the
temporary `Refraction px` display name, absent until UI labeling. They were
identified but not edited: the user pressed Escape to stop Computer Use.
No further AE input followed. The original remains unsaved and byte-identical;
the one saved recovery copy retains pre-restart state. No exports, commits,
pushes, releases or additional project checkpoints were created for this repair.

## Limits and retention

Acceptance covers this Windows AE 2026 reproduction and native frame, not all
host years, macOS, a long soak or every upstream sequence lifecycle selector.
The preview PNG is not evidence of final color accuracy; the native UI was
observed separately. Raw project/script evidence stays outside the repository
because it contains production source and project details. Its absolute paths
and hashes are retained in the manifest; binary/PDB evidence stays under
scripts/out. Existing user files and the historical release audit are preserved.

## Local source checkpoint

On 2026-09-10 the user authorized committing this remaining repair. All 33
source inputs match the frozen manifest and the retained final AEX still
hashes to `c91db8c0...`. Fresh default/editor runs each pass 230 tests. The
initial 220-test cached executable omitted the new tests; its output is
preserved and is not counted as verification of the current source.
[TR-OPEN-COMMIT-001](../TEST_MATRIX.md#tr-open-commit-001--project-open-repair-source-checkpoint)
and [raw evidence](evidence/project-open-commit-20260910/README.md) record
this verification. No runtime code changed during commit preparation.

The earlier no-commit statement describes the original host repair turn.
This local checkpoint does not publish the archival branch or change the
release tag, installed AEX or production project. The subsequent Windows
asset refresh is recorded separately in its existing evidence directory.

Next action: no further repair changes are needed for this checkpoint;
source publication requires a separately scoped public-history transfer.
The two production-project expression references remain deferred.

## Subsequent authorized Windows archive refresh

On 2026-09-09 the user separately authorized replacing the existing GitHub
v0.1.1 Windows AEX package with the final c91db8c0 repair, without changing
the version. Publication and redownload verification now PASS. The earlier
no-publication statements above describe the initial repair acceptance scope;
this subsequent authorization applies only to the Windows release asset and
its matching package metadata/checksums. Source history and tag were not pushed
or rewritten. Mac and Sample assets remain unchanged. See
[TR-REL-011-REFRESH-001](../TEST_MATRIX.md#tr-rel-011-refresh-001---existing-windows-asset-replacement)
and [downloaded-asset evidence](evidence/release-011-refresh-20260909/README.md).
