# Automatic host coverage: implementation preparation

Tracking [issue #9](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/9).
The user's latest 2026-09-10 direction retains only the original host layer's
visible coverage/alpha. The additional automatic contour-sampling input is
withdrawn; existing `hint:path` mask input remains unchanged.
[ADR-0049](../../docs/adr/0049-automatic-host-shape-input.md) keeps transport/ABI
details Proposed. Historical vector probes below remain evidence, not a new
contour feature or proof of coverage. [Scope record](../../docs/audits/evidence/host-coverage-scope-20260910/README.md).
This directory contains a reproduction and an independently built diagnostic
effect, not an implemented shader resource. The first native run reproduces
the alpha mismatch and rules out PF enumeration for shape-only contents.
[Native results](../../docs/audits/evidence/host-shape-native-20260910/README.md)
distinguish these observations from the unrun shader-resource acceptance.

## Latest diagnostic state

Version 0.0.9 proves automatic carrier creation, sharing, cleanup, Undo/Redo,
save/reopen and cold reads on the disposable AE 2026 fixture. The user accepted
the managed data mask. Earlier version descriptions below are historical.
[Managed-carrier evidence](../../docs/audits/evidence/host-shape-native-20260910/managed-carrier/README.md)
records the memory-handle failure and repair. Partial-alpha cost, original-mask
coverage and FFX remain unresolved; this is not a production resource.

## Diagnostic build

The [Windows-only crate](native/Cargo.toml) has its own match name,
`DynamicFx Host Shape Probe`, and does not link the production runtime.
Build without installation from the repository root:

```powershell
cargo test --offline --locked --manifest-path spike/host-outline/native/Cargo.toml --target-dir scripts/out/host-shape-probe-20260910/target
cargo build --offline --locked --manifest-path spike/host-outline/native/Cargo.toml --target-dir scripts/out/host-shape-probe-20260910/target
```

The first-run 0.0.1 artifact is frozen under
`scripts/out/host-shape-probe-20260910/DynamicFxHostShapeProbe.aex`.
The first background 0.0.2 diagnostic is frozen under
`scripts/out/host-shape-native-20260910/DynamicFxHostShapeProbe-v002.aex`.
The tested 0.0.3 layer-options diagnostic is frozen under
`scripts/out/host-shape-native-20260910/DynamicFxHostShapeProbe-v003.aex`.
The tested 0.0.4 auxiliary-channel diagnostic is frozen under
`scripts/out/host-shape-native-20260910/DynamicFxHostShapeProbe-v004.aex`.
Its build and eight tests pass. [Current results](../../docs/audits/evidence/host-shape-native-20260910/coverage-v003-v004/README.md)
record background alpha through the layer constructors and absent auxiliary
Coverage; none supplies adjustment coverage. Ordinary shape/input controls pass.
The historical 0.0.5 original-checkout diagnostic is frozen under
`scripts/out/host-shape-native-20260910/DynamicFxHostShapeProbe-v005.aex` and passes
nine local tests. Its direct PF checkouts also return adjustment background.
[Later feasibility evidence](../../docs/audits/evidence/host-shape-native-20260910/original-source-v005/README.md)
records pre-effect expression sampling and mode-None raster-block data transport,
including an edge-precision failure/fix and a remaining partial-opacity budget
failure. The data mask is fixture-managed; automated production ownership and
FFX support are unimplemented and user acceptance of that carrier is pending.
This is a development diagnostic build. Its separately authorized replacement
installation is recorded in the native evidence; it is not a release.
[Build evidence](../../docs/audits/evidence/host-shape-probe-20260910/README.md)
records its identity and local tests.

The user requires MCP-only regression with no foreground activation. Use
[mcp_client.py](mcp_client.py), [capture-native.py](capture-native.py) and,
after diagnostic 0.0.2 is loaded, [request-native.py](request-native.py).
The capture driver calls `saveFrameToPng` without opening a viewer. The
background driver waits for `BACKGROUND_READY`, then writes time, mode and
a new request serial through MCP. Initial persisted serials do not replay;
wait for readiness after reopening. Each request needs a unique evidence
label. Terminal failures and timeouts must be inspected before another request.

After a separately scoped installation and disposable fixture setup, add the
probe to the shape layer before DynamicFx and use **Layer time (s)**,
**Read mode**, then **Read**. Coverage reads only upstream of this probe.
Run PF paths, Vector streams and the supported coverage modes separately,
preserving each result in `%TEMP%/dynamicfx-host-shape-probe.log`.
Mode 5 is retired: AE 2026 rejects the plain=true/upstream option combination
with a modal 5027 error. Modes 6/7 require 0.0.3 and use RenderSuite5 layer and
downstream options. First try them only on the baseline with the diagnostic
as its sole effect; layer options include all effects. Neither mode has a
usable adjustment-coverage result. Their completed trial is in the current
results above; do not repeat the same variants without a new hypothesis.
The log records the requested layer time and the callback thread; successful
completion means that the read completed, not that the pixels are correct.

Coverage comparison requests 16-bpc Full-resolution worlds and records their
size, stride, rendered region and alpha at the three baseline coordinates.
Those coordinates are explicitly world-local; only compare them with the
identity fixture until coordinate mapping has been proven. Out-of-world
samples remain `UNAVAILABLE`. Do not mark Full/Half/Quarter or 32-bpc support
from this initial diagnostic. Vector reads log post-expression values and
raw Bezier vertices; they do not implement modifiers or transformed contours.

**Log PF paths and channels** is off by default. Enable it only for the
short, disposable PF enumeration experiment; that path propagates host
cancellation and contains no AEGP calls. Auxiliary enumeration reports count
and typed Coverage availability; an absent channel is not checked out.
Other modes run from the setup
thread's button or idle callback. Stream traversal has depth/node/vertex limits;
coverage requests use a cooperative ten-second cancellation callback.
This callback cannot guarantee a timeout if the host stops invoking it.
Host handles remain local to each read and are released during unwinding.
The probe does not yet establish safe production render transport or host
callback reentrancy. Stop native experiments on recursion or blocked renders.

## Baseline fixture

Use a disposable 800 x 600, square-pixel, 16-bpc comp with an opaque
full-frame background. Draw a closed white-filled triangle with vertices
`[[200,150],[600,150],[400,500]]`, zero tangents, no stroke and identity
group transform. Layer anchor and position are `[0,0]`; other transforms
are identity. Enable Adjustment and add DynamicFx.

Set Language to GLSL. Use [input-alpha-probe.glsl](input-alpha-probe.glsl)
as Source expression text, wrapped in a JavaScript template literal followed
by `;0`. Use Full preview and no canvas expansion for this first fixture.
The three values encoded as RGB are ordinary-input alpha at `(210,490)`,
`(400,300)` and `(50,50)`. The report observes a white triangle. That remains
the expected ordinary-input result after the feature: the separate resource
must give `[0,1,0]`. Green is not the expected result of this baseline shader.

The darker opaque background in [cases.json](cases.json) distinguishes
AE's final clip from the probe's white output. Sample well away from the
antialiased edge for this binary check. Save raw pixels and an unambiguous
artifact/host identity; a screenshot alone cannot establish alpha values.

## First implementation experiment

Prepare an isolated native diagnostic build before production graph changes:

1. Instrument PF `num_paths` / `path_info` / `checkout_path` on three
   disposable layers: shape-only, mask-only and both. Use actual requested
   frame time, pair every checkout/checkin, propagate cancellation, and log
   selector/thread, path count, closure and vertex samples with bounded output.
2. Separately, from a safe main-thread callback, get the host layer and
   enumerate internal vector match names, evaluated values and transform
   stacks. Include parametric rectangle/ellipse and a curved Bezier at
   `0`, `0.5`, `1` seconds. Record requested versus evaluated time explicitly.
3. If trying AEGP pre-effect layer checkout, prove whether alpha is actual
   shape coverage on an adjustment layer. This main-thread experiment alone
   does not prove a render-safe source. Compare correctly typed RenderSuite5
   with correctly constructed layer options. The completed RenderSuite4
   plain=true trial was rejected; do not repeat that incompatible combination.
4. Select a transport only after cold, shuffled frame requests with CTI
   elsewhere reproduce correct geometry and geometry edits invalidate the
   cache. Stop the candidate on stale frames, recursive rendering or waits
   for idle while a render holds host work.

Diagnostic 0.0.1 installation and the disposable fixture run were separately
authorized and completed. Version 0.0.2 was installed and loaded, and its
background vector/coverage experiment is recorded. Versions 0.0.3 and 0.0.4
were subsequently installed and tested using authorized background lifecycle
operations. Source, installation and host-result states stay separate.

## Integration order

Data-source proof -> resource/coordinate ADR -> frame-owned host adapter ->
shared GLSL/WGSL resource parsing and binding -> GPU upload -> full host matrix
-> portable material FFX fixture. Reuse `ExternalSource` and the renderer's
extra-input ordering after their required extension; do not allocate a
manual Path selector for an automatically owned resource.

Coverage must match the original visible alpha, including fill/stroke opacity,
masks, holes and antialiased edges for supported cases. Unsupported modifiers
must be explicit. A bounding box or raw path list is not coverage. Compare
against AE and keep coverage application at compositing from being duplicated
in the shader output. No separate contour ABI, upload or sampling test is
part of the current feature.

## Acceptance

[cases.json](cases.json) enumerates current HS-01 through HS-12 plus HS-16 and retains
the three withdrawn contour-specific definitions separately. Each host result
needs the exact source baseline, plugin artifact hash, installation identity,
AE version/build, GPU/backend, requested times and raw evidence. Initial
priority is the reported AE 2026 Windows / DX12 configuration, followed by
AE 2025 regression; other target years and macOS require their own results.
Existing project-open repair and ordinary-input compatibility must survive.

The verification record is
[TR-HOST-SHAPE-001](../../docs/TEST_MATRIX.md#tr-host-shape-001---automatic-host-shape-input-preparation).
