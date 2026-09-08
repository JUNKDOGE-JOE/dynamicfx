# Frozen macOS 0.1.0 package

This archive contains the bdfc bundle accepted in [host-bdfc](../host-bdfc/README.md), built from native source `8b0fe81f5f6869d1ced67fc3cd1dbf6d366dae36`. Packaging does not rebuild or re-sign it.

- ZIP: `DynamicFX-0.1.0-macos-arm64.zip`, 3,563,347 bytes.
- ZIP SHA-256: `f2fffc1808727231db8a713cddf024cbcf0e55a7f1064beff6d8897a129dcc3d`.
- Executable: `bdfc9f1d978cf052e01db97d69b1d61c1ad9cc073b7691a82e2cb2d90baaa141`.
- PiPL resource: `fe5f2d0ba4386a9cbb4b2695da5f0086f0d5d1f8a1a90ba3370085b91e07de04`.
- `ditto -x -k` extraction: architecture/signature verified, all 287 entries in internal `SHA256SUMS` matched.
- Archive includes 132 dependency packages and 253 license texts, the exact lockfile, examples and authoring Skill; optional editor disabled.

`package-bdfc-final.json` and `final-*` are the final archive records. The earlier `package-bdfc.json` and unprefixed extraction/checksum files retain the unpublished 3,563,283-byte ZIP with SHA `3a5272045e1d6f27add1917804a8ac49ff5211802f8e128a601274bd8b4a8632`. Independent review found stale acceptance wording in the packaged Skill reference; correcting only that document produced this final ZIP. The signed plugin bytes did not change.

`release-notes.md` is the reviewed publication text. These package checks precede upload; upload and re-download verification are recorded separately when performed. `curation.json` retains original and path-redacted file hashes. No ZIP, project, bridge credential or user file is included here.

Commands:

```sh
python3 scripts/release/package_macos.py --bundle target/macos-arm64/DynamicFx.plugin --build-record scripts/out/010/authoring-fix/build-record.json --third-party <verified-notices> --out scripts/out/010/release-bdfc-final/DynamicFX-0.1.0-macos-arm64.zip
ditto -x -k scripts/out/010/release-bdfc-final/DynamicFX-0.1.0-macos-arm64.zip <extracted>
python3 scripts/macos.py verify <extracted>/DynamicFx.plugin
```
