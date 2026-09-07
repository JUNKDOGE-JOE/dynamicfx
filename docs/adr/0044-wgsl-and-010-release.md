# ADR-0044: Production WGSL frontend and 0.1.0 release scope

- Status: Accepted
- Date: 2026-09-08
- Owners: DynamicFX project
- Authorization: the user requested completing WGSL, publishing 0.1.0 with Windows artifacts allowed later, then implementing the researched iOS 27 Siri effect locally.
- Extends: [ADR-0010](0010-stable-language-ids.md), [ADR-0011](0011-shader-abi-v1-core.md), [ADR-0043](0043-apple-silicon-host-protocol.md)
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md)
- Related tests: TR-WGSL-002 and TR-REL-010 in [../TEST_MATRIX.md](../TEST_MATRIX.md)

## Context

The Metal feasibility spike rendered WGSL through the existing Naga IR,
SPIR-V and wgpu path. Its generated GLSL interface adapter was deliberately
not production code. The user now authorizes the real frontend and release;
the preceding scope limited to feasibility no longer applies.

Only this Mac and AE 2026 are available. No connected Windows Codex host was
found. Prior releases were pre-releases with Windows artifacts. This release
publishes a macOS ARM artifact first; Windows can be appended from the same
tag after its independent build and host verification.

## Decision

1. Activate the permanently reserved Language ID 2 as WGSL. Append popup
   position 2; keep GLSL default and position 1. The explicit Language
   control remains authoritative for new commits. Preserve snapshot restore,
   unsupported-ID behavior, parameter IDs and sequence schema v1.
2. Parse WGSL directly to Naga IR and reflect the original module. Share
   annotation and neutral user-parameter reflection with GLSL; never generate
   a substitute GLSL interface in the production compiler. Reuse the graph,
   SPIR-V compiler, parameter upload, texture resources and renderer.
3. A WGSL module has exactly one fragment entry named `main`, one
   location-0 `vec4<f32>` output (direct or single-field struct). If UV input
   is declared, it is location 0 `vec2<f32>`; fragment position may be
   `@builtin(position) vec4<f32>`. No other input locations or builtins,
   fragment depth/sample-mask outputs, additional stages or entry points.
   Pure generators may omit UV, primary texture and sampler, matching the
   released generator behavior. Declared resources must satisfy the ABI even
   when unused. UV interpolation is perspective/center to match the runtime
   vertex output; flat, centroid and sample interpolation are rejected.
4. Group 0 binding 2 is a uniform structure whose first members are
   `u_resolution: vec2<f32>`, `u_time: f32`, `u_frame: f32` at offsets 0, 8,
   12. User members are f32/i32/vec2f/vec3f/vec4f with the existing annotation
   semantics (bool uses i32). Use original WGSL reflected offsets and span,
   including legal explicit padding; never impose a guessed Rust layout.
   Structure and variable names are presentation, not identity. The WGSL
   uniform span is capped at the runtime's 65536-byte binding limit, including
   explicit padding, before any GPU allocation.
5. Declared binding 0 is non-array, non-multisampled `texture_2d<f32>`;
   binding 1 is an ordinary non-comparison sampler. Bindings 3 through 15
   are the existing extra sampled inputs, limited by the graph's supplied
   input count. Other groups, storage resources, push constants, binding
   arrays and pipeline overrides are rejected. Layer, gradient, path,
   canvas and deterministic temporal features keep their existing limits.
   Optional WGSL features such as f16 and subgroups have no device
   negotiation in this version and are rejected using baseline validation
   capabilities.
6. Keep envelope v1 byte/escaping rules. Raw WGSL uses ordinary attributes;
   within an envelope pass a leading literal `@` is written `@@`. Compiler
   diagnostics identify the pass and source location, including the
   envelope body's source starting line where appropriate.
7. Append `E21 WgslParse` in the frontend diagnostic family. Preserve E17
   GLSL parse and the shared E18 ABI/E19 parameter/E20 emission meanings.
   Syntax or validation errors preserve source and fail closed.
8. No persistent hash/schema migration. The existing token fingerprint
   already includes LanguageId. GPU artifacts and pipelines remain
   process-local and are rebuilt from source after restart; compiler-version
   isolation derives from that lifetime, not from a nonexistent persistent
   shader cache. Existing identity golden vectors remain unchanged.
9. Publish v0.1.0 as a regular GitHub Release after current-artifact macOS
   AE 2026 checks. Ship an ARM-only, ad-hoc-signed development bundle; no
   Developer ID identity is available, so it is **not notarized**. Explain
   this and the scoped quarantine-removal installation step in the package.
   Do not disable system-wide Gatekeeper. The release must visibly state
   Windows artifacts are pending and this diff has no Windows acceptance.
10. Freeze the exact tested executable, package it without rebuilding or
    re-signing, record source/tag and dependency identities, publish hashes,
    download the asset again and verify archive/bundle/signature identity.
    Finish publication before starting the new iOS 27 Siri shader work.

## Consequences

WGSL becomes a supported authoring language on the verified host subset;
existing GLSL projects retain their language and parameter stream meaning.
The menu does not promise compute/WebGPU browser APIs or arbitrary resources.
An older GLSL-only build cannot render ID 2 and must keep its source intact.
Unsigned-by-Developer-ID distribution may require the documented user
installation step. Neither version numbering nor successful Metal execution
proves Windows/DX12 or other AE-year compatibility.

## Verification obligations

- Registry, ABI rejection, annotation/layout, source mapping, full graph and
  persistent round-trip tests; default/editor GLSL regression suites.
- Production WGSL GPU outputs compared with equivalent GLSL at all depths,
  including real multipass texture reads and resource types.
- Installed AE: menu/defaults, WGSL publication, values/keyframes, invalid
  source, language changes, real Undo/Redo, save/reopen, resource inputs,
  temporal random-time requests, Full/Half/Quarter and independent aerender.
- Source publication scan, version/tag match, release ZIP structure,
  architecture, symbol/PiPL/signature, installed and downloaded hashes.
- Record failures and exact untested host subsets in the release audit.

## Next action

Implement the WGSL frontend and its host integration, then validate the final
0.1.0 artifact before publication.
