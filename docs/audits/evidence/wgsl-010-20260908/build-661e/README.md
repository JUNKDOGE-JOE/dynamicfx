# 661e candidate: build and CPU evidence

Recorded on 2026-09-08. **Build/CPU checks passed; current-artifact AE host
acceptance is NOT_RUN in this record.** Administrator installation was still
awaiting the system authorization result when these artifacts were curated.
Neither this archive nor the preceding candidate's rendering checks establish
that the new candidate fixes GUI Undo/Redo. No release is claimed.

The candidate contains the supervised Source/Language and idle-publication
repair, plus fresh-default publication before the state/plan words. Exact
Color/Angle defaults retain AEGP floating-point writes; their Undo grouping
with PF writes remains a host-verification question.

## Identities

- Source commit: `f4ba578552e02c2a82170d2d116cf0400d8319a6`.
- Source tree: `50e205650cdc4dcc5263daa4deacd70d66f098a4`.
- Bundle version/architecture: 0.1.0, `arm64`; build target
  `aarch64-apple-darwin`.
- Packaged executable SHA-256:
  `661e89af6aa6c9cb595732146b01a42a4dad99fd1d6a19929723622a8f2a7371`
  (7,600,160 bytes).
- Raw pre-packaging dylib SHA-256:
  `d07b9cc194d22a1e7f807e6c912c8ec64c90a821f737b1688c6339bd0bfb917d`.
- Cargo.lock SHA-256:
  `4af148e8cb350674e90236096b536405bb74b54d5f9565f57573a43abfa2e306`;
  identical bytes are already retained in [Cargo.lock.txt](../build/Cargo.lock.txt).
- Rust/Cargo: 1.97.1, `stable` alias on this native Apple Silicon Mac.

[build-record.json](build-record.json) records individual source hashes and
bundle identity. [build-macos.log](build-macos.log) records the build and
packaging result: Mach-O arm64, PiPL flags and an ad-hoc signature. Packaging
context is explicitly dirty; it is not presented as a clean release tag.
Evidence curation did not rebuild, install or re-sign the bundle.

## Commands and observed results

Commands ran in the repository directory on the native arm64 host. The CPU
commands did **not** contain an explicit `--target` option.

```sh
cargo +stable test --locked --offline > scripts/out/010/undo-fix/tests-default-final.log 2>&1
cargo +stable test --locked --offline --features editor > scripts/out/010/undo-fix/tests-editor-final.log 2>&1
RUSTUP_TOOLCHAIN=stable bash scripts/build-macos.sh --locked --offline > scripts/out/010/undo-fix/build-macos.log 2>&1
```

| Run | Observed result | Raw evidence |
|---|---|---|
| Default CPU suite | 213 passed, 0 failed | [tests-default-final.log](tests-default-final.log) |
| Editor CPU suite | 213 passed, 0 failed | [tests-editor-final.log](tests-editor-final.log) |
| ARM release build/package | Build finished; bundle identity above | [build-macos.log](build-macos.log) |
| Final targeted default-publication suite | 5 passed, 0 failed, 208 filtered | [fresh-default-tests.log](fresh-default-tests.log) |
| Earlier targeted test fixture | 3 passed, 1 failed, 208 filtered | [fresh-default-first-test-failure.log](fresh-default-first-test-failure.log) |

The targeted command was:

```sh
cargo +stable test --locked --offline fresh_default_tests > /private/tmp/dynamicfx-ucp-fresh-default-tests.log 2>&1
```

The earlier failure occurred before the final source freeze. Its local test
fixture passed null `InData` to the after-effects wrapper, which rejected it
with `assertion failed: !ptr.is_null()`. This was a **CPU test-fixture failure,
not an AE host failure**. The fixture was corrected to use a local initialized
`PF_InData` object without host calls. The retained final run also includes
the added `PF_KeyIndex_NONE`/zero/positive-keyframe boundary regression. The
five targeted tests are included within the later 213-test suites; their
counts are not additional production tests.

## Provenance and integrity

All six raw artifacts are copied byte-for-byte. The
[file-manifest.json](file-manifest.json) maps original locations to retained
files and SHA-256 hashes. Machine-local build/registry paths inside those raw
artifacts are retained only to identify the build location and the wrapper
assertion that caused the test-fixture failure. There is no bridge output,
credential material, project file, binary bundle or screenshot in this set.

[SHA256SUMS](SHA256SUMS) covers the six raw files, this README and the local
manifest. Verify from this directory with:

```sh
shasum -a 256 -c SHA256SUMS
```

The parent evidence manifest and main verification documents are maintained
separately. Later installation/AE results must retain their own artifact
identity and evidence; they must not be inferred from this build record.
