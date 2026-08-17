# ADR-0022: 16-bpc working format is Rgba32Float

- Status: Accepted
- Date: 2026-08-13 (Accepted with user approval at the M5 report; completes the M5 exit)
- Owners: DynamicFX project
- Related decisions: supersedes the 16-bpc row of [ADR-0021](0021-precision-alpha-color-policy.md) §1-2; [ADR-0019](0019-intermediate-format-policy.md) unchanged
- Related tests/audits: TR-M5-001; `docs/audits/05-pixel-formats.md`

## Context

ADR-0021 §1 chose `Rgba16Unorm` as the 16-bpc working format, rejecting f32 on bandwidth grounds. The first live 16-bpc render refuted it: wgpu refuses 16-bit norm formats as render attachments regardless of adapter capability — `Device::create_render_pipeline` fails validation with **"Format Rgba16Unorm is not renderable"** (AE 2025, DX12, RTX 5080; the panic dialog is preserved in the M5 audit evidence). `TEXTURE_FORMAT_16BIT_NORM` enables sampling and storage use only. The renderable-format table is a wgpu invariant, not a host quirk: no target machine can render to it.

## Decision

1. 16-bpc comps use **`Rgba32Float`** as the working format — the same as 32-bpc — for I/O, pass targets, and intermediates.
2. Boundary conversions stay exact: U15 → f32 as `v / 32768.0` (every U15 integer is exact in f32's 24-bit mantissa; dividing by 2^15 only shifts the exponent); f32 → U15 as `round(clamp(v, 0, 1) × 32768)`, the clamp being AE's own 16-bpc range (no over-white/negatives exist in U15; NaN clamps to 0). The exhaustive 0..=32768 round-trip unit sweep transfers unchanged.
3. Everything else in ADR-0021 stands: depth-transparent shaders, carry-never-convert alpha, compute-in-comp-space color, per-depth cache keying, fixtures as the contract.

## Consequences

- **Better precision than promised:** 16-bpc pipelines now carry full f32 between passes — zero per-hop quantization of any kind; the single quantization to U15 happens once at the output boundary.
- **Higher cost than promised:** 16-bpc working memory is ×4 over 8-bpc rather than ADR-0021's projected ×2 — identical to 32-bpc cost. ADR-0020 aliasing still bounds the count; M7 owns measurement. ADR-0021's "always Rgba32Float above 8-bpc" alternative, rejected there on bandwidth, is what reality enforces.
- 16-bpc requires `FLOAT32_FILTERABLE` (not `TEXTURE_FORMAT_16BIT_NORM`); on adapters without it, 16/32-bpc fail closed to pass-through with a diagnostic log while 8-bpc keeps working.
- Hardening added with the fix: pipeline creation runs inside a wgpu validation error scope, so any future format/pipeline validation failure becomes the documented log + pass-through path instead of a panic dialog inside AE (the dialog also deadlocks scripted harness runs — measured).

## Revisit conditions

wgpu gaining renderable 16-bit norm support would allow restoring ADR-0021's mapping for the bandwidth win; that is an M7-measurement decision, not a correctness one.

## Verification obligations

Unchanged from ADR-0021 §Verification (the same TR-M5-001 fixture matrix must pass); the unit sweep asserts the f32 mapping's losslessness and clamp behavior.
