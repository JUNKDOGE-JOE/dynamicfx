# ADR-0043: Apple Silicon macOS host protocol

- Status: Accepted
- Date: 2026-09-08
- Owners: DynamicFX project
- Authorization: user requested native ARM macOS AE operation and explicitly authorized installing the plugin and testing temporary projects.
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md) §17, §19
- Related decisions: [ADR-0004](0004-breaking-rewrite-and-host-matrix.md), [ADR-0014](0014-windows-host-protocol.md)
- Related tests: TR-MAC-001 in [../TEST_MATRIX.md](../TEST_MATRIX.md)

## Context

Windows AE 2025 and 2026 have completed the M0-M7 and 0.0.6 host runs.
ADR-0004 already schedules Apple Silicon after Windows stability. The user
now starts that phase. The available macOS host is AE 2026. At the fetched
baseline `d4477ab`, the renderer defaults to DX12 on every OS, and the current
repository has no macOS bundle installer. ARM PiPL entries alone cannot make
that runtime render on macOS.

## Decision

1. Add an independently verified `aarch64-apple-darwin` native plugin bundle.
   The existing Windows matrix and DX12 default stand unchanged.
2. Use wgpu Metal by default on macOS, high-performance adapter selection,
   and record adapter/backend identity. Explicit diagnostic overrides remain
   available; no override is required for normal macOS use.
3. Build and validate an ARM Mach-O executable, ARM PiPL entry, bundle
   metadata, exported AE entry points and ad-hoc signature before local use.
   Local ad-hoc signing is not Developer ID notarization or a public release.
4. Install only into the selected year's
   `/Applications/Adobe After Effects <year>/Plug-ins/DynamicFx/DynamicFx.plugin`.
   Require an explicit year, refuse while AE/aerender runs, refuse shared
   MediaCore copies, preserve the replaced local bundle for rollback, and
   verify the installed executable against the tested artifact.
5. Start with installed AE 2026. Each additional year needs its own evidence;
   no Windows or other-year result implies a macOS pass.
6. Retain the released GLSL ABI, topology, language registry, persistence,
   depth and alpha contracts. Fixing the sampler to linear clamp-to-edge
   implements the existing ADR-0011 §5 contract and does not change the ABI.

## Alternatives considered

- Rosetta: unnecessary; the requested target is native ARM.
- Defaulting to every wgpu backend: hides the tested backend and does not
  provide a reproducible platform baseline.
- Shared MediaCore installation: violates the AE-only product boundary.

## Consequences

Native macOS work can be validated without modifying the Windows artifact
path. GPU and host differences remain independent validation obligations.
Existing shaders that relied on accidental nearest filtering may change at
fractional coordinates; explicit `texelFetch` retains discrete reads.

## Revisit conditions

A real unsupported Metal capability or additional host-year request requires
its own evidence and explicit supported-subset decision. Public distribution
requires a separate signing/notarization and release verification step.

## Verification obligations

- ARM bundle, resource, symbol, signature and installed hash checks.
- Native AE discovery, one addProperty call, expression-only publication,
  visible single/multi-pass output, invalid-source behavior, keyframes,
  save/reopen, 8/16/32-bpc and aerender checks.
- Subpixel texture sampling and AA/noise fixtures on the real Metal path.
- Preserve raw failures and passing artifacts in TR-MAC-001 and TR-QUALITY-001.
