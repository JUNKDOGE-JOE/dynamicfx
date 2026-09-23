# Automatic host coverage

This Windows feature ships in 0.2.0 and requires **After Effects 26.5 or newer**.
The [implementation status](IMPLEMENTATION_STATUS.md) and
[test matrix](TEST_MATRIX.md) identify accepted artifacts and remaining work.
The published 0.1.1 downloads do not contain this feature.

## Installation and authoring

Close AE and aerender, then install `DynamicFx.aex` and
`DynamicFxCoverageReader.aex` from the same verified build in the version-specific
`Support Files/Plug-ins/DynamicFx` directory. Never use shared MediaCore.

Declare `// @param coverage hint:coverage` in the shader and list `coverage`
as a graph input. Its RGBA channels contain the original layer alpha, with masks,
on the same canvas as `input`. It is independent of the background supplied to
an adjustment effect. Sampling it requires no Layer or Mask selector.

The plugin creates one hidden, shy, locked internal reader per original layer.
Keep editing the original layer normally. Do not edit or bind the internal
reader manually. Copies and FFX targets acquire their own validated link after
AE returns to idle. A missing or modified reader reports E60 rather than treating
another layer's coverage as valid. An unsupported host reports E59.

The shader does not need to multiply its output by coverage when AE already
uses the original adjustment-layer alpha for compositing. Coverage supplies
sampling information such as refraction direction. Existing `hint:path` remains
the separate, manually selected mask-path resource. Coverage with temporal
`prev` is explicitly refused with E7.

## Build the pair

Extract the user-provided Windows AE 26.5 SDK outside version control, set
`DYNAMICFX_AESDK_265_ROOT` to its SDK root, then run:

```powershell
cargo +1.97.1 build --release --locked --target x86_64-pc-windows-msvc
cargo +1.97.1 build --release --locked --target x86_64-pc-windows-msvc --manifest-path coverage-reader/Cargo.toml
```

Use the resulting `dynamicfx.dll` as `DynamicFx.aex` and
`dynamicfx_coverage_reader.dll` as `DynamicFxCoverageReader.aex`. SDK archives,
headers and local build trees stay ignored. Building without the SDK preserves
ordinary features but cannot supply the new coverage resource.

The Windows packager accepts `--coverage-reader`, `--coverage-verification`
and `--coverage-third-party` together. It verifies both artifact hashes against the host-acceptance record,
includes both binaries and the adapted dispatcher license, and labels the ZIP
as a coverage development candidate. Packaging is not host acceptance.
