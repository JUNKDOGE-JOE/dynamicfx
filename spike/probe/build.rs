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
        "leg_u1",
        "leg_u2",
        "leg_u2b",
        "leg_u3",
        "leg_u4",
        "leg_u146",
        "cui_legs",
        "cui_nil_seq",
    ] {
        println!("cargo:rustc-check-cfg=cfg({name})");
    }
    println!("cargo:rustc-cfg=catch_panics");
    println!("cargo:rerun-if-env-changed=DFXP_LEGS");

    let legs = std::env::var("DFXP_LEGS").unwrap_or_default();
    let mut leg_u1 = false;
    let mut leg_u2 = false;
    let mut leg_u2b = false;
    let mut leg_u3 = false;
    let mut leg_u4 = false;
    let mut leg_u146 = false;
    let mut nil_seq = false;
    for leg in legs.split(',').map(str::trim).filter(|leg| !leg.is_empty()) {
        match leg {
            "u1" => leg_u1 = true,
            "u2" => leg_u2 = true,
            "u2b" => leg_u2b = true,
            "u3" => leg_u3 = true,
            "u4" => leg_u4 = true,
            "u146" => leg_u146 = true,
            // u1 canvas with the sequence type swapped to () - the
            // discriminant for the Event-path sequence-handle crash.
            "u1nil" => {
                leg_u1 = true;
                nil_seq = true;
            }
            _ => panic!("unsupported DFXP_LEGS value: {leg}"),
        }
    }
    if nil_seq {
        println!("cargo:rustc-cfg=cui_nil_seq");
    }
    for (enabled, cfg_name) in [
        (leg_u1, "leg_u1"),
        (leg_u2, "leg_u2"),
        (leg_u2b, "leg_u2b"),
        (leg_u3, "leg_u3"),
        (leg_u4, "leg_u4"),
        (leg_u146, "leg_u146"),
    ] {
        if enabled {
            println!("cargo:rustc-cfg={cfg_name}");
        }
    }

    let custom_ui = leg_u1 || leg_u2 || leg_u2b || leg_u3 || leg_u4 || leg_u146;
    if custom_ui {
        // Any-leg marker: CUI builds hide the M0-era undrawn arb param, whose
        // bare ECW row raises a modal on AE 2025 and wedges the JSX bridge.
        println!("cargo:rustc-cfg=cui_legs");
    }
    pipl::plugin_build(pipl_properties(custom_ui));
    repair_pipl_resource(custom_ui);
}

/// pipl 0.1.1 embeds PiPL bytes in an RC string literal, so Windows code-page
/// conversion can replace bytes at or above 0x80. A file reference preserves
/// the high byte introduced by `OutFlags::CustomUI` exactly.
fn repair_pipl_resource(custom_ui: bool) {
    if !cfg!(target_os = "windows") {
        return;
    }
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is always set for build scripts");
    let bytes = pipl::build_pipl(pipl_properties(custom_ui)).expect("PiPL serialization");
    let bin = std::path::Path::new(&out_dir).join("pipl.bin");
    std::fs::write(&bin, &bytes).expect("write pipl.bin");

    let mut res = winres::WindowsResource::new();
    res.append_rc_content("16000 PiPL DISCARDABLE \"pipl.bin\"");
    res.compile().expect("compile the byte-exact PiPL resource");
    println!("cargo:rerun-if-changed=build.rs");
}

#[rustfmt::skip]
fn pipl_properties(custom_ui: bool) -> Vec<Property> {
    let mut out_flags =
        OutFlags::PixIndependent |
        OutFlags::UseOutputExtent |
        OutFlags::DeepColorAware |
        OutFlags::SendUpdateParamsUI;
    if custom_ui {
        out_flags |= OutFlags::CustomUI;
    }

    vec![
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
        Property::AE_Effect_Global_OutFlags(out_flags),
        Property::AE_Effect_Global_OutFlags_2(
            OutFlags2::SupportsGetFlattenedSequenceData
        ),
        Property::AE_Effect_Match_Name("DynamicFxProbe"),
        Property::AE_Reserved_Info(0),
        Property::AE_Effect_Support_URL("https://github.com/dynamicfx/dynamicfx-ae"),
    ]
}
