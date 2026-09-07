# Native build and package evidence

This directory preserves the actual build/test output and bundle verification
reports for two working-tree artifacts based on `d4477ab`. See
[`../build-identities.json`](../build-identities.json) for complete hashes.
These records establish build/package checks; they do not establish final AE
acceptance.

| Run | Actual command | Result and evidence |
|---|---|---|
| First compile attempt | `RUSTUP_TOOLCHAIN=stable bash scripts/build-macos.sh` | `FAIL`: signed/unsigned `A_Err_NONE` comparison, [log](build-arm64-failed-1.log) |
| Second compile attempt | same | `FAIL`: unconditional `user32` link on macOS, [log](build-arm64-failed-2.log) |
| First completed build | same | `PASS`: first signed binary `b3bd67cf…`, [log](build-arm64-pass.log), [bundle report](bundle-verification.json) |
| First default tests | `cargo +stable test --target aarch64-apple-darwin` | `PASS`: 181/181, [log](test-default.log) |
| First editor tests | `cargo +stable test --target aarch64-apple-darwin --features editor` | `PASS`: 181/181, [log](test-editor.log) |
| Siri example compile | Exact invocation retained by the root session; not printed in this log | `PASS`: 1 selected test, [log](test-siri-example.log) |
| Geometry-fix default tests | `cargo +stable test --target aarch64-apple-darwin` | `PASS`: 184/184 including Siri and two geometry regressions, [log](test-default-geometry.log) |
| Geometry-fix editor tests | `cargo +stable test --target aarch64-apple-darwin --features editor` | `PASS`: 184/184, [log](test-editor-geometry.log) |
| Geometry-fix build | `RUSTUP_TOOLCHAIN=stable bash scripts/build-macos.sh` | `PASS`: signed binary `e239ae45…`, [log](build-arm64-geometry.log), [bundle report](bundle-geometry-verification.json) |

The explicit `stable` alias resolved to Rust/Cargo **1.97.1**, matching the
repository's pinned compiler version. The actual dependency lockfile is
preserved byte-for-byte as [`Cargo.lock.txt`](Cargo.lock.txt) (the suffix avoids the
repository-wide `Cargo.lock` ignore rule); its SHA-256 is
`483aa5ad54679d0291453f31464d07c191f8967af6a7e11b9e4f7aeda3cab7fe`.

Both bundle reports verify native arm64, `EffectMain`, PiPL resource 16000,
the `DynamicFx` match name, system dynamic dependencies and the complete
ad-hoc signature. The first binary also passed
[`dlopen` and symbol lookup](dlopen.log), without making an AE host call.
The first installed artifact's identity and backup path are retained in
[`install-ae2026.json`](install-ae2026.json).

The first report called packaging-time context `build`; that historical report
is retained verbatim. This label did not independently establish provenance.
The final packager uses `packaging_context`, and actual build logs plus artifact
hashes establish which binary was tested.

The plug-in ZIP files remain under `scripts/out/macos/` and are referenced by
hash in `build-identities.json`. They are ordinary file-byte archives with
executable permissions and no extended attributes. The final archive's
[identity](final-archive-identity.json) and
[verification after extraction](final-archive-verification.json) confirm
`e239ae45…` without rebuilding, re-signing or changing bundle contents.

[`file-manifest.json`](file-manifest.json) records source paths, sizes and
SHA-256 for every copied raw file. Logs retain their original compiler and
artifact paths because those identify the commands and outputs in this run.
