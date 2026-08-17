# DynamicFX concept

> **Prototype snapshot:** 本文描述尚未发布的重写前实现。已确认的产品决策和目标架构以 [ARCHITECTURE.md](ARCHITECTURE.md) 为准；重写将保留 `DynamicFx` 名称，但不承诺本文所述参数顺序、transport 或持久化协议兼容。

Status: implemented MVP. The render loop, GLSL compilation, reflected effect
controls, multi-shader registry, and programmatic update path exist. The base
render path has been exercised in AE 2025; the current regression set still
needs live runs in the two explicit targets, AE 2025 and AE 2026. See
`SHADER.md` for the current contract and detailed validation status.

## Product idea

DynamicFX is an independent After Effects effect plug-in (AEX). The rough idea is to treat it like an effect whose exposed parameters include the source that defines its behavior, so a user or model can change the effect through normal After Effects controls.

## Relationship to ae-mcp

ae-mcp should treat DynamicFX like any other installed effect:

1. Find or apply the effect.
2. Write its source through an ordinary effect parameter.
3. Read its status and diagnostics.
4. Preview the result.

The existing `ae_exec` and effect-property mechanisms are the expected integration path. The ae-mcp issue is only a future integration tracker; DynamicFX implementation belongs here.

The implemented top-level parameters are:

- `Source`: a numeric slider whose expression carries the GLSL source.
- `Compile`: a button that forces a user-change/commit pass.
- `Status`: a numeric parameter whose label reports the current compile state.
- Reflected control pools: float, integer, checkbox, color, and point slots
  renamed and exposed from shader uniforms.

`SourceChannel` and legacy `SourceData` are hidden implementation parameters,
not user-facing controls.

## UI/render source transport

AE may keep UI and render work in separate projects even while concurrent
Multi-Frame Rendering is disabled. DynamicFX therefore does not rely on
sequence-local memory alone:

1. The main-thread idle hook observes each instance's `Source` expression and
   compiles changed source into a process-wide registry keyed by a stable,
   51-bit FNV-1a source hash.
2. A hidden, non-time-varying single `FloatSlider` parameter (`SourceChannel`)
   carries one atomically copied integer token between AE's UI and render
   projects. Its 51-bit source identity plus two state bits fit within the
   exactly representable f64 integer range.
3. Flattened sequence data persists explicit Unknown/Active/Inactive state,
   source identity, and the commit flag so projects can rebuild their registry
   entry.
4. Render-side copies consume `SourceChannel` plus flattened state. An
   explicitly missing, malformed, or non-compiling expression becomes clear
   and passes through; it cannot revive a stale shader.

The arbitrary `SourceData` parameter remains only for old-project parameter
ordering and callback compatibility. Its hash is not authoritative.

`%TEMP%\dynamicfx_source.txt` is disabled by default. It is available only as
an explicit diagnostic escape hatch when `DYNAMICFX_ENABLE_SIDECAR` is set to
`1`, `true`, or `on` before AE starts, and only while an instance's source
state is still unknown. It is process-global and not instance-safe, so it is
not a supported production transport.

## Host boundary

Packaging explicitly targets After Effects 2025 or 2026. Install into the
selected host's `Support Files\Plug-ins\DynamicFx` directory. Never use the
shared Adobe `Common\Plug-ins\7.0\MediaCore` directory because Premiere Pro
also scans it.

## Scope now

DynamicFX owns its GLSL compiler/runtime, wgpu renderer, reflected parameter
pool, persistence model, diagnostics, packaging, tests, and releases. It does
not require a dedicated MCP tool, private IPC channel, proxy service, or new
ae-mcp native primitive.
