# ADR-0017: Hash algorithm, canonical serialization, and domain separation

- Status: Accepted
- Date: 2026-08-12
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md) §14
- Related decisions: [ADR-0007](0007-identity-and-cache-boundaries.md), [ADR-0010](0010-stable-language-ids.md), [ADR-0011](0011-shader-abi-v1-core.md), [ADR-0013](0013-paramid-grammar-and-pools.md), [ADR-0015](0015-statetoken-and-diagnostics.md), [ADR-0016](0016-sequence-schema-v1.md)
- Related tests/audits: `docs/audits/03-persistence-render-clone.md`

## Context

ADR-0007 fixed the *semantic* identity layers (ModuleHash, ArtifactHash, GraphHash, DefinitionHash, PipelineKey, ExecutionPlanKey, FrameResourceKey) but deferred the algorithm, digest form, and canonical input encodings. M3 makes two of these persistent for the first time — the snapshot fingerprint and the StateToken payload — so the algorithm and domain rules must be fixed now, before accidental encodings freeze. The M1/M2 interim used truncated FNV-1a; FNV is fine for change detection but its collision behavior is too weak to anchor persistent identity.

## Decision

1. **Algorithm.** All identity hashes are **BLAKE3-256** (the `blake3` crate, version pinned by the lockfile and reflected in evidence records). Digests are stored/compared as the full 32 bytes except where a domain explicitly truncates. CRC-32 in ADR-0016 is integrity-checking, not identity, and stays CRC.
2. **Domain separation.** Every hash begins with an ASCII domain tag, length-prefixed like every other field: `dfx:module:v1`, `dfx:artifact:v1`, `dfx:graph:v1`, `dfx:definition:v1`, `dfx:token:v1`. Tags are permanent and append-only; changing a domain's inputs bumps its tag suffix through an ADR. Cross-domain equality is meaningless by construction.
3. **Canonical serialization.** Hash inputs use one deterministic encoding, shared with ADR-0016's codec style: little-endian fixed-width integers, u32 length-prefixed UTF-8 strings and byte runs, fields in the exact order this ADR lists, no map-iteration order anywhere (collections hash in their canonical declaration order), absent optionals encoded as a 0 flag byte and present ones as 1 + value.
4. **Domain inputs** (composition fixed here; the byte-exact golden vectors are pinned by implementation-time unit tests, ADR-0011-style):
   - `dfx:module:v1` — LanguageId (u32), frontend version (u32), ShaderAbiVersion (u32), canonical pass source bytes. (ADR-0007's "canonical pass source": v1 canonicalization is byte-identity — no whitespace or comment normalization; TR-M0-003 measured the host preserves bytes exactly, so the committed bytes are already canonical.)
   - `dfx:artifact:v1` — ModuleHash, compiler identity (naga version string), SPIR-V emit options tag.
   - `dfx:graph:v1` — ordered pass ModuleHashes, canonical topology (v1: the implicit single-pass chain, encoded explicitly so M4 graphs extend rather than replace), static resource declarations.
   - `dfx:definition:v1` — GraphHash, parameter schema in declaration order: per param ParamId, ShaderParamType tag, aliases (ordered), and `default` values. Labels and min/max are excluded (display-only; ADR-0010 §6 discipline), defaults are included (they change rendered output through fresh bindings). LanguageId rides in via ModuleHash.
   - `dfx:token:v1` — LanguageId (u32) + committed source bytes, digest truncated to the low 51 bits, zero mapped to 1 (the StateToken payload and snapshot fingerprint, ADR-0015/0016).
   - `PipelineKey`, `ExecutionPlanKey`, `FrameResourceKey` are structured keys (typed tuples over ArtifactHash/GraphHash + device/format state), not digests; they follow ADR-0007 and gain hashes only if a cache ever needs serialized keys, via a future tag.
5. **Stability contract.** From first release, a given input must produce the same digest forever: algorithm, tags, and canonical encoding changes all require superseding ADRs with migration notes. Pre-release amendments are allowed while no persisted project exists outside the test corpus, recorded in the test matrix.

## Alternatives considered

- Keep truncated FNV-1a everywhere: rejected; adequate for session-local change detection, too collision-weak to anchor persisted identity (the token keeps a truncated *BLAKE3* precisely so its 51 bits inherit strong mixing).
- SHA-256: workable, but BLAKE3 is substantially faster on large sources (the cap is 4 MiB), pure-Rust, and equally stable as a format.
- Self-describing hash inputs (serde-derived): rejected; derive output changes with library internals — hash inputs must be hand-canonical.
- Hashing labels/ranges into DefinitionHash: rejected; UI-only edits would churn definition identity, exactly what ADR-0007 forbids.

## Consequences

### Benefits

- Persistent fingerprints get real collision resistance before the first project is saved.
- Domain tags make cross-layer cache misuse a type error in practice, not a latent bug.
- Golden-vector tests turn "the hash changed" into a reviewed decision instead of an accident.

### Costs and risks

- One new dependency (`blake3`) enters the identity path; its version is part of evidence records.
- Byte-identity canonicalization means whitespace-only source edits produce new identities — correct for authority (the user committed different bytes) but means trivial edits recompile. Accepted; caching layers absorb it.
- The golden vectors are a permanent maintenance surface: every intentional format change must update them consciously.

## Revisit conditions

A demonstrated need for source canonicalization beyond byte identity (e.g. cross-platform expression newline mangling measured on a target host — none seen in TR-M0-003), a cryptographic break relevant to BLAKE3's structural use, or M4 graph encoding needs that the v1 topology encoding cannot extend, justify superseding tags.

## Verification obligations

- Rust unit tests: golden digest vectors per domain (including the token truncation and zero-mapping), cross-domain inequality for identical payloads, canonical-order independence from declaration containers, optional-field encoding.
- M3 integration: the snapshot fingerprint and StateToken payload agree end-to-end through flatten → unflatten → verify on a real save/reopen (part of TR-M3-001).
- UI label/order changes leave DefinitionHash unchanged (extends ADR-0007's obligation with concrete vectors).
