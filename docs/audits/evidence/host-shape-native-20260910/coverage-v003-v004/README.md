# Layer options and auxiliary coverage: native result

The additional direct SDK routes do not provide host coverage for the tested
shape adjustment layer. RenderSuite5 layer/downstream options still return
background alpha. An ordinary shape control returns correct coverage through
the same layer constructor. PF auxiliary Coverage is absent on all four
fixtures. Raw animated contour acquisition remains the successful source
observation from [0.0.2](../background-v002/README.md).

These results rule out the tested routes, not every possible implementation.
The feature is not integrated, and shader-resource acceptance is not complete.

## Identity and evidence

Windows AE 2026 26.3x87, AE-MCP 0.10.7, 16-bpc project. Diagnostic 0.0.3
`2a79f154...` ran the layer-options trial. Diagnostic 0.0.4 `9c61010d...`
ran render-side auxiliary enumeration and is currently installed. Its eight
local tests and build pass. Production remains source `ff89564` / AEX
`c91db8c0...`. [checks.json](checks.json) records hashes, requests/results,
native logs, PNG comparisons, fixture snapshot and installation records.

| Route | Adjustment shape | Ordinary shape control |
|---|---|---|
| RenderSuite5, NewFromLayer | Alpha 1,1,1 | Alpha 0,1,0 |
| RenderSuite5, NewFromDownstreamOfEffect | Alpha 1,1,1 | Not needed for this comparison |
| PF auxiliary Coverage | Absent, channel count 0 | Absent, channel count 0 |

PF auxiliary enumeration also returns count 0 / Coverage absent for mask-only
and combined shape/mask adjustment fixtures. Enumeration runs in the PF render
callback and performs no AEGP calls. No channel buffer is checked out because
the typed lookup returns absent. The SDK documents these as
[auxiliary source channels](https://ae-plugins.docsforadobe.dev/effect-details/useful-utility-functions/#accessing-auxiliary-channel-data);
the Coverage name alone does not promise a shape matte.

The ordinary shader's new PNG matches its first-run output across all RGBA
bytes. The ordinary shape/probe PNG matches the transparent reference across
all bytes. PNGs are 8-bit exports from a 16-bpc project; these do not establish
16-bit quantization or 32-bpc support.

## Autonomous lifecycle

The user explicitly authorized AE startup/closure without mouse control.
AE was already closed before the 0.0.3 replacement; the saved AEP matched the
previous standby checkpoint. Both launches used `Start-Process -WindowStyle
Hidden`. For the 0.0.4 replacement, MCP saved the disposable fixture and
scheduled `app.quit()`; process exit was observed before installation. No
process-tree termination, mouse/keyboard operation or explicit activation
call was used. OS foreground ownership was not independently measured.

Only the independent diagnostic was replaced. Each installer backed up the
previous probe and verified the production AEX hash. After the experiments,
request serials and render logging were cleared, mode was returned to vector
read, and the disposable AEP was saved. AE remains open for further work.

## Implementation boundary and next action

Subsequent user direction withdraws the additional contour input. The
[coverage-only scope revision](../../host-coverage-scope-20260910/README.md)
supersedes the two-resource planning below; all native results remain intact.

Adding a shader binding cannot fix the missing producer. Current ordinary
input and the tested adjustment-layer render receipts contain background;
the successful AEGP contour reader runs on the main thread. A last-CTI idle
snapshot would not satisfy random frame requests or geometry invalidation.

The next design must prove a legal, frame-exact geometry producer before
integrating either resource. Reconstructing coverage from those contours is
a candidate with its own fill/stroke/modifier, antialiasing and coordinate
costs, not a selected or implemented fallback. It must preserve coverage-only
independence from the optional contour resource. An automatically generated
AE-evaluated parameter bridge could be investigated separately, but its
capacity, FFX rebinding, Undo/project-dirty behavior and structural invalidation
are unproven and no new parameter contract has been accepted.

Exact next action: prepare a bounded geometry-transport feasibility design
under ADR-0049 and test its requested-time/invalidation contract before any
production ABI change or coverage rasterizer. Do not repeat the exhausted
checkout variants, toggle Adjustment on user layers, add helper layers, call
AEGP from rendering or wait synchronously for idle work during a render.
