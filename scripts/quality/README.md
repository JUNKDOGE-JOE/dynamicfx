# Headless Shader quality evidence

This runner uses the production `frontend`, `binding`, `definition`, `plan` and
`render.rs` modules by path. It does not link the After Effects plugin or use an
alternative renderer. It cannot prove AE installation, publication, project
persistence, color management or final U15 conversion. The 16-bpc measurements
inspect the production `Rgba32Float` working buffer before AE output conversion.

Dependencies: installed stable Rust, the repository's naga/wgpu major versions,
Python 3 with NumPy and Pillow. The Python script never installs dependencies.
In Codex Desktop the bundled workspace runtime supplies NumPy and Pillow.

From the repository root:

```sh
cargo +stable build --offline --manifest-path scripts/quality/Cargo.toml --target-dir scripts/out/quality/target
python3 scripts/quality/run.py --binary scripts/out/quality/target/debug/dynamicfx-quality --out scripts/out/quality/current
```

`--offline` deliberately keeps this evidence run from upgrading dependencies.
Remove it only when provisioning a machine without a populated Cargo cache.
The repository pins a Windows toolchain; `+stable` selects the installed native
toolchain explicitly. `scripts/out/` is ignored. On macOS a sandbox may hide
Metal adapters; run the local executable with GPU access if it reports no
adapter, and preserve that failure instead of calling the device unsupported.

To compare the old nearest sampler with the fixed production path, first
preserve the current executable, build a second executable against the exact
historical `render.rs`, and pass it as `--baseline`:

```sh
cp scripts/out/quality/target/debug/dynamicfx-quality scripts/out/quality/current-renderer
git show d4477ab:src/render.rs > scripts/out/quality/render-before.rs
DFX_QUALITY_RENDER_SOURCE="$PWD/scripts/out/quality/render-before.rs" cargo +stable build --offline --manifest-path scripts/quality/Cargo.toml --target-dir scripts/out/quality/target
cp scripts/out/quality/target/debug/dynamicfx-quality scripts/out/quality/baseline-renderer
python3 scripts/quality/run.py --binary scripts/out/quality/current-renderer --baseline scripts/out/quality/baseline-renderer --out scripts/out/quality/comparison
```

The only source substitution is the historical renderer; shader frontend,
fixtures, dimensions and input data remain identical. The baseline uses the
old diagnostic `DYNAMICFX_BACKEND=all` override because that renderer predates
the macOS backend default. Every render records the adapter that actually ran.
The historical snapshot is a local measurement input, never an installed plugin.

The runner fails if a shader does not compile, GPU execution fails, output is
non-finite, or a quantitative gate fails. It records RGBA little-endian float
frames, PNG previews, exact commands/stdout/stderr/exit, source and compiled
renderer identities, binary identity, adapter, baseline HEAD, and `summary.json`.
Each new measurement should use a new `--out` directory to preserve failures.

The fixtures measure:

- 2×2 bilinear center and quarter-texel displacement at 8/16/32-bpc;
- 256×1 gradient LUT magnification to 1024 pixels;
- circle coverage at Full/Half/Quarter versus an 8×8 supersampled hard mask;
- fBm filtering versus an 8×8 supersampled unfiltered field;
- mean adjacent-pixel jump on raw-hash versus quintic-interpolated cell borders;
- small intermediate values amplified in the next pass at 8/16/32-bpc;
- decode-before-filter for the shipped thermal example's packed temperature;
- the complete Siri-inspired example at three times and three preview scales,
  plus all three working depths, and a compile/render smoke of thermal.

The thresholds gate these specific fixtures, not an arbitrary shader's visual
quality. Full-image error is reported with a 99th-percentile error for the rim
preview comparison; enlarged crops should also be inspected.

The sampler oracles cover constant-center sampling, a fractional offset at
native scale, and LUT magnification. They do not independently test texture
minification. The procedural Half/Quarter renders test output-footprint
handling; they are not a minification-filter test.
