# ADR-0016: Sequence schema v1 — codec, limits, checksum

- Status: Accepted
- Date: 2026-08-12
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md) §13
- Related decisions: [ADR-0006](0006-state-and-persistence-boundary.md), [ADR-0010](0010-stable-language-ids.md), [ADR-0012](0012-source-envelope-marker-and-limits.md), [ADR-0013](0013-paramid-grammar-and-pools.md), [ADR-0015](0015-statetoken-and-diagnostics.md), [ADR-0017](0017-hash-domains.md)
- Related tests/audits: TR-M0-004 in [../TEST_MATRIX.md](../TEST_MATRIX.md); `docs/audits/03-persistence-render-clone.md`

## Context

The sequence snapshot is the only persistent definition carrier (ADR-0013 dropped DefinitionData on measured evidence) and the only path by which aerender and reopened projects can render without a UI observation. The spike measured the carrier itself: PF sequence flatten round-trips a 16 MiB checksummed payload byte-exact, persisting at roughly 2× as hex inside the `.aep` (TR-M0-004). ADR-0012 caps the serialized snapshot at 8 MiB for host stability; ADR-0010 assigns the persistent LanguageId encoding here; ADR-0013 requires the ParamId→slot map to survive save/reopen so keyframed streams stay aligned. Until this ADR is Accepted, `flatten()` writes zero bytes by design (ADR-0009).

## Decision

1. **Envelope.** Little-endian throughout. Layout:

   | Field | Type | Meaning |
   |---|---|---|
   | magic | 4 bytes | `DFXS` |
   | schema | u16 | `1` |
   | flags | u16 | `0` in v1; nonzero → `SnapshotSchemaUnknown` (no bit is silently ignorable) |
   | body_len | u32 | body byte count |
   | crc32 | u32 | CRC-32/ISO-HDLC over magic..body (all preceding fields with crc32 itself zeroed, then the body) |
   | body | bytes | §2 |

   Anything failing magic/length/CRC checks is `SnapshotCorrupt`: the snapshot is discarded whole, rendering fails closed, and the committed expression remains the recovery authority. An unknown `schema` is `SnapshotSchemaUnknown`: rendering fails closed with the diagnostic and **no re-binding happens implicitly** — fresh allocation could silently misalign keyframes, so re-binding an unknown-schema project requires the user's explicit Compile.
2. **Body.**

   | Field | Type | Meaning |
   |---|---|---|
   | language | u32 | `LanguageId` (ADR-0010's persistent encoding, fixed here) |
   | fingerprint | u64 | ADR-0017 SessionToken-domain digest of (language, source); cross-checks the StateToken |
   | source_len | u32 | UTF-8 byte count, ≤ 4 MiB (ADR-0012) |
   | source | bytes | exact committed source, unmodified |
   | map_count | u16 | ParamId→slots entries, ≤ 256 |
   | entries | — | per entry: id_len u8, id bytes (ADR-0013 grammar, ≤ 64), slot_count u8 (1-2 in v1), then per slot kind u8 (0 Float, 1 Integer, 2 Bool, 3 Color, 4 Point2D, 5 Angle — append-only registry aligned with ADR-0013's pool table) + index u16 |

   The map is the saved `BindingPlan`, and on restore it seeds `build_with_reuse` as the previous plan — the same inheritance path that already keeps keyframes across live edits keeps them across save/reopen. Nothing else persists: no SPIR-V, no pipelines, no diagnostics, no annotation metadata (re-derived from source), and never a session generation (repository invariant).
3. **Budget.** Worst case is bounded by construction: 20-byte header + 16-byte body head + 4 MiB source + 256 × 70-byte map entries ≈ 4.2 MiB, comfortably inside ADR-0012's 8 MiB. A snapshot that would exceed the budget is a construction bug, not a runtime path — `flatten()` asserts the bound and refuses (diagnostic, no partial write) rather than persisting an oversized payload.
4. **Restore semantics.** UI clones: the snapshot is a seed, never an override — on the first observation, the committed expression wins; if its content equals the snapshot's, the map seeds slot inheritance; if it differs, the map still seeds `build_with_reuse` (stable IDs inherit, everything else reallocates). Render clones (aerender, render-project copies): no observation exists, so verify fingerprint against the StateToken (ADR-0015 §2), compile from the snapshot source, bind with the snapshot map, and render — this is the path that turns today's measured aerender pass-through into shader output.
5. **Versioning.** `schema` bumps only through a superseding ADR; v2 readers must read v1 forever (the product's first persisted projects are v1). Pre-release, v1 itself may be amended only while no persisted project exists outside the test corpus, recorded in the test matrix.

## Alternatives considered

- Persisting only the source and re-binding fresh on load: rejected; fresh allocation after a definition history with holes misaligns keyframed streams (the exact failure ADR-0013's map exists to prevent).
- A self-describing format (JSON/CBOR): rejected; a fixed binary layout with a CRC is smaller, unambiguous byte-for-byte, and this schema changes only through ADRs anyway.
- Storing compiled SPIR-V for faster clone startup: rejected; artifacts are cache, not authority (ADR-0007), compile-on-restore is milliseconds, and persisted artifacts would couple projects to compiler versions.
- CRC over the body only: rejected; header corruption (schema/length) must also fail the checksum, not just parse luckily.

## Consequences

### Benefits

- aerender and reopened projects gain a complete, checksummed authority to render from — the M1-measured pass-through limitation closes.
- Keyframe alignment survives save/reopen through the same reuse path proven live in M2.
- Corruption and future-schema cases fail closed with distinct diagnostics and never guess.

### Costs and risks

- The kind byte registry and entry layout are permanent contracts; growth is append-only.
- Unknown-schema projects deliberately refuse implicit re-binding — users opening far-future projects in this build see Invalid until they explicitly recompile. Honest, but a support surface.
- The snapshot doubles the source in the `.aep` (expression stream + snapshot, ~3× total as hex) — bounded by ADR-0012's caps, measured safe at far larger sizes.

## Revisit conditions

Host evidence that PF sequence data is unreliable on any target year at these sizes (contradicting TR-M0-004), or a real need to persist more than source + map (e.g. measured compile times that make artifact caching worthwhile), justifies a superseding schema ADR.

## Verification obligations

- Rust unit tests: golden-bytes round-trip, CRC bit-flip rejection at every field, unknown-schema/flags rejection, map round-trip with non-contiguous slots, budget assertion, truncation rejection.
- TR-M3-001 host legs: save → reopen renders without a Compile click with keyframes aligned; aerender renders the shader from the snapshot (closing the TR-M1-004 pass-through note); a hex-corrupted snapshot in a saved project fails closed with `SnapshotCorrupt` and the expression path recovers it.
