use pipl::*;

fn main() {
    for name in ["does_dialog", "with_premiere", "threaded_rendering", "catch_panics"] {
        println!("cargo:rustc-check-cfg=cfg({name})");
    }
    println!("cargo:rustc-cfg=catch_panics");
    pipl::plugin_build(properties());
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    std::fs::write(output.join("pipl.bin"), pipl::build_pipl(properties()).unwrap()).unwrap();
    let mut resource = winres::WindowsResource::new();
    resource.append_rc_content("16000 PiPL DISCARDABLE \"pipl.bin\"");
    resource.compile().unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}

fn properties() -> Vec<Property> {
    vec![
        Property::Kind(PIPLType::AEEffect),
        Property::Name("DynamicFx Coverage Reader"),
        Property::Category("DynamicFx Internal"),
        Property::CodeWin64X86("CoverageReaderMain"),
        Property::AE_PiPL_Version { major: 2, minor: 0 },
        Property::AE_Effect_Spec_Version { major: 13, minor: 28 },
        Property::AE_Effect_Version { version: 0, subversion: 1, bugversion: 0, stage: Stage::Develop, build: 3 },
        Property::AE_Effect_Info_Flags(0),
        Property::AE_Effect_Global_OutFlags(OutFlags::DeepColorAware | OutFlags::UseOutputExtent | OutFlags::IExpandBuffer),
        Property::AE_Effect_Global_OutFlags_2(OutFlags2::SupportsSmartRender |
            OutFlags2::FloatColorAware | OutFlags2::SupportsThreadedRendering),
        Property::AE_Effect_Match_Name("DynamicFx Coverage Reader"),
        Property::AE_Effect_Support_URL("https://github.com/JUNKDOGE-JOE/dynamicfx"),
        Property::AE_Reserved_Info(0),
    ]
}
