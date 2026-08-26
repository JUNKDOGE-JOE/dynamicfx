# DynamicFX Reference

## @param syntax and type mapping

Full entry syntax: `// @param <identifier> [entry ...]`, placed near the corresponding field inside `FxUniforms`.

| Entry | Form | Applies to |
|---|---|---|
| `label:"text"` | quoted display name | all types (ignored by host for `layer`/`gradient`/`point3d`/`path` — see below) |
| `min:<n>` | number | numeric types — slider (drag) range, give both; not a clamp: the shader receives the row's value as shown, so `clamp()` in the shader if a hard limit matters (ADR-0037) |
| `max:<n>` | number | numeric types — see `min:` |
| `default:<value>` | number (1-4 components) or `#RRGGBB`/`#RRGGBBAA` | numeric types / `hint:color` only for hex form; 6-digit hex implies alpha 1.0 |
| `alias:<old-id>` | identifier | any — preserves keyframes across a parameter rename |
| `hint:` | one of `angle`, `color`, `bool`, `layer`, `gradient`, `point3d`, `path`, `canvas` | see mapping table below; `canvas` (0.0.6+) additionally makes the float the canvas boundary — exactly one per source (`E55`), float only (`E56`) |

Parsing is fail-closed: any unrecognized or misspelled entry key rejects the entire `@param` definition (compile error), it is never silently dropped. An `@param` comment that matches no declared uniform field is ignored (no error).

### Type → control mapping and pool capacities

| GLSL type | `hint` | AE control | Pool capacity |
|---|---|---|---|
| `float` | (none) | Slider | 48 |
| `float` | `angle` | Angle dial | 8 |
| `float` | `canvas` | Slider **and** the canvas boundary (logical px per side, keyframeable) | uses a float slot |
| `int` | (none) | Integer slider | 8 |
| `int` | `bool` | Checkbox | 16 |
| `vec2` | (none) | Normalized 0..1 point | 12 |
| `vec3` | (none) | Color | 12 |
| `vec3` | `point3d` | 3D point (x, y normalized; z in pixels) | 8 |
| `vec4` | (none) | Color + opacity | Consumes 1 color slot + 1 float slot |
| — | `layer` | Layer chooser | 4 |
| — | `gradient` | Gradient (256x1 LUT) | 2 |
| — | `path` | Mask chooser | 2 |

Exceeding any single pool's ceiling rejects the whole shader with `E32` — pools are independent per type/hint, not shared.

**Per-pass banks (0.0.6+):** the capacities above are the SHARED (`Main`) pools. In a multi-pass graph, each of the first twelve passes additionally owns a private bank — 8 floats, 2 ints, 2 checkboxes, 3 colors, 2 points, 1 angle — used by parameters exclusive to that pass (grouping and allocation follow uniform-block membership; see SKILL.md → "Parameter groups"). Bank overflow spills to `Main` gracefully (Status reports "(N spilled to Main)"); only exhausting a shared pool itself is `E32`. `layer`/`gradient`/`point3d`/`path` never bank.

### Binding behavior for resource-hint parameters

- `hint:layer`: not a `FxUniforms` field. It appears as a pass **input** in the `@graph` manifest and gets a `texture2D` binding at 3/4/5 (in manifest order), like any other extra pass input.
- `hint:gradient`: backed by a 256x1 LUT texture. Sample with `texture(sampler2D(u_ramp, u_s), vec2(t, 0.5))`. Up to 8 color stops per gradient.
- `hint:path`: backed by an Nx2 `Rgba32Float` vertex texture. Read with `texelFetch` by vertex index; `textureSize(...).x` gives the vertex count. Closed paths repeat the first vertex at the end (a rectangle mask = 5 vertices). When no path is assigned, the texture defaults to 1x2, all zero. Requires GPU support for `FLOAT32_FILTERABLE`.

### Host UI limitations (known, not bugs)

`layer`, `gradient`, `point3d`, and `path` controls display a generic host-assigned name (e.g. "Mask 01") in the Effect Controls panel instead of the shader's `label:` text — After Effects ignores rename requests for these four control types specifically. This does not affect values, keyframes, or rendering.

**Panel structure (0.0.6+):** `Setup` (head controls, always expanded) → `Main` (collapsed; shared parameters, each gradient as a nested sub-group) → per-pass groups named after the envelope's pass names, hidden while empty. A `Setup` header occasionally shows collapsed on first draw (recorded cosmetic observation). One instance's group/label names are per-instance UI state: instances saved under older releases keep the default names of THEIR era until re-bound.

