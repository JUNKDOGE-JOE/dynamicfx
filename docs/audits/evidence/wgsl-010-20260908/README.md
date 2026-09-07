# Production WGSL 0.1.0 evidence

This is evidence for the production frontend, separate from the older
feasibility spike. AE installation/host verification and publication are
still pending; this directory does not grant them.

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
  preparation only; the Mac is locked and installation awaits the user's
  system authorization. No host or release PASS is implied.

Every original `.f32` dump is stored losslessly as `.f32.zlib`. Recover using
Python `zlib.decompress(path.read_bytes())`, then read tight little-endian
RGBA float32 at the dimensions in the corresponding log/summary. Original
and stored SHA-256 identities are in `manifest.json`. JSON metadata retains
local run paths for provenance; public relative files are mapped by that
manifest. No binary is rebuilt or signed by evidence curation.
