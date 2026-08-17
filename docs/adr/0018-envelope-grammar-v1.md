# ADR-0018: Multi-pass source envelope grammar v1

- Status: Accepted
- Date: 2026-08-13
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md) §7, §8, §9
- Related decisions: [ADR-0003](0003-render-graph-is-core.md), [ADR-0011](0011-shader-abi-v1-core.md), [ADR-0012](0012-source-envelope-marker-and-limits.md), [ADR-0013](0013-paramid-grammar-and-pools.md), [ADR-0015](0015-statetoken-and-diagnostics.md), [ADR-0016](0016-sequence-schema-v1.md), [ADR-0017](0017-hash-domains.md)
- Related tests/audits: TR-M4-001 in [../TEST_MATRIX.md](../TEST_MATRIX.md); `docs/audits/04-multipass-graph.md`

## Context

ADR-0012 reserved `@dynamicfx <version>` and version `1` for this grammar; everything behind the marker has failed closed (`E3`) since M1. The grammar must satisfy architecture §7.1: readable, copy-pastable, embeddable in a JavaScript backtick string, safely boundable — and it becomes persistent user-authored text the moment it is Accepted. The committed text (envelope included) is already the unit of persistence and identity: ADR-0016 snapshots it whole and ADR-0017 fingerprints it whole, so this grammar changes neither.

## Decision

1. **Shape.** Line-oriented directives inside the reserved prefix; CRLF tolerated; directives are recognized after stripping leading whitespace; case-sensitive:

   ```text
   @dynamicfx 1
   @graph
   pass blur_h: input -> temp
   pass blur_v: temp -> output
   @end
   @pass blur_h
   ...GLSL 450 module (ABI v1)...
   @endpass
   @pass blur_v
   ...GLSL 450 module (ABI v1)...
   @endpass
   ```

   Between sections, blank lines and `//` comment lines are allowed and ignored. Anything else outside a section is `E6 EnvelopeSyntax` (a new code appended to the 1-15 source/envelope family of ADR-0015's registry; every syntax error reports its 1-based line).
2. **Graph manifest.** Exactly one `@graph`…`@end` block, before any `@pass`. Each line inside is `pass <name>: <in1>[, <in2>…] -> <out>` (or a blank/comment line). Names — pass names and resource names alike — use the ADR-0013 ParamId grammar (`[A-Za-z_][A-Za-z0-9_]*`, ≤ 64 bytes); `input` and `output` are reserved resource names for the effect input and final output. Limits (v1 contract): ≤ 16 passes, ≤ 4 inputs per pass, ≤ 15 intermediate resources.
3. **Graph rules (v1), each violation a distinct `E6` message:** the graph is a DAG (no cycles, no self-reads — feedback is M6); exactly one pass writes `output`, and `output` is never read; `input` is read-only; every intermediate has exactly one writer and at least one reader (an unread intermediate is an error, not a warning — silent typos must not ship); pass names and the manifest/pass-section sets must match one-to-one; duplicate pass or resource-writer declarations are errors.
4. **Pass sections.** `@pass <name>` … `@endpass`, one per manifest pass, any order. The body is the pass's module source, byte-preserved except for one escape: a body line whose first non-whitespace is `@` is a directive line — `@endpass` terminates the section, a leading `@@` is unescaped to a literal `@` line, anything else is `E6`. (GLSL has no line-leading `@` outside comments, so real collisions are comment-only; annotations like `// @param` start with `/`, not `@`, and are untouched.)
5. **Modules and the ABI.** Each pass body compiles as an independent ABI v1 module (ADR-0011: same entry, same `FxUniforms` head). Multi-input passes bind their manifest inputs in order: input 0 at binding 0 (`u_input`), input *i* ≥ 1 at binding 2+*i* (3, 4, 5 — the space ADR-0011 reserved; no ABI version bump, single-input v1 modules remain valid). Declaring a binding the manifest does not feed is an `E18` ABI violation.
6. **Parameters are effect-wide (ADR-0013).** User members across all passes share one ParamId namespace: same name = same parameter and must have the same type everywhere (`E19` on conflict); each pass declares only the members it uses. Annotations parse once over the whole committed text (duplicate `@param` ids stay errors regardless of which pass body they sit in).
7. **Identity.** Raw single-pass input remains exactly what it was — an implicit one-pass graph. Per-pass `ModuleHash` input is the unescaped pass body (ADR-0017's "canonical pass source bytes"); `GraphHash` covers the manifest's canonical topology; the token/snapshot fingerprint stays the whole committed text, so ADR-0016 and the M3 transport are untouched by this ADR.

## Alternatives considered

- JSON/structured manifest: rejected again (ADR-0012 grounds); the expression editor is the authoring surface and text must survive copy-paste through backticks.
- Inferring the graph from `layout(binding=…)` declarations without a manifest: rejected; topology would be implicit, unreadable, and unverifiable against typos.
- Allowing multiple writers per resource with last-writer-wins: rejected; silent ordering dependence is exactly what a validated DAG exists to prevent.
- Fenced/heredoc pass delimiters (```-style): rejected; the `@` directive family is already reserved, and one escape rule (`@@`) is simpler than fence-collision handling.

## Consequences

### Benefits

- Multi-pass effects are one readable committed string — same authority, same persistence, same transport as today, zero schema change.
- Errors are line-numbered and fail the whole definition closed; no partial graphs ever render.
- The reserved-binding space absorbs multi-input without breaking any existing module.

### Costs and risks

- Effect-wide sharing has a quiet failure shape: two passes that *coincidentally* declare the same member name with the same type merge into one slider silently (a type mismatch is caught, a type coincidence is not). Mitigations: the single shared slider is immediately visible in the panel and moves both passes at once; naming conventions (`radius_h`/`radius_v`) express independence explicitly; if real corpora make this a recurring trap, a per-pass-private parameter marker is a pure grammar append, not a break.
- The grammar is a permanent user-facing contract; renaming directives later means a v2 envelope version.
- One-writer/no-read-of-output rules exclude some exotic topologies (multi-tap of the final image) until a future version adds explicit copies.
- The `@@` escape is one more rule for users who write `@endpass` inside comments — rare, documented, and diagnosed by line.

## Revisit conditions

Real shader corpora requiring more passes/inputs/intermediates than the v1 limits, or M6 feedback needing graph-level syntax this version cannot append (history declarations are expected to arrive as new directive lines, which is an append), justify envelope version 2 through a superseding ADR.

## Verification obligations

- Rust unit tests: golden parse of the §1 example; every §3 rule violated in isolation with line-numbered `E6`; the `@@` escape round-trip; limit boundaries (17 passes, 5 inputs, unread intermediate); cross-pass parameter type conflict (`E19`); manifest/section mismatch both ways.
- TR-M4-001: parser/validator/scheduler suite green; a two-pass separable blur renders on AE 2025 with numeric probes; a single-pass envelope (`@dynamicfx 1`, one pass) matches the raw-source render of the same module byte-for-byte.