### Canvas and layer bounds (known, not a bug)

The render target is the layer's own frame: `v_uv` spans it, `u_resolution` is its size, and the effect cannot produce pixels outside it — AE composites only what the layer's buffer holds. Consequences, all measured on 0.0.5 / AE 2026 (record TR-BOUNDS-001 in the DynamicFX repository):

- glow / halo / shadow / bloom / displacement overshoot is clipped at the layer bounds; on a tight footage layer the result looks like a square cut;
- since 0.0.6 an upstream buffer-expanding effect (Grow Bounds) DOES enlarge an undeclared shader's canvas, and a `hint:canvas` declaration overrides it; on pre-0.0.6 builds both are ignored — pixel-identical to the plain instance;
- a padded precomp does: precompose the source with transparent margin ≥ the shader's maximum pixel reach and apply DynamicFx to the precomp. Padding also changes the look near the edges, because the passes then sample the margin instead of clamp-to-edge pixels.

Canvas expansion shipped in 0.0.6 (ADR-0039, `hint:canvas`); the precomp remains the answer only on older installs.

## Capacity limits (full)

| Limit | Value |
|---|---|
| Passes per graph | 16 |
| Inputs per pass | 4 |
| Intermediate textures | 15 |
| Source size | 4 MiB |
| `float` parameters | 48 |
| `float` + `hint:angle` parameters | 8 |
| `int` parameters | 8 |
| `int` + `hint:bool` parameters | 16 |
| `vec2` parameters | 12 |
| `vec3` (color) parameters | 12 |
| `vec3` + `hint:point3d` parameters | 8 |
| `vec4` parameters | shares color (12) + float (48) pools, 1 slot each |
| `hint:layer` parameters | 4 |
| `hint:gradient` parameters | 2 (max 8 stops each) |
| `hint:path` parameters | 2 |
| `prev` feedback window | default 16, max 64 |

## Error codes

| Code | Meaning | Typical fix |
|---|---|---|
| E6 | Envelope syntax violation (with line number); or reserved/resource name misuse (using `input`/`output`/`prev`, or a `layer`/`gradient`/`path`-fed name, as a pass name or output) | Re-check `@dynamicfx`/`@graph`/`@end`/`@pass`/`@endpass` structure and pass-name/manifest consistency; rename the offending pass |
| E7 | `prev` feedback input combined with a `layer` or `path` input in the same graph | Remove one; split into separate graphs/passes if both are needed |
| E18 | A `texture2D` binding declared in a pass body that the `@graph` manifest doesn't list as an input | Add the missing input to the pass's manifest line, matching binding order |
| E32 | A parameter type/hint pool exceeded its capacity ceiling | Merge, reduce, or remove parameters of that type — see pool table above |
| E19 | `@param` line malformed (bad entry key), or the same name annotated in more than one pass | Fix the entry key, or keep exactly one `@param` line per name across the whole source (repeat only the uniform member) |
| E53 | PublicationPending — shader compiled successfully but has not yet been published to the render pipeline; not renderable yet | Wait a few seconds; for scripts, poll readiness instead of assuming immediate availability (see below) |
| E54 | A gradient's stop values read back malformed | Fix the stop rows (positions monotone 0..1); the resource binds transparent black until then |
| E55 | More than one `hint:canvas` declaration in the source | Keep exactly one canvas authority |
| E56 | `hint:canvas` on a non-float parameter | Move the hint to a scalar float |
| E57 | Requested canvas exceeds the GPU texture limit | Runtime falls back to the layer frame and logs; lower the expansion (or the upstream Grow Bounds) |

Submitting a shader has a short unready window (a few seconds) after which renders during that window silently pass the original layer through unmodified.

## Diagnostics

- **Status line**: Effect Controls panel, truncated to 31 characters.
- **Show Full Status** button: opens the full error text including the E-code.
- **Log file**: `%TEMP%\dynamicfx.log`.
- Additional verbosity/perf environment toggles: `DYNAMICFX_VERBOSE_LOG`, `DYNAMICFX_PERF`.

## Scripting DynamicFx from ExtendScript

**Address rows BY NAME, never by numeric property index.** Indexes shifted once in 0.0.6 (group rows occupy positions: e.g. the StateToken moved from property 5 to 6) and names/matchNames are the stable identity — the runtime itself matches saved projects by id-derived matchName. Effect-parameter trees are FLAT in scripting: group rows appear as inert `NO_VALUE` entries, members stay top-level siblings.

