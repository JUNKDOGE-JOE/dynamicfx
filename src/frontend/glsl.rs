//! The GLSL frontend: GLSL 450 core through naga `glsl-in` (ADR-0011 §3),
//! validating the Shader ABI v1 per-pass fragment interface and reflecting
//! `FxUniforms` members after the builtin head into ADR-0013 parameter
//! declarations.

use super::annotation::Annotation;
use super::shared::{reflect_user_params, truncate};
use super::{FrontendError, LanguageFrontend, LanguageId, PassModule};
#[cfg(test)]
use crate::definition::param::ShaderParamType;

pub struct GlslFrontend;

impl LanguageFrontend for GlslFrontend {
    fn language(&self) -> LanguageId {
        LanguageId::GLSL
    }

    fn parse_module(
        &self,
        source: &str,
        annotations: &std::collections::HashMap<String, Annotation>,
        allowed_inputs: usize,
    ) -> Result<PassModule, FrontendError> {
        let mut frontend = naga::front::glsl::Frontend::default();
        let options = naga::front::glsl::Options::from(naga::ShaderStage::Fragment);
        let module = frontend.parse(&options, source).map_err(|errors| {
            let msg = errors
                .errors
                .iter()
                .map(|e| format!("{e:?}"))
                .collect::<Vec<_>>()
                .join(" | ");
            FrontendError::Parse(truncate(&msg, 300))
        })?;

        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .map_err(|e| FrontendError::Parse(truncate(&format!("{e:?}"), 300)))?;

        check_entry_point(&module)?;
        let extra_input_bindings = check_bindings(&module, allowed_inputs)?;
        let (params, layout) = reflect_user_params(&module, annotations)?;
        Ok(PassModule { module, params, layout, extra_input_bindings })
    }
}

/// Fragment entry `main` with a color result (ADR-0011 §3: exactly one color
/// target in v1). naga glsl-in produces exactly one entry point.
fn check_entry_point(module: &naga::Module) -> Result<(), FrontendError> {
    let entry = module
        .entry_points
        .first()
        .ok_or_else(|| FrontendError::Abi("no entry point".into()))?;
    if entry.name != "main" {
        return Err(FrontendError::Abi(format!(
            "entry point must be `main`, found `{}`",
            entry.name
        )));
    }
    if entry.function.result.is_none() {
        return Err(FrontendError::Abi(
            "missing fragment output (layout(location = 0) out vec4 outColor)".into(),
        ));
    }
    Ok(())
}

