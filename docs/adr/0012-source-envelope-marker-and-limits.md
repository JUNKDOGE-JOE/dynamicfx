# ADR-0012: Source envelope marker and size limits

- Status: Accepted
- Date: 2026-08-12
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md) §5.1, §7, §13
- Related decisions: [ADR-0001](0001-expression-authority-and-open-runtime.md), [ADR-0003](0003-render-graph-is-core.md), [ADR-0009](0009-staged-format-adr-acceptance.md), [ADR-0010](0010-stable-language-ids.md)
- Related tests/audits: TR-M0-002/003/004/007 and TR-M1-004 in [../TEST_MATRIX.md](../TEST_MATRIX.md); [../audits/00-architecture-contract.md](../audits/00-architecture-contract.md)

## Context

From the first target frame the runtime must decide whether a committed `Source.expression` is raw single-pass source for the selected language or a versioned multi-pass envelope (architecture §7.1), even though the full envelope grammar is deferred to M4 entry (ADR-0009). Whatever distinguishes the two becomes persistent the moment a project is saved, so the marker must be fixed before M1 code exists.

The M0 transport spike (AE 2025, 25.6.6x4) measured the host constraints:

- committed expressions carry byte-exact payloads to at least 16 MiB with no host ceiling below the probe cap; a 16 MiB write costs 235 ms (TR-M0-002);
- hostile punctuation, CRLF, and CJK/emoji content survive save/reopen byte-exact — the host applies no normalization (TR-M0-003);
- the sequence carrier round-trips a 16 MiB checksummed payload, but persists it at roughly 2× as hex, and the resulting 33 MB project crashed AE on overlapping open (TR-M0-004);
- a 1 MiB expression evaluates identically in GUI and aerender (TR-M0-007).

Transport capacity is therefore not the binding constraint — host stability under multi-tens-of-MB project payloads is. The committed source will also exist twice per project (the expression stream plus the exact source inside the persisted snapshot, §13), multiplying any oversize decision.

## Decision

1. **Encoding and measurement.** The committed source is treated as UTF-8; every limit in this ADR is measured in UTF-8 bytes. The runtime applies no normalization at any point (TR-M0-003 shows none is needed): it reads the committed text exactly and never modifies the user's expression.
2. **Reserved prefix and detection.** After skipping an optional UTF-8 BOM and ASCII whitespace (space, tab, CR, LF), if the source begins with the exact case-sensitive token `@dynamicfx` followed by whitespace or end-of-input, the input is an **envelope**; otherwise it is **raw single-pass source** handed unmodified to the selected `LanguageFrontend`. `@dynamicfx` is the only token this ADR reserves; all further directive syntax belongs to the M4 grammar ADR.
3. **No fallback.** Once the prefix matches, the input is an envelope forever: a malformed or unsupported envelope produces the Invalid state with a stable diagnostic and input pass-through, and is never compiled as raw source. Ambiguity fails closed rather than drifting silently.
4. **Marker line.** The first envelope line is `@dynamicfx <version>` where `<version>` is a decimal u32 ≥ 1 without leading zeros, followed only by whitespace before the line break (CRLF tolerated) or end-of-input. Version `1` is reserved for the full grammar fixed at M4 entry. Until that grammar ADR is Accepted and implemented, every envelope input — any version — fails closed: Invalid, stable diagnostic, pass-through, committed text preserved unmodified. Unknown or higher versions behave identically, so newer projects opened in older builds degrade safely (the same fail-closed shape as ADR-0010's unknown `LanguageId`).
5. **Committed source cap.** The committed source (raw or envelope alike) is limited to **4 MiB (4,194,304 UTF-8 bytes)**, checked before any parsing. Oversize input produces Invalid + stable diagnostic + pass-through; nothing is truncated, partially compiled, or written back to the expression. Rationale: 16 MiB is host-transportable but persists ~2× in the snapshot plus once in the expression stream, and a 33 MB project already crashed the host (TR-M0-004); 4 MiB keeps worst-case project growth an order of magnitude below the measured crash region while staying ~40× above any realistic multi-pass shader set (well under 100 KiB).
6. **Persisted definition payload budget.** The serialized persisted definition snapshot (sequence schema v1, M3 entry ADR) must fit **8 MiB** total, including its own header and checksum overhead. The M3 codec is designed inside this budget. The budget is host-stability evidence, not an encoding detail, which is why it lives here rather than in the M3 ADR.
7. **Diagnostic classes.** Three stable diagnostic classes must exist and be testable from M1: source-oversize, envelope-unsupported (unknown or unimplemented version), and envelope-malformed (bad marker line). Code values belong to the M3 diagnostic registry (ADR-0009).

## Alternatives considered

- Heuristic detection without a marker (e.g. scanning for `@graph`): rejected; unversionable and ambiguous against arbitrary language source.
- A structured container (JSON wrapper) around source: rejected; architecture §7.1 requires the envelope to be readable, copy-pastable, and embeddable in a JavaScript backtick string — a plain text marker preserves the expression-editor workflow.
- No size cap (16 MiB round-trips cleanly): rejected; TR-M0-004's crash shows project-size stability, not transport, is the limit that matters, and the payload multiplies (expression stream + hex snapshot).
- A much smaller cap (e.g. 256 KiB): rejected; no demand-side evidence forces it, and a too-small cap is the variant most likely to need a superseding ADR later. 4 MiB sits safely inside measured behavior in both directions.
- Truncating or rewriting oversize text in AE: rejected; the runtime never modifies the committed expression (ADR-0001 authority).

## Consequences

### Benefits

- Raw source and envelopes are distinguishable from the first persisted frame, before the M4 grammar exists.
- Newer envelope projects opened in older builds fail closed with the user's source intact.
- Project size stays an order of magnitude away from the measured host crash region.
- M4 designs the full grammar inside an already-reserved, already-versioned prefix.

### Costs and risks

- Raw source whose first non-whitespace token is `@dynamicfx` cannot be compiled as raw source. No supported language begins programs with `@`, so this is theoretical; a future frontend whose language legitimately allows it would need an escaping rule in its own ADR.
- 4 MiB / 8 MiB are contract numbers derived from one AE year's evidence; other years could measure differently. Loosening requires a superseding ADR; instability evidence below the caps forces a tightening revision.
- Until M4, users who paste an envelope see Invalid + pass-through; Status text must clearly say "envelope not supported yet", not look like a defect.

## Revisit conditions

Host-matrix evidence that substantially larger payloads are stable across AE 2023-2026 (e.g. chunked or compressed persistence measured on real hosts), or demonstrated real shader sets exceeding 4 MiB, justify a superseding ADR raising the caps. Evidence that any target year is unstable below the caps forces a tightening revision.

## Verification obligations

- Rust unit tests: BOM/whitespace skipping, case-sensitive prefix match, CRLF tolerance, version parsing (rejects `0`, leading zeros, garbage), oversize boundary at exactly 4 MiB ± 1 byte, and no-fallback (a malformed envelope never reaches a frontend).
- M1, alongside TR-M1-004's host run: an envelope input on a real host shows Invalid + pass-through with the correct diagnostic class; an oversize source is rejected without host instability.
- M3: snapshot serialization enforces the 8 MiB budget with a boundary test.
