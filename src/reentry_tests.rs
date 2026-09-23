use super::*;

const SOURCE: &str = "#version 450\n\
layout(location=0) out vec4 outColor;\n\
layout(set=0,binding=2) uniform FxUniforms {\n\
vec2 u_resolution;float u_time;float u_frame;float recovery_gain;};\n\
void main(){outColor=vec4(recovery_gain,0,0,1);}";

fn restored() -> LocalMutex {
    let (code, status, compiled) = evaluate_committed_source(LanguageId::GLSL, SOURCE, None);
    assert_eq!(code, Diag::Ok, "{status}");
    let (token, compiled) = compiled.unwrap();
    let snapshot = persistence::Snapshot::from_state(
        LanguageId::GLSL, token, SOURCE, &compiled.definition.binding,
    );
    LocalMutex::new(Local { snapshot: Some(snapshot), ..Local::default() })
}

fn observe(instance: &LocalMutex) -> Result<Option<bool>, Error> {
    observe_with(instance, false, || {
        Ok((Some(LanguageId::GLSL), Observation::Committed(SOURCE.into())))
    })
}

#[test]
fn source_read_can_reenter_flatten_without_losing_the_restored_snapshot() {
    let instance = restored();
    let before = instance.flatten().unwrap();
    let result = observe_with(&instance, false, || {
        assert!(instance.try_lock().is_ok(), "host source read holds the instance lock");
        assert_eq!(instance.flatten().unwrap(), before);
        for _ in 0..2 {
            assert_eq!(observe_with(&instance, false, || panic!("nested source read"))?, None);
        }
        Ok((Some(LanguageId::GLSL), Observation::Committed(SOURCE.into())))
    }).unwrap();
    assert_eq!(result, Some(true));
    assert_eq!(instance.flatten().unwrap(), before);
    assert_eq!(observe(&instance).unwrap(), Some(false));
    let local = instance.lock().unwrap();
    assert_eq!(local.status_code, Diag::Ok);
    assert!(local.compiled.as_ref().unwrap().definition.binding.bindings[0].inherited);
}

#[test]
fn ui_publication_can_reenter_flatten_and_defers_recursive_host_work() {
    let instance = restored();
    observe(&instance).unwrap();
    let before = instance.flatten().unwrap();
    with_ui_publication(&instance, |ui| {
        assert!(instance.try_lock().is_ok(), "host UI write holds the instance lock");
        assert_eq!(instance.flatten().unwrap(), before);
        for _ in 0..2 {
            assert_eq!(observe(&instance)?, None);
            with_ui_publication(&instance, |_| panic!("recursive UI publication"))?;
        }
        ui.configured_token = Some(ui.token);
        ui.visibility_token = Some(ui.token);
        ui.status = "published".into();
        Ok(())
    }).unwrap();
    assert_eq!(instance.flatten().unwrap(), before);
    let local = instance.lock().unwrap();
    assert_eq!(local.configured_token, Some(local.token));
    assert_eq!(local.visibility_token, Some(local.token));
    assert_eq!(local.status, "published");
}

#[test]
fn source_read_error_or_unwind_preserves_state_and_releases_the_scope() {
    let instance = restored();
    let before = instance.flatten().unwrap();
    assert!(observe_with(&instance, false, || Err(Error::InvalidCallback)).is_err());
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = observe_with(&instance, false, || panic!("host read failed"));
    }));
    assert!(panic.is_err());
    assert_eq!(instance.flatten().unwrap(), before);
    assert_eq!(observe(&instance).unwrap(), Some(true));
}

#[test]
fn nested_observation_of_a_different_instance_is_allowed() {
    let first = restored();
    let second = restored();
    let result = observe_with(&first, false, || {
        assert_eq!(observe(&second)?, Some(true));
        Ok((Some(LanguageId::GLSL), Observation::Committed(SOURCE.into())))
    }).unwrap();
    assert_eq!(result, Some(true));
    assert_eq!(first.flatten().unwrap(), second.flatten().unwrap());
}

#[test]
fn unknown_snapshot_schema_still_requires_explicit_compile() {
    let instance = LocalMutex::new(Local { block_rebind: true, ..Local::default() });
    assert_eq!(observe_with(&instance, false, || panic!("blocked source read")).unwrap(), Some(false));
    assert!(instance.lock().unwrap().block_rebind);
    assert_eq!(observe_with(&instance, true, || {
        Ok((Some(LanguageId::GLSL), Observation::Committed(SOURCE.into())))
    }).unwrap(), Some(true));
    assert!(!instance.lock().unwrap().block_rebind);
}

#[test]
fn stale_ui_completion_cannot_mark_a_new_definition_as_configured() {
    let instance = restored();
    observe(&instance).unwrap();
    with_ui_publication(&instance, |ui| {
        ui.configured_token = Some(ui.token);
        ui.visibility_token = Some(ui.token);
        let mut local = instance.lock().unwrap();
        local.clear_definition();
        local.status = "new state".into();
        local.status_code = Diag::NoExpression;
        Ok(())
    }).unwrap();
    let local = instance.lock().unwrap();
    assert_eq!(local.status, "new state");
    assert_eq!(local.configured_token, None);
    assert_eq!(local.visibility_token, None);
    assert_eq!(local.status_code, Diag::NoExpression);
}
