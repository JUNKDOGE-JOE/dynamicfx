# DynamicFX examples

Complete, working shaders you can paste into an effect. Each file is the
shader *source*; After Effects needs it wrapped as an expression.

## How to use one

1. Apply **DynamicFx** to a layer.
2. Alt-click (Option-click) the stopwatch on the `Source` parameter to open
   the expression field.
3. Type a backtick `` ` ``, paste the whole file, then type `` ` `` and `;0`.

The result looks like this — the backticks carry the source text verbatim,
and the `;0` makes the expression evaluate to a number, which is what the
numeric `Source` parameter requires:

```javascript
`@dynamicfx 1
@graph
pass trail: input, prev -> output
@end
...the rest of the file...
@endpass
`;0
```

Click away from the field to commit. The `Status` row reports the compile
result; `Show Full Status` prints the untruncated text with its `E<code>`
diagnostic if something is wrong. The declared `@param` controls appear
underneath as ordinary, keyframeable AE properties.

If you are applying effects from a script rather than by hand, read
[Scripting: wait for readiness before you render](../README.md#scripting-wait-for-readiness-before-you-render)
first — writing an expression does not compile it, and a script that holds
the main thread prevents the compile from ever happening.

## The examples

### [`thermal.glsl`](thermal.glsl) — six-pass heat signature

A thermal/infrared look built as a real multi-pass graph: a warped fBm heat
field, two separable blur chains at different radii, and a compositing pass
that maps everything through a six-stop palette.

Demonstrates multi-pass graphs (six passes, two independent blur chains
feeding one compositor), effect-wide parameters shared across passes, and
`hint:color default:#RRGGBB` controls.

Drive it with a layer that has an **alpha channel** — text or a logo works
well. The graph reads the input's alpha as the heat source, so a fully
opaque solid gives a flat result.

### [`orb.glsl`](orb.glsl) — orbiting light with a temporal trail

A glowing orb circling the frame, leaving a decaying trail. One pass that
reads both the layer and `prev` (the previous frame's output).

Demonstrates temporal feedback: `prev` as a pass input plus `// @window 16`.
DynamicFX re-simulates the last 16 frames for every request, so scrubbing,
the render queue, and aerender all produce exactly the same pixels — there
is no hidden playback state to get out of sync. It also shows `hint:angle`
and `hint:bool` controls alongside colors.

Turn **Composite Over Layer** off to see the orb alone on transparency.

## Verification status

Both files are compiled through the real frontend by
`cargo test example_tests` on every build, so a grammar, ABI, or annotation
change cannot silently break them. That test proves they **compile**; the
palettes and default values are authored choices and are checked visually at
release time, not by the test.
