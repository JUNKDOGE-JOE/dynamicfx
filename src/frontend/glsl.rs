//! The GLSL frontend: GLSL 450 core through naga `glsl-in` (ADR-0011 §3),
//! validating the Shader ABI v1 per-pass fragment interface and reflecting
//! `FxUniforms` members after the builtin head into ADR-0013 parameter
//! declarations.

use super::annotation::{Annotation, Hint};
use super::{FrontendError, LanguageFrontend, LanguageId, PassModule, UniformBlockLayout, UniformEntry};
use crate::definition::param::{ParamDeclaration, ParamId, ParamUiMeta, ShaderParamType};

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

/// The fixed std140 builtin head (ADR-0011 §4): name, then a type predicate,
/// then the byte offset pinned by the ADR's verification obligations.
const ABI_HEAD: [(&str, HeadType, u32); 3] = [
    ("u_resolution", HeadType::Vec2F, 0),
    ("u_time", HeadType::F32, 8),
    ("u_frame", HeadType::F32, 12),
];

enum HeadType {
    Vec2F,
    F32,
}

impl HeadType {
    fn matches(&self, inner: &naga::TypeInner) -> bool {
        match self {
            Self::Vec2F => matches!(
                inner,
                naga::TypeInner::Vector {
                    size: naga::VectorSize::Bi,
                    scalar: naga::Scalar { kind: naga::ScalarKind::Float, width: 4 },
                }
            ),
            Self::F32 => matches!(
                inner,
                naga::TypeInner::Scalar(naga::Scalar {
                    kind: naga::ScalarKind::Float,
                    width: 4,
                })
            ),
        }
    }
}

