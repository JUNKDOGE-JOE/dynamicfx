# Production WGSL 0.1.0 evidence

This is evidence for the production frontend, separate from the older
feasibility spike. The first installed candidate failed real GUI Undo. Its
repair is built; installation/host verification of the replacement and
publication remain pending. The latest replacement is `bdfc9f1d`, after the
intermediate `661e` exposed additional native authoring defects.

- [host-661e-authoring-failure](host-661e-authoring-failure/README.md): Language
  Undo/Redo passed; bare CR annotations and short-name termination failed.
- [build-bdfc](build-bdfc/README.md): repair at `8b0fe81`, default/editor
  220 tests each; installation/host acceptance still pending at curation.

- [host-91c8-undo-failure](host-91c8-undo-failure/README.md): retired candidate,
  217 numerical assertions passed but one real Language Undo failed.
- [build-661e](build-661e/README.md): repair at `f4ba578`, default/editor
  213 tests each; new executable identity, not yet a host PASS.

- `build/`: default/editor 205 tests each, integration tests, locked build,
  source identity and unsigned-input/signed-bundle identities.
- `gpu-production/`: 18 GLSL/WGSL pairs through the production frontend and
  Metal renderer; all byte-equal, finite, source unchanged during run.
- `gpu-first-instrument-failure/`: retained failed comparison caused by
  missing GLSL annotations in the instrument. The algorithms received
  different parameter defaults. The corrected fixture supplies the same
  metadata to both frontends; no runtime fix was needed for this failure.
- `examples/`: 40 authored-example renders and 96 checks, with actual
  Full/Half/Quarter images and enlarged crops. Working formats 16 and 32
  share float GPU storage; this does not claim AE boundary equivalence.
- `independent-review/`: source audit and actual GPU tests for 24-byte,
  96-byte padded and 64KiB-tail uniform layouts; all maximum errors zero.
- `package/`: candidate ZIP checks, unchanged signature verification after
  extraction and the dependency-notice manifest. These establish packaging
  preparation only, for the retired 91c8 candidate. No host or release PASS
  is implied.

Every original `.f32` dump is stored losslessly as `.f32.zlib`. Recover using
Python `zlib.decompress(path.read_bytes())`, then read tight little-endian
RGBA float32 at the dimensions in the corresponding log/summary. Original
and stored SHA-256 identities are in `manifest.json`. JSON metadata retains
local run paths for provenance; public relative files are mapped by that
manifest. No binary is rebuilt or signed by evidence curation.

The original manifest covers the original build/GPU/example/package groups.
The appended `host-91c8-undo-failure` and `build-661e` directories have their
own manifests and checksums. Host curation explicitly records path redaction
and both the original and distributed hashes.
