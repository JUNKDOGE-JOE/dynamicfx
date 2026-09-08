# macOS ARM 0.1.1 backfill evidence

Native build and static bundle verification: **PASS**. Default and editor
CPU suites: **222 passed each**. Exact-new-byte AE execution: **NOT_RUN**.
This is a build/distribution backfill, not another AE installation or sample
visual-refinement task. Metal valid-shader smoke: **5 renders / 23 checks PASS** on Apple M5.
[Quality evidence](quality/README.md) retains the 98-test suite, dependency
parity and first malformed-fixture failure. Publication and fresh-download verification: **PASS**.
Windows-only FXC failure injection is not a Metal test.

## Frozen native inputs

- Source tag `v0.1.1`: `2428cfd94b4556cc3bc5ed63f7950ceca42a6b92`.
- Rust 1.97.1 (`stable` alias resolves to the pinned compiler), macOS 26.5.2
  (25F84), native `aarch64-apple-darwin`, default features (editor disabled).
- Exact [published dependency lock](../release-011-20260909/Cargo.lock.txt):
  SHA-256 `125d8418a9eb26068503cd15717ae7f04a2d23e680162285730c273a49d1f8bc`.
- Signed executable, 7,602,160 bytes:
  `3d2fc6e88415f512988b02036779a48dcbae7b1f4e4d1b525020e71450c8133f`.
- PiPL: `b64852ea2a0bb7cc2548398843bf2bdf8d8613b6e7f60b884707a696ed8af8a0`.
- [Build record](build-record.json) records all 30 inputs, raw dylib identity,
  native source tree, bundle architecture, entry point, system dependencies,
  ad-hoc signature and exact command. No Developer ID signing/notarization.

## Commands and retained results

The first [offline attempt](build-macos.log) failed before compilation because
four locked dependencies were absent from the Mac cache. [Locked fetch](fetch-locked.log)
filled that cache; neither the source nor lock was changed. The subsequent
[locked offline build](build-macos-locked.log) exited 0:

```sh
RUSTUP_TOOLCHAIN=stable bash scripts/build-macos.sh --locked --offline
RUSTUP_TOOLCHAIN=stable cargo test --locked --offline --lib
RUSTUP_TOOLCHAIN=stable cargo test --locked --offline --lib --features editor
```

[Default](tests-default.log) and [editor](tests-editor.log) output retain all
222 test names each. [Dependency notices](dependency-notices.json) cover
133 target-reachable packages and 254 license files; they ship outside the
signed bundle. The lock referenced above is also included as `build/Cargo.lock`
in the final archive.

[Redaction manifest](evidence-redactions.json) records original and published
hashes. Temporary checkout prefixes are replaced with `$BUILD_ROOT`; log
headers label that transformation. Original logs remain under the isolated
build checkout's ignored `scripts/out/011/` directory.

The release packager records `packaging_docs_commit` separately from the frozen
native `source_commit`: only packaging instructions, current README and
release evidence change after the source tag. It checks all frozen native
inputs and copies the already signed bundle without rebuilding or re-signing.

Source-expression single Undo/Redo remains a known failure. The independent
Windows sample's host evidence does not establish macOS sample acceptance.

## Published asset

[Regular v0.1.1](https://github.com/JUNKDOGE-JOE/dynamicfx/releases/tag/v0.1.1)
now includes `DynamicFX-0.1.1-macos-arm64.zip`, **6,934,170 bytes**, SHA-256
`e8b4b01dab9df5567a93b27ee3f30ec174bb48098319826f57a22703bb2c3cc7`.
[Package result](package-result.json), [independent candidate checks](verify-candidate.json)
and [actual re-download checks](verify-downloaded.json) all match. Both the
candidate and a fresh downloaded extraction pass all **297 internal file
hashes**, ARM/PiPL/entry-point/signature/executable-mode checks, exact lock and
native source identity. The 315 download assertions are byte/distribution
checks, not AE host tests. The six IOS27Siri files equal the existing tag.

Packaging instructions/README come from
`4bb7196d9c72a23700e44b67c6e68eb78f7c7ab5`, separately recorded as
`packaging_docs_commit`; native source remains `2428cfd`. Neither the source
tag nor signed bundle was changed. Package-only links use the frozen document
commit so newly added documents do not resolve against the older source tag.

[Published metadata](published-release.json), [updated description](release-notes.md)
and [combined checksums](SHA256SUMS.txt) retain the existing Windows/Sample
archive identities. Their asset IDs, sizes, digests and update timestamps
are unchanged. The previous checksum asset's 267 bytes remain an exact prefix;
three ARM archive/executable/PiPL lines were appended. The checksum asset was
replaced, as authorized, and the download matches those new bytes exactly.
[Previous metadata](release-before-backfill.json) retains the original state.

[Publication-input scan](preflight-package-summary.json) covers the specified
prepared snapshot with zero matched credentials/restricted findings and
15 archive checks passing. Final document checks are recorded separately.
The native 0.1.1 Mac AE and Mac Sample checks remain **NOT_RUN**.

The [downloaded Windows lock comparison](windows-lock-parity.json) additionally
verifies the original Windows ZIP digest and its actual `build/Cargo.lock`
bytes against both the Mac build and the source tag.
