# ADR-0011: Shader ABI v1 core

- Status: Accepted
- Date: 2026-08-11
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md) §6.1, §8, §14, §16
- Related decisions: [ADR-0003](0003-render-graph-is-core.md), [ADR-0007](0007-identity-and-cache-boundaries.md), [ADR-0009](0009-staged-format-adr-acceptance.md)
- Related tests/audits: TR-M1-001/003/004 in [../TEST_MATRIX.md](../TEST_MATRIX.md); `docs/audits/01-first-frame.md`

## Context

Every pass module compiles against a runtime-provided interface. That interface enters `ModuleHash` (ADR-0007), so it must be versioned and explicit before M1 code freezes an accidental contract. Numeric conventions (UV orientation, texel centers) are exactly the details implementation exposes, so per ADR-0009 they are stated here as intended values and finalized against M1 fixtures; pre-release ABI version bumps remain allowed and must be recorded in the test matrix.

## Decision

1. **Versioning.** `ShaderAbiVersion` is an unsigned 32-bit integer; this ADR defines version `1`. It is a `ModuleHash` input. Pre-release bumps are permitted with a matrix note; after first release, changes require a superseding ADR.
2. **Scope.** This ADR binds the GLSL frontend's per-pass fragment interface. Other language frontends must lower to the same neutral interface; their surface syntax is their own ADR's concern.
3. **Per-pass fragment interface** (GLSL 450 core through naga `glsl-in`):
   - entry point `main`, fragment stage only;
   - `layout(location = 0) in vec2 v_uv;`
   - `layout(location = 0) out vec4 outColor;` — exactly one color target in v1;
   - `layout(set = 0, binding = 0) uniform texture2D u_input;` — primary pass input;
   - `layout(set = 0, binding = 1) uniform sampler u_sampler;`
   - `layout(set = 0, binding = 2) uniform FxUniforms { ... };` — std140.
   - Set 0 bindings 3-15 are reserved for the M4 multi-input extension; descriptor sets ≥ 1 are reserved. A v1 module that declares reserved bindings is rejected with a stable diagnostic.
4. **Builtin uniform head.** `FxUniforms` begins with a fixed std140 head, in this order:
   ```glsl
   vec2  u_resolution;   // pass target size in pixels
   float u_time;         // comp time of the evaluated frame, seconds
   float u_frame;        // comp frame index as float
   ```
   User parameters follow the head in declaration order (types: `float`, `int`, `bool` as i32, `vec2`, `vec3`, `vec4`; declaration/ID grammar is ADR-0013's contract). Appending a new builtin later bumps the ABI version; reordering or removing head fields is forbidden while v1 exists.
5. **Semantics.** `u_resolution` is the current pass target extent (M1: the layer extent). `u_time` and `u_frame` never enter any identity hash or `PipelineKey` (ADR-0007). Default sampling is linear filtering with clamp-to-edge addressing.
6. **Fixture-pinned numeric conventions** — intended values, each requiring an M1 pixel-fixture before the ABI is considered implemented; a fixture contradiction amends this list pre-release with a version bump:
   - UV origin top-left; `v_uv.y` increases downward, matching AE raster order;
   - texel centers at `(i + 0.5) / N`;
   - output rows top-down, matching the AE output world;
   - v1 performs no color-space conversion: 8-bpc unorm input passes to the shader as stored; alpha enters and leaves with AE's premultiplication state unchanged. The full alpha/color policy is the M5 format ADR; v1 makes no claim beyond "unchanged".
7. **Out of scope for v1:** compute passes, storage buffers/textures, multiple render targets, explicit mip LOD control, history resources (M6 entry ADR), push constants, and user-controlled vertex stages (the fullscreen pass geometry is runtime-owned). Declaring them is a stable-diagnostic rejection, not undefined behavior.
8. **Validation.** A module missing required interface elements, or whose `FxUniforms` head mismatches, is rejected at compile with a stable diagnostic code and input pass-through; nothing silently passes.

## Alternatives considered

- Adopt the prototype head (`u_resolution`, `u_time` only): rejected; the roadmap needs a frame index, and the unreleased rewrite is the only cheap moment to add it.
- Provide builtins as separate uniforms instead of one UBO head: rejected; one std140 block keeps reflection, hashing, and binding plans simple and matches the prototype's proven path.
- Defer all numeric conventions to whatever M1 happens to produce: rejected; unstated conventions become accidental contracts — stating intended values first makes fixture deviations visible decisions.

## Consequences

### Benefits

- `ModuleHash` gains a real ABI version input from the first compiled module.
- Multi-pass (M4) extends bindings into pre-reserved space instead of breaking v1 modules.
- Fixtures test stated conventions instead of blessing incidental behavior.

### Costs and risks

- Reserved-binding rejection may surprise users porting Shadertoy-style code with extra inputs before M4 lands.
- The std140 head is a permanent layout once released; mistakes before release cost an ABI bump, after release a superseding ADR.
- naga `glsl-in` acceptance defines the practical GLSL dialect; compiler upgrades can shift edge-case acceptance (tracked by `ArtifactHash`, but user-visible).

## Revisit conditions

M1 fixture evidence contradicting an intended numeric convention amends §6 with an ABI bump (pre-release). Adding capabilities listed in §7 happens through their scheduled milestone-entry ADRs, not by widening v1.

## Verification obligations

- Rust unit tests: interface validation, reserved-binding rejection, std140 head offsets (16-byte head: 0/8/12).
- M1 pixel fixtures for each §6 convention (UV orientation gradient, texel-center probe, row-order probe) with numeric comparison per test-matrix rules.
- TR-M1-004: raw GLSL lowered to a one-pass graph renders through this interface on at least one Windows AE year.
