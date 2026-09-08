# 0.1.1 Windows patch and IOS27Siri sample

Local release checks pass; publication awaits the uploaded-asset verification.
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
Next action: publish and verify downloaded package identities.
