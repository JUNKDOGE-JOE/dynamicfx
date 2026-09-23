# Host coverage and contour diagnostic build

The user approved coverage as the primary refraction resource and a separate
contour-sampling resource in the same feature batch. The follow-up was posted
to [issue #9](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/9#issuecomment-5620243801)
and read back. Transport and shader ABI remain Proposed in ADR-0049.

Production baseline: local `ff89564` on `codex/fix-project-open-lock`.
The independent [probe crate](../../../../spike/host-outline/native/Cargo.toml)
builds a separate effect with no production source changes. It compares PF
path enumeration, raw post-expression vector/transform streams, and AEGP
upstream layer checkout using RenderSuite5 and RenderSuite4 with its plain
flag off/on. The probe is Windows-only, uses normal 8/16-bpc pass-through
rendering, and requests 16-bpc worlds for the coverage experiment.

## Local results

- Five tests pass: invalid/negative time handling, pixel bounds and padded
  stride, bounded geometry traversal, cleanup on early error, and nested-read
  guard lifetime. [Test log](test.log).
- Native development build succeeds. [Build log](build.log).
- [checks.json](checks.json) records source/lockfile and artifact hashes,
  fixture consistency, production preservation and governance output.
- The initial compile reported four integration errors: missing PiPL support
  URL, a zero-sized instance, an incorrect PF vertex accessor and a time-mode
  integer type mismatch. These were corrected before the passing logs.
- Both installed Rust toolchains lack rustfmt. No components were installed;
  automated formatting is not claimed.

## Unverified work

The current AE-MCP health probe returned unreachable. No installation, AE
restart, project mutation, native acquisition or feature rendering ran.
Neither host-safety nor pixel correctness follows from a successful DLL build.
HS-01 through HS-15 remain NOT_RUN. The three sample coordinates are currently
world-local and only suitable for the identity baseline, not arbitrary layer
transforms or expanded canvases. Modifier evaluation, contour encoding,
thread-safe per-frame transport and invalidation remain open.

Source and diagnostic controls are not a public shader ABI. The two shader
resources, refraction material and portable FFX are not implemented yet.
See the [procedure](../../../../spike/host-outline/README.md) before native
experiments. A scoped diagnostic installation and disposable fixture run
are the next step once an AE connection is available.
