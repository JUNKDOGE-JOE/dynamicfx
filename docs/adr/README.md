# Architecture Decision Records

ADR files preserve why durable decisions were made. They are historical records, not editable summaries of current code.

## Statuses

- `Proposed`: under active decision review; implementation must not freeze a persistent contract yet.
- `Accepted`: approved and binding.
- `Superseded by ADR-NNNN`: replaced by a later Accepted ADR; retain the file unchanged except the status/reference.
- `Rejected`: considered and explicitly not selected.

## Rules

Create an ADR before changing product authority, persistent AE topology, Language IDs/frontends, source envelope grammar, Shader ABI, RenderGraph schema, ParamId/binding semantics, StateToken/sequence schema, identity/cache domains, history behavior, host matrix, installation boundary, or delivery ordering.

An Accepted ADR is immutable. To change it:

1. write a new ADR with new evidence;
2. obtain approval;
3. mark the old ADR `Superseded by ADR-NNNN`;
4. update architecture, status, roadmap, test matrix, and audits;
5. never rewrite the old rationale to make history appear linear.

Use [TEMPLATE.md](TEMPLATE.md). Ordinary implementation details and bug fixes do not require ADRs unless they change a listed durable contract.

## Accepted decisions

| ADR | Decision | Status |
|---|---|---|
| [0001](0001-expression-authority-and-open-runtime.md) | Expression authority and open runtime | Accepted |
| [0002](0002-extensible-language-frontends.md) | Language popup and extensible frontends | Accepted |
| [0003](0003-render-graph-is-core.md) | Multi-pass RenderGraph is core | Accepted |
| [0004](0004-breaking-rewrite-and-host-matrix.md) | In-place breaking rewrite and Windows AE 2023-2026 | Accepted |
| [0005](0005-stable-parameter-ids.md) | Stable Param IDs over a fixed AE pool | Accepted |
| [0006](0006-state-and-persistence-boundary.md) | New StateToken and sequence schema boundary | Accepted |
| [0007](0007-identity-and-cache-boundaries.md) | Layered identities and cache keys | Accepted |
| [0008](0008-product-scope-and-delivery-order.md) | Product scope and delivery order | Accepted |
| [0009](0009-staged-format-adr-acceptance.md) | Staged format-ADR acceptance and M0 transport spike | Accepted |
| [0010](0010-stable-language-ids.md) | Stable Language numeric IDs | Accepted |
| [0011](0011-shader-abi-v1-core.md) | Shader ABI v1 core | Accepted |
| [0012](0012-source-envelope-marker-and-limits.md) | Source envelope marker and size limits | Accepted |
| [0013](0013-paramid-grammar-and-pools.md) | ParamId grammar, parameter pools, and growth policy | Accepted |
| [0014](0014-windows-host-protocol.md) | Windows AE 2023-2026 build/install/test protocol | Accepted |
| [0015](0015-statetoken-and-diagnostics.md) | StateToken v1 layout, publication semantics, diagnostic registry | Accepted |
| [0016](0016-sequence-schema-v1.md) | Sequence schema v1 — codec, limits, checksum | Accepted |
| [0017](0017-hash-domains.md) | Hash algorithm, canonical serialization, domain separation | Accepted |
| [0018](0018-envelope-grammar-v1.md) | Multi-pass source envelope grammar v1 | Accepted |
| [0019](0019-intermediate-format-policy.md) | Intermediate format policy (v1) | Accepted |
| [0020](0020-executionplan-aliasing.md) | ExecutionPlan resource aliasing (v1) | Accepted |
| [0021](0021-precision-alpha-color-policy.md) | Working precision, alpha, and color policy (M5) | Accepted (16-bpc rows superseded by 0022) |
| [0022](0022-16bpc-working-format-f32.md) | 16-bpc working format is Rgba32Float | Accepted |
| [0023](0023-temporal-seek-reset.md) | Temporal history v1 — `prev` input, continuity/reset, MFR-compatible | Accepted (state model superseded by 0025) |
| [0024](0024-history-format-policy.md) | History storage format, lifetime, and update discipline | Accepted (storage model superseded by 0025) |
| [0025](0025-windowed-resimulation.md) | Temporal v2 — windowed re-simulation (`@window`, self-contained frames) | Accepted |
| [0026](0026-color-parameter-default-annotation.md) | Color parameter `default:` annotation | Accepted |
| [0027](0027-0.0.1-prerelease-scope.md) | 0.0.1 pre-release scope (verified-host subset; four-year gate kept for 1.0) | Accepted |
| [0028](0028-details-button-and-slider-precision.md) | Details button (topology append) and float-slider precision | Accepted |
| [0029](0029-logical-resolution-abi.md) | `u_resolution` is the logical full-resolution frame size | Accepted |
| [0030](0030-layer-input-parameters.md) | Layer input parameters (`hint:layer`) — Layer pool, comp-space `uv`, temporal refused | Accepted |
| [0031](0031-gradient-parameters.md) | Gradient parameters (`hint:gradient`) — 8-stop format, 256×1 float LUT, custom-UI editor | Accepted (§2 superseded by 0032; §3/§6/§7 by 0033) |
| [0032](0032-gradients-are-graph-resources.md) | Gradients are graph resources — one extra-input binding rule for layers and gradients alike | Accepted |
| [0033](0033-gradient-stops-are-ordinary-parameters.md) | Gradient stops are ordinary parameters — no arbitrary data, AE owns persistence and keyframes | Accepted |
| [0034](0034-point3d-parameters.md) | Point 3D parameters (`hint:point3d`) — closes ADR-0013's reserved kind | Accepted |
| [0035](0035-path-parameters.md) | Path parameters (`hint:path`) — masks as an N×2 vertex texture, count from `textureSize` | Accepted |
| [0036](0036-single-repository-record.md) | Single-repository record — the public repo is the whole record; one document withheld, redactions listed | Accepted |
| [0037](0037-pool-valid-range-and-slider-range.md) | Pool slider valid ranges are wide and fixed at `PARAMS_SETUP`; `@param min:/max:` is the slider range; the runtime never clamps | Accepted |