/// Find `FxUniforms` at (set 0, binding 2), require the exact builtin head,
/// and reflect every following member as a user parameter declaration plus
/// its std140 upload entry. A module without the block, or with a mismatched
/// head, is rejected (ADR-0011 §8: nothing silently passes).
fn reflect_user_params(
    module: &naga::Module,
    annotations: &std::collections::HashMap<String, Annotation>,
) -> Result<(Vec<ParamDeclaration>, UniformBlockLayout), FrontendError> {
    let block = module.global_variables.iter().find(|(_, var)| {
        var.binding
            .as_ref()
            .is_some_and(|b| b.group == 0 && b.binding == 2)
    });
    let Some((_, var)) = block else {
        return Err(FrontendError::Abi(
            "missing FxUniforms block at (set = 0, binding = 2)".into(),
        ));
    };
    let naga::TypeInner::Struct { members, span } = &module.types[var.ty].inner else {
        return Err(FrontendError::Abi("FxUniforms is not a struct block".into()));
    };
    if members.len() < ABI_HEAD.len() {
        return Err(FrontendError::Abi(
            "FxUniforms head must start with u_resolution, u_time, u_frame".into(),
        ));
    }
    for (member, (name, ty, offset)) in members.iter().zip(ABI_HEAD.iter()) {
        let member_name = member.name.as_deref().unwrap_or_default();
        if member_name != *name
            || !ty.matches(&module.types[member.ty].inner)
            || member.offset != *offset
        {
            return Err(FrontendError::Abi(format!(
                "FxUniforms head mismatch at `{member_name}` (expected `{name}` at offset {offset})"
            )));
        }
    }

    let mut params = Vec::new();
    let mut entries = Vec::new();
    for member in members.iter().skip(ABI_HEAD.len()) {
        let name = member.name.clone().unwrap_or_default();
        let mut ty = map_member_type(&module.types[member.ty].inner).ok_or_else(|| {
            FrontendError::Param(format!("`{name}`: type outside the v1 parameter set"))
        })?;
        let id =
            ParamId::new(&name).map_err(|e| FrontendError::Param(format!("`{name}`: {e:?}")))?;

        // Merge the annotation, if one names this member. Unmatched
        // annotations are ignored (stale leftovers are harmless); matched
        // ones are contract and fail closed on any inconsistency.
        let mut aliases = Vec::new();
        let mut ui = ParamUiMeta::default();
        if let Some(annotation) = annotations.get(&name) {
            match (annotation.hint, ty) {
                (Some(Hint::Angle), ShaderParamType::Float) => ty = ShaderParamType::AngleFloat,
                (Some(Hint::Angle), _) => {
                    return Err(FrontendError::Param(format!(
                        "`{name}`: hint:angle applies to float members only"
                    )));
                }
                // std140 has no host-shareable bool: a checkbox parameter is
                // an int member with hint:bool (ADR-0011 "bool as i32").
                (Some(Hint::Bool), ShaderParamType::Int) => ty = ShaderParamType::Bool,
                (Some(Hint::Bool), _) => {
                    return Err(FrontendError::Param(format!(
                        "`{name}`: hint:bool applies to int members only"
                    )));
                }
                (Some(Hint::Color), ShaderParamType::Vec3Color | ShaderParamType::Vec4Color) => {}
                (Some(Hint::Color), _) => {
                    return Err(FrontendError::Param(format!(
                        "`{name}`: hint:color applies to vec3/vec4 members only"
                    )));
                }
                (None, _) => {}
            }
            if let Some(default) = &annotation.default {
                let expected = match ty {
                    ShaderParamType::Float
                    | ShaderParamType::AngleFloat
                    | ShaderParamType::Int
                    | ShaderParamType::Bool => 1,
                    // Writing color/point defaults needs AEGP stream-value
                    // plumbing that v1 does not have; fail closed instead of
                    // silently ignoring the user's default.
                    ShaderParamType::Vec2 | ShaderParamType::Vec3Color
                    | ShaderParamType::Vec4Color => {
                        return Err(FrontendError::Param(format!(
                            "`{name}`: default is scalar-only in v1 (color/point defaults are not supported yet)"
                        )));
                    }
                };
                if default.len() != expected {
                    return Err(FrontendError::Param(format!(
                        "`{name}`: default needs {expected} component(s), got {}",
                        default.len()
                    )));
                }
            }
            if let (Some(min), Some(max)) = (annotation.min, annotation.max) {
                if min > max {
                    return Err(FrontendError::Param(format!("`{name}`: min > max")));
                }
            }
            aliases = annotation.aliases.clone();
            ui = ParamUiMeta {
                label: annotation.label.clone(),
                min: annotation.min,
                max: annotation.max,
                default: annotation.default.clone(),
            };
        }

        let (words, int) = match ty {
            ShaderParamType::Float | ShaderParamType::AngleFloat => (1, false),
            ShaderParamType::Int | ShaderParamType::Bool => (1, true),
            ShaderParamType::Vec2 => (2, false),
            ShaderParamType::Vec3Color => (3, false),
            ShaderParamType::Vec4Color => (4, false),
        };
        entries.push(UniformEntry { offset: member.offset as usize, words, int });
        params.push(ParamDeclaration { id, ty, aliases, ui });
    }
    let layout = UniformBlockLayout { block_size: (*span as usize).max(16), entries };
    Ok((params, layout))
}

/// ABI v1 user parameter types (ADR-0011 §4) to their ADR-0013 defaults:
/// vec3 is Color by default; the `angle` hint arrives with M2 annotations.
fn map_member_type(inner: &naga::TypeInner) -> Option<ShaderParamType> {
    use naga::{ScalarKind, TypeInner, VectorSize};
    match inner {
        TypeInner::Scalar(naga::Scalar { kind: ScalarKind::Float, width: 4 }) => {
            Some(ShaderParamType::Float)
        }
        TypeInner::Scalar(naga::Scalar { kind: ScalarKind::Sint, width: 4 }) => {
            Some(ShaderParamType::Int)
        }
        TypeInner::Scalar(naga::Scalar { kind: ScalarKind::Bool, .. }) => {
            Some(ShaderParamType::Bool)
        }
        TypeInner::Vector { size, scalar: naga::Scalar { kind: ScalarKind::Float, width: 4 } } => {
            match size {
                VectorSize::Bi => Some(ShaderParamType::Vec2),
                VectorSize::Tri => Some(ShaderParamType::Vec3Color),
                VectorSize::Quad => Some(ShaderParamType::Vec4Color),
            }
        }
        _ => None,
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &s[..end])
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
        // Color/point defaults are scalar-only in v1: explicit rejection,
        // never a silent no-op.
        let color_default = format!("{head}// @param tint default:1,0.5,0.25\n{body}");
        let err = parse(&color_default).expect_err("color default is v1-rejected");
        assert!(matches!(err, FrontendError::Param(ref m) if m.contains("scalar-only")), "{err:?}");
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
