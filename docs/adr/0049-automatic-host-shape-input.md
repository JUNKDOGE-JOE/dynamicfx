# ADR-0049: Automatic host coverage input

- Status: Proposed
- Product scope: revised by the user on 2026-09-10 to visible coverage only.
  The additional automatic contour-sampling resource is withdrawn.
  The user subsequently accepted a plugin-managed mode-None data mask visible
  in the mask list, with no manual binding. Transport and shader ABI details
  remain Proposed pending native evidence.
- Delivery decision (2026-09-20): the user requires the original full scope
  to pass before delivery. A reduced initial 2D/16/32-bpc scope is rejected.
  Existing approval for an automatically managed data mask remains in force.
- Fidelity decision (2026-09-20): image accuracy takes priority. Preview speed
  may be optimized only while preserving validated accuracy. No silent quality
  reduction or relaxed correctness threshold is authorized.
- Internal-object decision (2026-09-20): after the native return bridge passed,
  the user explicitly accepted a plugin-maintained invisible reading layer,
  without manual binding. This supersedes the earlier no-helper-layer product
  constraint. Resource ABI and production ownership semantics remain Proposed.
- Host-version decision (2026-09-20): the user explicitly accepts AE 26.5+
  as the minimum for this new feature after AE 2025.6.6 refused StreamSuite7
  and a preset exported by AE 26.5. Existing features keep their previous host
  compatibility. The 8/16/32-bpc, 3D, mask, modifier and FFX requirements are
  unchanged; this is not approval to reduce their scope.
- Date: 2026-09-10
- Owners: DynamicFX project
- Issue: [#9](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/9)
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md)
- Related decisions: [0006](0006-state-and-persistence-boundary.md),
  [0029](0029-logical-resolution-abi.md),
  [0030](0030-layer-input-parameters.md),
  [0035](0035-path-parameters.md), [0039](0039-canvas-expansion.md)
