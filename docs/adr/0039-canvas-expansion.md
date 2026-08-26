# ADR-0039: Canvas expansion beyond the layer frame

- Status: Accepted
- Date: 2026-08-26 (Proposed and Accepted the same day with explicit user approval, after two review rounds; round 2 replaced the built-in control with the source-declared authority)
- Owners: DynamicFX project
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md)
- Related implementation: `src/lib.rs` (`Command::SmartPreRender` / `Command::SmartRender`), `src/host/params.rs`, `build.rs` (PiPL), executor extents
- Related tests/audits: [TR-BOUNDS-001](../TEST_MATRIX.md#tr-bounds-001--shader-canvas-is-the-layer-frame-field-observation), evidence [`field-20260821-layer-bounds/`](../audits/evidence/field-20260821-layer-bounds/README.md), public issue [#8](https://github.com/JUNKDOGE-JOE/dynamicfx/issues/8)

## Context

The shader canvas is the layer's own frame by contract ([ADR-0011](0011-shader-abi-v1-core.md) §5; [ADR-0029](0029-logical-resolution-abi.md) changes only the units). A shader that paints beyond its source — glow, halo, shadow, displacement overshoot — is clipped at the layer bounds, and an upstream buffer-expanding effect does not help, because `SmartPreRender` overwrites the checkout rect with the source layer's `width/height` and discards the `PF_CheckoutResult` extent (measured: Grow Bounds is pixel-identical to the plain instance, tiles A ≡ B of TR-BOUNDS-001). The PiPL sets no `PF_OutFlag_I_EXPAND_BUFFER`, so the output world never exceeds the layer frame either.

The user-side workaround is a padded precomp (tile C of the evidence; documented in the shader skill). It works, renders the full halo, and its margin samples as transparent black — that behaviour is what users accept as correct. The cost is manual: one precomp per layer, sized by hand to the shader's reach.

A request-derived canvas was rejected on sight when the defect was recorded: AE's ROI request varies per render (a few pixels for `sampleImage`, TR-M7-004), and a canvas that moves with the request breaks WYSIWYG and every `uv`-space shader.

Decision direction (pad-precomp-equivalent semantics) approved by the user 2026-08-26; the same day's round 2 replaced the round-1 built-in `Canvas Expansion` control with the source-declared model — when the shader declares an expansion parameter that declaration is the boundary; when it declares none, the canvas is the original boundary plus the upstream expansion (Grow Bounds style).

## Decision

1. **No new AE-side parameter. The canvas authority is the shader's own declaration when present, else the upstream extent** (user decision 2026-08-26, round 2 — replacing the round-1 draft's built-in `Canvas Expansion` head control):
   - **Undeclared source:** the canvas is `U` — the input's upstream extent, read from the `max_result_rect` in the `PF_CheckoutResult` that `checkout_layer` already returns and the released code discards. `U` equals the layer frame when nothing upstream expands (today's contract, byte-for-byte) and is the stable, request-independent signal an upstream Grow Bounds emits — so a buffer-expanding upstream effect enlarges the canvas out of the box, with zero shader changes.
   - **Declared source:** one `@param` of Float kind annotated as the canvas-expansion parameter (grammar token, e.g. `hint:canvas`, fixed at implementation as ADR-0013/ADR-0018 annotation growth) provides the expansion `m` in logical pixels per side ([ADR-0029](0029-logical-resolution-abi.md) units, clamped at 0): the canvas is **the layer frame grown by that parameter's value at the frame's time**, and the declaration **replaces** the upstream signal — the author's boundary is law, upstream content beyond it is cropped. The parameter is an ordinary keyframeable pool slot the shader may also read as a uniform: the natural pattern is the shader's own reach/radius control doubling as the canvas authority (exactly the "expose the reach as pixel `@param`s" advice the workaround already gives). More than one canvas declaration in a source is a compile diagnostic with a stable code.

