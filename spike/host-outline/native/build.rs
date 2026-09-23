use pipl::*;

fn properties() -> Vec<Property> {
    vec![
        Property::Kind(PIPLType::AEEffect),
        Property::Name("DynamicFx Host Shape Probe"),
        Property::Category("DynamicFx Diagnostics"),
        Property::CodeWin64X86("EffectMain"),
        Property::AE_PiPL_Version { major: 2, minor: 0 },
        Property::AE_Effect_Spec_Version { major: 13, minor: 28 },
        Property::AE_Effect_Version {
            version: 0, subversion: 0, bugversion: 29, stage: Stage::Develop, build: 0,
        },
        Property::AE_Effect_Info_Flags(0),
        Property::AE_Effect_Global_OutFlags(OutFlags::DeepColorAware | OutFlags::SendUpdateParamsUI | OutFlags::WideTimeInput | OutFlags::IExpandBuffer),
        Property::AE_Effect_Global_OutFlags_2(OutFlags2::SupportsSmartRender | OutFlags2::FloatColorAware),
        Property::AE_Effect_Match_Name("DynamicFx Host Shape Probe"),
        Property::AE_Effect_Support_URL("https://github.com/JUNKDOGE-JOE/dynamicfx/issues/9"),
        Property::AE_Reserved_Info(0),
    ]
}

fn main() {
    assert_eq!(std::env::var("CARGO_CFG_TARGET_OS").unwrap(), "windows");
    println!("cargo:rustc-check-cfg=cfg(stage_sdk)");
    println!("cargo:rerun-if-env-changed=DYNAMICFX_AESDK_265_ROOT");
    println!("cargo:rerun-if-changed=src/stage.cpp");
    if let Some(root) = std::env::var_os("DYNAMICFX_AESDK_265_ROOT") {
        let headers = std::path::PathBuf::from(root).join("Examples/Headers");
        cc::Build::new().cpp(true).define("_WINDOWS", None).file("src/stage.cpp")
            .include(&headers).include(headers.join("SP"))
            .compile("host_stage_probe");
        println!("cargo:rustc-cfg=stage_sdk");
    }
    for name in ["does_dialog", "with_premiere", "threaded_rendering", "catch_panics"] {
        println!("cargo:rustc-check-cfg=cfg({name})");
    }
    println!("cargo:rustc-cfg=catch_panics");
    pipl::plugin_build(properties());
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    // A binary resource avoids Windows code-page conversion of PiPL flag bytes.
    std::fs::write(output.join("pipl.bin"), pipl::build_pipl(properties()).unwrap()).unwrap();
    let mut resource = winres::WindowsResource::new();
    resource.append_rc_content("16000 PiPL DISCARDABLE \"pipl.bin\"");
    resource.compile().unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
