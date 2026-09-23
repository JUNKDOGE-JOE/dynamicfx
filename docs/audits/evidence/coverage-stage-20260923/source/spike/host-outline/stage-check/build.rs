fn main() {
    println!("cargo:rerun-if-env-changed=DYNAMICFX_AESDK_265_ROOT");
    println!("cargo:rerun-if-changed=check.cpp");
    println!("cargo:rerun-if-changed=../../../src/host/coverage_stage.cpp");
    let root = std::env::var_os("DYNAMICFX_AESDK_265_ROOT").expect("SDK 26.5 root required");
    let headers = std::path::PathBuf::from(root).join("Examples/Headers");
    let mut build = cc::Build::new();
    build.cpp(true).file("check.cpp").include(&headers).include(headers.join("SP"));
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        build.define("_WINDOWS", None);
    }
    build.compile("coverage_stage_check");
}
