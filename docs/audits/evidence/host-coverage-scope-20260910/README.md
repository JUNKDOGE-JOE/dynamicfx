# Coverage-only scope revision

The user withdrew the additional automatic contour input on 2026-09-10:
"那不做这个了 保证原始图层的可视区域能被读就行". The current deliverable
is one automatically owned, opt-in coverage/alpha resource for the original
host layer. The existing background input and selected-mask `hint:path` stay.
This supersedes the earlier same-day request for two new resources.

## Acceptance boundary

The coverage image must distinguish transparent outside regions and holes,
opaque interiors, partial opacity and antialiased edges. Supported fills,
strokes, masks and modifiers must agree with AE at the requested time and
align with the background input. The shape adjustment-layer fixture remains
the first target; other original-layer source types need separate verification.
No manual shape recreation, name binding or helper layer/precomp is required
by the proposed product contract. This is a requirement, not proven behavior.

Automatic contour encoding, upload, arc-length/direction sampling and the
three former contour-specific test definitions are withdrawn from this issue.
Those definitions are retained in `withdrawn_cases` in the
[fixture matrix](../../../../spike/host-outline/cases.json). Existing mask-path
compatibility remains required. Internal geometry would still need to prove
visible-alpha fidelity if selected as a coverage producer.

## Current evidence

Production source remains at `ff89564`; this revision changes planning and
fixtures only. No build, install, restart, project edit or native render runs
as part of the scope edit. The independent diagnostic code and all prior
failure/success artifacts are preserved. The
[last native run](../host-shape-native-20260910/coverage-v003-v004/README.md)
found background alpha through the tested adjustment render checkouts and
no auxiliary Coverage channel. Those findings still apply; dropping contours
does not establish a coverage producer.

The [Proposed ADR](../../../adr/0049-automatic-host-shape-input.md),
[current status](../../../IMPLEMENTATION_STATUS.md),
[roadmap](../../../ROADMAP.md) and fixture matrix now describe coverage only.
Public [issue #9](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/9) records
the current requirement above the initial historical investigation.
Verification outputs and issue readback are recorded in [checks.json](checks.json).
No native feature row becomes PASS from these checks.

## Exact next action

Identify and test a render-safe source of original host coverage against the
revised alpha/coordinate contract. Prove requested-time evaluation and edit
invalidation before accepting the ABI or integrating production code. Do not
repeat failed checkout variants, toggle the user's layer switches during a
render, or treat the main-thread vector probe as the coverage result.
