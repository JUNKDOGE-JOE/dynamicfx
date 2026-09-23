use after_effects::sys;

#[repr(C)]
#[derive(Default, Debug)]
#[cfg(stage_sdk)]
struct StageReport {
    step: i32, layer: i32, before: i32, limit: i32, after: i32, applied: i32,
}

#[cfg(stage_sdk)]
extern "C" {
    fn host_stage_probe(basic: *mut sys::SPBasicSuite, id: i32,
        effect: sys::AEGP_EffectRefH, index: i32, action: i32, report: *mut StageReport) -> i32;
    fn host_stage_bind(basic: *mut sys::SPBasicSuite, id: i32,
        effect: sys::AEGP_EffectRefH, index: i32, layer: i32, stage: i32, apply: i32, changed: *mut i32) -> i32;
}

pub unsafe fn bind(basic: *mut sys::SPBasicSuite, id: i32, effect: sys::AEGP_EffectRefH,
    index: i32, layer: i32, stage: i32, apply: bool) -> Result<bool, String> {
    #[cfg(stage_sdk)]
    {
        let mut changed = 0;
        let error = host_stage_bind(basic, id, effect, index, layer, stage, i32::from(apply), &mut changed);
        if error != 0 { return Err(format!("stage bind index={index} layer={layer} stage={stage}: AE error {error}")); }
        Ok(changed != 0)
    }
    #[cfg(not(stage_sdk))]
    {
        let _ = (basic, id, effect, index, layer, stage, apply);
        Err("stage binding requires DYNAMICFX_AESDK_265_ROOT".into())
    }
}

pub unsafe fn inspect(basic: *mut sys::SPBasicSuite, id: i32,
    effect: sys::AEGP_EffectRefH, action: i32) -> Result<String, String>
{
    #[cfg(stage_sdk)]
    {
        let _quiet = after_effects::aegp::suites::Utility::new()
            .and_then(|suite| suite.start_quiet_errors(false)).map_err(|e| format!("{e:?}"))?;
        let mut result = String::new();
        for index in [6, 9] {
            let mut report = StageReport::default();
            let error = host_stage_probe(basic, id, effect, index, action, &mut report);
            result.push_str(&format!("STAGE param={index} action={action} error={error} {report:?}\n"));
        }
        Ok(result)
    }
    #[cfg(not(stage_sdk))]
    {
        let _ = (basic, id, effect, action);
        Err("StreamSuite7 probe requires a build with DYNAMICFX_AESDK_265_ROOT".into())
    }
}

#[cfg(all(test, not(stage_sdk)))]
#[test]
fn missing_sdk_never_enters_host_callbacks() {
    let result = unsafe { inspect(std::ptr::null_mut(), 0, std::ptr::null_mut(), 0) };
    assert!(result.unwrap_err().contains("DYNAMICFX_AESDK_265_ROOT"));
}
