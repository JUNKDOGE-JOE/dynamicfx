# ADR-0050: Host coverage resource and production integration

- Status: Accepted
- Decision 6 ready encoding superseded by [ADR-0052](0052-coverage-instance-readiness.md).
- Date: 2026-09-23
- Scope authority: user request to integrate the verified reader into production development;
  [ADR-0049](0049-automatic-host-shape-input.md) records approved coverage-only,
  internal-reader and AE 26.5+ boundaries.
- Related: [ADR-0013](0013-paramid-grammar-and-pools.md),
  [ADR-0019](0019-intermediate-format-policy.md),
  [ADR-0039](0039-canvas-expansion.md), [ADR-0040](0040-parameter-groups-and-id-identity.md).
- Verification: TR-COVERAGE-INTEGRATION-001 in [TEST_MATRIX](../TEST_MATRIX.md).

## Decision

1. `// @param <id> hint:coverage` declares a read-only graph texture, with no
   uniform member or user layer/path selector. GLSL and WGSL share the syntax.
   Names follow existing ParamId/alias rules. Repeated uses of one ID share one
   resource; at most one distinct coverage ID is allocated per effect.
2. Add `ShaderParamType::Coverage` and a separate main-only `PoolKind::Coverage`
   of capacity one. Its persistence tag is 10, appended after Path (9).
   Register its hidden Layer parameter after every existing parameter, followed
   by hidden non-time-varying `CoverageState`. Existing IDs and physical indexes
   remain unchanged. Existing manual Layer capacity is not consumed.
3. Coverage contains the original source's native alpha with masks applied,
   before the host DynamicFx output and independent of the adjustment background.
   Encode `(a,a,a,a)` in the current working format. U15 conversion uses 32768;
   float32 alpha words are copied without arithmetic. Do not clamp or apply a
   color transform. Padding is zero, and cropped worlds retain their origin.
4. The texture spans the same canvas as `input`. Coverage placement uses the
   checked-out world's origin and canvas origin in the same downsampled space.
   ROI does not redefine the canvas. Do not stretch a cropped coverage world
   across the full texture. Final output is not automatically multiplied by
   coverage: AE retains adjustment-layer compositing responsibility.
5. Main-thread ownership connects the original ONLY_MASKS stage to an invisible
   reader, then its ALL_EFFECTS stage to the hidden coverage parameter. Frame
   callbacks only use declared SmartFX dependencies. No AEGP acquisition, idle
   wait, geometry reconstruction or expression sampling in render callbacks.
6. `CoverageState` is host-managed transport metadata: 0 pending, 1 verified
   ready, 2 unsupported host, 3 ownership conflict. E59 reports unsupported
   coverage capability; E60 reports pending/invalid/unavailable coverage.
   Until ownership/readiness is integrated, the state stays pending and shaders
   requesting coverage fail closed with E60. An unset input never silently
   becomes the background alpha or a valid-looking transparent resource.
7. Undeclared shaders perform no coverage work. This feature requires AE 26.5+
   (StreamSuite7), with older hosts retaining their existing features. Coverage
   combined with `prev` follows the existing layer/temporal refusal E7 until
   replay-time acquisition has its own accepted contract and evidence.
8. Ownership, copy/FFX-before-idle readiness and helper renderer integration are
   explicit production gates. Do not infer them from diagnostic tests. Ordinary
   Source.expression remains authoritative; internal metadata is not an alternate
   shader authoring channel. Any additional persistent helper topology or clone
   protocol must be recorded before implementing it.

## Delivery sequence

Resource/reflection/persistence and exact canvas encoding -> explicit readiness
gates -> native owner adapter and original-instance binding -> helper frame path
-> production pixel tests -> FFX, ROI/downsample/PAR and MFR/aerender acceptance.
The earlier diagnostic remains evidence, not the production implementation.

## Verification obligations

Both frontends compile the resource; graph writes/collisions are rejected;
uniform budget is unchanged; parameter IDs/indexes remain stable; persistence
round-trips tag 10 and rejects unknown tags; native alpha conversion and cropped
placement preserve bytes/words. A missing binding yields E60, unsupported host
yields E59, and existing shaders/resources pass regression. Production release
still requires the full native host and lifecycle gates from ADR-0049.
