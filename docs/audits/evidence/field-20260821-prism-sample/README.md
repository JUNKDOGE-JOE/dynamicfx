# Field evidence — prism sample: interrupted-render cache poisoning and copy flicker (2026-08-21)

Two field reports investigated on the user's `prism` sample project (`BugSample/"prism"_存在缺帧的情况.aep`), on the released 0.0.4 artifact. One is a **new** correctness defect (dropped frames), filed as issue [#7](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/7); the other is the visible symptom of the **already-open** issue [#6](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/6) / [TR-BIND-002](../../../TEST_MATRIX.md#tr-bind-002--copied-instance-corrupts-slot-mapping-field-defect).

- Finding A — **dropped/“missing” frames after an interrupted preview**: recorded as [TR-CACHE-001](../../../TEST_MATRIX.md#tr-cache-001--interrupted-render-poisons-the-frame-cache-field-defect), public issue [#7](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/7). New bug.
- Finding B — **copying the effect makes the original and the copy flicker**: same root cause as issue #6 (shared process-registry entry). Confirmed here; no separate fix track needed.

## Environment

- Windows 11 Pro 10.0.26200; **After Effects 2026 (26.3x87)** — the sample was saved by AE 2026 and will not open in AE 2025.
- Installed plug-in: the 0.0.4 release artifact `DynamicFx.aex` (`dynamicfx.dll` 8,544,768 B), SHA-256 `BFE1AB9FBE20F64E9098599C57F89B9D721C9DA8735F24F480384F85E5B858C3`, at `C:\Program Files\Adobe\Adobe After Effects 2026\Support Files\Plug-ins\DynamicFx\` — **installed hash re-verified equal** to the TR-REL-004 record this session.
- GPU `NVIDIA GeForce RTX 5080`, backend Dx12, driver `32.0.15.9621` (from the plug-in log).
- Project: comp `test` 1920×1080, 30 fps, 480 frames, 8-bpc. Two “solid” layers (1 and 2) each carry a `DynamicFx` instance running the same 1-pass, 12-param **Chromatic Dispersion** shader (an additive flare over fractal-noise + blurs); the two instances differ only in parameter values (L1 `Dispersion Distance` 0.05 / `Color Strength` 0.25; L2 0.15 / 1.0). L1 reads `Status: idle`, L2 `Status: compiled` — both render from the one shared registry entry (same `token=3763537382884453`).
- Date: 2026-08-21. Harness: warm AE session driven through the ae-mcp `/exec` panel; frames written by `comp.saveFrameToPng` after `app.purge(ALL_CACHES)` unless noted. Scripts in this folder.

---

## Finding A — interrupted preview poisons the frame cache (dropped frames)

### What the user reported

“奇怪的缺帧” — occasional missing/odd frames during normal interactive work on this project.

### Reproduction (this session)

`missing_frame_repro.py`: purge → run `mf_preview_interrupt.jsx` (8 cycles of *play, then move the CTI to interrupt* over work area 3.0 s + 4.0 s = frames 90…209) → sample the 120 work-area frames from whatever AE then holds in cache (no purge) → re-read the dipped frames, then purge and re-render them.

Observed 7 frames whose luminance sits well below both neighbours, from cache (`reproduction-report.txt`):

```
dips (luma > 0.006 below neighbour median):
  f111: 0.1740 vs 0.1896 (-0.0156)   f153: 0.1802 vs 0.1992 (-0.0190)
  f154: 0.1801 vs 0.1994 (-0.0192)   f155: 0.1912 vs 0.1996 (-0.0084)
  f156: 0.1915 vs 0.2007 (-0.0093)   f181: 0.1791 vs 0.2006 (-0.0215)
  f183: 0.1926 vs 0.2003 (-0.0077)
```

### The dropped content is exactly one DynamicFx layer

`copy_flicker_part2.py` phase G re-rendered each bad frame with individual layers hidden and compared, pixel-mean, to the cached bad frame:

```
 frame  bad luma |  none_hidden |    L1_hidden |    L2_hidden |  both_hidden   (mean |diff| vs cached bad frame)
   111    0.1740 | 0.01156(0.19) | 0.00000(0.174)| 0.01310(0.187)| 0.00258(0.171)  -> closest: L1_hidden
   153    0.1802 | 0.01426(0.20) | 0.00000(0.180)| 0.01868(0.191)| 0.00722(0.171)  -> closest: L1_hidden
   154    0.1801 | 0.01444(0.20) | 0.00000(0.180)| 0.01886(0.191)| 0.00720(0.171)  -> closest: L1_hidden
   181    0.1791 | 0.01559(0.20) | 0.00000(0.179)| 0.01960(0.193)| 0.00639(0.171)  -> closest: L1_hidden
```

`L1_hidden` reproduces each cached bad frame to **mean |diff| = 0.00000** — i.e. the cached frame is the correct composite with **Layer 1’s entire contribution absent**. In an additive-flare composite, a DynamicFx instance that delivers transparent black is indistinguishable from its layer being switched off, so the signature “layer 1 hidden” == “DynamicFx on layer 1 delivered transparent black for that frame”.

### It is a cache artefact, not a shader error

- **Persists on re-read** from cache (f111: sampled 0.174, re-read 0.174) and **recovers after `app.purge(ALL_CACHES)`** (f111 → 0.19). Other frames recovered on the first re-read because AE re-rendered that region.
- **Clean Render-Queue output is unaffected.** Three full RQ passes rendered earlier this session (`rq_baseline` 480 frames, `rq_pass2`/`rq_pass3` 120 frames over the work area) are **bit-identical** where they overlap: `max |diff| across the 3 passes = 0.0000`, 0 frames differ. Batch/`aerender` exports do not interrupt, so they never hit this path.
- **Correlates 1:1 with interrupt log lines.** Each interrupted preview adds `smart render input checkout failed: InterruptCancel` pairs to the plug-in log (38 → 48 → 58 over the session). See `dynamicfx.log.interrupt-window.txt`.

### Cause (from the source)

`src/lib.rs`, `Command::SmartRender` arm:

```rust
let checked_out = match cb.checkout_layer_pixels(0) {
    Ok(v) => v,
    Err(e) => {                                  // InterruptCancel lands here
        diag::log(&format!("smart render input checkout failed: {e:?}"));
        None                                     // ← cancel flattened to “no input”
    }
};
…
if let Ok(Some(mut out_layer)) = cb.checkout_output() {
    if let Some(in_layer) = &checked_out {
        AdobePluginInstance::render(self, plugin, in_layer, &mut out_layer)?;
    } else {
        out_layer.buffer_mut().fill(0);          // transparent black
    }
}
…
Ok(())                                           // ← reports success to AE
```

When the user interrupts a preview, `checkout_layer_pixels(0)` returns `InterruptCancel`. The arm logs it and sets `checked_out = None`, which then takes the **same branch meant for a genuine empty input** (an adjustment layer over nothing, per ADR-0030 §5): it fills the output with transparent black and returns `Ok(())`. Returning `Ok` tells After Effects the layer rendered successfully, so AE **caches** the transparent-black layer output. In the comp that frame is then missing that DynamicFx layer’s flare, and the poisoned frame survives in the cache until the region is re-rendered or the cache is purged.

The defect is that a **cancel** and an **empty input** are conflated. A cancel must propagate; only a real empty input should become transparent black. This is the CLAUDE.md rule “never convert a failure into pass-through without a stable diagnostic code and test expectation”, applied to `InterruptCancel`.

### Fix direction (runtime; not applied here)

In the `SmartRender` arm, treat an aborted input checkout as an abort: return `Err(Error::InterruptCancel)` (AE then discards the frame instead of caching it) rather than falling into the `fill(0)` branch. Keep `fill(0)` only for the true `Ok(None)` no-input case. A harness leg should assert that an interrupted work-area render leaves no cached frame missing a layer (compare cache-served vs purged samples over an interrupted preview). Runtime change — to be delegated and verified on the host.

### Workaround for the user today

Purge the cache (`Edit ▸ Purge ▸ All Memory & Disk Cache`) after interrupting a preview, or export through the Render Queue / `aerender` (batch renders never interrupt, so they are always clean).

### Artifacts

- `cached-frame-missing-layer1.png` — f111 (cached vs purged) and f181 (cached vs re-read) side by side with a ×6 difference map; the difference is exactly Layer 1’s flare.
- `reproduction-report.txt` — full run log: purge → interrupted preview → sampling → persistence/recovery table.
- `dynamicfx.log.interrupt-window.txt` — plug-in log for the reproduction window, epoch stamps converted to local time.
- `missing_frame_repro.py`, `mf_preview_interrupt.jsx`, `frame_stats.py` — the harness.

---

## Finding B — copying the effect makes both instances flicker (issue #6)

### What the user reported

“效果不能直接复制，会导致原效果和复制效果闪烁” — copy/pasting a DynamicFx instance makes the original and the copy flicker.

### What this is

The visible symptom of the open defect [#6](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/6) / TR-BIND-002: two instances of the same source share **one** process-registry entry (keyed by the source fingerprint only), and each render/resolve makes them take turns owning and rebuilding it. TR-BIND-002 already documents the severe form — when one instance’s `BindingPlan` is *migrated* (slot order ≠ declaration order after in-place source edits), the shared entry permutes the other instance’s parameter roles, and both flicker while alternating.

### What this session adds

On this sample the shader is simple (1 pass, 12 params) and both instances carry **identical, fresh declaration-order plans**, so I could test the copy path without the plan-migration confound. `copy_flicker_repro.py` / `copy_flicker_part2.py`, driven by script: copy/paste the effect onto a third layer, re-commit the copy’s `Source`, edit a parameter on the copy and on the original, then `layer.duplicate()`.

- No corruption: all instances kept their **own** values throughout (after paste L1 dist 0.05 / L2 0.15 / L3 0.15; after edits L2 0.20 / L3 0.30; `describe()` per second in the reports), and 5 purged renders at each step were **bit-identical** (`max |diff| = 0.0000`).
- The registry still churned: `resolved from process registry` 67 → 72 and `pipelines built` 71 → 76 as the instances alternately rebuilt the shared entry (`reproduction-report.txt`).

So the **corruption** half of #6 needs a migrated plan; the **flicker** half is the shared-entry contention and is what the user sees. A scripted, purged single-frame render does not expose the flicker because it has no live re-render timing — the flicker is an interactive-timing artefact of the same shared entry. This strengthens #6’s cause and fix (the registry value must not carry per-instance plan state, or must be keyed by plan identity) and needs **no separate fix track**.

### Workaround for the user today

As in #6: don’t copy/paste a DynamicFx instance; add a fresh DynamicFx and paste only the `Source` expression into it. To clear an already-flickering pair, remove and re-add the effect with the same expression.

### Artifacts

- `copy_flicker_repro.py`, `copy_flicker_part2.py` — the copy/paste/duplicate/edit harness and the layer-hidden decomposition (phase G, reused by Finding A).
- `reproduction-report.txt` — the per-second instance readouts and registry counters through every step.
