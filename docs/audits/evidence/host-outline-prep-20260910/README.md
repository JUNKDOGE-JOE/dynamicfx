# Host-shape input preparation

Issue [#9](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/9) was created and
read back with the supplied body unchanged after newline normalization.
The GitHub connector succeeded after the local gh read returned HTTP 401.
No authentication configuration was changed.

Baseline: `17e12a7`, Windows, existing dirty branch
`codex/fix-project-open-lock`. The prepared feature does not alter the
project-open callback repair, build code, installed AEX or an AE project.
The public/archive distinction recorded in IMPLEMENTATION_STATUS still
applies; no source commit or push was performed.

## Source and SDK findings

- `src/lib.rs::read_path` uses the Path pool's selected ID and PF checkout;
  `ExternalSource` and `ExternalPixels` are the existing staging boundary.
- `src/path.rs` encodes one vertex sequence, with a recorded unresolved PF
  tangent interpretation. Do not infer a multi-contour AEGP ABI from it.
- Installed wrapper `after-effects` 0.4.0 provides PF enumeration, AEGP
  stream traversal, outline vertices and upstream layer-render options.
  Their availability is source evidence, not a successful host experiment.
- The SDK [threading documentation](https://ae-plugins.docsforadobe.dev/aegps/implementation/#threading)
  constrains ordinary AEGP calls to the main thread. Its
  [render-suite documentation](https://ae-plugins.docsforadobe.dev/aegps/aegp-suites/#render-suites)
  describes layer-frame checkout for non-render-time use. A UI snapshot is
  insufficient to prove cold/out-of-order per-frame input.

## Prepared artifacts and checks

[ADR-0049](../../../adr/0049-automatic-host-shape-input.md) records a Proposed
direction without freezing grammar, texture layout or persistence.
The [spike](../../../../spike/host-outline/README.md) contains a GLSL baseline,
fixture JSON and native probe sequence. Check outputs and file identities
are recorded in [checks.json](checks.json).

All native feature scenarios remain NOT_RUN. No shader compile, GPU result,
FFX, native probe executable or implemented resource is claimed here. The
submitted reproduction remains CLAIMED_UNVERIFIED for this local baseline.

Next exact action: implement the isolated PF-enumeration/native host-shape
probe specified in the spike, establishing frame-exact acquisition and
invalidation before accepting ADR-0049 or integrating the resource.
