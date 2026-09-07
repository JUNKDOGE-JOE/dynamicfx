use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=DFX_QUALITY_RENDER_SOURCE");
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../..");
    let source = env::var_os("DFX_QUALITY_RENDER_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("src/render.rs"))
        .canonicalize().unwrap();
    println!("cargo:rerun-if-changed={}", source.display());
    let generated = format!("#[path = {0:?}] mod render;\nconst RENDER_SOURCE: &str = include_str!({0:?});\n", source);
    fs::write(PathBuf::from(env::var("OUT_DIR").unwrap()).join("render_module.rs"), generated).unwrap();
}