/// Binding-space rules (ADR-0011 §3 + ADR-0018 §5): set 0 only; bindings
/// 0/1/2 are the fixed interface; binding 2+i (3, 4, 5) carries manifest
/// input i for multi-input passes — declaring one the manifest does not
/// feed is the `E18` violation; everything else stays reserved.
fn check_bindings(
    module: &naga::Module,
    allowed_inputs: usize,
) -> Result<Vec<u32>, FrontendError> {
    let max_extra_binding = 2 + allowed_inputs.saturating_sub(1) as u32;
    let mut extra = Vec::new();
    for (_, var) in module.global_variables.iter() {
        let Some(binding) = &var.binding else { continue };
        if binding.group >= 1 {
            return Err(FrontendError::Abi(format!(
                "descriptor set {} is reserved (v1 uses set 0 only)",
                binding.group
            )));
        }
        match binding.binding {
            0 | 1 | 2 => {}
            b @ 3..=15 => {
                if b > max_extra_binding {
                    return Err(FrontendError::Abi(format!(
                        "binding {b} has no manifest input feeding it (this pass declares {allowed_inputs} input(s))"
                    )));
                }
                if !matches!(module.types[var.ty].inner, naga::TypeInner::Image { .. }) {
                    return Err(FrontendError::Abi(format!(
                        "binding {b} must be a texture (it carries a manifest input)"
                    )));
                }
                extra.push(b);
            }
            other => {
                return Err(FrontendError::Abi(format!(
                    "binding {other} is outside the v1 interface"
                )));
            }
        }
    }
    extra.sort_unstable();
    extra.dedup();
    Ok(extra)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Result<PassModule, FrontendError> {
        parse_with_inputs(source, 1)
    }

    fn parse_with_inputs(source: &str, inputs: usize) -> Result<PassModule, FrontendError> {
        let annotations = crate::frontend::annotation::parse_annotations(source)
            .map_err(|e| FrontendError::Param(format!("@param line {}: {}", e.line, e.message)))?;
        GlslFrontend.parse_module(source, &annotations, inputs)
    }

    /// The exact "invert" pass the M4 harness chains after a generator:
    /// samples its primary input through the separate texture/sampler
    /// interface. Pinned here so a naga acceptance change breaks in CI, not
    /// on the host.
    #[test]
    fn sampling_pass_fixture_compiles() {
        let src = r#"
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_input;
layout(set = 0, binding = 1) uniform sampler u_sampler;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
};
void main() {
    vec4 c = texture(sampler2D(u_input, u_sampler), v_uv);
    outColor = vec4(1.0 - c.rgb, 1.0) + vec4(0.0) * (u_time + u_frame + u_resolution.x);
}
"#;
        let pm = parse(src).expect("sampling fixture must compile");
        assert!(pm.extra_input_bindings.is_empty());
        crate::render::compile_spirv(&pm.module).expect("sampling fixture SPIR-V");
    }

    /// Multi-input rules (ADR-0018 §5): binding 3 with two manifest inputs
    /// is fine; with one it is the E18 shape; non-texture extras rejected.
    #[test]
    fn extra_input_bindings_follow_the_manifest_budget() {
        let src = r#"
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D u_input;
layout(set = 0, binding = 1) uniform sampler u_sampler;
layout(set = 0, binding = 3) uniform texture2D u_second;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
};
void main() {
    outColor = texture(sampler2D(u_input, u_sampler), v_uv)
        + texture(sampler2D(u_second, u_sampler), v_uv)
        + vec4(u_time + u_frame) + vec4(u_resolution, 0.0, 1.0);
}
"#;
        let pm = parse_with_inputs(src, 2).expect("two inputs allow binding 3");
        assert_eq!(pm.extra_input_bindings, vec![3]);

        let err = parse_with_inputs(src, 1).expect_err("one input forbids binding 3");
        assert!(matches!(err, FrontendError::Abi(ref m) if m.contains("no manifest input")), "{err:?}");
    }

    const VALID: &str = r#"
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float speed;
    int steps;
    vec3 tint;
    vec4 overlay;
};
void main() {
    outColor = vec4(tint, 1.0) * overlay * speed * float(steps) * u_time
        * u_frame / vec4(u_resolution, 1.0, 1.0) + vec4(v_uv, 0.0, 0.0);
}
"#;

    #[test]
    fn valid_module_reflects_declarations_in_order() {
        let pm = parse(VALID).expect("valid ABI module");
        let ids: Vec<&str> = pm.params.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, vec!["speed", "steps", "tint", "overlay"]);
        let types: Vec<ShaderParamType> = pm.params.iter().map(|p| p.ty).collect();
        assert_eq!(
            types,
            vec![
                ShaderParamType::Float,
                ShaderParamType::Int,
                ShaderParamType::Vec3Color,
                ShaderParamType::Vec4Color,
            ]
        );
        // std140: head 16B, speed@16, steps@20, tint@32 (vec3 aligns 16),
        // overlay@48 (vec4 aligns 16), span 64.
        let offsets: Vec<usize> = pm.layout.entries.iter().map(|e| e.offset).collect();
        assert_eq!(offsets, vec![16, 20, 32, 48]);
        assert_eq!(pm.layout.block_size, 64);
        assert!(pm.layout.entries[1].int);
        assert!(!pm.layout.entries[0].int);
    }

    #[test]
    fn bad_glsl_is_a_parse_error() {
        assert!(matches!(parse("not glsl"), Err(FrontendError::Parse(_))));
    }

    #[test]
    fn annotations_merge_into_declarations() {
        let src = r#"
#version 450
layout(location = 0) out vec4 outColor;
// @param sweep hint:angle default:90 label:"Sweep Angle"
// @param level min:0 max:2 default:0.5 alias:gain
// @param ghost min:0 max:1
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float sweep;
    float level;
};
void main() { outColor = vec4(sweep + level + u_time + u_frame) + vec4(u_resolution, 0.0, 1.0); }
"#;
        let pm = parse(src).expect("annotated module");
        // hint:angle retypes the float and re-routes its pool slot.
        assert_eq!(pm.params[0].ty, ShaderParamType::AngleFloat);
        assert_eq!(pm.params[0].ui.label.as_deref(), Some("Sweep Angle"));
        assert_eq!(pm.params[0].ui.default, Some(vec![90.0]));
        assert_eq!(pm.params[1].ui.min, Some(0.0));
        assert_eq!(pm.params[1].ui.max, Some(2.0));
        let aliases: Vec<&str> = pm.params[1].aliases.iter().map(|a| a.as_str()).collect();
        assert_eq!(aliases, vec!["gain"]);
        // `ghost` names no member: stale annotations are ignored.
        assert_eq!(pm.params.len(), 2);
    }

    #[test]
    fn canvas_hint_marks_a_scalar_float_without_retyping_it() {
        let src = r#"
#version 450
layout(location = 0) out vec4 outColor;
// @param reach min:0 max:512 default:64 hint:canvas
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float reach;
};
void main() { outColor = vec4(reach + u_time + u_frame + u_resolution.x); }
"#;
        let pm = parse(src).expect("canvas float");
        assert_eq!(pm.params[0].ty, ShaderParamType::Float);
        assert!(pm.params[0].canvas);

        let wrong_kind = src.replace("float reach;", "int reach;");
        assert!(matches!(
            parse(&wrong_kind),
            Err(FrontendError::CanvasWrongKind(name)) if name == "reach"
        ));
    }

    /// ADR-0034. The whole point of the hint is that it is the ONLY way to
    /// reach the kind: an un-annotated `vec3` must stay a colour, or every
    /// existing shader's colours silently retype (ADR-0026).
    #[test]
    fn point3d_needs_the_hint_and_a_vec3() {
        let src = r#"
#version 450
layout(location = 0) out vec4 outColor;
// @param light_dir hint:point3d
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    vec3 light_dir;
    vec3 tint;
};
void main() { outColor = vec4(light_dir + tint + u_resolution.xyy + u_time + u_frame, 1.0); }
"#;
        let pm = parse(src).expect("annotated module");
        assert_eq!(pm.params[0].ty, ShaderParamType::Point3D);
        assert_eq!(
            pm.params[1].ty,
            ShaderParamType::Vec3Color,
            "an un-annotated vec3 is still a colour"
        );
        // Both occupy three block words, so the hint changes the AE control
        // and the value encoding, never the layout.
        assert_eq!(pm.layout.entries[0].words, 3);
        assert_eq!(pm.layout.entries[1].words, 3);

        // The pool routing is what makes it a different control.
        assert_eq!(
            pm.params[0].ty.slot_requirements(),
            &[crate::binding::PoolKind::Point3D]
        );

        // Any other member type is a mismatch, not a silent reinterpretation.
        for (member, use_it) in [
            ("float", "thing"),
            ("int", "float(thing)"),
            ("vec2", "thing.x"),
            ("vec4", "thing.x"),
        ] {
            let bad = format!(
                r#"
#version 450
layout(location = 0) out vec4 outColor;
// @param thing hint:point3d
layout(set = 0, binding = 2) uniform FxUniforms {{
    vec2 u_resolution;
    float u_time;
    float u_frame;
    {member} thing;
}};
void main() {{ outColor = vec4(u_resolution, u_time + u_frame + {use_it}, 1.0); }}
"#,
            );
            match parse(&bad).expect_err("hint:point3d on a non-vec3 member") {
                FrontendError::Param(message) => assert!(
                    message.contains("hint:point3d applies to vec3 members only"),
                    "{member}: unexpected message {message}"
                ),
                other => panic!("{member}: expected a param error, got {other:?}"),
            }
        }
    }

    /// ADR-0034 §4: a Point 3D default would need the AEGP ThreeD stream-value
    /// plumbing Point 2D also lacks, so it is refused rather than half-applied.
    #[test]
    fn point3d_defaults_are_refused() {
        let src = r#"
#version 450
layout(location = 0) out vec4 outColor;
// @param light_dir hint:point3d default:0,0,1
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    vec3 light_dir;
};
void main() { outColor = vec4(light_dir + u_resolution.xyy + u_time + u_frame, 1.0); }
"#;
        match parse(src).expect_err("defaults are unsupported for this kind") {
            FrontendError::Param(message) => {
                assert!(message.contains("point defaults are not supported yet"), "{message}");
            }
            other => panic!("expected a param error, got {other:?}"),
        }
    }

    /// The exact multi-kind fixture shader the M2 harness uses (m2h): every
    /// v1 kind in one block. Pinning it here means a naga acceptance change
    /// breaks in CI, not on the host.
    #[test]
    fn multi_kind_fixture_shader_compiles() {
        let src = r#"
#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
// @param count label:"Count" min:0 max:10 default:3
// @param flag hint:bool default:1
// @param sweep hint:angle default:90
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    int count;
    int flag;
    vec3 tint;
    vec2 center;
    float sweep;
};
void main() {
    float x = v_uv.x;
    vec3 c;
    if (x < 0.2)      c = vec3(float(count) / 10.0);
    else if (x < 0.4) c = vec3(flag != 0 ? 1.0 : 0.0);
    else if (x < 0.6) c = tint;
    else if (x < 0.8) c = vec3(center, 0.0);
    else              c = vec3(sweep / 360.0);
    outColor = vec4(c, 1.0);
}
"#;
        let pm = parse(src).expect("multi-kind fixture must compile");
        let types: Vec<ShaderParamType> = pm.params.iter().map(|p| p.ty).collect();
        assert_eq!(
            types,
            vec![
                ShaderParamType::Int,
                ShaderParamType::Bool,
                ShaderParamType::Vec3Color,
                ShaderParamType::Vec2,
                ShaderParamType::AngleFloat,
            ]
        );
        assert_eq!(pm.params[0].ui.default, Some(vec![3.0]));
        assert_eq!(pm.params[1].ui.default, Some(vec![1.0]));
        assert_eq!(pm.params[4].ui.default, Some(vec![90.0]));
        // SPIR-V emission must also hold for the fixture.
        crate::render::compile_spirv(&pm.module).expect("fixture SPIR-V");
    }

    #[test]
    fn annotation_inconsistencies_fail_closed() {
        let head = r#"
#version 450
layout(location = 0) out vec4 outColor;
"#;
        let body = r#"
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    vec3 tint;
};
void main() { outColor = vec4(tint + vec3(u_time + u_frame), 1.0) + vec4(u_resolution, 0.0, 1.0); }
"#;
        // ADR-0026: color defaults accept 3 or 4 components; a 2-component
        // color default is still a rejection, and point defaults stay
        // unsupported.
        let color_default = format!("{head}// @param tint default:1,0.5,0.25\n{body}");
        assert!(parse(&color_default).is_ok(), "3-component color default accepted");
        let bad_color_default = format!("{head}// @param tint default:1,0.5\n{body}");
        let err = parse(&bad_color_default).expect_err("2-component color default rejected");
        assert!(matches!(err, FrontendError::Param(ref m) if m.contains("component")), "{err:?}");
        // angle hint on a vec3.
        let bad_hint = format!("{head}// @param tint hint:angle\n{body}");
        assert!(matches!(parse(&bad_hint), Err(FrontendError::Param(_))));
        // min > max.
        let bad_range = format!("{head}// @param tint min:2 max:1\n{body}");
        assert!(matches!(parse(&bad_range), Err(FrontendError::Param(_))));
        // Malformed annotation line anywhere is a rejection.
        let bad_line = format!("{head}// @param tint mim:0\n{body}");
        assert!(matches!(parse(&bad_line), Err(FrontendError::Param(_))));
    }

    #[test]
    fn missing_fx_uniforms_block_is_an_abi_error() {
        let src = r#"
#version 450
layout(location = 0) out vec4 outColor;
void main() { outColor = vec4(1.0); }
"#;
        assert!(matches!(parse(src), Err(FrontendError::Abi(_))));
    }

    #[test]
    fn head_missing_u_frame_is_an_abi_error() {
        let src = r#"
#version 450
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
};
void main() { outColor = vec4(u_time) + vec4(u_resolution, 0.0, 1.0); }
"#;
        assert!(matches!(parse(src), Err(FrontendError::Abi(_))));
    }

    #[test]
    fn head_out_of_order_is_an_abi_error() {
        let src = r#"
#version 450
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 2) uniform FxUniforms {
    float u_time;
    vec2 u_resolution;
    float u_frame;
};
void main() { outColor = vec4(u_time + u_frame) + vec4(u_resolution, 0.0, 1.0); }
"#;
        assert!(matches!(parse(src), Err(FrontendError::Abi(_))));
    }

    #[test]
    fn reserved_binding_is_an_abi_error() {
        let src = r#"
#version 450
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
};
layout(set = 0, binding = 3) uniform Extra { float x; };
void main() { outColor = vec4(x + u_time + u_frame) + vec4(u_resolution, 0.0, 1.0); }
"#;
        let err = parse(src).expect_err("binding 3 needs a manifest input");
        assert!(
            matches!(err, FrontendError::Abi(ref m) if m.contains("no manifest input")),
            "{err:?}"
        );
    }

    #[test]
    fn reserved_param_id_is_a_param_error() {
        let src = r#"
#version 450
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    float dfx_secret;
};
void main() { outColor = vec4(dfx_secret + u_time + u_frame) + vec4(u_resolution, 0.0, 1.0); }
"#;
        assert!(matches!(parse(src), Err(FrontendError::Param(_))));
    }

    #[test]
    fn matrix_member_is_outside_the_v1_type_set() {
        let src = r#"
#version 450
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 2) uniform FxUniforms {
    vec2 u_resolution;
    float u_time;
    float u_frame;
    mat4 transform;
};
void main() { outColor = transform * vec4(u_resolution, u_time, u_frame); }
"#;
        assert!(matches!(parse(src), Err(FrontendError::Param(_))));
    }
}
