# 0.1.1 Windows patch, IOS27Siri sample and macOS ARM backfill

## Active macOS ARM backfill

The user now separately authorizes an ARM archive from the unchanged v0.1.1
source `2428cfd94b4556cc3bc5ed63f7950ceca42a6b92` and its published exact
dependency lock. Add it to the existing regular release, preserving the tag
and Windows/Sample archive bytes. [ADR-0047](../adr/0047-011-macos-backfill.md)
qualifies the earlier Windows-only delivery scope; no native code change or
old local Siri work is included.

Native ARM build and static bundle verification **PASS**. The signed
executable is 7,602,160 bytes, SHA-256
`3d2fc6e88415f512988b02036779a48dcbae7b1f4e4d1b525020e71450c8133f`;
PiPL SHA-256 is
`b64852ea2a0bb7cc2548398843bf2bdf8d8613b6e7f60b884707a696ed8af8a0`.
The published lock SHA-256 is
`125d8418a9eb26068503cd15717ae7f04a2d23e680162285730c273a49d1f8bc`.
Environment: Apple M5, macOS 26.5.2 (25F84), Rust 1.97.1. Command:
`RUSTUP_TOOLCHAIN=stable bash scripts/build-macos.sh --locked --offline`.
The first offline build lacked four cached dependencies; a locked dependency
fetch completed, then the locked offline build passed. The source checkout
was clean and its 30 recorded build-input hashes include the exact lock.
See the [build identity](evidence/macos-011-20260909/build-record.json),
[passing build log](evidence/macos-011-20260909/build-macos-locked.log),
[initial offline-cache failure](evidence/macos-011-20260909/build-macos.log)
and [locked fetch](evidence/macos-011-20260909/fetch-locked.log).
The [redaction manifest](evidence/macos-011-20260909/evidence-redactions.json)
preserves original/published hashes and identifies checkout-path substitutions.

Default and editor CPU suites **PASS**, 222 tests each:
[default log](evidence/macos-011-20260909/tests-default.log),
[editor log](evidence/macos-011-20260909/tests-editor.log).

The quality suite **PASS** contains 98 tests. Five actual Metal renders on
Apple M5 pass **23 assertions**: equivalent GLSL/WGSL inputs have equal
pixels at 8- and 32-bit working depth, and the tag's eight-pass
`examples/siri-reference.glsl` produces finite pixels with alpha 1. The
first smoke fixture omitted its required envelope and failed before shader
compilation; a corrected fixture and fresh result are retained separately.
The successful summary is `scripts/out/011/quality/smoke-summary-v2.json`
in the tagged-source checkout; the
[public evidence index](evidence/macos-011-20260909/README.md) records its
curation. This runner uses zero-valued Point defaults: it establishes valid
Metal execution, not calibrated Siri appearance, native AE output or Windows
FXC failure recovery.

Package and new-asset download verification are in progress and
remain **NOT_RUN** until evidence is recorded. New-byte native
AE execution and Mac Sample acceptance are **NOT_RUN** and outside this
request. Source-expression Undo remains a known failure. The prior 0.1.0 Mac
host pass does not certify the new binary.

Next action: verify the exact-source ARM build and signed package, append it
and its checksum to the existing release, then verify a fresh download and
the unchanged Windows/Sample hashes.

## Original Windows and Sample publication

PASS: v0.1.1 is published as a regular release; the original three downloaded assets match the frozen packages.
Baseline is v0.1.0 (`f0cd526`) plus the two runtime repairs and 0.1.1 metadata.

The renderer captures Validation, Internal and OutOfMemory pipeline errors as
E58. Rejected replacements clear stale pipelines. No Shader ABI, parameter
pool, persistence format or texture filtering behavior changes.

