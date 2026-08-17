use pipl::*;

const PF_PLUG_IN_VERSION: u16 = 13;
const PF_PLUG_IN_SUBVERS: u16 = 28;

#[rustfmt::skip]
fn main() {
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
        // Display name and match name deliberately differ from the real
        // effect so the probe can never be confused with DynamicFx.
        Property::Name("DynamicFx Probe"),
        Property::Category("DynamicFX"),

        #[cfg(target_os = "windows")]
        Property::CodeWin64X86("EffectMain"),
        #[cfg(target_os = "macos")]
        Property::CodeMacIntel64("EffectMain"),
        #[cfg(target_os = "macos")]
        Property::CodeMacARM64("EffectMain"),

        Property::AE_PiPL_Version { major: 2, minor: 0 },
        Property::AE_Effect_Spec_Version { major: PF_PLUG_IN_VERSION, minor: PF_PLUG_IN_SUBVERS },
        Property::AE_Effect_Version {
            version: 0,
            subversion: 1,
            bugversion: 0,
            stage: Stage::Develop,
            build: 0,
        },
        Property::AE_Effect_Info_Flags(0),
        Property::AE_Effect_Global_OutFlags(
            OutFlags::PixIndependent |
            OutFlags::UseOutputExtent |
            OutFlags::DeepColorAware |
            OutFlags::SendUpdateParamsUI
        ),
        Property::AE_Effect_Global_OutFlags_2(
            OutFlags2::SupportsGetFlattenedSequenceData
        ),
        Property::AE_Effect_Match_Name("DynamicFxProbe"),
        Property::AE_Reserved_Info(0),
        Property::AE_Effect_Support_URL("https://github.com/dynamicfx/dynamicfx-ae"),
    ])
}
