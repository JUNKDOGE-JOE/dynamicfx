# ADR-0013: ParamId grammar, parameter pools, and growth policy

- Status: Accepted
- Date: 2026-08-12
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md) §4, §11, §13
- Related decisions: [ADR-0005](0005-stable-parameter-ids.md), [ADR-0007](0007-identity-and-cache-boundaries.md), [ADR-0009](0009-staged-format-adr-acceptance.md), [ADR-0010](0010-stable-language-ids.md), [ADR-0011](0011-shader-abi-v1-core.md)
- Related tests/audits: TR-M0-004/006 and TR-M2-001 in [../TEST_MATRIX.md](../TEST_MATRIX.md); [../audits/00-architecture-contract.md](../audits/00-architecture-contract.md)

## Context

ADR-0005 fixes the model: a fixed AE parameter pool declared at `PARAMS_SETUP`, stable `ParamId`s, atomic `BindingPlan` publication, ordinary keyframed AE streams. What it left open — the ID grammar, alias semantics, pool kinds and capacities, and how pools may grow across builds — becomes persistent with the first saved project, so it must be fixed before M1.

The M0 transport spike (AE 2025, 25.6.6x4) measured two host facts this ADR depends on:

- A Popup's menu is fixed at `PARAMS_SETUP`: in-plugin `set_options` + `PF_UpdateParamUI` return Ok, but AE keeps the original items and label, and `setValue(5)` on a 4-item popup is rejected out of range (TR-M0-006). Pool slots therefore cannot carry per-definition menus.
- Writing a hidden arbitrary parameter's value through the plugin's ParamDef path is ineffective, while the sequence carrier round-trips a 16 MiB checksummed payload (TR-M0-004). A parameter-level definition payload is not a usable transport.

AE matches effect parameter streams by declaration order across project loads (the reason ADR-0004 discards prototype indexes), so every kind/capacity decision below is a persistent contract from the first release build onward.

## Decision

1. **ParamId grammar.** A `ParamId` is an ASCII string matching `[A-Za-z_][A-Za-z0-9_]*`, 1-64 bytes, case-sensitive — deliberately the shared identifier subset of GLSL/WGSL, so "uniform member name is the initial ID" (architecture §11.1) is always well-formed. Explicit annotation IDs use the same grammar. Reserved and rejected as user IDs: the exact builtin head names (`u_resolution`, `u_time`, `u_frame`; ADR-0011) and any ID beginning with `dfx_` (runtime-reserved namespace).
2. **Aliases.** A declaration may carry an alias list; each alias uses the same grammar. Slot inheritance during `BindingPlan` construction: exact current-ID match first; on miss, an alias matching the previous binding's ID inherits that slot and its keyframed stream. Alias resolution is single-generation, never a recursive chain. All current IDs and aliases in one definition share one namespace; any duplicate is an atomic rejection. Aliases are param-schema data: they enter `DefinitionHash` and never `ModuleHash` or `PipelineKey` (ADR-0007).
3. **v1 pool table.** A single configuration source defines the pools; `PARAMS_SETUP` declares exactly this, in this order, after the fixed head parameters:

   | Kind | AE type | Capacity | Carries |
   |---|---|---:|---|
   | Float | Float Slider | 48 | `float` (default) |
   | Integer | Integer Slider | 8 | `int` |
   | Bool | Checkbox | 16 | `bool` |
   | Color | Color (RGB) | 12 | `vec3` (default); RGB part of `vec4` |
   | Point2D | Point | 12 | `vec2` |
   | Angle | Angle | 8 | `float` with `angle` hint |

   104 pool slots total. A `vec4` binds atomically as one Color slot plus one Float slot (alpha); if either pool lacks a free slot, the whole definition is rejected. Capacities are v1 contract values estimated from ecosystem evidence (typical Shadertoy/ISF-style effects use ≤16 user parameters; the studied competitor's actual pool capacity is not statically determinable) with 3-4× headroom. Underestimation is repairable by the growth policy; overestimation only costs hidden slots. Per §4.2 the numbers must be re-checked against real effect samples before first release.
4. **Kinds deliberately absent from v1.**
   - **Popup**: excluded — TR-M0-006 proves menus and labels are immutable after `PARAMS_SETUP`, so per-definition menus are impossible, and a generic fixed menu ("Option 1..N", unrenamable) is worse UI than an Integer slider. Enum-annotated parameters map to Integer slots; enum labels remain definition metadata without a native menu UI. The Language popup (ADR-0010) is unaffected — its menu only appends across builds.
   - **Point3D**: kind reserved, not declared — §4.2 requires host evidence before enabling it.
   - **Layer**: not declared — Shader ABI v1 is single-input; extra inputs are the M4 multi-input extension and its entry ADR's concern.
   Adding any new kind requires a new ADR and follows the growth policy below.
5. **Append-only growth policy.** Because streams match by declaration order, for every released build:
   - a declared index's AE type and pool kind never change; indexes are never deleted, reordered, or reused;
   - all growth — enlarging a pool or introducing a new kind — appends at the tail of the parameter list; a pool's slot set may therefore be non-contiguous, and `BindingPlan` uses explicit slot tables, never contiguity assumptions;
   - shrinking is forbidden; hiding unused slots is the only removal mechanism;
   - capacity changes ship as a new build and are recorded in the configuration source; cross-build compatibility (an old project in a newer build takes defaults for appended tail slots; a newer project in an older build) is a host-behavior assumption that must be measured before any released capacity change — see verification.
6. **DefinitionData is dropped.** The target topology contains no hidden arbitrary-data parameter. The persisted definition snapshot's only carrier is sequence data (schema v1 at M3 entry, inside ADR-0012's 8 MiB budget). This resolves architecture §13's open question with TR-M0-004's evidence: the parameter-value path is ineffective, and a second carrier would create a second payload authority. The fixed head topology becomes: 0 Input, 1 Language, 2 Source, 3 Compile, 4 Status, 5 StateToken, then the pools.
7. **Atomic validation and diagnostics.** Grammar violations, reserved-ID use, alias conflicts, unsupported kinds, and pool overflow each reject the whole definition atomically (stable diagnostic; previously published definition and streams untouched, per §11.1). Four stable diagnostic classes must exist and be testable: param-grammar/reserved-id, alias-conflict, unsupported-kind, pool-overflow. Code values belong to the M3 diagnostic registry.
8. **Value-encoding semantics are not this ADR.** Numeric semantics of each mapping (Point coordinate normalization, Angle units, Color working-space handling, int rounding) are fixed by M2 fixtures under ADR-0011 §6's fixture-pinning discipline. This ADR fixes identity, topology, and capacity only.

