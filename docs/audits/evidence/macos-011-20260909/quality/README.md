# v0.1.1 ARM standalone quality evidence

Source: exact tag v0.1.1, commit `2428cfd94b4556cc3bc5ed63f7950ceca42a6b92`. No tracked source changed; `git status --short` was empty after verification.

- CPU: 98 tests passed, 0 failed, exit 0. See `tests-quality.log` and `test-execution.json`.
- Standalone build: exit 0. See `build-quality.log`.
- Actual GPU: Apple M5, Metal; 5 valid-shader renders, 23 checks passed. See `smoke-summary-v2.json`, `smoke-executions-v2.json`, per-case logs and PNG/f32 outputs.
- Simple GLSL/WGSL source produced identical output bytes at each working depth (8 and 32). Numeric reference max absolute errors: 8-bit 0.0019608139991760254; float32 5.960464477539063e-8.
- Exact tagged `examples/siri-reference.glsl` completed all 8 passes, 585x1266 physical / 1170x2532 logical, t=1s, float32; finite and opaque, changed input. Its Point controls have zero-valued defaults in this runner. This is compilation/execution smoke, not a calibrated Siri visual or AE positioning test.
- All shared dependency name/version/source/checksum rows match the frozen native lock. See `dependency-parity.json`, `Cargo.lock.production`, `Cargo.lock.quality`.

## Commands

Run in the frozen worktree, with `DYNAMICFX_BACKEND`, `DFX_QUALITY_RENDER_SOURCE`, and `DFX_QUALITY_BENCH_FRAMES` unset:

```sh
cargo +stable test --offline --manifest-path scripts/quality/Cargo.toml --target aarch64-apple-darwin
cargo +stable build --locked --offline --manifest-path scripts/quality/Cargo.toml --target aarch64-apple-darwin
python3 scripts/out/011/quality/run_smoke.py
```

The smoke script requires NumPy and Pillow. Here it ran with the bundled Python runtime. It launches the built binary directly; each exact command and exit code is retained in the logs.

## Preserved initial fixture failure

The first GLSL test fixture mistakenly omitted the mandatory `@dynamicfx 1` envelope. It exited 1 at grammar parsing before any GPU execution. Original source, script, stderr and checksums remain in `attempt-01-missing-envelope/`. Corrected fixtures use fresh `smoke-v2.*` names and WGSL line-leading `@@` escapes. This was a test-fixture error, not a runtime failure. Production/tagged source was not modified.

## Scope

This evidence does not test After Effects, packaging, signing, release delivery, host persistence/Undo, or DX12/FXC failure recovery. The Windows-only error-recovery test is compile-time excluded on macOS and is not covered by the 98-test count. Working depth here denotes the standalone renderer format, not the AE native 16-bit boundary.

## Public-copy layout

Original output hashes in `smoke-summary-v2.json` identify the locally retained
PNG/f32 files; those render buffers and the duplicate executions file are not
included here. The native lock is the previously published release lock.
To rerun, copy this directory to `scripts/out/011/quality/` in the exact tagged
checkout and copy `Cargo.lock.quality` to `scripts/quality/Cargo.lock`. The
script resolves its root from that location. Use a fresh output directory or
retain existing attempts before rerunning. Path-normalized source copies have
separate public hashes in the parent redaction manifest; the initial failure
manifest continues to identify original bytes.