```javascript
function dfxProp(fx, name) {
    for (var i = 1; i <= fx.numProperties; i++) {
        try { if (fx.property(i).name === name) return fx.property(i); } catch (e) {}
    }
    return null;
}
```

Reserved rows by name: `"Source (use expression)"`, `"Compile"`, `"State Token (internal)"`, `"Plan Token (internal)"`, `"Details"`; the Status row's NAME is the status text (starts `"Status: "` — find it by prefix). Bound shader parameters carry their `label:` text; unbound slots carry pool defaults (`"Float 01"`…). Two scripting gotchas measured on the host: `setValue` on a HIDDEN row (any unbound slot) throws "property or parent is hidden" — a script that lands there has a stale index or wrong name; and opening a project saved by an older AE year pops a version-conversion modal that blocks all scripting until dismissed.

### Readiness polling

The StateToken value is `(payload << 2) | state`; state `1` means renderable. Never block the main thread with `$.sleep()` while waiting — it starves the idle-time compile hook DynamicFX relies on. Poll instead with `app.scheduleTask`:

```javascript
function dfxIsReady(fx) {
    try {
        var t = dfxProp(fx, "State Token (internal)");
        return t.value % 4 === 1;
    } catch (e) { return false; }
}

var DFX_FX = [];
var DFX_TRIES = 0;

function dfxPoll() {
    var ready = 0;
    for (var i = 0; i < DFX_FX.length; i++) {
        if (dfxIsReady(DFX_FX[i])) { ready++; }
    }
    if (ready === DFX_FX.length) {
        dfxRender();
    } else if (++DFX_TRIES > 60) {
        alert("DynamicFX not ready: " + ready + "/" + DFX_FX.length);
    } else {
        app.scheduleTask("dfxPoll()", 500, false);
    }
}

dfxPoll();
```

The Status row's name holds the human-readable diagnostic string shown in the Effect Controls panel (find the row whose name starts with `"Status: "`).

### Reshaping an existing instance (regroup / new-source migration)

Bound parameters keep their slots across source edits (keyframes outrank regrouping), so applying a regrouped source to a live instance does not visually regroup it. The verified recipe (bit-identical renders on both AE years):

1. Capture every LABELED value: walk the flat tree, skip `NO_VALUE` rows and pool-default names — match defaults EXACTLY (`/^(Float|Int|Bool|Color|Point|Angle|Layer|Mask|Gradient) \d\d$/` etc.), never by prefix: user labels like "Color Boost" begin with pool words.
2. Render a before-frame for comparison (`app.purge(PurgeTarget.ALL_CACHES)` first).
3. In an undo group: remove the effect, `addProperty('DynamicFx')`, set the new source expression, poll readiness.
4. Restore captured values by label; keyframed streams need per-key copies (capture `numKeys`/`keyValue`/`keyTime` before removal).
5. Render an after-frame; expect mean-abs-diff 0 when only blocks moved.

Old instances may also carry rows under PREVIOUS releases' default names (e.g. `"G01 01 Pos"` from 0.0.3-era stop naming) — AE preserves saved stream names per instance. Harmless leftovers; exclude them from capture by matching both old and current default patterns.

## Plugin install (quick reference)

- Download `DynamicFX-<version>-win-x64.zip` from GitHub Releases (contains `DynamicFx.aex`, `INSTALL.txt`, `SHA256SUMS.txt`), or build from source with `cargo build --release`.
- Install by copying `DynamicFx.aex` into the **version-specific** plug-ins folder, e.g. `C:\Program Files\Adobe\Adobe After Effects 2025\Support Files\Plug-ins\DynamicFx\DynamicFx.aex`, then restart AE. The effect appears under Effect > DynamicFx > DynamicFx.
- **Never install into the shared `Common\Plug-ins\7.0\MediaCore` folder** — Premiere Pro scans that folder too.
- From a source build, run `scripts\install.bat <2023|2024|2025|2026>` as administrator. The version argument is required precisely so the plug-in is never copied to MediaCore; the script refuses to overwrite while AfterFX/aerender is running and never deletes, moves, launches, or terminates anything.

## Platform support matrix

| Platform / host | Status |
|---|---|
| Windows | Supported |
| macOS | Planned, not yet available |
| After Effects 2025 | Verified |
| After Effects 2026 | Verified |
| After Effects 2024 | Unverified |
| After Effects 2023 | Blocked (not supported) |