- Verification: [TR-HOST-SHAPE-001](../TEST_MATRIX.md#tr-host-shape-001---automatic-host-shape-input-preparation)
- Fixtures and implementation entry: [host-outline spike](../../spike/host-outline/README.md)

## Context

On the reported shape adjustment layer, ordinary `input` supplies the
background composite; AE clips the effect with the layer's shape later.
Opaque background alpha therefore cannot describe the triangle boundary.
The report requests an additional resource so an FFX material can follow the
existing host shape without geometry controls, names or helper layers.
The submitted reproduction is a user observation, not a new host run.

`read_path` in `src/lib.rs` currently obtains a selected PF mask ID from the
Path pool, checks it out at render time, and stages vertices. `src/path.rs`
encodes one sequence into an N x 2 float texture. Neither that selector nor
the texture layout supplies the original layer's visible coverage. The existing
mask path feature remains unchanged. The latest user direction supersedes the
earlier same-day request for both coverage and contours; historical probes and
their failures remain in the evidence record.

The public SDK exposes AEGP layer streams and mask-outline vertices, but
these calls belong on the main thread. The SDK describes layer-frame
checkout through `AEGP_RenderAndCheckoutLayerFrame` as a non-render-time
operation. A static tree read consequently does not establish a legal
per-frame data source for SmartRender, MFR or aerender.
See the SDK [threading rules](https://ae-plugins.docsforadobe.dev/aegps/implementation/#threading)
and [render suites](https://ae-plugins.docsforadobe.dev/aegps/aegp-suites/#render-suites).

## Proposed decision

Add one opt-in, read-only graph resource containing the effect host's original
visible coverage/alpha, suitable for refraction sampling. Keep ordinary `input`
and the existing `hint:path` ABI intact. No new automatic contour input is
part of this feature.
The material declares the dependency in source; applying its FFX requires
no layer/path selector, shape-specific expression or geometry parameter.
Shaders that do not declare the resource incur no new host-shape work.

Coverage must agree with AE's original visible alpha for supported cases:
transparent outside regions and holes, opaque interiors, partial opacity
and antialiased edges. Its source is the original host layer's coverage
before DynamicFX output, independent of the underlying adjustment composite.
The first acceptance fixture remains a shape adjustment layer; other source
types need separate evidence before support is claimed.

Generate/upload coverage only when declared. A path list, visible outline
stroke, bounding box or the underlying composite's alpha must never silently
substitute for coverage. If geometry is used internally, its fill, stroke,
mask, modifier and transform evaluation must meet the same visible-alpha
contract. It does not introduce an exposed contour representation or any
along-contour sampling guarantee.
The resource contract is now fixed by [ADR-0050](0050-host-coverage-resource.md).
This feasibility record does not by itself accept the production owner adapter,
helper renderer or copy/FFX readiness protocol.

The active acquisition candidate is the [native reader bridge](../audits/evidence/host-coverage-direct-source-20260920/README.md):
original ONLY_MASKS -> disabled reader layer -> reader ALL_EFFECTS -> host
effect Layer input. Configuration belongs on the main thread; frame acquisition
uses declared PF/SmartFX dependencies. The tested original alpha returns with
zero native-word differences at 8/16/32 bpc. No expression sampling or shape
duplication is needed for those cases. Automatic lifecycle, arbitrary times,
coordinate/host matrices and production persistence remain unaccepted.

Resolve the data-source question before accepting those contracts:

1. **PF enumeration probe.** The installed `after-effects` 0.4.0 wrapper
   exposes `PathQuery::num_paths`, `path_info` and `checkout_path`. Compare
   a shape-only layer with a mask-only control, then a layer containing
   both. Record exactly which paths AE returns; the existing selector's
   behavior is not proof that enumeration exposes vector contents.
2. **Main-thread vector probe (historical feasibility).** Resolve the effect's layer, traverse
   `ADBE Root Vectors Group` by internal match names and evaluate path and
   transform streams at explicit times. Record rectangle/ellipse parameter
   streams as well as Bezier paths; do not assume every shape is an
   `ADBE Vector Shape`. Dispose effect/stream/value handles on every exit.
3. **Coverage probe.** Inspect supported pre-effect layer-render options
   on the main thread. Test whether the result is shape coverage or the
   adjustment background, and establish coordinates and recursion behavior.
   Never toggle Adjustment, enable flags or the effect stack to obtain it.
   The native comparison now finds background alpha from upstream options
   through both RenderSuite5 and RenderSuite4 plain=false. RenderSuite4
   plain=true with those options triggers modal 5027 and AE error 3; do not
   repeat it. RenderSuite5 layer/downstream constructors also return background
   alpha on the adjustment fixture, while the ordinary shape control returns
   correct coverage. PF auxiliary Coverage is absent on shape/mask/combined
   and ordinary control fixtures. [Recorded experiments](../audits/evidence/host-shape-native-20260910/coverage-v003-v004/README.md).
   These results exhaust the tested direct routes, not all possible producers.
4. **Render transport proof.** A candidate must supply the exact requested
   time in cold/out-of-order rendering and invalidate AE's cached result
   after a geometry edit. No render-thread AEGP, synchronous wait for an
   idle callback, persisted live handles or latest-CTI-only snapshot.
   A compute cache stores valid data; it does not make AEGP calls legal.

If none passes item 4, the feature remains a feasibility result. Do not
ship an idle-preview implementation as automatic animated coverage support.

## Coordinates and visible coverage

Coverage must align with the effect canvas's logical pixel space. If a
candidate internally uses geometry, it must transform points and Bezier
control points through the complete group stack, including anchor, position,
scale, rotation and skew. Distinguish absolute control points from tangent
offsets; the existing PF tangent interpretation is not evidence for AEGP.

Measure whether AE has already applied layer/parent transforms for the
continuously rasterized shape input. Do not apply them twice. Once a point
is in the shared logical space, normalize as `(point - canvas_origin) /
canvas_size`. Offset tangents get the linear transform and scale only.
Use the same stable canvas as `input`, not a downstream ROI's small rect;
preview downsampling must not be applied twice.

AE remains responsible for final adjustment-layer compositing. In
particular, do not multiply output alpha by coverage a second time without
proving edge equivalence. Antialiased boundary pixels need separate
comparison from fully exterior pixels. Define how fill opacity, strokes,
masks, holes and modifiers relate to the resource before claiming support.

An unsupported host or operator must produce a visible, stable diagnostic
with a tested failure behavior. No silent rectangle/ellipse approximation,
partial shape, stale coverage or transparent texture that looks valid.
Diagnostic numbers are assigned during implementation through the existing
registry. Temporal graphs using `prev` initially require explicit refusal
unless the ADR-0025 replay window can obtain the shape at every replay time;
ordinary animated paths in stateless graphs remain required.

## Implementation boundaries

The next isolated diagnostic compares legacy `PF_CHECKOUT_PARAM` for input 0
with an additional `PF_Param_LAYER` defaulting to `PF_LayerDefault_MYSELF`.
Its own sixth parameter is diagnostic-only, appended after the request serial;
production parameter topology is unchanged. A temporary `sampleImage` read
with `postEffect=false` distinguishes the original shape from the adjustment
composite, but excludes masks. The PF route must be validated independently,
including the SmartFX difference for input 0 described in the
[checkout documentation](https://ae-plugins.docsforadobe.dev/effect-details/interaction-callback-functions/).
No expression-per-pixel transport or new production self-layer parameter is
accepted from the three-point expression result.

An additional disposable experiment stores adaptively sampled coverage blocks
in a mask with mode `None`, then reads that data via PF PathQuery during render.
This mask carries raster blocks, not an exposed contour input. The user has
explicitly accepted an automatically created and managed mode-None mask visible
in the mask list. It currently requires fixture setup: ownership, FFX recreation,
cleanup, opacity cost, expressions, invalidation and cold-render behavior must
still be settled before production integration. The product request continues
to require no manual geometry setup or mask binding.

The isolated lifecycle diagnostic appends parameter 7, `Manage coverage carrier`,
default false and non-time-varying. Its main-thread idle hook creates a shared
carrier per opted-in layer only in guarded disposable `HS_auto_` compositions.
The exact expression identifies ownership; the displayed mask name is not a
binding. Missing data after Undo must not be silently recreated in the same
enable interval. This is a lifecycle probe, not a production parameter index,
ownership contract, or shader ABI acceptance. It retains the bounded sampler;
partial-opacity cost remains an independent acquisition failure. Diagnostic
0.0.9 now demonstrates basic ownership, cleanup, Undo/Redo and saved/cache-purged
retrieval. The null-expression memory repair and earlier modal failures are in
the [managed-carrier audit](../audits/evidence/host-shape-native-20260910/managed-carrier/README.md).
It retains AE's default mask name because the tested native rename call returns
`Parameter`. FFX recreation and complete visible-alpha semantics remain open.

| Area | Existing entry | Work after the transport proof |
|---|---|---|
| Host ownership and timing | `src/host/idle.rs`, `src/lib.rs` | Isolated host-shape adapter; explicit callback/thread and handle lifetimes |
| Resource declarations | `src/frontend/annotation.rs`, `shared.rs`, `grammar.rs` | Slot-free opt-in declaration shared by GLSL/WGSL; read-only graph validation |
| Resource staging | `ExternalSource`, `ExternalPixels`, SmartFX dispatch in `src/lib.rs` | Frame-owned resource and complete cleanup/cancel propagation |
| Coverage and canvas | `src/path.rs`, `src/canvas.rs` | Align the coverage image; internal geometry only if needed and verified; preserve released mask texel semantics |
| GPU upload and binding | `src/render.rs`, `src/plan.rs` | Existing extra-input order; explicit format, capability and size limits |
| State and invalidation | `src/identity.rs`, `src/persistence.rs` | Audit definition serialization and frame dependency identity; never key shape data by shader source alone |

The project-open callback repair is preserved in local commit `ff89564`
(2026-09-10). Runtime integration needs an isolated baseline that includes
that repair, with the public-source/archive distinction resolved before a
future push. This proposal remains uncommitted and does not publish that history.

## Alternatives considered

- Derive the boundary from ordinary input alpha: contradicts the reproduction.
- Self-reference with `hint:layer`: reported to return background alpha too.
- Require masks, helper precomps or shape-name expressions: fails FFX portability.
- Traverse AEGP streams inside SmartRender: violates the current host boundary.
- Cache only the current UI-time outline: cannot satisfy random frame requests.
- Reconstruct only rectangles and ellipses: omits required custom Beziers.

## Consequences

The material can stay independent of geometry and reuse the normal background
input. Coverage drives the initial refraction material. Withdrawing the
additional contour resource removes its encoding, upload and sampling contract,
but does not solve coverage acquisition. The remaining costs are frame
dependencies, coverage production and one resource contract. Animated transforms, modifiers,
dependency invalidation and cold renders are the main correctness risks.

No Accepted ADR is superseded by this Proposed document. If the eventual
design changes a persistent or shader ABI rule, identify and accept the
necessary extension or superseding decision before integrating it.

## Revisit conditions

Revisit acquisition or encoding when a supported SDK source improves visible
coverage fidelity. Adding an automatic contour input later would require a
new explicit product decision; it is not deferred work inside this issue.

## Verification obligations

Use the scenarios in [cases.json](../../spike/host-outline/cases.json).
Record each host/build/artifact separately. Required acceptance includes
the three alpha probes, renamed FFX reuse, animated curves, nested and layer
transforms, Full/Half/Quarter, expanded origins and ROI, holes/modifiers,
outside-shape preservation, warm/cold shuffled rendering, Undo/Redo,
duplicate isolation, save/reopen and aerender. GLSL/WGSL and existing
Layer/Gradient/Path resources must remain compatible at 8/16/32 bpc.

Pure geometry and parser tests cannot mark those native host rows PASS.
Also verify coverage opt-in, unchanged undeclared shaders and sampled partial
alpha against AE. No contour-only/both-resource or along-contour sampling
acceptance is required. Earlier contour-specific fixture definitions remain
archived separately and do not gate this coverage-only scope.

## 2026-09-20 acquisition evidence and pending product boundary

The [fidelity investigation](../audits/evidence/host-coverage-fidelity-20260920/README.md)
records exact native-depth sampling and a lossless diagnostic float32 transport.
The original 800x600 half-opacity case is correct but takes 91.529 s. The native
mask API matches basic add/subtract cases but omits expansion. These results do
not accept a production ABI. The managed mask's previous sampler is unchanged.

A user question is pending on whether automatically managed reference
layers/precomps may change the original no-helper-object requirement. No answer
or approval is inferred. Original full-scope and fidelity-first requirements
remain binding while that structural alternative is evaluated.
