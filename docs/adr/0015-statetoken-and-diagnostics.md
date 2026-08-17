# ADR-0015: StateToken v1 layout, publication semantics, and the diagnostic code registry

- Status: Accepted
- Date: 2026-08-12
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md) §4, §13
- Related decisions: [ADR-0006](0006-state-and-persistence-boundary.md), [ADR-0009](0009-staged-format-adr-acceptance.md), [ADR-0016](0016-sequence-schema-v1.md), [ADR-0017](0017-hash-domains.md)
- Related tests/audits: TR-M0-004/005 in [../TEST_MATRIX.md](../TEST_MATRIX.md); M1/M2 host runs (interim token proven live); `docs/audits/03-persistence-render-clone.md`

## Context

The hidden StateToken parameter is the UI→render fast path: AE copies primitive parameter state into render clones, so a single exactly-representable f64 can carry "which published definition should this clone resolve, or why not". M1/M2 proved the mechanics live on AE 2025: a 51-bit token published by the idle observer via `AEGP_SetStreamValue`, resolved through the process registry, failing closed to pass-through on a miss. The spike measured the host behaviors this ADR must respect: AEGP stream writes dirty the project, save clears it, an idle republication of an *unchanged* value does not re-dirty a saved project, and undo reverts the committed expression (TR-M0-005).

What is unfixed: the token's permanent bit layout, what its payload means across sessions, how publication interacts with undo/dirty, and the stable diagnostic codes that repository policy requires for every fail-closed state. All become persistent with the first M3 release, and ADR-0016's snapshot needs the token as its cross-check.

## Decision

1. **Layout.** The token is one f64 holding an exact non-negative integer < 2^53: `(payload << 2) | state`, payload 51 bits.
   - state `0b00` — Uninitialized: fresh instance, nothing published; payload must be 0.
   - state `0b01` — Active: payload is the **definition fingerprint** (ADR-0017's SessionToken domain: 51-bit truncated hash of LanguageId + committed source, nonzero). It keys the process registry and cross-checks the snapshot.
   - state `0b10` — Invalid: a bad source/definition was explicitly observed; payload is the **diagnostic code** (§4), so render clones display the real reason without re-observing.
   - state `0b11` — reserved; decodes as corrupt.
   - Any non-integral, negative, out-of-range, or reserved-state value decodes as **Corrupt** and fails closed (pass-through + `TokenCorrupt` diagnostic). Nothing is ever clamped or guessed.
2. **Meaning across sessions.** The payload is a content fingerprint, never a session counter — reopening a project re-derives the same fingerprint from the same content, so the compile generation stays session-local (repository invariant). Resolution order in a render clone: process registry by fingerprint (fast path) → on miss, rebuild from the ADR-0016 snapshot and **verify** the snapshot's own fingerprint equals the token payload. A mismatch means AE handed the clone a torn pair (parameter stream and sequence data copied at different moments): the snapshot — checksummed, content-complete — wins, the frame renders from it, and the observer corrects the token on the next main-thread pass. A fresh observation on the UI side always outranks both (architecture §5.1).
3. **Publication semantics.**
   - Writes are idempotent: token, slot names, visibility, and defaults are written only when the target value actually differs. Consequence (measured, TR-M0-005): republication after a save does not re-dirty the project; a real content change does, which is correct — the project genuinely changed.
   - Publication never participates in the user's undo story as an independent entry: after any undo/redo the observer re-observes the committed expression and republishes, so rendering always follows the expression state. The token may lag one idle tick behind an undo; it must converge without user action. Token/UI writes use non-undoable AEGP paths where the SDK offers the choice.
   - Programmatic publication must never overwrite user-authored stream values on inherited bindings (ADR-0013 discipline, already enforced for defaults).
4. **Diagnostic code registry.** Codes are `u16`, permanent and append-only from first release; the registry is one Rust table whose uniqueness/append-only shape is unit-guarded, and Status text renders as `E<code> <short text>` (the 31-char PF name limit truncates text, never the code). Domains are pre-partitioned so future codes append within their family:
   - 0 — OK/none;
   - 1-15 source/envelope: 1 `SourceOversize`, 2 `EnvelopeMalformed`, 3 `EnvelopeUnsupported`, 4 `NotSourceBlock`, 5 `NoExpression`;
   - 16-31 frontend: 16 `LanguageUnknown`, 17 `GlslParse`, 18 `AbiViolation`, 19 `ParamRejected`, 20 `SpirvEmit`;
   - 32-47 binding: 32 `PoolOverflow`, 33 `AliasConflict`;
   - 48-63 runtime/transport: 48 `GpuUnavailable`, 49 `RegistryMiss`, 50 `SnapshotCorrupt`, 51 `SnapshotSchemaUnknown`, 52 `TokenCorrupt`;
   - 64+ reserved for future families.
   A code's number is never reused or renumbered; retiring a meaning strikes the row but keeps the number burned.

## Alternatives considered

- A session counter as payload: rejected; it would persist a session-local generation (forbidden invariant) and carries no cross-session verification value.
- Full-width content hash in an arbitrary parameter: rejected; the arb value path is measured-ineffective (TR-M0-004) and the 51-bit primitive path is measured-good.
- Diagnostic text without stable codes: rejected by repository policy; text truncates at 31 chars and localizes badly, codes do not.
- Making token writes undoable so undo restores them atomically with the expression: rejected; the observer's converge-on-observation model already guarantees eventual consistency, and undoable programmatic writes would stuff the user's undo stack with entries they never authored.

## Consequences

### Benefits

- Render clones fail closed with the *actual* diagnostic instead of a generic miss.
- Torn token/snapshot pairs are detected, not silently rendered.
- Saved projects do not go dirty from mere republication (measured), so autosave/dirty UX stays honest.
- The code registry gives tests, logs, and future UI one stable vocabulary.

### Costs and risks

- 51 bits of fingerprint means a ~2^-51 collision can mis-resolve within one process; the registry insert already refuses cross-content collisions, and the snapshot cross-check catches cross-session ones. Accepted and documented.
- Undo convergence via idle means up to ~1 s of stale render after undo before correction (matches the M0-accepted observer cadence).
- The code table is a permanent contract; sloppy early additions are burned numbers forever.

## Revisit conditions

Host evidence that AE fails to copy the primitive parameter into render clones reliably on any target year (breaks the fast path), or that non-undoable AEGP publication measurably corrupts undo/dirty behavior in real projects, justifies a superseding ADR. Payload width changes only with a new token version in a superseding ADR.

## Verification obligations

- Rust unit tests: encode/decode round-trip for every state, corrupt-value rejection (NaN, negatives, fractions, reserved state, >2^53), registry uniqueness/append-only guard, `E<code>` formatting.
- TR-M3-001 host legs: reopen resolves via snapshot with fingerprint verification; a deliberately torn token (scripted overwrite) renders from the snapshot and is corrected by idle; undo of a source edit converges to the prior render; save → idle republication leaves the project clean (extends the TR-M0-005 measurement to the target code).
