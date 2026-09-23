# First native host-shape acquisition regression

The reported alpha mismatch is reproduced on AE 2026 26.3x87. PF path
enumeration reads masks but returns no path for the shape-only adjustment
layer. This rules out PF enumeration as the direct automatic shape source
for this fixture. This page records the first 0.0.1 run. The subsequent
[0.0.2 background run](background-v002/README.md) reads animated AEGP vertices
and records failed coverage candidates, including a rejected plain flag.
The [later layer/auxiliary run](coverage-v003-v004/README.md) records additional
failed adjustment-coverage sources and passing ordinary-shape controls.

The user authorized launching AE and native regression, then required MCP
only with no foreground activation. All fixture setup, readback and renders
in this run used MCP over loopback HTTP. No window activation, clicks,
`openInViewer`, process restart or foreground capture occurred in this run.
The empty initial project was populated with five disposable compositions
and saved under `scripts/out/host-shape-native-20260910/host-shape-native.aep`.

## Evidence and results

Production source baseline is `ff89564`. Installed production AEX remains
`c91db8c0...`; this first run used diagnostic 0.0.1 / `cf867972...`. AE-MCP is
0.10.7. The project is 16-bpc; saved PNG evidence is 8-bit RGBA and does not
establish 16-bit precision. [checks.json](checks.json) records full hashes,
requested times, evidence paths and the later background candidate.

| Procedure | Observed result | Status |
|---|---|---|
| Ordinary shader encodes alpha at `(210,490)`, `(400,300)`, `(50,50)` | Interior RGB is `[255,255,255]`; exterior stays background `[51,76,102]` | PASS reproduction |
| Shape over transparent background, same three coordinates | Alpha `[0,255,0]` | PASS reference |
| Shape-only adjustment / PF enumeration | 0 paths | PASS observation; source candidate fails |
| Mask-only adjustment / PF enumeration | 1 closed path, 3 segments | PASS |
| Shape plus mask / PF enumeration | 1 closed path, matching mask | PASS |
| Animated mask requests `0`, `1`, `0.5` with CTI at `0` | Requested vertices and interpolated handles read correctly | PASS mask control |
| MCP native tree traversal | Reaches `ADBE Vector Shape`, type `mask`; value marked `unsupported` by MCP | PASS metadata only |

Raw frame logs, pixels and MCP responses are adjacent. The full native
tree responses remain in `scripts/out/host-shape-native-20260910/native-tree-*.json`;
the summarized leaf is in checks.json. Traversal uses only returned native
locators. The MCP serializer's unsupported value is not evidence that the
underlying AEGP mask-outline suite cannot read vertices.

PF tangent fields in the animated mask control contain absolute control
positions, not the JSX tangent offsets. This is a recorded observation for
this API and fixture; no production Path change or AEGP tangent assumption
is included in this batch.

## First failures and recovery

The first setup call timed out after 45 seconds. No blind retry ran. MCP
health recovered, readback found all five comps, and the AEP file was saved
after the timeout. The original call remains an uncertain transport result.
Its script repeatedly accessed `app.effects`; subsequent preparation caches
the collection once. That is a source-supported performance lead, not a
timed causal proof of the timeout.

Readback also found an empty Source expression because the setup used an
exact label instead of `Source (use expression)`. The script now matches the
prefix, and one separate MCP write populated only that fixture's Source.
The successful renders occurred after this correction.

Native tree development initially omitted a required composition reference
and then used the wrong local JSON key for a property locator. The former
was rejected before host execution; the latter was a local parser failure.
Neither is a shape-acquisition failure. The corrected native traversal passed.

## Background candidate and next step

Diagnostic 0.0.1 exposes coverage/vector reads only through a button, which
JSX reports as `NO_VALUE`. To honor the no-foreground requirement, 0.0.2 adds
`Background request serial` at diagnostic parameter 5. After its idle scan
logs `BACKGROUND_READY`, MCP writes time, mode and a changed positive serial.
An idle callback performs at most one request per scan on the setup thread,
without holding the effect-instance mutex or entering render-side AEGP.
It records a terminal result and never retries a consumed request. Initial
persisted serials seed the observer without replaying reads after reopen.

The separate 0.0.2 candidate compiles and passes seven local tests, including
serial validation and suppression of persisted/repeated requests. The first
compile's `Proj` alias error is retained beside the passing logs. It was
installed after the user closed AE at 22:47:51 +08:00, with no AfterFX/aerender
process present. The installer backed up 0.0.1, verified the frozen 0.0.2 hash
`8d896b68...`, and confirmed the production AEX was unchanged.
[Installation record](install-v002.json). The earlier checks.json remains the
pre-installation snapshot. Loading and host verification of 0.0.2 remain unrun.
The request driver is
[request-native.py](../../../../spike/host-outline/request-native.py).

The planned 0.0.2 reload and comparison have since completed; see the
[background result and next action](background-v002/README.md).
No general shape input, contour shader ABI, production integration,
FFX delivery, cold-cache shape transport or other host-year acceptance is
claimed from this first run.
