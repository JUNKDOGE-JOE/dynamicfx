use super::*;

fn shader(language: LanguageId, ty: &str, annotation: &str) -> String {
    if language == LanguageId::GLSL {
        format!("#version 450\n// @param value {annotation}\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=2) uniform FxUniforms {{vec2 u_resolution;float u_time;float u_frame;{ty} value;}};\nvoid main(){{outColor=vec4(1.0);}}")
    } else {
        format!("// @param value {annotation}\nstruct FxUniforms {{u_resolution:vec2f,u_time:f32,u_frame:f32,value:{ty},}};\n@group(0) @binding(2) var<uniform> fx:FxUniforms;\n@fragment fn main()->@location(0) vec4f {{return vec4f(1.0);}}")
    }
}

fn compile(language: LanguageId, source: &str, previous: Option<&binding::BindingPlan>) -> (u64, Arc<CompiledEffect>) {
    let (code, status, result) = evaluate_committed_source(language, source, previous);
    assert_eq!(code, Diag::Ok, "{status}");
    result.unwrap()
}

#[test]
fn rgb_hex_defaults_follow_declared_width_and_rgba_never_loses_alpha() {
    for (language, rgb, rgba) in [(LanguageId::GLSL,"vec3","vec4"),(LanguageId::WGSL,"vec3f","vec4f")] {
        for (ty, literal, expected) in [
            (rgb,"#4080FF",vec![64.0/255.0,128.0/255.0,1.0]),
            (rgba,"#4080FF",vec![64.0/255.0,128.0/255.0,1.0,1.0]),
            (rgba,"#4080FF80",vec![64.0/255.0,128.0/255.0,1.0,128.0/255.0]),
        ] {
            let (_, effect) = compile(language,&shader(language,ty,&format!("hint:color default:{literal}")),None);
            assert_eq!(effect.definition.params[0].ui.default.as_ref(),Some(&expected));
            let configs=slot_configs(&effect.definition);
            if ty==rgba {
                let alpha=effect.definition.binding.bindings[0].slots[1];
                assert_eq!(configs[&alpha].default,Some(expected[3]));
            }
        }
        for literal in ["#4080FFFF","#ééé","#GGGGGG"] {
            assert_eq!(evaluate_committed_source(language,&shader(language,rgb,&format!("hint:color default:{literal}")),None).0,Diag::ParamRejected);
        }
    }
}

#[test]
fn percent_metadata_preserves_raw_defaults_binding_and_snapshot() {
    for (language, scalar) in [(LanguageId::GLSL,"float"),(LanguageId::WGSL,"f32")] {
        let source=shader(language,scalar,"hint:percent min:0 max:100 default:37.125");
        let (token,effect)=compile(language,&source,None);
        let slot=effect.definition.binding.bindings[0].slots[0];
        let configs=slot_configs(&effect.definition);
        assert!(configs[&slot].percent);
        assert_eq!(configs[&slot].default,Some(37.125));
        assert_eq!(configs[&slot].max,Some(100.0));
        let (_,plain)=compile(language,&source.replace("hint:percent ",""),Some(&effect.definition.binding));
        assert_eq!(plain.definition.binding.bindings[0].slots[0],slot);
        assert!(plain.definition.binding.bindings[0].inherited);
        assert!(!slot_configs(&plain.definition)[&slot].percent);
        let local=LocalMutex::new(Local {compiled:Some(Arc::clone(&effect)),token,..Local::default()});
        let (version,bytes)=<LocalMutex as AdobePluginInstance>::flatten(&local).unwrap();
        let restored=<LocalMutex as AdobePluginInstance>::unflatten(version,&bytes).unwrap();
        let mut restored=restored.lock().unwrap();
        resolve_from_snapshot(&mut restored);
        let rebuilt=restored.compiled.as_ref().unwrap();
        assert!(slot_configs(&rebuilt.definition)[&slot].percent);
        assert!(plan_mappings_equal(&effect.definition.binding,&rebuilt.definition.binding));
    }
}

#[test]
fn percent_rejects_nonfloat_members() {
    for (language,types) in [(LanguageId::GLSL,["int","vec3","vec4"]),(LanguageId::WGSL,["i32","vec3f","vec4f"])] {
        for ty in types {
            assert_eq!(evaluate_committed_source(language,&shader(language,ty,"hint:percent"),None).0,Diag::ParamRejected);
        }
    }
}