## Format ADRs (staged per ADR-0009)

Format ADRs are accepted at the entry of the milestone that first implements or persists each contract ([ADR-0009](0009-staged-format-adr-acceptance.md)). Create numbered ADRs from 0010 onward. Until its ADR is Accepted, a staged contract remains session-local and non-contractual.

Required before M0 exit (gate M1 implementation):

- 0010 stable Language numeric IDs — Accepted 2026-08-11;
- 0011 Shader ABI v1 core builtins and semantics (numeric conventions finalized by M1 fixtures) — Accepted 2026-08-11;
- [0012 source envelope version marker and size limits](0012-source-envelope-marker-and-limits.md) (full grammar deferred to M4 entry) — Accepted 2026-08-12 from TR-M0-002/003/004 data;
- [0013 ParamId grammar, aliases, initial pool capacities, and append-only growth policy](0013-paramid-grammar-and-pools.md) — Accepted 2026-08-12 from TR-M0-004/006 data;
- 0014 Windows AE 2023-2026 build/install/test protocol, including wgpu backend/adapter policy and automated harness requirements — Accepted 2026-08-11.

Required at M3 entry:

- [0015 StateToken layout, undo/redo and project-dirty semantics, stable diagnostic code registry](0015-statetoken-and-diagnostics.md) — Accepted 2026-08-12;
- [0016 sequence schema v1 codec, limits, and checksum](0016-sequence-schema-v1.md) — Accepted 2026-08-12;
- [0017 hash algorithm, canonical serialization, and domain separation](0017-hash-domains.md) — Accepted 2026-08-12.

Required at M4 entry:

- [0018 full multi-pass source envelope grammar and escaping](0018-envelope-grammar-v1.md) — Accepted 2026-08-13;
- [0019 intermediate format policy](0019-intermediate-format-policy.md) — Accepted 2026-08-13;
- [0020 ExecutionPlan resource aliasing](0020-executionplan-aliasing.md) — Accepted 2026-08-13.

Required at M5 entry (staged by the deferrals in ADR-0011 §6 and ADR-0019 §2):

- [0021 working precision, alpha, and color policy](0021-precision-alpha-color-policy.md) — Accepted 2026-08-13.

Required at M6 entry:

- [0023 temporal seek/reset semantics](0023-temporal-seek-reset.md) — Accepted 2026-08-13;
- [0024 history format policy](0024-history-format-policy.md) — Accepted 2026-08-13.

Post-M7 (follow-up features):

- [0026 color parameter `default:` annotation](0026-color-parameter-default-annotation.md) — Accepted 2026-08-14;
- [0027 0.0.1 pre-release scope](0027-0.0.1-prerelease-scope.md) — Accepted 2026-08-14 (pre-release host subset; ADR-0014 §7 four-year gate kept for 1.0); its two-repository publication clause superseded by 0036;
- [0028 Details button and float-slider precision](0028-details-button-and-slider-precision.md) — Accepted 2026-08-14 (first-user-feedback fixes);
- [0029 logical-resolution ABI](0029-logical-resolution-abi.md) — Accepted 2026-08-14 (preview-downsample invariance; user defect report);
- [0030 layer input parameters (`hint:layer`)](0030-layer-input-parameters.md) — Accepted 2026-08-15 (public issue #1; Layer pool kind, graph-grammar extension, comp-space `uv`, temporal combination refused in the first release);
- [0031 gradient parameters (`hint:gradient`)](0031-gradient-parameters.md) — Accepted 2026-08-15 (public issue #2; Gradient pool kind, 8-stop persistent format, 256×1 float LUT, custom-UI editor); §2 superseded by 0032;
- [0032 gradients are graph resources](0032-gradients-are-graph-resources.md) — Accepted 2026-08-15 (one extra-input binding rule; corrects an unexecutable rule authored into 0031 §2);
- [0033 gradient stops are ordinary parameters](0033-gradient-stops-are-ordinary-parameters.md) — Accepted 2026-08-15 (withdraws 0031's arbitrary-data value after it crashed AE; parameter structure matched against a shipping reference effect);
- [0034 Point 3D parameters](0034-point3d-parameters.md) — Accepted 2026-08-15 (closes the ADR-0013 §3 reservation; `vec3` stays a colour unless annotated);
- [0035 path parameters](0035-path-parameters.md) — Accepted 2026-08-15 (masks as vertex textures; Beziers delivered, not flattened);
- [0036 single-repository record](0036-single-repository-record.md) — Accepted 2026-08-17 (public repo becomes the whole record after the private one was archived; competitor analysis withheld, its six citation redactions listed);
- [0037 pool valid range and slider range](0037-pool-valid-range-and-slider-range.md) — Accepted 2026-08-19 (public issue #5: `PF_UpdateParamUI` cannot change `valid_*`, so the registered `0..1`/`0..10` ranges clamped every float above 1 and int above 10 at render; wide registered range, annotation range = slider range, runtime never clamps).
