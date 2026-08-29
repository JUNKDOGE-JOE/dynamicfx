# The gradient editor — implementation method (shelved capability)

**State (2026-08-29):** complete and host-verified on AE 2025 + 2026 ([TR-0042-001](TEST_MATRIX.md#tr-0042-001--gradient-editor-adr-0042-probe-leg-prelude-and-editor-host-legs)), then **shelved by user decision** — default builds ship the 0.0.6 surface with no custom UI. The contract is [ADR-0042](adr/0042-gradient-editor-presentation-contract.md); this page records *how* it is built and how to bring it back.

## Re-enable

```
cargo build --release --features editor
```

One cargo feature drives the whole custom-UI atom, together and never separately (ADR-0042 Decision 3):

- the PiPL `CustomUI` out-flag and the PiPL subversion (6 with editor, 5 without) — `build.rs` reads `CARGO_FEATURE_EDITOR`;
- every `ParamUIFlags::CONTROL` / `ui_width` / `ui_height` declaration;
- the `register_ui` call.

The default (editor-off) build declares byte-for-byte the shipped 0.0.6 out-flags/topology words. A unit boundary test pins the coupling in both flavors; `cargo test` and `cargo test --features editor` must both stay green.

## The safety invariant this exists around

Declaring `PF_PUI_CONTROL` on any parameter **without calling `PF_REGISTER_UI` in `params_setup` kills After Effects**: AE's Effect Controls paint path dereferences the missing registration record (null), Windows escalates to `C000041D` inside the window-procedure callback, and WER sees nothing — the 2026-08-15 silent-death signature (TR-CUI-001 Rounds 2–4, reproduced 3/3 and read from a debugger). The implementation therefore:

- derives the CONTROL declarations and the registration condition from **one** table (`host::params::control_surface`); `requires_custom_ui()` gates the `register_ui` call at the end of `params_setup`, and its failure fails `params_setup` (visible refusal to load, never a declared-unregistered state);
- keeps the fatal arm reachable only through the `spike/probe` instrument, never a shippable artifact.

`OutFlags::CustomUI` is bit 15 — the code-page-vulnerable high byte of the PiPL word; `repair_pipl_resource` (root `build.rs`) already re-emits the bytes file-referenced, and the built artifact's `eGLO` word is decoded per flavor as a release check (editor `0x06008644`, off `0x06000644`).

## Shape

- **Presentation only.** Gradient values live entirely in the ADR-0033 ordinary parameters (`Stops`, per-stop `Position`/`Color`/`Alpha`). The editor reads them for painting and writes them through the host's normal commit path; removing it costs nothing but the canvas. Projects interchange between flavors with zero value loss — measured both directions (the canvas stream is valueless, so the off flavor silently drops it; ADR-0040 id matching re-attaches every real stream).
- **Topology:** one `ParamKey::GradientCanvas(g)` per gradient (feature-gated), declared as the first row of each `Gradient NN` sub-topic — a standard **Color** param (`Pixel8` default) + `CONTROL`, 200×80, `CANNOT_TIME_VARY`, label `Preview`. Color-not-Float matters: a Float+CONTROL canvas rests collapsed, a Color+CONTROL canvas paints on apply (measured). The canvas joins the golden id table (editor flavor) and the visibility predicate that hides gradient rows while the anchor is unbound. It is in no pool, no `BindingPlan`, no identity domain, no `DFXS` field.
- **Draw** (`src/lib.rs` `paint_gradient_preview`): on `Event::Draw` for a canvas, map `param_index` → key (`key_for_param_index`), read the gradient through the same `read_gradient` the renderer uses, bake the 256-sample LUT, and paint per-column `paint_rect` slices into **`current_frame`** — never the declared size (AE stretches the control to the panel column width; declared 200 renders ≈280). Alpha composites over mid-gray; stop ticks draw at `RampGeometry::position_to_x` with luminance-contrast color. An invalid gradient paints a flat dark-red degraded fill + one `log::warn!` — never an error out of the event (ADR-0042 Decision 7).
- **Geometry** is one pure struct (`gradient::RampGeometry`, x ↔ position) shared by draw and hit-test, plus pure `nearest_stop` (±6 px, nearest wins, tie → lower index) and `clamp_position` (monotone against the live neighbors — the editor cannot author `E54`). All unit-tested, including the 280-px stretch case.
- **Editing** (stage 2): `Click` on a canvas hit-tests the live ticks and stores the grab in one process-global `Mutex<Option<GrabbedStop>>` (session-local, cleared on gesture end, cross-canvas click, or any failure); every `Drag` event re-reads the live stops, clamps, and commits the grabbed stop's `Position` via `Parameters::get_mut → as_float_slider_mut → set_value` — the crate couples that setter to `PF_ChangeFlag_CHANGED_VALUE`, so undo, keyframing (a drag at the CTI writes a keyframe exactly like a hand edit) and rendering all follow the host's ordinary semantics. `set_send_drag(true)` keeps the drag stream coming; the `last=true` write is the guaranteed terminal state.
- Events are registered for the Effect Controls window only (`CustomEventFlags::EFFECT`).

## Measured behavior notes

- **Viewport follows a drag live; the canvas repaints when the drag ends.** `EventOutFlags::UPDATE_NOW` does not change this — AE repaints rows whose values changed, and the canvas value never changes by design. Open follow-up: call `update_param_ui` on the canvas from the drag path (probe-testable).
- The host's ExtendScript **Redo** command id is **2035** on this machine's AE 2025 (17 is a silent no-op) — relevant to any scripted undo probing.
- **Scripted `setValue` cannot grow the visible stop set**: rows hidden by the DynStream mechanism refuse `setValue` by design, and a scripted write to `Stops` does not reach the plugin as a commit (the M0 finding), so the visibility never re-applies without a real hand gesture on the slider. Measured 2026-08-29 on a field project; recorded in TR-0042-001.
- Editing colors/alpha/`Stops` stays on the ordinary rows in this version; the color-picker call (`pf::suites::App::color_picker_dialog`) is probe-verified host-safe from the Click path but not wired to a gesture.

## Verified artifacts (identity by hash; build is not byte-reproducible)

- editor `BB39AC1156F52F02F79B3E8FB0AFC6BEBC01B450C91F9A066A151C5636994E3E` (8,643,584 B) — the TR-0042-001 pass artifact, archived at `scripts/out/hostpass-0042/DynamicFx-editor.aex` (local, gitignored).
- off `AC85FA0D3AEEC133CF69FB1BE43966E2EE2B61322C2BD0643403B6B10667265E` (8,614,400 B) — same tree, 0.0.6-shape words.

Shipping the editor later means: re-run the TR-0042-001 release gate on the artifact that ships (both AE years), per the ADR-0042 verification obligations — the shelved evidence stands for the shelved artifact, not for a rebuilt one.
