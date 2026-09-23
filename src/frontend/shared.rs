//! Language-neutral ABI head and parameter reflection from the original Naga IR.
//! GLSL and WGSL share annotation semantics and upload layouts (ADR-0044).
//! The frontend validates its resource/entry surface before calling this module;
//! offsets and span come from that module, never a generated interface schema.

use super::annotation::{Annotation, Hint};
use super::{FrontendError, UniformBlockLayout, UniformEntry};
use crate::definition::param::{ParamDeclaration, ParamId, ParamUiMeta, ShaderParamType};

/// The fixed ABI builtin head (ADR-0011 §4): name, then a type predicate,
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
                    scalar: naga::Scalar {
                        kind: naga::ScalarKind::Float,
                        width: 4
                    },
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
/// its reflected upload entry. A module without the block, or with a mismatched
/// head, is rejected (ADR-0011 §8: nothing silently passes).
pub(super) fn reflect_user_params(
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
        return Err(FrontendError::Abi(
            "FxUniforms is not a struct block".into(),
        ));
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
        let mut canvas = false;
        if let Some(annotation) = annotations.get(&name) {
            match (annotation.hint, ty) {
                (Some(Hint::Canvas), ShaderParamType::Float) => canvas = true,
                (Some(Hint::Canvas), _) => {
                    return Err(FrontendError::CanvasWrongKind(name));
                }
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
                // ADR-0034 §2: a vec3 stays a colour unless it says otherwise.
                // Flipping the default would silently retype every existing
                // shader's colours, which is what ADR-0026 exists to prevent.
                (Some(Hint::Point3D), ShaderParamType::Vec3Color) => ty = ShaderParamType::Point3D,
                (Some(Hint::Point3D), _) => {
                    return Err(FrontendError::Param(format!(
                        "`{name}`: hint:point3d applies to vec3 members only"
                    )));
                }
                (Some(Hint::Color), ShaderParamType::Vec3Color | ShaderParamType::Vec4Color) => {}
                (Some(Hint::Color), _) => {
                    return Err(FrontendError::Param(format!(
                        "`{name}`: hint:color applies to vec3/vec4 members only"
                    )));
                }
                // ADR-0030/0032/0035: all three are texture bindings, not block
                // storage. Reaching here means the id names both a
                // graph-resource annotation and an FxUniforms member — the two
                // would fight over one ParamId, so reject rather than pick a
                // winner.
                (Some(Hint::Path), _) => {
                    return Err(FrontendError::Param(format!(
                        "`{name}`: hint:path names a graph input, so it must not \
                         also be an FxUniforms member"
                    )));
                }
                (Some(Hint::Coverage), _) => {
                    return Err(FrontendError::Param(format!("`{name}`: coverage is a texture, not a uniform member")));
                }
                (Some(Hint::Layer), _) => {
                    return Err(FrontendError::Param(format!(
                        "`{name}`: hint:layer names a graph input, so it must not \
                         also be an FxUniforms member"
                    )));
                }
                (Some(Hint::Gradient), _) => {
                    return Err(FrontendError::Param(format!(
                        "`{name}`: hint:gradient names a graph input, so it must \
                         not also be an FxUniforms member"
                    )));
                }
                (None, _) => {}
            }
            if let Some(default) = &annotation.default {
                // ADR-0026: color defaults are supported (3 components, or 4
                // with explicit alpha — the hex forms decode to exactly
                // these). Vec2 (point) defaults still lack stream plumbing.
                let accepted: &[usize] = match ty {
                    ShaderParamType::Float
                    | ShaderParamType::AngleFloat
                    | ShaderParamType::Int
                    | ShaderParamType::Bool => &[1],
                    // vec3 has no alpha companion slot — a 4th component
                    // would be silently dropped, so reject it.
                    ShaderParamType::Vec3Color => &[3],
                    ShaderParamType::Vec4Color => &[3, 4],
                    // Unreachable: reflection only ever yields member types,
                    // and the hint:layer arm above already rejected the id.
                    ShaderParamType::Layer | ShaderParamType::Gradient | ShaderParamType::Path | ShaderParamType::Coverage => {
                        &[]
                    }
                    // ADR-0034 §4 keeps Point 3D where Point 2D already is:
                    // a default would need AEGP ThreeD stream-value plumbing
                    // neither kind has, so the pool default is AE's own.
                    ShaderParamType::Vec2 | ShaderParamType::Point3D => {
                        return Err(FrontendError::Param(format!(
                            "`{name}`: point defaults are not supported yet"
                        )));
                    }
                };
                if !accepted.contains(&default.len()) {
                    return Err(FrontendError::Param(format!(
                        "`{name}`: default needs {} component(s), got {}",
                        accepted
                            .iter()
                            .map(|n| n.to_string())
                            .collect::<Vec<_>>()
                            .join(" or "),
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
            ShaderParamType::Vec3Color | ShaderParamType::Point3D => (3, false),
            ShaderParamType::Vec4Color => (4, false),
            // Unreachable for the same reason as above; a layer occupies no
            // block words, so zero is also the honest answer if it were.
            ShaderParamType::Layer | ShaderParamType::Gradient | ShaderParamType::Path | ShaderParamType::Coverage => {
                (0, false)
            }
        };
        entries.push(UniformEntry {
            offset: member.offset as usize,
            words,
            int,
        });
        params.push(ParamDeclaration {
            id,
            ty,
            aliases,
            ui,
            canvas,
            bank: None,
        });
    }
    let layout = UniformBlockLayout {
        block_size: (*span as usize).max(16),
        entries,
    };
    Ok((params, layout))
}

/// ABI v1 user parameter types (ADR-0011 §4) to their ADR-0013 defaults:
/// vec3 is Color by default; the `angle` hint arrives with M2 annotations.
fn map_member_type(inner: &naga::TypeInner) -> Option<ShaderParamType> {
    use naga::{ScalarKind, TypeInner, VectorSize};
    match inner {
        TypeInner::Scalar(naga::Scalar {
            kind: ScalarKind::Float,
            width: 4,
        }) => Some(ShaderParamType::Float),
        TypeInner::Scalar(naga::Scalar {
            kind: ScalarKind::Sint,
            width: 4,
        }) => Some(ShaderParamType::Int),
        TypeInner::Scalar(naga::Scalar {
            kind: ScalarKind::Bool,
            ..
        }) => Some(ShaderParamType::Bool),
        TypeInner::Vector {
            size,
            scalar:
                naga::Scalar {
                    kind: ScalarKind::Float,
                    width: 4,
                },
        } => match size {
            VectorSize::Bi => Some(ShaderParamType::Vec2),
            VectorSize::Tri => Some(ShaderParamType::Vec3Color),
            VectorSize::Quad => Some(ShaderParamType::Vec4Color),
        },
        _ => None,
    }
}

pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &s[..end])
}
