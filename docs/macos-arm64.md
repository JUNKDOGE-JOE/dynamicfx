# Apple Silicon macOS

DynamicFX's native macOS target is `aarch64-apple-darwin`, using wgpu's
Metal backend. This extends the existing runtime under
[ADR-0043](adr/0043-apple-silicon-host-protocol.md). Windows retains DirectX 12.
The shader language, source authority, parameter identities and project schema
are shared with Windows. Intel macOS and Rosetta are outside this target.

The exact tested AE years, artifact and remaining host checks belong in
[TEST_MATRIX.md](TEST_MATRIX.md); a successful bundle check alone is not a host
support claim. The shelved gradient editor remains disabled in default builds.

## Build and inspect

Prerequisites: native Apple Silicon macOS, Xcode Command Line Tools, Python 3.9
or newer, and the Rust version in `rust-toolchain.toml`. The scripts honor
`RUSTUP_TOOLCHAIN` and do not install or change a toolchain. If an installed
alias resolves to the pinned compiler, it can be selected explicitly; record
`rustc --version` and `cargo --version` with the build evidence.

```sh
bash scripts/build-macos.sh
python3 scripts/macos.py verify target/macos-arm64/DynamicFx.plugin
```

The build writes `target/aarch64-apple-darwin/release/libdynamicfx.dylib` and
`dynamicfx.rsrc`, then packages `target/macos-arm64/DynamicFx.plugin` with:

- an arm64 Mach-O executable exporting `EffectMain`;
- a classic resource file containing PiPL resource 16000 and the `ma64`
  entry point, with the existing `DynamicFx` match name;
- `Contents/Info.plist` and `PkgInfo` declaring the AE `eFKT` bundle type;
- packaging-time checkout/toolchain context, the input binary hash and the
  packaging checkout's dependency-lock hash;
- an ad-hoc signature applied after the complete bundle is assembled.

The verifier checks the executable architecture, exported entry point,
system-only dynamic dependencies, PiPL identity and signature. It emits a JSON
report with signed executable and resource hashes. The package command also
saves the report beside the bundle as `DynamicFx.verification.json`.
The report's `packaging_context` and the bundle's
`Resources/packaging-context.json` describe the environment when packaging ran.
They do not establish which source revision, lockfile or toolchain built a
previously existing binary; preserve the actual build log and its artifact
hash separately to establish that provenance.

To package a previously verified build without rebuilding it:

```sh
python3 scripts/macos.py package target/aarch64-apple-darwin/release
```

Do not rebuild between host acceptance and distribution. Retain the lockfile,
working-tree baseline and raw logs with the accepted binary: `Cargo.lock` is
currently ignored by the repository, so its evidence hash alone does not make
a future dependency resolution reproducible.

## Install for one AE year

Close After Effects and aerender, then install the packaged artifact for the
explicit year. The script requires write access to that application's plug-in
directory; it does not invoke `sudo` itself.

```sh
bash scripts/install.sh 2026
```

If Adobe installed that directory as `root:wheel` without user write access,
the installer reports that condition before making a backup or staging a
replacement. Authorize only the installer, keeping the build under your normal
user account:

```sh
sudo bash scripts/install.sh 2026
```

An administrator password is entered into the system's prompt. The script
still checks the invoking user's shared plug-in folder when run through `sudo`.

The destination is
`/Applications/Adobe After Effects 2026/Plug-ins/DynamicFx/DynamicFx.plugin`.
The installer verifies the source, refuses a running host or a duplicate
DynamicFx bundle, and refuses shared MediaCore copies. An existing version is
copied to a `DynamicFx-backup-*` temporary directory before replacement; the
printed JSON includes its exact backup path. Staging is verified before the
swap, and a failed post-install check restores the previous bundle. Installation
does not launch or terminate AE.

The accepted year arguments describe install destinations, not a claim that
every year is verified. AE 2024 and AE 2026 must have separate host records.
The per-year installation boundary deliberately avoids Adobe shared MediaCore.

## Native macOS fixes

The Windows baseline selected DX12 unconditionally; on macOS that produces no
usable adapter. The macOS default is now Metal, and adapter/device errors are
logged instead of disappearing during initialization. The same audit found
that `after-effects-sys` exposes `A_Err_NONE` as an unsigned constant on macOS
while stream APIs return signed `A_Err`; the host comparison must use the API's
error type. Neither change alters stream values or serialization.

The Details button also linked Windows `user32` unconditionally, preventing
the native macOS binary from linking. Windows keeps its existing dialog;
macOS now uses AE's Unicode `AEGP_ReportInfoUnicode` dialog from the same UI
callback.

A real Half-preview check also exposed a canvas regression in the 0.0.6
baseline: `PF_InData.width/height` remain full-resolution, while SmartFX
requests, upstream rectangles and pixel worlds use render pixels. The canvas
now converts each axis by its downsample ratio before requesting or resolving
its extent. Full/Half/Quarter use the same physical geometry in PreRender
and the SmartRender fallback; authored expansion margins retain their
existing per-axis conversion. Odd source sizes round outward to a complete
render pixel. The established `u_resolution` contract reconstructs logical
size from that physical canvas, so odd sizes can differ by less than one
render-pixel footprint at reduced resolution.

Set `DYNAMICFX_VERBOSE_LOG=1` before launching AE to record logical dimensions,
downsample ratios, requested and upstream rectangles, and resolved physical
canvas geometry in the temporary `dynamicfx.log`. Normal operation does not
require this diagnostic setting.

The PiPL advertises only the CPU architecture actually compiled into a native
build. This package is arm64-only and does not advertise an Intel entry point.
The Windows resource-repair path remains independent.

The bundle layout and requirement to sign complete modern AE macOS plug-ins
are corroborated by the upstream
[after-effects build recipe](https://github.com/virtualritz/after-effects/blob/master/AdobePlugin.just).
For 0.1.0, [ADR-0044](adr/0044-wgsl-and-010-release.md) authorizes public
distribution of this ad-hoc-signed bundle. It is not Developer ID signed or
notarized. Verify the published checksum before installation; if macOS blocks
the downloaded bundle, remove only its quarantine attribute as described in
the release INSTALL.txt. Do not disable Gatekeeper system-wide.
