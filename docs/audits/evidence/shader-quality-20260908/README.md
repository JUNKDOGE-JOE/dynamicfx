# Shader quality GPU evidence — 2026-09-08

**PASS: 45 headless GPU renders, all numerical gates satisfied.** This record
does not claim an After Effects host result. The production renderer is
included directly by `scripts/quality`; no alternative image renderer is used.

- Date: 2026-09-07 20:43 UTC / 2026-09-08 04:43 Asia/Shanghai.
- Baseline HEAD: `d4477ab`; changed working-tree renderer and example source
  identities are recorded per render in [`summary.json`](summary.json).
- Platform: macOS **26.5.2 arm64**; GPU **Apple M5**, actual backend **Metal**.
- Rust: `rustc 1.97.1 (8bab26f4f 2026-07-14)`;
  Cargo: `1.97.1 (c980f4866 2026-06-30)`; naga/wgpu **29.0.4**.
- Current headless binary SHA-256:
  `edc319fdc3bc26f47a30beee1f3f77322ab7ba0e381dcad39453eddd6cd0bb40`.
- Historical-renderer binary SHA-256:
  `95d86eaea1f45d56130e424f4c06c2c25a4cc79d58fb429c3b32eb31c1c6839b`.
- Final Siri source SHA-256:
  `4fe06faa3f4f0f8127e80d77511df3c87e448cd0f86f5c09a28cf06806d62d9f`.
- AE host/version/plugin installation: **not applicable to this headless run**.
  There was no AE process interaction in this procedure.

## Procedure and artifacts

Build and baseline-reproduction commands are in
[`scripts/quality/README.md`](../../../../scripts/quality/README.md).
The final command used the bundled Python runtime (NumPy + Pillow) and was:

```sh
python3 scripts/quality/run.py --binary scripts/out/quality/dynamicfx-quality-current --baseline scripts/out/quality/dynamicfx-quality-before --out scripts/out/quality/run5
```

The first binary includes the working-tree `src/render.rs`; the second was
built with `DFX_QUALITY_RENDER_SOURCE` pointing at a `git show d4477ab:src/render.rs`
snapshot. Both use the same frontend and fixture code. The old renderer uses
its diagnostic `DYNAMICFX_BACKEND=all` override; every returned adapter identity
is Metal. Compiled-in renderer content hashes make this distinction inspectable.

- [`summary.json`](summary.json): all metrics, 45 render records, source hashes,
  compiled renderer hashes, dimensions, time, depth, adapter and binary identity.
- [`renders.log`](renders.log): per-render command, stdout/stderr, and exit 0.
  The repository's absolute local prefix is replaced by `${REPO}` for publication;
  no other output content is changed.
- [`siri-t2.png`](siri-t2.png): 1280×720 F32 example at t=2 s, black input.
- [`preview-corners.png`](preview-corners.png): nearest-expanded inspection crops
  covering the same 160×160 logical-pixel region at Full/Half/Quarter.
- [`comparison.png`](comparison.png): noise/cell-filtering contact sheet.
- [`failed-runs.txt`](failed-runs.txt): preserved failure descriptions, including
  sandbox adapter visibility, a bad LUT fixture declaration, and an overly
  aggressive first noise cutoff. Failing inputs are kept in the raw run folders.
- Raw RGBA f32 frames, original commands/logs, fixture GLSL and per-probe PNGs:
  `scripts/out/quality/run5/` (ignored local evidence). Earlier failures/results:
  `scripts/out/quality/current/`, `run2/`, `run3/`, `run4/`.

The full raw output is intentionally not committed: the canonical script and
compact numeric/image record provide reproducibility without a large binary
dump. Every reported metric derives from raw f32 arrays, not PNG screenshots.
The 16-bpc test uses the production f32 working representation; it does not
exercise AE's final U15 conversion.

## Outcome and limitations

The linear-sampler probes, circle coverage against an 8×8 supersampled mask,
noise against an 8×8 supersampled field, lattice-boundary continuity,
intermediate precision, packed-temperature decode-before-filter, and complete
example render all passed. Details and interpretation are in
[`docs/shader-quality.md`](../../../shader-quality.md).

The corrected `apple-thermal.glsl` also completed a 10-pass 256×256 F32 smoke
render with checker input and unassigned external gradient. This proves the
changed shader bodies and pipeline execute, not a new artistic acceptance of
that legacy example on logo footage. Historical AE screenshots remain separate.

Next action: use the same source and fixed plugin artifact for the separately
recorded AE macOS host acceptance; do not promote this headless PASS to AE PASS.
