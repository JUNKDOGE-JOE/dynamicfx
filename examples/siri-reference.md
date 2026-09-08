# Continuous Siri ribbon study

[`siri-reference.glsl`](siri-reference.glsl) is an eight-pass analytic ribbon
and glass study. Four colored sheets rotate through a shared wave, change
their projected width, and retain continuously varying phase offsets. Every requested time
is evaluated directly. There are no stored motion samples, texture lookups
for animation, frame interpolation, or five-second endpoint clamps.

Apply the shader to an adjustment layer above the background. It requires
no motion-data footage. The background remains an ordinary image or precomp.
The 1170 × 2532 study uses **Glass Center** `[582, 179.4]` and **Pill Center** `[582, 93]`.
Activation goes from 0 at 0 s to 1 at 0.4 s; Wave Amp goes from 0 at 0 s to 1
at 0.1 s. Geometry, breathing, phase offset, speed, sheet separation, color,
refraction and reflected light remain ordinary editable DynamicFx controls.
The delivery composition runs at 60 fps and 16 bpc. Changing frame rate
changes temporal sampling; it does not select a different animation table.

**Ribbon Flow** sets angular speed; **Ribbon Phase** sets initial orientation.
**Phase Settling** controls only the small initial excess separation. Blue/cyan
and yellow/red retain separate, slowly varying phase gaps: about 25–46 degrees
after settling at default settings. Their drift rates differ, so a temporary
overlap does not turn into two permanently locked pairs. **Sheet
Separation** and **Ribbon Spread** control spatial separation; **Ribbon Tail**
and **Ribbon Edge** control the projected soft body and its sharper boundary.
The phase equations are a designed approximation, not a recovered Apple model.

**Glass Breath** now uses a complete cosine cycle. The expansion and contraction
join smoothly, with no flat half-cycle between pulses. Emit and glass passes
use the same curve, amplitude and timing.

**Ribbon Transmission** controls the falloff from the bright surface into its
colored body. **Ribbon Sheen** adds an orientation-dependent highlight that fades
toward the ribbon ends. The blue/red bodies are broader than the cyan/yellow
light layers, with independently weighted color contributions. These are
art-directed shading profiles, not a physical thin-film interference model.
The material refinement preserves sheet flow speed, and projected thickness
has a smooth turnaround. The later persistent-phase correction affects the
relative offsets while keeping the material and glass breath unchanged.

Validate the analytic example with the production renderer:

```powershell
python scripts/quality/check_siri_continuous.py --background background.f32 --out validation
```

The reference still differs in ribbon shape, brightness and lower-glass
refraction. This is not perfect-match acceptance. The historical native
32-bpc PNG export issue remains outside the 16-bpc delivery. See the
[test matrix](../docs/TEST_MATRIX.md) for the exact measured scope.
