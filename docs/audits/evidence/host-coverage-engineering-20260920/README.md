# Diagnostic engineering corrections — 0.0.10

The idle cleanup and small-world diagnostic failures are corrected in the
isolated probe. This does **not** clear the full coverage acceptance gate.
The user requires the original full scope before delivery and rejected a
reduced initial 2D/16/32-bpc scope. The accepted managed data mask remains the
only approved project structure addition. No separate contour resource is added.
No new commit, push or main update occurred.

## Diagnosis and change

The last-owner fixture remains at two masks without an idle wake. A read-only
native MCP request immediately yields one mask: [before](raw/wake-before.json),
[request](raw/native-wake-read.json), [after](raw/wake-after.json). This isolates
idle starvation as the cause of that observed cleanup failure.

The probe caches `AEGP_CauseIdleRoutinesToBeCalled` on the main thread and holds
its suite until a bounded 500-ms wake worker has stopped and joined. The worker
only requests idle; all layer/stream/mask access stays in the main-thread idle
hook. No render callback waits for it or calls AEGP. The installed SDK wrapper
explicitly documents this one function as asynchronous and thread-safe while
suite acquisition is not thread-safe. Shutdown stops/unparks/joins the worker
before suite release. This remains diagnostic infrastructure, not a finalized
production scheduling design.

Original-source diagnostic samples now scale their three reference coordinates
to the checked-out world. The 800x600 reference positions are unchanged; small
worlds no longer abort logging before PF path retrieval.

## Verification

- [16 tests](raw/test.log) and [locked build](raw/build.log) pass. The first
  build's suite-version type mismatch is recorded in [build note](raw/initial-build-error.txt).
- [Installed diagnostic](raw/install.json): 0.0.10,
  SHA-256 `64a14201b272142e1e171e357ed0e7047a66f2e7be3398dbc49b46480b95f507`.
  Production AEX remains `c91db8c0...`, and production source is unchanged.
- [Lifecycle checks](raw/lifecycle-checks.json): sharing, disable-first,
  disable-last, re-enable, Undo and Redo all PASS on AE 2026 26.5x89.
  The regular user mask is preserved. These calls use JSX MCP only; no native
  read wake workaround or foreground activation is used in acceptance.
- [Native PF pixel checks](raw/pixel-checks.json): 128x96 / 818 vertices and
  800x600 / 3648 vertices both agree with fresh reference alpha to 1/255.
  These remain 8-bit PNG comparisons from a 16-bpc project.
- [Clean shutdown](raw/v010-shutdown-check.json) and [restart/reopen](raw/v010-reopen.json)
  pass: 55 items, two base masks, owner flags [1,0], clean saved project.
  The earlier v009 quit closed the MCP connection before its reply; the saved
  AEP and exited process were verified before installation. No request was replayed.

## Remaining gate and decisions

Original masks/holes, full-frame partial-alpha cost, 8-bpc edge fidelity,
production coverage binding, FFX and true MFR acceptance remain open. The
coverage sampler itself is unchanged, so none of its prior failures is erased.
The prior [failed regression](../host-coverage-regression-20260920/README.md)
is preserved.

Engineering defaults: preserve coverage correctness rather than silently
truncating, simplify user controls while keeping automatic ownership, use
caching only with correct time/geometry invalidation, and retain the host-thread
boundary. Algorithm and scheduling choices do not need product confirmation.
A future decision would need the user only if it changes the visible result,
supported scope, project structure, or required workflow. No such additional
change is authorized by this correction.

Next exact action: solve full visible-alpha acquisition (masks, partial opacity,
8-bpc edges) under bounded cost, then finish the original acceptance matrix.
The [manifest](manifest.json) maps original/evidence hashes; local usernames and
MCP IDs are visibly redacted in copied text. Source snapshots and binaries are
identified separately. The diagnostic source is in the working tree only.

## User priority clarification

Image fidelity takes priority. Preview speed is optimized only after correctness
is established, and optimizations must preserve the validated output. No implicit
preview downgrade or loosened acceptance threshold is authorized. Existing
pixel, partial-alpha and mask failures remain blocking. This is a product
priority decision, not a newly passing test.
