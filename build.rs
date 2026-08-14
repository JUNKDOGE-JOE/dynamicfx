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

    pipl::plugin_build(vec![
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
            subversion: 4,
            bugversion: 0,
            stage: Stage::Develop,
            build: 0,
        },
        Property::AE_Effect_Info_Flags(0),
        Property::AE_Effect_Global_OutFlags(
            OutFlags::PixIndependent |
            OutFlags::UseOutputExtent |
            OutFlags::DeepColorAware |
            OutFlags::SendUpdateParamsUI |
            // The shader output depends on u_time (and the shader source
            // itself) even when no AE parameter changes — without this flag
            // AE caches the frame forever and the preview never updates.
            OutFlags::NonParamVary
        ),
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
    ])
}
