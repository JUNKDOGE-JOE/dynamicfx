//! Minimal `EffectDefinition` / `RenderGraph` / `PassDefinition` for M1:
//! one-pass graphs only. Single-pass is a one-node graph, not a separate
//! runtime (ADR-0003); edges, resources, and the multi-pass grammar arrive
//! with the M4 entry ADRs.

use crate::binding::{build_fresh, build_with_reuse, BindingError, BindingPlan};
use crate::definition::param::ParamDeclaration;
use crate::frontend::grammar::ManifestPass;
use crate::frontend::LanguageId;

#[derive(Debug, Clone)]
pub struct PassDefinition {
    pub name: String,
    /// Exact unescaped module source for this pass.
    pub source: String,
    /// Manifest input resource names (`input` = the effect input).
    pub inputs: Vec<String>,
    /// Manifest output resource name (`output` = the final output).
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct RenderGraph {
    /// Manifest declaration order.
    pub passes: Vec<PassDefinition>,
}

#[derive(Debug, Clone)]
pub struct EffectDefinition {
    pub language: LanguageId,
    /// Effect-wide merged parameters (ADR-0018 §6), first-appearance order.
    pub params: Vec<ParamDeclaration>,
    pub graph: RenderGraph,
    pub binding: BindingPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LowerError {
    Binding(BindingError),
    /// ADR-0018 §6: same name = same parameter; types must agree.
    ParamTypeConflict { name: String },
}

impl From<BindingError> for LowerError {
    fn from(e: BindingError) -> Self {
        Self::Binding(e)
    }
}

/// Lower a validated graph (any pass count — raw single-pass input arrives
/// as an implicit one-pass manifest, ADR-0003) into an `EffectDefinition`
/// with a fully validated `BindingPlan`, plus each pass's member→merged-
/// parameter index map for uniform upload. With a previous plan, slots
/// follow stable IDs and aliases (ADR-0013 §2); rejection stays atomic.
pub fn lower_graph(
    language: LanguageId,
    manifest: &[ManifestPass],
    bodies: &[String],
    per_pass_params: &[Vec<ParamDeclaration>],
    previous: Option<&BindingPlan>,
) -> Result<(EffectDefinition, Vec<Vec<usize>>), LowerError> {
    // Effect-wide merge: same name = same parameter, same type required.
    let mut merged: Vec<ParamDeclaration> = Vec::new();
    let mut maps: Vec<Vec<usize>> = Vec::with_capacity(per_pass_params.len());
    for params in per_pass_params {
        let mut map = Vec::with_capacity(params.len());
        for decl in params {
            match merged.iter().position(|m| m.id == decl.id) {
                Some(index) => {
                    if merged[index].ty != decl.ty {
                        return Err(LowerError::ParamTypeConflict {
                            name: decl.id.as_str().to_owned(),
                        });
                    }
                    map.push(index);
                }
                None => {
                    merged.push(decl.clone());
                    map.push(merged.len() - 1);
                }
            }
        }
        maps.push(map);
    }

    let binding = match previous {
        Some(previous) => build_with_reuse(&merged, previous)?,
        None => build_fresh(&merged)?,
    };
    let passes = manifest
        .iter()
        .zip(bodies)
        .map(|(pass, body)| PassDefinition {
            name: pass.name.clone(),
            source: body.clone(),
            inputs: pass.inputs.clone(),
            output: pass.output.clone(),
        })
        .collect();
    Ok((
        EffectDefinition { language, params: merged, graph: RenderGraph { passes }, binding },
        maps,
    ))
}

/// The implicit one-pass manifest for raw single-pass input.
pub fn single_pass_manifest() -> Vec<ManifestPass> {
    vec![ManifestPass {
        name: "main".to_owned(),
        inputs: vec![crate::frontend::grammar::RES_INPUT.to_owned()],
        output: crate::frontend::grammar::RES_OUTPUT.to_owned(),
        line: 1,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::param::{ParamId, ShaderParamType};

    fn decl(id: &str, ty: ShaderParamType) -> ParamDeclaration {
        ParamDeclaration {
            id: ParamId::new(id).unwrap(),
            ty,
            aliases: vec![],
            ui: Default::default(),
        }
    }

    #[test]
    fn raw_source_lowers_to_a_one_pass_graph() {
        let manifest = single_pass_manifest();
        let (def, maps) = lower_graph(
            LanguageId::GLSL,
            &manifest,
            &["void main() {}".to_string()],
            &[vec![decl("speed", ShaderParamType::Float)]],
            None,
        )
        .unwrap();
        assert_eq!(def.graph.passes.len(), 1);
        assert_eq!(def.graph.passes[0].name, "main");
        assert_eq!(def.graph.passes[0].source, "void main() {}");
        assert_eq!(def.graph.passes[0].inputs, vec!["input"]);
        assert_eq!(def.graph.passes[0].output, "output");
        assert_eq!(def.binding.bindings.len(), 1);
        assert_eq!(maps, vec![vec![0]]);
        assert_eq!(def.language, LanguageId::GLSL);
    }

    /// Cross-pass merge: same name = same parameter (one slot, shared map
    /// index); a type conflict rejects the whole definition (ADR-0018 §6).
    #[test]
    fn cross_pass_parameters_merge_by_name() {
        let manifest = vec![
            ManifestPass {
                name: "a".into(),
                inputs: vec!["input".into()],
                output: "t".into(),
                line: 1,
            },
            ManifestPass {
                name: "b".into(),
                inputs: vec!["t".into()],
                output: "output".into(),
                line: 2,
            },
        ];
        let bodies = vec!["A".to_string(), "B".to_string()];
        let (def, maps) = lower_graph(
            LanguageId::GLSL,
            &manifest,
            &bodies,
            &[
                vec![decl("radius", ShaderParamType::Float), decl("tint", ShaderParamType::Vec3Color)],
                vec![decl("radius", ShaderParamType::Float)],
            ],
            None,
        )
        .unwrap();
        assert_eq!(def.params.len(), 2);
        assert_eq!(def.binding.bindings.len(), 2);
        assert_eq!(maps, vec![vec![0, 1], vec![0]]);

        let conflict = lower_graph(
            LanguageId::GLSL,
            &manifest,
            &bodies,
            &[
                vec![decl("radius", ShaderParamType::Float)],
                vec![decl("radius", ShaderParamType::Int)],
            ],
            None,
        );
        assert!(matches!(conflict, Err(LowerError::ParamTypeConflict { .. })));
    }

    #[test]
    fn pool_overflow_rejects_the_whole_definition() {
        let params: Vec<_> =
            (0..49).map(|i| decl(&format!("f{i}"), ShaderParamType::Float)).collect();
        let manifest = single_pass_manifest();
        assert!(matches!(
            lower_graph(LanguageId::GLSL, &manifest, &["src".to_string()], &[params], None),
            Err(LowerError::Binding(BindingError::PoolOverflow { .. }))
        ));
    }
}
