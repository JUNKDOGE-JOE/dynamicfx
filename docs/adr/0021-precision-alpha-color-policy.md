# ADR-0021: Working precision, alpha, and color policy (M5)

- Status: Accepted; the §1-2 16-bpc working-format rows are superseded by [ADR-0022](0022-16bpc-working-format-f32.md) (live wgpu evidence) — every other decision stands
- Date: 2026-08-13 (Proposed and Accepted with explicit user approval)
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md) §16
- Related decisions: [ADR-0008](0008-product-scope-and-delivery-order.md), [ADR-0011](0011-shader-abi-v1-core.md), [ADR-0014](0014-windows-host-protocol.md), [ADR-0019](0019-intermediate-format-policy.md), [ADR-0020](0020-executionplan-aliasing.md)
- Related tests/audits: TR-M5-001 in [../TEST_MATRIX.md](../TEST_MATRIX.md); `docs/audits/05-pixel-formats.md`

## Context

Everything above 8-bpc is currently a lie of omission: 16-bpc frames are squeezed through the 8-bpc path (a measured, temporary normalization), 32-bpc is unhandled, and ADR-0011 §6 deliberately claimed nothing about alpha or color beyond "unchanged". ADR-0019 §2 designed the intermediates so precision could be promoted in one policy place. AE's own formats are fixed facts this ADR must map onto: 8-bpc ARGB u8; 16-bpc ARGB in Adobe's U15 encoding (white = 32768, not 65535); 32-bpc ARGB float, linear-light capable of over-white and negative values. The spike also showed color management is real on target machines (the default-ACES host), so color claims must be fixture-pinned per management mode, not assumed.

## Decision

1. **Working format follows the comp depth**, applied to the whole pipeline — I/O, every pass target, and every intermediate (the ADR-0019 §2 wholesale swap; no per-texture choice exists):
   - 8-bpc → `Rgba8Unorm` (unchanged);
   - 16-bpc → `Rgba16Unorm`;
   - 32-bpc → `Rgba32Float`.
2. **Boundary conversions are exact and reversible where the target permits:**
   - 8-bpc: byte-passthrough with ARGB↔RGBA reorder (today's path);
   - 16-bpc: U15→unorm16 as `round(v × 65535 / 32768)` and back — injective in both directions over U15's 0..32768 domain, so a unit-pinned round-trip is lossless (this is why `Rgba16Unorm`, not `Rgba16Float`, whose ~11-bit mantissa would silently discard up to 4 of U15's 15 bits);
   - 32-bpc: float passthrough with reorder only — over-white and negative values survive untouched.
3. **Shaders are depth-transparent.** The module interface does not change (no ABI bump): colors arrive normalized (0..1 for 8/16; linear float with 1.0 = white and unbounded range for 32). One module runs at all three depths; the depth is a host fact, not a shader parameter.
4. **Alpha policy: carry, never convert.** The runtime performs no premultiplication or unpremultiplication anywhere — shaders see exactly the alpha relationship AE delivers at the effect boundary, per channel and per depth. The *measured* boundary semantics (straight vs premultiplied, per depth) are host behavior; the M5 fixtures pin them and the audit records them as the documented contract. A convenience unpremultiply helper is future ABI surface, not silent runtime behavior.
5. **Color policy: compute in the comp's working space.** The runtime performs no color transforms; whatever space AE hands the effect (unmanaged, managed, linearized) is the space shaders compute in. Fixtures run once in an unmanaged project and once in an ACES-managed project, and the audit records both measured boundary behaviors. Consequence stated plainly: the same shader produces different numbers under different project color settings — that is AE's contract, and this runtime does not editorialize on top of it.
6. **Identity and caches.** The working format already parameterizes `PipelineKey`, `ExecutionPlanKey`, and `FrameResourceKey` (ADR-0007/0019 §5); per-depth pipelines and plans cache independently with no new hash domain. Snapshot, token, and all persistence formats are untouched (depth is per-render, never per-project-file).
7. **Fixture obligations are the contract.** Every numeric claim above exists only when its fixture passes: U15 round-trip vectors (0, 1, 16384, 32767, 32768), 32-bpc over-white (2.0) and negative (-0.5) survival through an identity pass, a multi-pass chain at 16/32-bpc showing no 8-bit quantization staircase (the recorded M4 handoff closing), alpha-semantics probes on semi-transparent input at each depth, and the managed/unmanaged color pair.

## Alternatives considered

- One universal `Rgba16Float` working format: rejected; it quietly truncates U15's precision and clamps nothing for 32-bpc while paying half-float conversion costs everywhere.
- Always `Rgba32Float` above 8-bpc: workable but 2× the bandwidth of `Rgba16Unorm` for 16-bpc comps with zero precision benefit over the exact unorm mapping; rejected for the common case, revisitable with M7 measurements.
- Automatic unpremultiply/premultiply around shaders: rejected; it destroys effects that are premultiplication-aware (glows, keys) and hides the host's actual contract — carry-and-document is honest and reversible.
- Normalizing color to a fixed space (e.g. always linear): rejected; it would silently fight AE's project color management and double-transform managed projects.

## Consequences

### Benefits

- 16-bpc becomes bit-honest end to end; 32-bpc HDR values survive, including through multi-pass chains.
- The M4-recorded per-hop quantization disappears at high depths with zero envelope/grammar change.
- Depth-transparent shaders mean users author once and render correctly at any comp depth.

### Costs and risks

- Memory/bandwidth scale with depth (×2 at 16, ×4 at 32, including every intermediate); ADR-0020 aliasing contains the count, M7 owns the measurement.
- The alpha and color contracts are *measured host behavior* rather than chosen behavior — if AE's boundary semantics differ across the 2023-2026 matrix, per-year fixtures will say so and the audit inherits a per-year table instead of one sentence.
- `Rgba16Unorm` renderability is required of the DX12 adapter (universally supported on target hardware; the Unavailable diagnostic path covers exotic failures).

## Revisit conditions

M7 measurements showing `Rgba16Unorm` costs exceeding `Rgba16Float` benefits on real graphs, host-matrix evidence of inconsistent boundary alpha semantics that a carry-only policy cannot document coherently, or user demand for in-runtime premultiply helpers justify a superseding ADR (helpers would be an ABI append).

## Verification obligations

- Rust unit tests: U15↔unorm16 round-trip over the full golden vector set plus an exhaustive 0..=32768 sweep; 32-bpc reorder preserving NaN/±inf/negatives; conversion-path selection by depth.
- TR-M5-001 host fixtures on AE 2025 (per ADR-0014 discipline): identity-shader round-trips at 8/16/32-bpc with numeric probes in AE's own values (U15 for 16); over-white/negative survival at 32; the 16/32-bpc multi-pass staircase check; alpha-semantics probes; the managed/unmanaged pair. Other years inherit per-year rows.
