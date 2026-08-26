# Field evidence — the shader canvas is the layer frame (2026-08-21)

Field question on the user's `AppleVison` project (AE 2026): the `apple-thermal`
shader renders fully when DynamicFx sits on the 1024×1024 padded precomp
`apple-logo (pad 1024)`, but is clipped to a hard square when the same effect is
applied directly to the 512×512 `apple-logo.png` footage layer. Is that a plugin
defect or a shader-authoring defect?

**Answer: neither.** It is the ABI v1 canvas contract. `v_uv` ∈ [0,1]² maps onto
the layer's own frame and `u_resolution` is that frame's size
([ADR-0011](../../../adr/0011-shader-abi-v1-core.md) §5, "the layer extent";
[ADR-0029](../../../adr/0029-logical-resolution-abi.md) only changes the units).
`src/lib.rs` `Command::SmartPreRender` checks out the input as `0,0,width,height`
of the layer and declares `max_result_rect` as that frame (unioned with the
request); the PiPL sets no `PF_OutFlag_I_EXPAND_BUFFER`. A shader cannot paint
outside its canvas, and the runtime never offers a canvas larger than the layer.
Canvas expansion is therefore a **missing feature that needs an ADR**, recorded in
`IMPLEMENTATION_STATUS.md` → *Recorded, not scheduled*. The record row is
[TR-BOUNDS-001](../../../TEST_MATRIX.md#tr-bounds-001--shader-canvas-is-the-layer-frame-field-observation);
public issue [#8](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/8) (enhancement, filed 2026-08-21).

## Measurement

Three fresh `DynamicFx` instances of the same source (the expression copied from
the user's tuned instance; all 94 copyable parameter values verified identical —
the instance runs the shipped `examples/apple-thermal.glsl` defaults) placed side
by side in a new 3072×1024 test comp `DFX 边界测试 (可删除)`, each layer centred
in its 1024-wide tile, rendered at t = 2.0 s after `app.purge(ALL_CACHES)`:

| Tile | Layer | Visible bbox | Visible px outside the 512×512 logo square |
|---|---|---|---|
| A | `apple-logo.png` 512×512 + DynamicFx | exactly 512×512 (x256..768, y256..768) | **0** |
| B | `apple-logo.png` 512×512 + **Red Giant GrowBounds** (Pixels 256) + DynamicFx | exactly 512×512 | **0** — pixel-identical to A |
| C | `apple-logo (pad 1024)` precomp + DynamicFx | 766×855 (x119..885, y92..947) | 253,110 (49.1 % of its visible pixels) |

- B ≡ A: an upstream effect that enlarges the layer buffer does **not** enlarge
  the DynamicFx canvas (A–C and B–C diffs are identical: mean |diff| 13.80,
  437,288 px differing by > 8).
- C's halo reaches at most 179 px beyond the logo at these settings (top 164,
  bottom 179, left 137, right 117), so the 256 px pad suffices.
- The inside changes too: inside the logo square A differs from C by mean
  |diff| 20.07/255 — the blur/halo passes sample the transparent margin instead
  of clamp-to-edge pixels.
- Adobe's own *Grow Bounds* (`ADBE Grow Bounds`) is **not present** in this AE
  2026 install (1,536 effects enumerated; the only match is
  `Red Giant GrowBounds`, Trapcode's free utility), so the Red Giant one stood in.
  The mechanism under test — an upstream effect expanding the buffer handed to
  the next effect — is the same.

Render: `abc-t2.0-a-clipped-b-growbounds-c-padded-precomp.png` (left A, middle B,
right C). Numbers: `analysis-report.txt`, produced by `analyze.py` from that PNG.

## Environment

- Windows 11 Pro 10.0.26200; **After Effects 2026 (26.3x87)**; 8-bpc project.
- Installed plug-in: the released **0.0.5** artifact `DynamicFx.aex`, 8,564,736 B,
  SHA-256 `FF1197D9DB09CCE81D8E2CE26AC4FD3FA752FEA8705109FBCA8D9725670A0344`, at
  `C:\Program Files\Adobe\Adobe After Effects 2026\Support Files\Plug-ins\DynamicFx\`
  — hash re-verified this session, equal to
  [TR-REL-005](../../../TEST_MATRIX.md#tr-rel-005--005-release-verification).
- Working tree on `main` at `8da89d8` (documentation only touched).
- Driven through the warm ae-mcp session (`ae_exec` for setup and
  `app.purge(PurgeTarget.ALL_CACHES)`, `ae_previewFrame` → `comp.saveFrameToPng`
  for the frame). The user's project file is not part of the repository; the
  test comp was left in it, clearly named, in three undo groups
  (`DFX bounds test: setup`, `setup 2`, `copy values`).

## What a runtime change would touch

Listed so the eventual ADR starts from the measured surface rather than from
memory; none of this was changed.

- An explicit, **stable** expansion amount (a new persistent parameter, e.g.
  `Canvas Expansion (px)`) — a topology change under
  [ADR-0013](../../../adr/0013-paramid-grammar-and-pools.md). Deriving the
  canvas from AE's request instead was rejected on sight: the ROI request varies
  per render (a few pixels for `sampleImage`, see TR-M7-004), and a canvas that
  moves with it breaks WYSIWYG and every `uv`-space shader.
- The canvas definition itself: what `u_resolution`/`v_uv` span, and where
  `input` sits inside the expanded frame ([ADR-0011](../../../adr/0011-shader-abi-v1-core.md) §5,
  [ADR-0029](../../../adr/0029-logical-resolution-abi.md)).
- [ADR-0030](../../../adr/0030-layer-input-parameters.md) §4 comp-space
  alignment of `hint:layer` inputs must keep holding on the expanded canvas.
- Temporal history extents ([ADR-0023](../../../adr/0023-temporal-seek-reset.md),
  [ADR-0024](../../../adr/0024-history-format-policy.md)) and extents inside the
  identity/cache keys ([ADR-0007](../../../adr/0007-identity-and-cache-boundaries.md),
  [ADR-0017](../../../adr/0017-hash-domains.md)).
- The SmartFX protocol: `PF_OutFlag_I_EXPAND_BUFFER` in the PiPL, the
  `max_result_rect` declaration, and the output placement math in
  `Command::SmartRender` (origin − request window), plus the downsample path.
- Why an upstream Grow Bounds is invisible today, mechanically: `SmartPreRender`
  overwrites the checkout rect with `0,0,in_data.width(),in_data.height()` —
  the *source layer's* dimensions, which the SDK keeps at the layer size even
  when an upstream effect has enlarged the buffer — so the added margin is
  never checked out, the GPU canvas is the checked-out world (512×512), and
  the write-back fills the rest of the output world with transparent black.
  The stable signal a future design can use is the upstream's own extent in
  the `PF_CheckoutResult` that `checkout_layer` returns (`max_result_rect`),
  which does not move with the downstream ROI request; the current code
  discards it (`Ok(_)`). Not measured — read from `src/lib.rs`; the
  pixel-identity of tiles A and B is the observable consequence.

## User-side answer today

Precompose the source into a comp larger by at least the shader's maximum pixel
reach on every side and apply DynamicFx to the precomp (what the user's project
already does). Expose the reach as pixel `@param`s so the needed margin is
explicit. Documented in the `dynamicfx-shaders` skill.
