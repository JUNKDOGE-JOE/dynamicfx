# WGSL feasibility instrument

Non-shipping probe; the plugin remains GLSL-only. See [the assessment](../../docs/wgsl-feasibility.md).

From the repository root:

```sh
python3 spike/wgsl/run.py
```

Requires Python 3, Rust `+stable`, the pinned Cargo dependencies, and native GPU access. The runner uses `--offline --locked`; populate an empty cache first with `cargo +stable fetch --manifest-path spike/wgsl/Cargo.toml`. It neither launches After Effects nor installs a plugin. A process sandbox may hide Metal adapters; run from a GPU-enabled terminal or obtain narrowly scoped GPU access.

The crate imports production graph/binding/render modules directly. It parses WGSL to Naga IR, checks an ABI v1 subset, emits SPIR-V and MSL, and runs equivalent WGSL/GLSL field and two-input blur passes through the production renderer. Parameter reflection uses a temporary generated GLSL schema solely to reuse existing annotation rules; shader expressions stay WGSL IR. A shipping implementation should extract shared IR reflection instead.

Results: [test output](evidence/tests.log), [GPU output](evidence/gpu.log), [baseline and source/artifact hashes](evidence/run.json), [8-bit preview](evidence/U8-2pass.png). The `.rgba.zlib` files contain exact raw outputs compressed with zlib: 256×128 tightly packed RGBA, U8 bytes or little-endian f32 components for U15/F32 working targets. Generated uncompressed buffers and temporary builds live in ignored `out/` and `target/`.

The initial [HEX color-default failure](evidence/initial-hex-default-failure.log) and [sandbox GPU failure](evidence/gpu-initial.log) remain recorded. The assessment separates these from the final passing run and lists the AE/front-end shipping work that has not been performed.
