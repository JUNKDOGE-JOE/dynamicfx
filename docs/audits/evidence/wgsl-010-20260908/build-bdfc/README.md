# bdfc candidate: CR parsing/name repair build evidence

Recorded on 2026-09-08. **Build/CPU checks passed; current-artifact AE host
acceptance is NOT_RUN in this record.** The new installation was still
awaiting system authorization when this set was curated. No empty install
result or host PASS is included. This is build evidence for a new executable,
not a transfer of the retired 661e candidate's passing Language test.

## Identities

- Source commit: `8b0fe81f5f6869d1ced67fc3cd1dbf6d366dae36`.
- Source tree: `45fd2c3c62a008022d675bdc0376996894800d51`.
- Bundle version/architecture: 0.1.0, `arm64`;
  target `aarch64-apple-darwin`.
- Packaged executable SHA-256:
  `bdfc9f1d978cf052e01db97d69b1d61c1ad9cc073b7691a82e2cb2d90baaa141`
  (7,600,208 bytes).
- Raw pre-packaging dylib SHA-256:
  `4ac92477fec1f698ed20d994365090ba3e1e4764bfeec76029bd6691495e7e5f`.
- Cargo.lock SHA-256:
  `4af148e8cb350674e90236096b536405bb74b54d5f9565f57573a43abfa2e306`;
  identical bytes remain in [Cargo.lock.txt](../build/Cargo.lock.txt).
- Rust/Cargo: 1.97.1, `stable` alias on the native Apple Silicon Mac.

[build-record.json](build-record.json) records source-file hashes, build
command and bundle verification metadata. [build-macos.log](build-macos.log)
contains the successful build/package result with arm64 Mach-O, PiPL flags
and ad-hoc signature. Its packaging context remains explicitly dirty; it is
not represented as a clean release tag. Curation did not rebuild, re-sign,
install or publish the bundle.

## CPU results

| Run | Observed result | Raw evidence |
|---|---|---|
| Default CPU suite | 220 passed, 0 failed | [tests-default.log](tests-default.log) |
| Editor CPU suite | 220 passed, 0 failed | [tests-editor.log](tests-editor.log) |
| Targeted CR/CRLF parser regression | 6 passed, 0 failed, 214 filtered | [source-line-ending-tests.log](source-line-ending-tests.log) |
| ARM release build/package | Build finished; bundle identity above | [build-macos.log](build-macos.log) |

The six parser tests cover the exact bare-CR expression captured during the
661e native UI leg, raw GLSL/WGSL LF/CR/CRLF input, envelope resource
annotations and hints, temporal windows, original-byte source-size limits,
and consistent idle classification. They pin original source/token identity
while normalizing only a parsing copy. They are included within each full
220-test suite and are not additional production tests. The full suites also
include the short-name termination regression. These CPU checks do not prove
that the repaired authoring paths behave correctly in installed AE.

The full CPU commands ran on the native arm64 host without an explicit
`--target` option. The recorded commands were:

```sh
cargo +stable test --locked --offline > scripts/out/010/authoring-fix/tests-default.log 2>&1
cargo +stable test --locked --offline --features editor > scripts/out/010/authoring-fix/tests-editor.log 2>&1
RUSTUP_TOOLCHAIN=stable bash scripts/build-macos.sh --locked --offline > scripts/out/010/authoring-fix/build-macos.log 2>&1
cargo +stable test --locked --offline source_line_ending_tests > /private/tmp/dynamicfx-source-line-ending-tests.log 2>&1
```

## Provenance and integrity

All five source artifacts are copied byte-for-byte.
[file-manifest.json](file-manifest.json) records original locations, retained
filenames, byte counts and SHA-256 hashes. Machine-local checkout paths in
raw compiler/signature output are retained only for build provenance,
consistent with the preceding build archive. No bridge/request output,
credential material, AEP, screenshot, bundle or empty installation record is
included. The original build-log hash and previously retained Cargo.lock
hash were checked against the new build record.

[SHA256SUMS](SHA256SUMS) covers the five raw files, this README and the local
manifest. Verify from this directory:

```sh
shasum -a 256 -c SHA256SUMS
```

The parent manifest and main verification documents are maintained
separately. Later installed-artifact results must be recorded independently;
the accepted Source single-Undo limitation remains a FAIL under [ADR-0045](../../../../adr/0045-010-source-undo-release-boundary.md),
not something these build or parser tests resolve.
