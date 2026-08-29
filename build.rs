use pipl::*;

const PF_PLUG_IN_VERSION: u16 = 13;
const PF_PLUG_IN_SUBVERS: u16 = 28;

#[rustfmt::skip]
fn main() {
    // after-effects-rs exposes these as destination-crate cfgs from its
    // generated entry point. Declare all of them for rustc's cfg checking and
    // keep a release panic boundary around EffectMain: several upstream host
    // handle Drop implementations still unwrap disposal failures.
    for name in [
        "does_dialog",
        "with_premiere",
        "threaded_rendering",
        "catch_panics",
    ] {
        println!("cargo:rustc-check-cfg=cfg({name})");
    }
    println!("cargo:rustc-cfg=catch_panics");

    // `plugin_build` emits the PIPL_* env vars the after-effects entry point
    // reads at GLOBAL_SETUP, and also writes the PiPL resource. Its resource
    // path is byte-unsafe (see `repair_pipl_resource`), so the resource is
    // rebuilt afterwards; the env vars from this call are correct and stay.
    let editor = std::env::var_os("CARGO_FEATURE_EDITOR").is_some();
    pipl::plugin_build(pipl_properties(editor));
    repair_pipl_resource(editor);
}

/// pipl 0.1.1 serializes the PiPL as an RC **string literal** of `\xNN`
/// escapes under `#pragma code_page(65001)`. Every byte >= 0x80 is then
/// code-page converted on the way into the binary and lands as `?` (0x3F).
///
/// This went unnoticed for the project's whole life because every out-flags
/// byte was <= 0x7F. `OutFlags::CustomUI` is bit 15, which makes byte 1 of the
/// little-endian global out-flags word 0x84 — the first byte to cross the
/// line. Measured on AE 2025 (2026-08-15): the built resource carried
/// `0x6003F44` while the code returned `0x6008444`, and AE refused to load the
/// effect with "global out-flags mismatch".
///
/// The repair writes the same bytes to a file and points the resource at it.
/// An RC file reference is copied verbatim, so no code page can touch it.
/// Recompiling overwrites pipl's `resource.rc` in OUT_DIR, so exactly one PiPL
/// reaches the linker.
fn repair_pipl_resource(editor: bool) {
    if !cfg!(target_os = "windows") {
        return;
    }
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is always set for build scripts");
    let bytes = pipl::build_pipl(pipl_properties(editor)).expect("PiPL serialization");
    let bin = std::path::Path::new(&out_dir).join("pipl.bin");
    std::fs::write(&bin, &bytes).expect("write pipl.bin");

    let mut res = winres::WindowsResource::new();
    // Relative to the generated .rc, which winres also writes into OUT_DIR.
    res.append_rc_content("16000 PiPL DISCARDABLE \"pipl.bin\"");
    res.compile().expect("compile the byte-exact PiPL resource");
    println!("cargo:rerun-if-changed=build.rs");
}

#[rustfmt::skip]
fn pipl_properties(editor: bool) -> Vec<Property> {
    let mut out_flags =
        OutFlags::PixIndependent |
        OutFlags::UseOutputExtent |
        OutFlags::DeepColorAware |
        OutFlags::SendUpdateParamsUI |
        // ADR-0039: the output world may exceed the layer frame (the
        // canvas — declared expansion or upstream extent). Without this
        // flag AE clips the output world to the layer regardless of
        // max_result_rect. The repair path below re-emits the PiPL
        // bytes, so the added bit is byte-safe like every other flag.
        OutFlags::IExpandBuffer |
        // The shader output depends on u_time (and the shader source
        // itself) even when no AE parameter changes — without this flag
        // AE caches the frame forever and the preview never updates.
        OutFlags::NonParamVary;
    if editor {
        // ADR-0042: the gradient-editor canvases carry PF_PUI_CONTROL, and AE
        // validates at PARAMS_SETUP — "no custom ui outflag, but param has
        // ui_width or ui_height or PF_PUI_TOPIC/CONTROL flags" refuses the
        // whole effect — so this bit rides the same `editor` feature as every
        // CONTROL declaration and the register_ui call; they can only appear
        // together. Bit 15 sets the high bit of flags byte 1 (0x86 with
        // today's flags): code-page-unsafe, covered by `repair_pipl_resource`.
        out_flags |= OutFlags::CustomUI;
    }

    vec![
        Property::Kind(PIPLType::AEEffect),
        Property::Name("DynamicFx"),
        Property::Category("DynamicFx"),

        #[cfg(target_os = "windows")]
        Property::CodeWin64X86("EffectMain"),
        #[cfg(target_os = "macos")]
        Property::CodeMacIntel64("EffectMain"),
        #[cfg(target_os = "macos")]
        Property::CodeMacARM64("EffectMain"),

        Property::AE_PiPL_Version { major: 2, minor: 0 },
        Property::AE_Effect_Spec_Version { major: PF_PLUG_IN_VERSION, minor: PF_PLUG_IN_SUBVERS },
        Property::AE_Effect_Version {
            // Subversion bumps with out-flag changes (M5 SmartFX entry,
            // M6 threaded rendering) so AE's plugin cache re-reads the PIPL.
            version: 1,
            subversion: if editor { 6 } else { 5 },
            bugversion: 0,
            stage: Stage::Develop,
            build: 0,
        },
        Property::AE_Effect_Info_Flags(0),
        Property::AE_Effect_Global_OutFlags(out_flags),
        Property::AE_Effect_Global_OutFlags_2(
            // Deliberately NOT SupportsThreadedRendering yet: AE still uses a
            // separate render thread/project, but serializes this effect's
            // render calls. Source identity crosses that boundary through a
            // hidden primitive stream; AEGP calls remain main-thread-only.
            //
            // SmartFX entry (M5): float worlds only reach smart effects —
            // FLOAT_COLOR_AWARE requires SUPPORTS_SMART_RENDER (ADR-0021).
            //
            // MFR (M6, ADR-0023 §4): thread-safe by construction (per-instance
            // mutex, mutex/OnceLock globals, locked log writer, thread-local
            // ROI hand-off; temporal state lives outside sequence data).
            OutFlags2::SupportsGetFlattenedSequenceData |
            OutFlags2::SupportsSmartRender |
            OutFlags2::FloatColorAware |
            OutFlags2::SupportsThreadedRendering
        ),
        Property::AE_Effect_Match_Name("DynamicFx"),
        Property::AE_Reserved_Info(0),
        Property::AE_Effect_Support_URL("https://github.com/dynamicfx/dynamicfx-ae"),
    ]
}
