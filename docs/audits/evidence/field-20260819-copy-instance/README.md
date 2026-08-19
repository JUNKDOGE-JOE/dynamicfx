# Field evidence — copy/paste of an effect instance corrupts the parameter slot mapping (2026-08-19)

Observed on the user's real project, not in a harness. Recorded as [TR-BIND-002](../../../TEST_MATRIX.md#tr-bind-002--copied-instance-corrupts-slot-mapping-field-defect); public issue [#6](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/6).

## Environment

- Windows 11 Pro 10.0.26200; After Effects 2025 (25.6.6x4); installed plug-in: the 0.0.4 release artifact per TR-REL-004 (installed hash not re-verified in this session); GPU `NVIDIA GeForce RTX 5080`, backend Dx12, driver `32.0.15.9621` (from the plug-in log).
- Project: 8-bpc; comp `AppleVison` 3840x2160; effect on a 1024x1024 precomp layer; shader `examples/apple-thermal.glsl` (10 passes, 22 params: 18 float, 2 angle, 1 bool, 1 gradient).
- History of the instance that matters: it had been re-bound many times during authoring (source edited in place ~15 times), so its `BindingPlan` was a *migrated* plan — early ParamIds kept their first slots (`speed`=F01 … `grain`=F13) and later-added IDs were appended (`thickness`=F14, `wall_heat`=F15, `line_heat`=F16, `halo`=F17, `halo_radius`=F18; `bias_angle`=A01, `extrude_angle`=A02). Its GLSL declaration order is different: `speed, heat_depth, edge_soft, thickness, wall_heat, rim_heat, line_heat, core_temp, flow_scale, flow_amount, turbulence, contrast, bias_amount, halo, halo_radius, bloom, bloom_radius, grain; extrude_angle, bias_angle`.

## What the user did and saw

1. Copied the DynamicFx effect to a layer in another comp (the plug-in log shows renders at 512x512 and 31x31 appearing at that time).
2. The pasted instance rendered as **black-and-white noise**; after pressing `Compile` it rendered as **coloured noise**.
3. Back in the original comp the original instance **flickered**, then settled on a wrong image: the whole logo pink-white with fine noise and a thin blue/orange edge line (user screenshot, kept out of the record).

## What was measured on the original instance afterwards (ae-mcp `/exec` readback, values by display name)

```
Flow Speed 2.07 | Heat Depth (px) 56 | Edge Line (px) 5 | Wall Thickness (px) 0.5 | Wall Heat 0.03 |
Rim Warmth 1.6 | Edge Line Heat 1 | Core Temp 1 | Flow Scale 1 | Flow Amount 0.15 | Turbulence 0.4 |
Contrast 3 | Heat Bias 0 | Outer Glow 2 | Outer Glow Radius (px) 4 | Softness 0.6 | Softness Radius (px) 1 |
Grain 0.3 | Use Custom Ramp 0 | Wall Direction 180 | Heat Bias Direction 205
Status: compiled: 10 passes, 22 params — no diagnostic, no error.
```

The values the instance had before the incident (set by script, verified by render):

```
Flow Speed 1 | Heat Depth 56 | Edge Line 5 | Rim Warmth 0.5 | Core Temp 0.03 | Flow Scale 1.6 | Flow Amount 1 |
Turbulence 0.6 | Contrast 1 | Heat Bias 0.15 | Softness 0.4 | Softness Radius 24 | Grain 0 | Wall Thickness 140 |
Wall Heat 1 | Edge Line Heat 0.6 | Outer Glow 1 | Outer Glow Radius 70 | Wall Direction 205 | Heat Bias Direction 180
```

Reading the "after" table by **slot** instead of by name explains every value: the labels moved to the fresh declaration-order mapping while the AE streams kept their old-slot values —
F04 (was `rim_heat` 0.5) is now labelled *Wall Thickness* → 0.5; F05 (was `core_temp` 0.03) → *Wall Heat* 0.03; F06 (`flow_scale` 1.6) → *Rim Warmth* 1.6; F07 (`flow_amount` 1) → *Edge Line Heat* 1; F09 (`contrast` 1) → *Flow Scale* 1; F10 (`bias_amount` 0.15) → *Flow Amount* 0.15; F11 (`bloom` 0.4) → *Turbulence* 0.4; F12 (`bloom_radius` 24) → *Contrast*, **clamped to the new slider max 3**; F13 (`grain` 0) → *Heat Bias* 0; F14 (`thickness` 140) → *Outer Glow*, **clamped to max 2**; F15 (`wall_heat` 1) → *Outer Glow Radius*, **clamped to min 4**; F16 (`line_heat` 0.6) → *Softness* 0.6; F17 (`halo` 1) → *Softness Radius* 1; F18 (`halo_radius` 70) → *Grain*, **clamped to max 0.3**; A01 (`bias_angle` 180) → *Wall Direction* 180; A02 (`extrude_angle` 205) → *Heat Bias Direction* 205. (`Flow Speed` 2.07 and `Core Temp` 1 do not fit the permutation exactly; the user had been dragging controls while investigating.) The four clamped-to-range values are the signature: values stayed in their slots, the roles moved.

## Log excerpt

[`dynamicfx.log.tail.txt`](dynamicfx.log.tail.txt): repeated `definition resolved from process registry` → `pipelines built … at 1024x1024` alternating with builds `at 512x512` / `31x31` (the pasted instance), each followed by `idle slot ui applied: 21 bound, 0 defaults written` on the original — i.e. the original instance re-applied slot UI against a definition it did not compile.

## Repair applied to the project

Removed the effect and added a fresh DynamicFx with the same expression: the fresh instance binds in declaration order (which is what the registry currently holds), all `@param` defaults were written, and the render returned to normal ([`after-rebuild_t2.png`](after-rebuild_t2.png)). Undo group: "DynamicFX: rebuild effect instance (fresh binding)".

## Reading

Two instances of the *same source* (same DefinitionHash) with *different* BindingPlans cannot both be right if the render path resolves "the definition" from a process-wide registry keyed by the hash: whichever compiled last owns the entry, and the other instance's streams are read through the wrong slot table. The M3 "duplicate isolation" probe passed because both duplicates carried identical (fresh) plans. The per-instance plan persisted in the sequence data must stay authoritative for that instance's stream reads and slot UI, or the registry key must include the plan identity.
