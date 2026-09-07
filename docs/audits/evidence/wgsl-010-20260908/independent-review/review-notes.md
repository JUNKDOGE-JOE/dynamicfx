# Independent WGSL review

No production source edits. No new runtime defect found in the reviewed surface.

- Reviewed WGSL entry/resource/type rejection, Naga capability validation, native reflected offsets and the 65536-byte span cap, shared annotation semantics, source-envelope line mapping, original-source snapshot storage, and frontend error classification.
- Compared the extracted shared reflection against origin/main GLSL after rustfmt and comment removal: code is identical after removal of pub(super) visibility.
- Metal GPU tests in summary.json read a native 24-byte block, explicit alignment/size with a 96-byte block, and the final scalar at offset 65532 in a 65536-byte block. All pixels equal the expected f32 values, max error 0, including negative and HDR values.
- Source body lines and UTF-8 byte columns are diagnostic coordinates after @@ unescaping; the host reports the original body start separately and does not claim original envelope columns. Exact committed text remains the snapshot source.
- Minor test-helper limitation: src/wgsl_tests.rs::envelope prepends @ before indentation, so future fixtures with indented attribute lines would be malformed. Current fixtures have no indented attribute lines; production grammar preserves indentation correctly. This does not demonstrate a runtime bug.

Scope: read-only code review plus the three named headless Apple M5 / Metal layout cases, not exhaustive compiler fuzzing, Windows validation, or AE host acceptance.
