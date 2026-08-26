//! Minimal `EffectDefinition` / `RenderGraph` / `PassDefinition` for M1:
//! one-pass graphs only. Single-pass is a one-node graph, not a separate
//! runtime (ADR-0003); edges, resources, and the multi-pass grammar arrive
//! with the M4 entry ADRs.

use crate::binding::{
    bank_capacity, build_fresh_counted, build_with_reuse_counted, BindingError, BindingPlan,
    BANK_GROUPS,
};
use crate::definition::param::{ParamDeclaration, ParamId};
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
    /// Scalar Float parameter that defines canvas expansion, when declared.
    pub canvas_param: Option<ParamId>,
    pub graph: RenderGraph,
    pub binding: BindingPlan,
    pub bank_spills: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LowerError {
    Binding(BindingError),
    /// ADR-0018 §6: same name = same parameter; types must agree.
    ParamTypeConflict { name: String },
    /// More than one merged parameter claims the effect-wide canvas role.
    DuplicateCanvas { first: String, second: String },
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
                    merged[index].canvas |= decl.canvas;
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

    if manifest.len() >= 2 {
        for (index, decl) in merged.iter_mut().enumerate() {
            let mut sole_pass = None;
            let mut shared = false;
            for (pass, map) in maps.iter().enumerate() {
                if map.contains(&index) {
                    if sole_pass.replace(pass).is_some() {
                        shared = true;
                        break;
                    }
                }
            }
            if !shared
                && sole_pass.is_some_and(|pass| pass < BANK_GROUPS)
                && decl.ty.slot_requirements().iter().all(|kind| bank_capacity(*kind) > 0)
            {
                decl.bank = sole_pass;
            }
        }
    }

    let mut canvas_param: Option<ParamId> = None;
    for decl in merged.iter().filter(|decl| decl.canvas) {
        if let Some(first) = &canvas_param {
            return Err(LowerError::DuplicateCanvas {
                first: first.as_str().to_owned(),
                second: decl.id.as_str().to_owned(),
            });
        }
        canvas_param = Some(decl.id.clone());
    }

    let (binding, bank_spills) = match previous {
        Some(previous) => build_with_reuse_counted(&merged, previous)?,
        None => build_fresh_counted(&merged)?,
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
        EffectDefinition {
            language,
            params: merged,
            canvas_param,
            graph: RenderGraph { passes },
            binding,
            bank_spills,
        },
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
            canvas: false,
            bank: None,
        }
    }

    fn canvas_decl(id: &str) -> ParamDeclaration {
        ParamDeclaration {
            canvas: true,
            ..decl(id, ShaderParamType::Float)
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
        assert_eq!(
            def.binding.bindings[0].slots[0],
            crate::binding::SlotRef { kind: crate::binding::PoolKind::Float, index: 0 }
        );
        assert_eq!(def.bank_spills, 0);
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
        assert_eq!(def.binding.bindings[0].slots[0].index, 0, "shared radius stays in Main");

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
    fn canvas_authority_is_resolved_after_cross_pass_merge() {
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
        let (shared, _) = lower_graph(
            LanguageId::GLSL,
            &manifest,
            &bodies,
            &[vec![canvas_decl("reach")], vec![canvas_decl("reach")]],
            None,
        )
        .unwrap();
        assert_eq!(shared.params.len(), 1);
        assert_eq!(
            shared.canvas_param.as_ref().map(ParamId::as_str),
            Some("reach")
        );

        let (plain, _) = lower_graph(
            LanguageId::GLSL,
            &manifest,
            &bodies,
            &[vec![decl("reach", ShaderParamType::Float)], vec![]],
            None,
        )
        .unwrap();
        assert_eq!(plain.canvas_param, None);

        let duplicate = lower_graph(
            LanguageId::GLSL,
            &manifest,
            &bodies,
            &[vec![canvas_decl("reach")], vec![canvas_decl("spread")]],
            None,
        );
        assert!(matches!(
            duplicate,
            Err(LowerError::DuplicateCanvas { first, second })
                if first == "reach" && second == "spread"
        ));
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

    fn manifest(count: usize) -> (Vec<ManifestPass>, Vec<String>) {
        let passes = (0..count)
            .map(|pass| ManifestPass {
                name: format!("p{pass}"),
                inputs: vec!["input".into()],
                output: if pass + 1 == count { "output".into() } else { format!("t{pass}") },
                line: pass + 1,
            })
            .collect();
        let bodies = (0..count).map(|pass| format!("body {pass}")).collect();
        (passes, bodies)
    }

    #[test]
    fn two_pass_parameters_allocate_to_their_pass_banks() {
        let (manifest, bodies) = manifest(2);
        let (def, _) = lower_graph(
            LanguageId::GLSL,
            &manifest,
            &bodies,
            &[
                vec![decl("left", ShaderParamType::Float)],
                vec![decl("right", ShaderParamType::Float)],
            ],
            None,
        )
        .unwrap();
        assert_eq!(def.binding.bindings[0].slots[0].index, 48);
        assert_eq!(def.binding.bindings[1].slots[0].index, 56);
        assert_eq!(def.bank_spills, 0);
    }

    #[test]
    fn bank_overflow_spills_the_parameter_to_main_and_counts_it() {
        let (manifest, bodies) = manifest(2);
        let first: Vec<_> =
            (0..9).map(|i| decl(&format!("f{i}"), ShaderParamType::Float)).collect();
        let (def, _) = lower_graph(
            LanguageId::GLSL,
            &manifest,
            &bodies,
            &[first, vec![]],
            None,
        )
        .unwrap();
        assert_eq!(def.binding.bindings[7].slots[0].index, 55);
        assert_eq!(def.binding.bindings[8].slots[0].index, 0);
        assert_eq!(def.bank_spills, 1);
    }

    #[test]
    fn pass_beyond_the_declared_banks_allocates_in_main() {
        let (manifest, bodies) = manifest(13);
        let mut per_pass = vec![Vec::new(); 13];
        per_pass[12].push(decl("late", ShaderParamType::Float));
        let (def, _) =
            lower_graph(LanguageId::GLSL, &manifest, &bodies, &per_pass, None).unwrap();
        assert_eq!(def.binding.bindings[0].slots[0].index, 0);
        assert_eq!(def.bank_spills, 0);
    }

    #[test]
    fn exact_id_inheritance_keeps_main_when_assignment_moves_to_a_bank() {
        let single = single_pass_manifest();
        let (old, _) = lower_graph(
            LanguageId::GLSL,
            &single,
            &["old".into()],
            &[vec![decl("stable", ShaderParamType::Float)]],
            None,
        )
        .unwrap();
        assert_eq!(old.binding.bindings[0].slots[0].index, 0);

        let (manifest, bodies) = manifest(2);
        let (new, _) = lower_graph(
            LanguageId::GLSL,
            &manifest,
            &bodies,
            &[
                vec![decl("stable", ShaderParamType::Float)],
                vec![decl("other", ShaderParamType::Float)],
            ],
            Some(&old.binding),
        )
        .unwrap();
        assert_eq!(new.binding.bindings[0].slots[0].index, 0);
        assert!(new.binding.bindings[0].inherited);
        assert_eq!(new.bank_spills, 0);
    }
}
