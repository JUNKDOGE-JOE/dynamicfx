# Original alpha and raster-block transport feasibility

Original shape alpha is accessible through AE's `sampleImage` expression
with `postEffect=false`. An experimental mode-None mask can transport sampled
raster blocks into the native render callback via PF PathQuery. The first
opaque triangle/animated-curve subset matches reference PNG alpha to one
8-bit level. This is not a production solution: 50% opacity exceeds the
sampling budget, original masks are excluded, and automated data-mask ownership
and FFX lifecycle are unimplemented. The user has been asked whether adding
such a managed mask to the layer is acceptable.

## Baseline and native procedure

Windows, AE 2026 26.3x87, AE-MCP 0.10.7, 16-bpc disposable fixture; source
`ff89564` and production AEX `c91db8c0...` remain intact. Independent diagnostic
0.0.5 `5da92f37...` is installed in the AE 2026-specific DynamicFx folder.
MCP saved/scheduled quit of PID 58660; exit preceded installation. Hidden-style
startup launched PID 57256. There were no mouse/keyboard or explicit window
activation calls; OS foreground ownership was not measured.

The diagnostic adds a sixth Layer parameter defaulting to self. Existing
instances initially read None; explicit self assignment is separately recorded.
Legacy PF checkout of input 0 and of this extra parameter both read background
alpha `[1,1,1]` on the adjustment shape. Both read `[0,1,0]` on the ordinary
shape control. The checkout is legal but does not isolate the required source.
A native Set Matte control also returned background alpha from its own
adjustment layer. The production input/binding implementation is unchanged.

The SDK distinguishes parameter checkout from the ordinary upstream input in
its [interaction callback reference](https://ae-plugins.docsforadobe.dev/effect-details/interaction-callback-functions/).
Those general semantics do not supersede these measured adjustment-layer
results. Adobe's [expression reference](https://helpx.adobe.com/after-effects/desktop/work-with-expressions/expression-language-reference/expression-language-reference.html)
documents pre-mask/pre-effect sampling. The original-shape tests return
`[0,1,0]`; the mask-only test returns `[1,1,1]`, so this expression source must
not be described as complete masked coverage.

## Raster transport experiment

The fixture expression recursively samples image rectangles and encodes opaque
or partially covered raster blocks as data in an open mask path. The mask is
set to None and has no compositing role. This does not reconstruct or expose
shape contours. The production plugin does not create or use this mask.

The diagnostic's existing PF path reader obtains these values at render time
on ThreadId(2). A local decoder validates the header, block bounds and overlap,
then reconstructs an 800x600 alpha image. The marker and block format are
experimental and not an accepted ABI. Sources:
[sampling expression](../../../../../spike/host-outline/coverage-blocks-expression.js),
[native reader](../../../../../spike/host-outline/native/src/read.rs),
[decoder/comparison](../../../../../spike/host-outline/verify-coverage-blocks.py).

| Case | Different pixels / 480000 | Maximum 8-bit error |
|---|---:|---:|
| Triangle | 49 | 1 |
| Animated curve at 1 s, CTI 0 | 31 | 1 |
| Animated curve at 0 s | 22 | 1 |
| Animated curve at 0.5 s, CTI 0 | 16 | 1 |
| Edited original key, first sampler | 20 | 31 — FAIL |
| Same edited key, revised sampler | 19 | 1 |

The edit changes 8509 decoded pixels, proving this subset does not reuse a
stale pre-edit image. The largest first-sampler error is pixel `(199,299)`,
decoded 0 versus native 31. A large rectangle's rounded average hid that tiny
coverage. The revision samples smaller rectangles before merging identical
blocks, and fixes this case. It does not establish a precision bound for every
depth/content type. All comparisons are 8-bit PNGs from a 16-bpc project.

Initial cached property reads took 167 ms for the triangle; the revised
sampler's measured property reads took 221 ms for the triangle and 252 ms
for the curve at zero. These are neither cold-render benchmarks nor per-frame
plugin timing guarantees. The revised expression is frozen separately from
the first version, and raw requests retain their exact source text.

## Failures and remaining risks

- A 50% fill exceeds the first sampler's budget; the revision still exceeds
  32768 queries. The first property-read measurement was 1334 ms. The later
  reported 0 ms excludes evaluation during expression assignment and is not
  a performance result. The disabled expression returns an ordinary fallback
  mask; its missing data header is invalid, never a transparent success.
- Original masks/feathering, general partial alpha, mask-list visibility,
  dependency ownership, auto-creation/removal, structural edits, duplicates,
  FFX, Undo/Redo, cold aerender and MFR are not accepted.
- One capture returned before PNG creation. The original empty log/false
  immediate file observation are preserved, along with the later completed
  PNG/log; the driver now waits for a stable file without rerendering.
- The first Set Matte inspection used a group-only property as a leaf and
  failed after fixture creation. A readback inspected that same fixture before
  proceeding. An edit record aliased its `before` array; the source fixture
  and prior PNGs provide the baseline, and later records copy the array.
- One local patch invocation was rejected for deleting and adding the same
  path together; no host action ran from that rejected patch. It was replaced
  by a normal update.

Nine diagnostic unit tests/build pass. The saved disposable project has 18
items; original animated paths are restored, all diagnostic requests/logging
are reset and the failing opacity expression is disabled. Production source
and AEX are unchanged. Exact hashes, raw filenames, source snapshots and
verification are in [checks.json](checks.json). All newly added masks are
confined to these disposable test compositions.

## Exact next action

Resolve whether a plugin-managed mode-None data mask is acceptable in the host
layer. If accepted, first fix bounded partial-alpha acquisition and establish
automatic ownership/FFX/Undo/cold-render behavior before production ABI work;
otherwise retire this carrier and continue investigating direct sources.
Do not treat this feasibility subset as the requested feature being complete.
