# DynamicFX internal coverage reader

Native pixel-forwarding component for [ADR-0051](../docs/adr/0051-native-coverage-reader-component.md).
The main plugin creates and binds its invisible reading layer automatically.
Users author shaders through the ordinary DynamicFx Source expression.

Build on Windows with `cargo +1.97.1 build --release --locked --offline
--manifest-path coverage-reader/Cargo.toml --target x86_64-pc-windows-msvc`.
The first local dependency resolution creates the ignored lockfile with
`cargo +1.97.1 test --offline --manifest-path coverage-reader/Cargo.toml`.
Rename the resulting DLL to `DynamicFxCoverageReader.aex` and package it beside
the main AEX. Installation follows the main repository's closed-host,
version-specific directory rules. This is an internal utility, not a glass
material or a substitute for the main plugin.

The current integration is under acceptance. First-frame native comparisons
and the tested lifecycle cases pass; copy/FFX-before-idle readiness and the
complete delivery matrix remain open. See the
[integration audit](../docs/audits/11-host-coverage-integration.md).
