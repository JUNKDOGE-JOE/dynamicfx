# ADR-0010: Stable Language numeric IDs

- Status: Accepted
- Date: 2026-08-11
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md) §4.1, §6
- Related decisions: [ADR-0002](0002-extensible-language-frontends.md), [ADR-0009](0009-staged-format-adr-acceptance.md)
- Related tests/audits: TR-M1-003 in [../TEST_MATRIX.md](../TEST_MATRIX.md)

## Context

ADR-0002 fixes an extensible `LanguageFrontend` registry selected by a non-time-varying `Language` popup. The selected language persists across save/reopen and enters `ModuleHash` (ADR-0007), so its identity must be a stable number, not a display string or a menu position. AE popup parameters persist their 1-based menu position, and popup menus are declared at `PARAMS_SETUP`; menu edits in later builds must never reinterpret older projects.

## Decision

1. `LanguageId` is an unsigned 32-bit integer with a permanent, append-only registry:
   - `0` — reserved; means invalid/unknown and is never a selectable language;
   - `1` — GLSL; the default language;
   - `2` — WGSL; reserved now, not implemented in Phase 1;
   - `3+` — future languages, assigned in ascending order, never reused, reordered, or repurposed.
2. The Language popup declares only implemented languages. The v1 menu is exactly `["GLSL"]`. Across plugin builds the menu may only append entries; existing menu positions never change meaning. Each build carries a fixed position→`LanguageId` table (v1: position 1 → ID 1).
3. Persistence authority: the sequence snapshot stores the `LanguageId` (encoding fixed by the M3 sequence-schema ADR). The popup stream is derived UI state. On disagreement the snapshot ID wins and the popup is corrected on the next main-thread commit opportunity.
4. An ID unknown or unimplemented in the running build produces the Invalid state with a stable diagnostic and input pass-through. The stored ID and committed source are preserved unmodified; nothing is clamped to a supported language.
5. The observer maps a committed popup position to a `LanguageId` at commit time; a Language change triggers immediate observation per architecture §5.3.
6. Display names and menu labels are presentation only; they never participate in identity, persistence, or hashing. Frontend versions are separate `ModuleHash` inputs (ADR-0007) and are not encoded in the ID.

## Alternatives considered

- String identifiers (`"glsl"`): rejected; the popup persists numbers, strings invite rename drift, and a parallel string registry would still need stable numbers for hashing.
- Popup position as the persistent identity: rejected; any menu edit would silently reinterpret saved projects.
- Auto-detecting language from source text: rejected by ADR-0002 as the authoritative mechanism.

## Consequences

### Benefits

- Saved projects survive menu growth and localization unchanged.
- Unsupported languages fail closed with the user's data intact.
- New frontends are additive: one registry row, one appended menu entry.

### Costs and risks

- The ID registry is a permanent contract from the first persisted project onward.
- Correcting a mismatched popup requires a main-thread commit path that must not fight the user's own edits.
- Menu append-only discipline must be enforced by review; nothing in AE enforces it.

## Revisit conditions

Only evidence that AE cannot reliably persist the popup/menu model across the 2023-2026 matrix (host rows in the test matrix) justifies replacing the popup as the selection surface; the numeric registry itself would survive any such change.

## Verification obligations

- Rust unit tests: position↔ID mapping, unknown-ID rejection, append-only registry guard.
- TR-M1-003: default GLSL on fresh apply; save/reopen retains the ID.
- Invalid-language diagnostic and pass-through behavior on at least one Windows AE year at M1; per-year rows per ADR-0014.
