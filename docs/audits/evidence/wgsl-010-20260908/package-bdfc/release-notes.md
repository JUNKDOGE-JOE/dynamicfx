DynamicFX 0.1.0 adds native WGSL authoring and an Apple Silicon macOS build for After Effects.

- **WGSL:** original WGSL goes through Naga into the existing GPU render graph. The Language control selects GLSL or WGSL; GLSL remains the default. Parameters, resource bindings, multipass graphs and temporal rendering use the same host workflow. This is the documented DynamicFX fragment ABI, not arbitrary WebGPU application code.
- **Apple Silicon:** native arm64 plug-in, Metal rendering and corrected macOS registration and parameter UI. Actual acceptance used After Effects 2026 **26.3x87**, Apple M5, macOS **26.5.2**.
- **Shader quality:** linear sampling, correct logical/physical preview coordinates and packed pixel decoding before filtering, plus an authoring guide for smooth SDF edges, filtered noise and subtle glow.
- **Mac authoring:** CR/CRLF source pasted through AE preserves annotations and multipass envelopes; shorter parameter names no longer retain a previous suffix.

Validation: 220 Rust tests in each default/editor configuration; production GLSL/WGSL GPU comparisons; 558 recorded AE/export assertions; 24 independently rendered PSDs covering Full/Half/Quarter. Actual GUI Language Undo/Redo was checked both with existing keyframes and with first-publication color/angle defaults.

### Downloads and installation

The attached `DynamicFX-0.1.0-macos-arm64.zip` is the exact ad-hoc signed bundle used in the final native tests. It includes installation instructions, shader examples, the AI authoring Skill, source identity, the exact Cargo lockfile and third-party license notices. Compare its SHA-256 against `SHA256SUMS.txt` before installing. Close AE first and follow `INSTALL.txt`.

This build is **ad-hoc signed, not Developer ID signed or notarized**. It supports native Apple Silicon; it is not an Intel/Rosetta package. Other AE versions were not accepted with this artifact.

### Known limits

- **Source-expression single Undo:** background state publication can occupy Undo history after a Source edit. A single Undo may leave the source unchanged and make Redo unavailable. Keep previous shader text in a file and restore it explicitly. This known failure is disclosed for 0.1.0 under [ADR-0045](https://github.com/JUNKDOGE-JOE/dynamicfx/blob/v0.1.0/docs/adr/0045-010-source-undo-release-boundary.md); its architecture repair is separate follow-up work. It is not included in the verified Language-switch Undo/Redo result.
- For the first script-driven gradient color assignment, open that effect's Effect Controls and expand its gradient once so AE initializes the dynamic child rows.
- The optional gradient editor remains disabled. WGSL supports the documented fragment ABI and resource set; general WebGPU compute/storage/stages are outside this release.
- **Windows 0.1.0 binaries are pending.** They will be built from this tag, checked in Windows AE and appended to this release separately. No Windows 0.1.0 host result is claimed here. [Backfill procedure](https://github.com/JUNKDOGE-JOE/dynamicfx/blob/v0.1.0/docs/windows-010-backfill.md).

[WGSL guide](https://github.com/JUNKDOGE-JOE/dynamicfx/blob/v0.1.0/skills/dynamicfx-shaders/wgsl.md) · [Shader-quality guide](https://github.com/JUNKDOGE-JOE/dynamicfx/blob/v0.1.0/docs/shader-quality.md) · [Native acceptance evidence](https://github.com/JUNKDOGE-JOE/dynamicfx/tree/v0.1.0/docs/audits/evidence/wgsl-010-20260908/host-bdfc)