2. **Canvas semantics: expansion is equivalent to a centered padded precomp.**
   - `u_resolution` is the canvas size; `v_uv` ∈ [0,1]² spans the canvas ([ADR-0011](0011-shader-abi-v1-core.md) §5's "layer extent" becomes "canvas extent").
   - `input` is delivered as a canvas-sized texture with the checked-out upstream pixels (all of `U`, not just the layer frame — this is what makes Grow Bounds pixels reachable) placed at their extent's position and cropped to the canvas when a declaration bounds it tighter than `U`, margin transparent black — `texture(input, v_uv)` therefore reads exactly what the padded-precomp workaround reads today. Margin sampling is transparent, not clamp-to-edge, matching the accepted tile-C behaviour.
   - With no canvas declaration and no upstream expansion, every value above reduces to the released contract byte-for-byte; existing projects and shaders are untouched. (An existing project that already has an upstream Grow Bounds under a DynamicFx instance changes appearance once — from clipped to expanded — which is the behaviour the user asked Grow Bounds for; the release notes state it.)
   - The equivalence is the acceptance contract: a declared expansion `m` must render comparably to the same shader on a precomp padded by `m`, and an undeclared instance under a 256 px upstream expander must match the 256 px padded precomp (the TR-BOUNDS-001 A/B/C scene is the fixture).

3. **All graph resources adopt the canvas extent.** Intermediate pass targets, temporal history frames ([ADR-0025](0025-windowed-resimulation.md)) and the ADR-0030 §4 comp-space placement of `hint:layer` inputs are computed against the canvas; the canvas rect has its own origin in layer space (`U`'s origin, or `(-m, -m)` under a declaration), and comp-space alignment adds that origin offset. History frames carry the canvas extent in their identity: changing expansion re-simulates the window from reset by construction, never resamples old frames.

4. **SmartFX protocol.** The PiPL adds `PF_OutFlag_I_EXPAND_BUFFER`. `SmartPreRender` checks out the input over `U` (no longer overwriting the checkout rect with `in_data.width/height`), reads `U` from the returned `PF_CheckoutResult`, resolves the instance's own published plan to learn whether a canvas parameter is declared and reads its value at the frame's time when it is, and declares `max_result_rect` = the canvas per §1, `result_rect` = its intersection with the request. An instance with no published artifact yet has no declaration to read and takes the undeclared branch — consistent with its fail-closed pass-through anyway. `SmartRender` places the canvas-space result into the output world using the canvas origin, fills any outside-canvas remainder transparent, and keeps the ADR-0029 downsample factors applied to `m` and `U` like any other logical length. The documented `EXPAND_BUFFER` edge — a null/empty input buffer for an empty input — must keep the TR-CACHE-001 discipline: cancel propagates, only a true empty checkout renders as transparent. External `hint:layer` inputs are likewise checked out over their own upstream extents and placed by the ADR-0030 §4 comp-space mapping onto the canvas.

5. **Identity and caches ([ADR-0007](0007-identity-and-cache-boundaries.md)/[ADR-0017](0017-hash-domains.md)).** Expansion changes no source, so `DefinitionHash` and `PipelineKey` are untouched. The canvas extent joins the execution-plan/frame-resource identities exactly as ROI extents already do since M7. No new hash domain is created.

6. **Canvas authority is exactly one source at a time:** the shader's declared parameter when present, else the upstream extent `U`. The per-render ROI request never shapes the canvas (WYSIWYG; rejected on sight in the TR-BOUNDS-001 record). The canvas changes only through deliberate acts — editing the source, keyframing/adjusting the declared parameter, or editing the upstream stack. Device texture limits cap either source with a stable diagnostic and pass-through, never a crash.

## Alternatives considered

- **Request-derived canvas** — rejected before this ADR (TR-BOUNDS-001 record): the ROI request is per-render and would move the canvas under the shader.
- **Layer-frame-anchored `v_uv` with margin outside [0,1]** — preserves in-frame pixels exactly under expansion changes, but the margin then samples clamp-to-edge (not the accepted transparent behaviour), `fragCoord`-style ports see negative coordinates, and `input` sampling needs a second coordinate space. Rejected by the user 2026-08-26 in favour of pad-precomp equivalence.
- **A built-in AE-side `Canvas Expansion` head control (the round-1 draft)** — superseded by the user's round-2 decision: authority belongs to the source (coherent with [ADR-0001](0001-expression-authority-and-open-runtime.md)), it adds no persistent topology, and only a declared `@param` can double as the shader's own reach uniform. A head control would also exist, meaninglessly, on shaders that never paint outside their frame.
- **Always-union (declared ∪ upstream extent)** — rejected: the author's declared boundary is authoritative; a union would let an upstream effect silently inflate a tuned canvas.
- **Per-side margins** — deferred; a revisit condition, not needed to close issue #8.

## Consequences

### Benefits

- Glow/halo/shadow/displacement shaders render whole without manual precomps; the workaround's mental model becomes the feature's contract, and the shader's own reach control can be the canvas authority in one declaration.
- An upstream Grow Bounds finally does what users already tried to make it do (TR-BOUNDS-001 tile B was pixel-identical to the clipped tile A; it becomes equivalent to the padded tile C) — with zero shader changes.
- **No new persistent AE topology at all**: the canvas parameter is an ordinary declared pool slot; undeclared-and-unexpanded instances keep the released contract byte-for-byte.
- One parameter, one semantics: no per-render canvas drift, WYSIWYG preserved.

### Costs and risks

- The `@param` annotation grammar grows by one token plus a multiple-declaration diagnostic (ADR-0013/ADR-0018 growth, fixed at implementation).
- Canvas-sized intermediates and history multiply VRAM and fill cost by `area(canvas)/area(layer)`; large expansions on large frames are the author's explicit trade, and an aggressive upstream expander inflates it invisibly on undeclared shaders (capped only by the device-limit diagnostic).
- A keyframed canvas parameter animates the canvas: each size change re-simulates the temporal window and re-keys extent-dependent caches — legal and the author's choice, but a new way to spend memory and time that the docs must state plainly.
- `SmartPreRender` now resolves the plan and reads one parameter value; PreRender and SmartRender must agree on both (the same class of split that produced the M3-era "more checkout requests than expected" abort) — a new consistency surface needing fixtures.
- A released project that already carries an upstream Grow Bounds under DynamicFx changes appearance once at upgrade (clipped → expanded); release notes must call it out.
- The `input` upload path gains a placement step (checked-out extent into canvas position); the write-back gains origin math — both are new correctness surfaces that need pixel-exact fixtures.
- `EXPAND_BUFFER` changes AE's buffer allocation for every instance, including undeclared/unexpanded ones; the release gate must re-run the full battery to prove the flag alone regresses nothing.

## Revisit conditions

- Field demand for per-side margins or for combining the declared boundary with the upstream extent (always-union) — either supersedes §1's replacement rule.
- Device texture-limit failures at legal expansions (would force a tighter clamp or tiled rendering).

## Verification obligations

- Unit: canvas/origin placement math golden-tested for both authority branches; the undeclared-and-unexpanded reduction pinned; the canvas-annotation grammar (accept, reject-duplicate, kind check) pinned; PreRender/SmartRender canvas agreement pinned.
- Host (AE 2025 + 2026, released-artifact procedure), on the TR-BOUNDS-001 scene: undeclared + plain input ≡ released output; **undeclared + Grow Bounds 256 ≡ the padded tile C** (the former no-op becomes the positive test); declared 256 ≡ tile C; declared 64 under Grow Bounds 256 stays bound to 64 (author priority, crop visible); a keyframed declared parameter spot-checked at two frames; an uncompiled instance falls back undeclared.
- Regression: M1–M7 batteries green with the `EXPAND_BUFFER` flag present on undeclared/unexpanded content; ROI rows (TR-M7-004) and temporal rows (M6) re-run on an expanded canvas; 8/16/32-bpc spot checks on an expanded canvas.
- Evidence recorded under a new TR row with artifact hash, host builds, and PNGs per the evidence policy.