| Check | Outcome and evidence |
|---|---|
| Default / editor tests | PASS, 222 each: [default](evidence/release-011-20260909/tests-default.log), [editor](evidence/release-011-20260909/tests-editor.log) |
| Quality suite | PASS, 98: [log](evidence/release-011-20260909/tests-quality.log) |
| Real DX12 compiler rejection and next valid pipeline | PASS, E58 without unwind or poisoned lock: [log](evidence/release-011-20260909/test-gpu-recovery.log) |
| Release compilation | PASS: [log](evidence/release-011-20260909/build-release.log), [exact lock](evidence/release-011-20260909/Cargo.lock.txt) |
| Windows artifact | PASS, AMD64/PE32+/DLL/EffectMain, version 0.1.1 and 342-byte PiPL: [identity](evidence/release-011-20260909/artifact.json) |
| Sample relocation / source / original project preservation | PASS: [validation](evidence/release-011-20260909/sample-validation.json) |
| Native preview | PASS, 300 unique PNG frames, 560 × 420, 60 fps and full decode: [probe](evidence/release-011-20260909/preview-probe.json), [sample](../../examples/IOS27Siri/) |

AEX SHA-256: `b10632fa77bee85443813758958f506033fe59d8e3bf00528f4bafd89e685e1d`.
Environment: Windows 11, RTX 5080 DX12, Rust 1.97.1 MSVC; native Sample checks
in AE 2026 26.3x87 at 16 bpc. The native project was copied, reduced to 10
items, relinked to its two included assets, opened from a different directory,
and rendered at 65/60 seconds. Sample, relocated and original renders have
identical PNG hashes. The original project matches its immediate checkpoint.
All AE operations used MCP.

The previously installed repair passed valid-source rendering, source
replacement and save/reopen. [Runtime-code equivalence](evidence/release-011-20260909/runtime-equivalence.json)
is narrowly checked: 25 modules unchanged, lib.rs adds only two compile tests;
Cargo/PE and PiPL version metadata change. Newly versioned bytes were not
installed during packaging. Exact-new-byte AE execution, other AE years and
macOS 0.1.1 remain NOT_RUN. Native bad-shader injection was BLOCKED by automatic
review; only the isolated real-GPU test establishes failure-path recovery.
Source single-Undo/Redo remains the known failure from ADR-0045, not a fixed bug.

## Commands and packaging findings

`cargo test --offline --lib`; same with `--features editor`;
`cargo test --manifest-path scripts/quality/Cargo.toml --offline`;
same with `backend_compile_error_returns_and_next_pipeline_succeeds -- --ignored --nocapture`;
`cargo build --offline --release`.

The initial PNG export was incomplete because project switching interrupted
asynchronous writes. The successful rerun waited for every PNG IEND between
batches. Dependency-notice collection initially rejected CRLF-converted
licenses; the exact upstream LF hashes were restored, with supplementary
notices for three Windows dependencies. These were packaging failures, not
additional plugin fixes.

The first proposed push was rejected before dispatch because it included
historical raw audits and assets outside the requested publication payload.
This release branch starts from the public baseline and includes only source,
the current sample and this necessary validation record. Original archives
remain on the unpublished local development branch. Build logs visibly redact
machine paths; [redaction manifest](evidence/release-011-20260909/evidence-redactions.json)
records the original and published hashes. No original evidence is deleted.
Historical visual perfection remains unaccepted; the sample is an approximation.

[Release boundary](../adr/0046-011-windows-patch-release.md).
The original publication is complete. The separately authorized Mac backfill
above is the current action; Source Undo repair and native host acceptance
remain separate work.

## Published identity

[Release](https://github.com/JUNKDOGE-JOE/dynamicfx/releases/tag/v0.1.1), source commit `2428cfd94b4556cc3bc5ed63f7950ceca42a6b92`. [Downloaded-asset verification](evidence/release-011-20260909/published-verification.json), [checksums](evidence/release-011-20260909/SHA256SUMS.txt), [public payload manifest](evidence/release-011-20260909/publication-manifest.json). The historical raw-archive commit was not pushed.

Temporary render sequences, relocated test project, export comparisons and staging copies were removed after validation; [cleanup record](evidence/release-011-20260909/temporary-cleanup.json). Final release archives and the verified Sample remain.
