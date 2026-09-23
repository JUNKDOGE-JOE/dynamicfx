# Windows 0.2.0 release

## Outcome and scope

The user deferred #12 and authorized merging/releasing completed #9–#11.
Windows 0.2.0 includes native original-layer coverage and its reader, the
licensed Liquid Glass shader/FFX, RGB hex defaults and percent display.
#12 remains open and is explicitly not claimed fixed. Source-expression single
Undo remains a disclosed limitation. macOS stays at its separate 0.1.1 asset.
[ADR-0061](../adr/0061-020-windows-release.md).

## Release artifacts and evidence

Main: `b4505fd38a7799a21e7d933032ac17118effba3162b8ffe0b4673466b16b4a7e`.
Reader: `b0f476ab29016d2070898284ea145a6bac48a1e2c8cb63afc28c50cf645f4887`.
Versioned bytes are installed and checked in Windows AE 26.5x89. Root product
version is 0.2.0; the reader retains its internal component version. Runtime
code is unchanged from the accepted issue batch. Release changes are metadata,
monotonic PiPL cache generation and build-path remapping in both binaries.

The three library configurations pass 257 tests each; reader tests pass four.
Final formal AE frames use Full/Half/Quarter, 8/16/32 project depth and three
animation times. Display PSD output is 8-bit; the earlier native/GPU float
comparisons establish the deeper precision boundary, not those PSD files.
Recorded final frame equivalence and identities are in the
[release evidence](evidence/release-020-20260923/README.md).

An initial comparison used the user's changed workspace geometry and failed.
That workspace is preserved. Acceptance uses a separate frozen test fixture;
no user geometry is changed to force the test to pass. Initial build binaries
contained home paths and were withheld; final builds remap USERPROFILE to
`/user-home` and the checkout to `/dynamicfx` through RUSTFLAGS.

## Publication procedure

Verify exact outgoing source/evidence and archive members for sensitive data,
SDKs, private projects and unrelated files. Merge the tested feature branch
normally. Tag the resulting main commit as v0.2.0, package the same accepted
artifact pair with pinned dependency notices and locks, then publish. The
package's source identity must name that merged commit and match its tracked
build inputs. Download the public files again and check SHA-256 and ZIP CRC.
The new release must close only #9–#11, retain #12 open, and preserve every
existing v0.1.1 asset. Release checksums and source identity ship as assets at
[v0.2.0](https://github.com/JUNKDOGE-JOE/dynamicfx/releases/tag/v0.2.0).

## Reproduction and next action

Use Rust 1.97.1, the locked Windows target and the locally supplied 26.5 SDK
root. Run library default/editor/no-SDK tests and reader tests. Build the main
and reader with home/checkout prefix remapping; install only while AE/aerender
are closed into AE 2026's dedicated DynamicFx folder. The recorded harness
prepares disposable projects and independent MFR-enabled aerender frames.

Next development action: obtain an affected-host reproduction for #12 using
the [diagnostic guide](../project-reopen-diagnostics.md).
