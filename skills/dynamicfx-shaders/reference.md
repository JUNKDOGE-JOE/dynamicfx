# DynamicFX Reference

## @param syntax and type mapping

Full entry syntax: `// @param <identifier> [entry ...]`, placed near the corresponding field inside `FxUniforms`.

| Entry | Form | Applies to |
|---|---|---|
| `label:"text"` | quoted display name | all types (ignored by host for `layer`/`gradient`/`point3d`/`path` — see below) |
| `min:<n>` | number | numeric types |
| `max:<n>` | number | numeric types |
| `default:<value>` | number (1-4 components) or `#RRGGBB`/`#RRGGBBAA` | numeric types / `hint:color` only for hex form; 6-digit hex implies alpha 1.0 |
| `alias:<old-id>` | identifier | any — preserves keyframes across a parameter rename |
| `hint:` | one of `angle`, `color`, `bool`, `layer`, `gradient`, `point3d`, `path` | see mapping table below |

Parsing is fail-closed: any unrecognized or misspelled entry key rejects the entire `@param` definition (compile error), it is never silently dropped. An `@param` comment that matches no declared uniform field is ignored (no error).

### Type → control mapping and pool capacities

| GLSL type | `hint` | AE control | Pool capacity |
|---|---|---|---|
| `float` | (none) | Slider | 48 |
| `float` | `angle` | Angle dial | 8 |
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

### Binding behavior for resource-hint parameters

- `hint:layer`: not a `FxUniforms` field. It appears as a pass **input** in the `@graph` manifest and gets a `texture2D` binding at 3/4/5 (in manifest order), like any other extra pass input.
- `hint:gradient`: backed by a 256x1 LUT texture. Sample with `texture(sampler2D(u_ramp, u_s), vec2(t, 0.5))`. Up to 8 color stops per gradient.
- `hint:path`: backed by an Nx2 `Rgba32Float` vertex texture. Read with `texelFetch` by vertex index; `textureSize(...).x` gives the vertex count. Closed paths repeat the first vertex at the end (a rectangle mask = 5 vertices). When no path is assigned, the texture defaults to 1x2, all zero. Requires GPU support for `FLOAT32_FILTERABLE`.

### Host UI limitations (known, not bugs)

`layer`, `gradient`, `point3d`, and `path` controls display a generic host-assigned name (e.g. "Mask 01") in the Effect Controls panel instead of the shader's `label:` text — After Effects ignores rename requests for these four control types specifically. This does not affect values, keyframes, or rendering.

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
| E53 | PublicationPending — shader compiled successfully but has not yet been published to the render pipeline; not renderable yet | Wait a few seconds; for scripts, poll readiness instead of assuming immediate availability (see below) |

Submitting a shader has a short unready window (a few seconds) after which renders during that window silently pass the original layer through unmodified.

## Diagnostics

- **Status line**: Effect Controls panel, truncated to 31 characters.
- **Show Full Status** button: opens the full error text including the E-code.
- **Log file**: `%TEMP%\dynamicfx.log`.
- Additional verbosity/perf environment toggles: `DYNAMICFX_VERBOSE_LOG`, `DYNAMICFX_PERF`.

## Scripting: readiness polling

The effect's 5th property is a StateToken (`(payload << 2) | state`). State `1` means renderable. Never block the main thread with `$.sleep()` while waiting — it starves the idle-time compile hook DynamicFX relies on. Poll instead with `app.scheduleTask`:

```javascript
function dfxIsReady(fx) {
    try { return fx.property(5).value % 4 === 1; } catch (e) { return false; }
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

Property 4 (`Status`) holds the human-readable diagnostic string shown in the Effect Controls panel.

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
