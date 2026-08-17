# ADR-0019: Intermediate format policy (v1)

- Status: Accepted
- Date: 2026-08-13
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md) §9, §16
- Related decisions: [ADR-0008](0008-product-scope-and-delivery-order.md), [ADR-0011](0011-shader-abi-v1-core.md), [ADR-0018](0018-envelope-grammar-v1.md)
- Related tests/audits: TR-M4-001 in [../TEST_MATRIX.md](../TEST_MATRIX.md); `docs/audits/04-multipass-graph.md`

## Context

Multi-pass introduces textures the user never sees directly: the intermediates between passes. Their format decides precision loss at every hop, and their policy must not pre-empt M5 — the milestone that owns 16/32-bpc and the alpha/color contract (image correctness before performance, ADR-0008; ABI v1 deliberately claims nothing beyond "unchanged", ADR-0011 §6).

## Decision

1. **v1 intermediate format: `Rgba8Unorm`** — identical to the current I/O boundary, so a value crossing a pass hop loses exactly as much as it loses crossing the AE boundary today, and a one-pass graph stays bit-identical to the raw single-pass path.
2. **No per-texture format syntax.** The envelope grammar (ADR-0018) carries no format field; format is a pipeline-level policy. When M5's format ADR raises the working precision, intermediates follow the pipeline working format wholesale — a policy swap, not an envelope change. (This boundary is the point of this ADR: the grammar must not fossilize premature per-texture precision knobs.)
3. **Geometry.** v1 intermediates share the pass target extent (the layer extent, ADR-0011 §5); no per-pass resolution scaling (that is M7 performance territory).
4. **Sampling and content semantics.** Intermediates are sampled exactly like the input: linear filtering, clamp-to-edge (ADR-0011 §5). Bytes pass between passes unchanged — no premultiplication conversion, no color transform (extends ADR-0011 §6's "unchanged" claim across hops; the full alpha/color policy remains M5's).
5. **Identity.** The intermediate format participates in `ExecutionPlanKey` and pipeline target state (ADR-0007) exactly as the output format already does; no new hash domain is needed.

## Alternatives considered

- RGBA16F intermediates now ("free" precision): rejected; it would smuggle an untested precision claim past M5's fixture discipline and double intermediate memory before aliasing (ADR-0020) is even measured.
- Per-texture format declarations in the envelope: rejected; users would encode today's hardware folklore into permanent project text, and M5 would inherit a syntax it did not design.
- Matching AE's comp bit depth automatically in v1: rejected; the render path is 8-bpc-normalized at the boundary today (16-bpc converts), so "matching" would claim precision the pipeline does not yet deliver — exactly the dishonesty M5 exists to remove.

## Consequences

### Benefits

- Multi-pass lands with zero new precision claims; every hop is as honest as the existing boundary.
- M5 upgrades precision in one policy place, with the envelope grammar untouched.
- Plan/pipeline identity already accounts for format, so the M5 swap invalidates exactly the right caches.

### Costs and risks

- Chained passes accumulate 8-bit quantization per hop (banding risk in gradients-through-blur chains). This is the known, accepted cost of correctness-first ordering; M5 removes it, and the M4 audit must record a visible example honestly.
- Memory per intermediate is extent×4 bytes; ADR-0020's aliasing is the containment.

## Revisit conditions

M5's format ADR supersedes the format choice by design. Independent of M5, measured artifacts that make 8-bpc intermediates unusable for the flagship two-pass scenarios would justify an early targeted bump — through a superseding ADR, with fixtures.

## Verification obligations

- TR-M4-001 includes a two-pass identity check: a pass chain whose net transform is identity (e.g. copy → copy) must be byte-exact against the single-pass render.
- The M4 audit records one deliberate quantization example (documented, not hidden) as the M5 handoff.