## Alternatives considered

- Dynamic per-definition AE parameter creation: rejected by ADR-0005 (index drift destroys keyframes); TR-M0-006 adds that even menu content is immutable, so "dynamic" parameter UI is not host-supported anyway.
- Numeric ParamIds behind a registry (like ADR-0010's LanguageIds): rejected; IDs originate in user source as uniform names, which are naturally strings; numeric indirection burdens shader authors for no stability gain.
- Keeping DefinitionData as a redundant carrier: rejected; the measured write path is ineffective and a second authority is exactly what §13 warns against.
- A Popup pool with a fixed generic menu: rejected for v1; labels are also immutable (TR-M0-006), and unrenamable "Option N" entries are strictly worse than an Integer slider.
- Folding `int` into the Float pool: rejected; integer UI (stepping, display) is worth one 8-slot pool, and a float↔int change correctly reads as a different parameter.
- Rejecting `vec4` or pinning alpha to 1.0: rejected; ABI v1's type set includes `vec4` (ADR-0011), and the paired-slot mapping preserves it at the cost of bookkeeping.

## Consequences

### Benefits

- Rename-with-alias keeps keyframes under an exactly testable rule instead of folklore.
- One configuration source feeds `PARAMS_SETUP`, validation, and documentation; capacity drift between them becomes impossible.
- Dropping DefinitionData removes the dual-authority risk and simplifies the M3 snapshot contract.
- Every failure mode is an atomic, diagnosable rejection; nothing silently truncates or partially binds.

### Costs and risks

- Enum parameters get no native menu UI in v1; labels exist only as metadata.
- A float↔int type change allocates a new slot and orphans old keyframes (visible and intended, but potentially surprising).
- Capacities rest on ecosystem-typical usage, not a local corpus; a low guess waits for the next build's append.
- The append-only policy depends on AE tail-append stream matching being safe across 2023-2026 — measured nowhere yet. If a host year breaks it, this policy needs a superseding ADR (e.g. full pre-allocation from day one).
- `vec4`'s paired slots make `BindingPlan` bookkeeping slightly more complex and consume two pools at once.

## Revisit conditions

Host evidence that tail-appended parameters corrupt or misalign existing projects on any target year (breaks the growth policy); a real effect corpus showing systematic capacity shortfall (forces new numbers via append or supersession); or a host mechanism making per-definition menus genuinely mutable (would justify a Popup-kind ADR).

## Verification obligations

- Rust unit tests: grammar acceptance/rejection including reserved names and the `dfx_` prefix, alias-conflict detection, single-generation alias resolution, atomic pool-overflow rejection, vec4 paired allocation, non-contiguous slot tables.
- TR-M2-001: the stable ParamId + BindingPlan unit/integration suite covers slot reuse, alias inheritance, and kind-change reallocation.
- M2 host runs: rename-with-alias preserves keyframed values on at least one Windows AE year; pool overflow rejects with the previous definition still rendering.
- M2/M3 (new matrix rows when reached): cross-build compatibility of a tail-appended build — old project in new build, new project in old build — on at least one target year before any released capacity change.
- A test or governance check asserts the pool configuration source is unique (PARAMS_SETUP, validation, and documentation all derive from it).
