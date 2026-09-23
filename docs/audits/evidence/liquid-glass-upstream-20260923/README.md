# Upstream Liquid Glass acceptance — 2026-09-23

Pinned upstream: iyinchao/liquid-glass-studio at
f7b28c36305a862f5cffed3ddd51511cf1204f56. Original shader/color source and MIT
notices are retained in upstream/; final material identities are separate.

gpu-summary.json: 26 final PASS cases. ae-matrix-summary.json: 27 final formal
queue frames. material-summary.json: four native alpha/hole/preset checks.
apply-twice.json and final-demo-state.json verify the saved handoff. The three
PNG images are decoded from formal Photoshop output (8-bit display evidence).
Full float/PSD/AEP intermediates remain local and are not source publication.

Failure logs remain visible: initial missing SciPy, inverse-trig discrepancy,
invalid diagnostic and wrong SDK-container path. gpu-algebraic.log and
rust-correct-sdk.log record final passing replacements. Host JSON contains
MCP script requests/readback; local home paths are normalized to <user-home>.
The scratch directory's 20260924 suffix is a run label, not the run date.

The harnesses are evidence for this Windows host; paths require adjustment to
rerun on another checkout. verify-gpu.py uses the production quality runner and
the earlier generated coverage/background fixture. No upstream web app code
was executed. Native binaries and SDK are excluded.
