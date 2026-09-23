# Production coverage stage adapter

The production build now compiles the stage adapter against the locally supplied
SDK 26.5 headers when `DYNAMICFX_AESDK_265_ROOT` is set. No SDK content is included
here. The adapter atomically binds a layer/stage pair, verifies both values, and
balances every acquired value, stream and suite on success and failure.

Baseline: public `922845c`, the
[resource foundation](../coverage-integration-20260923/manifest.json), and this
frozen source overlay. Windows; Rust 1.97.1. No AE installation or execution.
The new API still awaits its production owner caller. Current unused-API
warnings are preserved; this is not an operational coverage feature.

Actual commands and results:

| Command | Environment | Result |
|---|---|---|
| `cargo +1.97.1 test --offline --lib` | SDK variable set | 239 PASS; updates local ignored lockfile for cached cc dependency |
| `cargo +1.97.1 test --offline --locked --lib --features editor` | SDK variable set | 239 PASS |
| `cargo +1.97.1 test --offline --locked --lib` | SDK variable absent | 239 PASS |
| `cargo +1.97.1 test --offline --manifest-path spike/host-outline/stage-check/Cargo.toml --target-dir scripts/out/coverage-stage-20260923/check-target` | SDK variable set | 22 PASS |
| `cargo +1.97.1 build --release --locked --offline --target x86_64-pc-windows-msvc` | SDK variable set | PASS |

The standalone checker includes the production C++ source and uses SDK-declared
callback types. It verifies unchanged/read-only bindings, atomic mutations,
incorrect layer and stage readback, null/missing callbacks, acquisition/read/
write failures, disposal errors, and exact cleanup counts. Rust tests reject
wrong-thread access and missing compiled capability. A no-SDK build leaves
existing features available and refuses this new capability.

Candidate SHA-256:
`b9e4db45517fad8dbf1d85aade4f3cc3c0309783f36f3bd6a85ae7e1738dd6cd`.

[Manifest](manifest.json) hashes the frozen sources and actual logs. Absolute
workspace paths in logs are replaced with `<workspace>`; ignored SDK files,
local lockfiles and binaries are excluded. Ownership, helper rendering,
first-frame clone readiness and the complete production AE gate remain
NOT_RUN. See the [integration audit](../../11-host-coverage-integration.md).
