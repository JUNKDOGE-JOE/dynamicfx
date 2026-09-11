# Background host acquisition on diagnostic 0.0.2

Raw animated contour acquisition passes on AE 2026 26.3x87. Automatic shape
coverage remains unavailable through the two tested upstream checkout routes.
The third route triggers a host validation error. This is diagnostic evidence,
not implementation or shader-resource acceptance.

The installed diagnostic is `8d896b68...`; production source is `ff89564`
and production AEX remains `c91db8c0...`. Full hashes, raw requests/results,
native logs, numeric comparisons and the 0.0.3 candidate are in
[checks.json](checks.json). All AE operations used MCP without foreground
activation. The user opened AE and dismissed the diagnostic error dialog.

## Results

| Read | Outcome | Meaning |
|---|---|---|
| Baseline shape vectors | Triangle vertices and closure recovered | Raw AEGP extraction works |
| Animated curve at 1, 0, 0.5 seconds; CTI 0 | Vertices, relative tangent offsets and animated group position match JSX, including expression displacement | PASS 3/3 acquisition comparison; no coordinate composition or render transport claim |
| RenderSuite5, upstream options | Alpha 1,1,1 in 800x600 16-bit world | Background, not required coverage 0,1,0 |
| RenderSuite4, upstream options, plain=false | Same alpha 1,1,1 | Same failed coverage candidate |
| RenderSuite4, upstream options, plain=true | Modal 5027, then AE error 3 after dismissal | Invalid options combination; no returned pixels |

AEGP shape tangent fields are relative offsets in this fixture. The earlier
PF mask experiment returned absolute control positions. The two API records
must not be conflated when defining the future contour encoding. Skew, skew
axis, rotation, anchor and scale are read as raw group streams; this probe
does not transform or rasterize the geometry.

## Failures and recovery

The first request checked `numProperties == 5`; AE exposes six properties
because it appends Compositing Options. The guard rejected before mutation.
Readback identified the extra group; the corrected driver validates the
diagnostic's match name and request property's index/name instead.

The legacy plain=true request exceeded the driver's 30-second observation
window. A read-only MCP call still succeeded while the modal existed; loopback
health and JSX responsiveness therefore were not proof that no dialog existed.
The user supplied [the error dialog](plain-flag-error.png). After dismissal,
the append-only native log records `BACKGROUND_FAILED` with AE error 3.
The timed-out log and later terminal log are both retained. No request replay,
window activation, click or process termination was performed by the agent.

The first animated-fixture expression incorrectly called `value.points()`.
After the retained failure, a scoped correction used `thisProperty.points()`
and its tangent methods, consistent with the
[Adobe expression reference](https://helpx.adobe.com/after-effects/desktop/work-with-expressions/expression-language-reference/expression-language-reference.html).
Only corrected expression results enter the successful numeric comparisons.
The failed result was saved before a separate GBK stdout encoding error;
the local runner now prints UTF-8.

## Prepared diagnostic 0.0.3

The frozen candidate `scripts/out/host-shape-native-20260910/DynamicFxHostShapeProbe-v003.aex`
has SHA-256 `2a79f154ded226d3c84ab0bac087a0929d1daad43d1d5e4cec5a896b32582e9b`.
Eight local tests pass, including rejection of retired mode 5 before host-suite
access. Modes 6 and 7 pair RenderSuite5 with `AEGP_NewFromLayer` and
`AEGP_NewFromDownstreamOfEffect`, respectively. The current SDK binding exposes
both constructors; neither has yet proven useful shape coverage. Layer options
include all effects, so the first trial must use the saved baseline host with
only the pass-through diagnostic. See the
[SDK render-options guidance](https://ae-plugins.docsforadobe.dev/aegps/aegp-suites/#render-suites).

The candidate is built, frozen and not installed. The replacement script
verifies AE/aerender closure, backs up 0.0.2, checks the frozen candidate hash,
replaces only the version-specific diagnostic, and verifies production identity.
The saved disposable AEP has all request serials reset to zero and read mode 2;
an immutable copy is `scripts/out/host-shape-native-20260910/host-shape-native-v002-frozen.aep`.

The subsequent 0.0.3/0.0.4 installations and tests are complete; see the
[layer/auxiliary result and current next action](../coverage-v003-v004/README.md).
Do not reuse retired mode 5. No current result proves cold/shuffled render transport,
invalidation, modifiers, coordinate mapping, portable FFX or other host years.
