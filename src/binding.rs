//! v1 parameter pools and BindingPlan allocation (ADR-0013).
//!
//! `V1_POOLS` is the single configuration source (ADR-0013 §3): the
//! PARAMS_SETUP declaration, validation, and documentation all derive from
//! it. Growth is append-only — a released index never changes kind, moves,
//! or dies — so plans carry explicit slot tables, never contiguity
//! assumptions.

use crate::definition::param::{
    validate_declarations, DeclarationError, ParamDeclaration, ParamId,
};

/// AE pool kinds declared in v1 (ADR-0013 §3). Popup is deliberately absent
/// (TR-M0-006: menus are immutable after PARAMS_SETUP); Point 3D and Layer
/// are reserved for their own entry evidence/ADRs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoolKind {
    Float,
    Integer,
    Bool,
    Color,
    Point2D,
    Angle,
}

/// The v1 pool table: 104 slots total. Capacity changes append at the tail
/// of the parameter list and ship as a new build (ADR-0013 §5).
pub const V1_POOLS: &[(PoolKind, usize)] = &[
    (PoolKind::Float, 48),
    (PoolKind::Integer, 8),
    (PoolKind::Bool, 16),
    (PoolKind::Color, 12),
    (PoolKind::Point2D, 12),
    (PoolKind::Angle, 8),
];

pub fn pool_capacity(kind: PoolKind) -> usize {
    V1_POOLS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, capacity)| *capacity)
        .unwrap_or(0)
}

/// A slot within one kind's pool. Indexes are kind-local; after future
/// appends a pool's slot set may be non-contiguous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotRef {
    pub kind: PoolKind,
    pub index: usize,
}

/// One bound parameter. `Vec4Color` carries two slots (Color + Float alpha);
/// everything else carries one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamBinding {
    pub id: ParamId,
    pub slots: Vec<SlotRef>,
    /// Slots carried over from the previous plan. Annotation defaults are
    /// written only to fresh bindings, so user values and keyframes on
    /// inherited slots are never overwritten.
    pub inherited: bool,
}

/// Immutable result of a successful, fully validated allocation (§11.1 of
/// the architecture: nothing touches AE UI until the whole plan validates).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingPlan {
    pub bindings: Vec<ParamBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingError {
    /// Diagnostic classes param-grammar/reserved-id and alias-conflict.
    Declarations(DeclarationError),
    /// Diagnostic class pool-overflow. The whole definition is rejected;
    /// nothing partially binds.
    PoolOverflow { kind: PoolKind, capacity: usize, required: usize },
}

impl From<DeclarationError> for BindingError {
    fn from(e: DeclarationError) -> Self {
        Self::Declarations(e)
    }
}

/// Fresh allocation for a definition with no previous plan (the first apply
/// path): equivalent to reuse against an empty plan.
pub fn build_fresh(decls: &[ParamDeclaration]) -> Result<BindingPlan, BindingError> {
    build_with_reuse(decls, &BindingPlan { bindings: Vec::new() })
}

/// Allocation against a previous plan (ADR-0013 §2, architecture §11.1):
/// an exact current-ID match inherits its previous slots; on miss, an alias
/// matching a previous binding's ID inherits (single-generation, never a
/// chain). Inheritance requires the slot-kind requirements to be unchanged —
/// a kind change correctly reads as a different parameter and reallocates.
/// Unmatched declarations take ascending free slots per kind. Capacity is
/// validated over the complete plan before anything is returned (atomic
/// rejection; keyframed streams on inherited slots survive untouched).
pub fn build_with_reuse(
    decls: &[ParamDeclaration],
    previous: &BindingPlan,
) -> Result<BindingPlan, BindingError> {
    validate_declarations(decls)?;

    let prev_by_id: std::collections::HashMap<&str, &ParamBinding> =
        previous.bindings.iter().map(|b| (b.id.as_str(), b)).collect();

    // Pass 1: inheritance decisions. `taken` guards against two declarations
    // claiming one slot (the shared ID/alias namespace already prevents the
    // ordinary routes to that; this is defense in depth).
    let mut taken: std::collections::HashSet<SlotRef> = Default::default();
    let inherited: Vec<Option<Vec<SlotRef>>> = decls
        .iter()
        .map(|decl| {
            let required = decl.ty.slot_requirements();
            let candidate = prev_by_id.get(decl.id.as_str()).or_else(|| {
                decl.aliases
                    .iter()
                    .find_map(|alias| prev_by_id.get(alias.as_str()))
            });
            let slots = candidate.and_then(|binding| {
                let kinds: Vec<PoolKind> = binding.slots.iter().map(|s| s.kind).collect();
                (kinds == required && binding.slots.iter().all(|s| !taken.contains(s)))
                    .then(|| binding.slots.clone())
            });
            if let Some(slots) = &slots {
                taken.extend(slots.iter().copied());
            }
            slots
        })
        .collect();

    // Capacity over the complete plan: inherited occupancy plus fresh needs.
    let mut required: std::collections::HashMap<PoolKind, usize> = Default::default();
    for slot in &taken {
        *required.entry(slot.kind).or_default() += 1;
    }
    for (decl, inherited) in decls.iter().zip(&inherited) {
        if inherited.is_none() {
            for kind in decl.ty.slot_requirements() {
                *required.entry(*kind).or_default() += 1;
            }
        }
    }
    for (kind, needed) in &required {
        let capacity = pool_capacity(*kind);
        if *needed > capacity {
            return Err(BindingError::PoolOverflow {
                kind: *kind,
                capacity,
                required: *needed,
            });
        }
    }

    // Pass 2: fill unmatched declarations from ascending free indexes,
    // skipping inherited slots (plans carry explicit tables, so holes are
    // fine — ADR-0013 §5).
    let mut next_index: std::collections::HashMap<PoolKind, usize> = Default::default();
    let mut allocate = |kind: PoolKind| -> SlotRef {
        let cursor = next_index.entry(kind).or_default();
        loop {
            let slot = SlotRef { kind, index: *cursor };
            *cursor += 1;
            if !taken.contains(&slot) {
                return slot;
            }
        }
    };
    let bindings = decls
        .iter()
        .zip(inherited)
        .map(|(decl, inherited)| {
            let was_inherited = inherited.is_some();
            let slots = inherited.unwrap_or_else(|| {
                decl.ty
                    .slot_requirements()
                    .iter()
                    .map(|kind| allocate(*kind))
                    .collect()
            });
            ParamBinding { id: decl.id.clone(), slots, inherited: was_inherited }
        })
        .collect();
    Ok(BindingPlan { bindings })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::param::ShaderParamType;

    fn decl(id: &str, ty: ShaderParamType) -> ParamDeclaration {
        ParamDeclaration { id: ParamId::new(id).unwrap(), ty, aliases: vec![], ui: Default::default() }
    }

    #[test]
    fn pool_table_matches_the_adr_totals() {
        let total: usize = V1_POOLS.iter().map(|(_, c)| c).sum();
        assert_eq!(total, 104, "ADR-0013 fixes 104 v1 slots");
        // Every kind appears exactly once: the table is the single source.
        let mut kinds: Vec<PoolKind> = V1_POOLS.iter().map(|(k, _)| *k).collect();
        kinds.dedup();
        assert_eq!(kinds.len(), V1_POOLS.len());
    }

    #[test]
    fn full_float_pool_binds_and_one_more_rejects() {
        let full: Vec<_> =
            (0..48).map(|i| decl(&format!("f{i}"), ShaderParamType::Float)).collect();
        let plan = build_fresh(&full).expect("48 floats fit exactly");
        assert_eq!(plan.bindings.len(), 48);
        assert_eq!(plan.bindings[47].slots[0], SlotRef { kind: PoolKind::Float, index: 47 });

        let mut over = full;
        over.push(decl("f48", ShaderParamType::Float));
        assert_eq!(
            build_fresh(&over),
            Err(BindingError::PoolOverflow {
                kind: PoolKind::Float,
                capacity: 48,
                required: 49
            })
        );
    }

    #[test]
    fn vec4_pairs_color_and_float_atomically() {
        let one = [decl("tint", ShaderParamType::Vec4Color)];
        let plan = build_fresh(&one).unwrap();
        assert_eq!(
            plan.bindings[0].slots,
            vec![
                SlotRef { kind: PoolKind::Color, index: 0 },
                SlotRef { kind: PoolKind::Float, index: 0 },
            ]
        );

        // 12 vec4s fill Color exactly and take 12 Float slots with them.
        let twelve: Vec<_> =
            (0..12).map(|i| decl(&format!("c{i}"), ShaderParamType::Vec4Color)).collect();
        assert!(build_fresh(&twelve).is_ok());

        let mut thirteen = twelve.clone();
        thirteen.push(decl("c12", ShaderParamType::Vec4Color));
        assert!(matches!(
            build_fresh(&thirteen),
            Err(BindingError::PoolOverflow { kind: PoolKind::Color, .. })
        ));

        // The Float half of the pairing counts against the Float pool too:
        // 12 vec4 + 37 plain floats needs 49 Float slots.
        let mut float_squeeze = twelve;
        for i in 0..37 {
            float_squeeze.push(decl(&format!("f{i}"), ShaderParamType::Float));
        }
        assert_eq!(
            build_fresh(&float_squeeze),
            Err(BindingError::PoolOverflow {
                kind: PoolKind::Float,
                capacity: 48,
                required: 49
            })
        );
    }

    #[test]
    fn kinds_allocate_independently_in_declaration_order() {
        let decls = [
            decl("speed", ShaderParamType::Float),
            decl("steps", ShaderParamType::Int),
            decl("invert", ShaderParamType::Bool),
            decl("center", ShaderParamType::Vec2),
            decl("glow", ShaderParamType::Vec3Color),
            decl("sweep", ShaderParamType::AngleFloat),
            decl("gain", ShaderParamType::Float),
        ];
        let plan = build_fresh(&decls).unwrap();
        assert_eq!(plan.bindings[0].slots, vec![SlotRef { kind: PoolKind::Float, index: 0 }]);
        assert_eq!(plan.bindings[1].slots, vec![SlotRef { kind: PoolKind::Integer, index: 0 }]);
        assert_eq!(plan.bindings[2].slots, vec![SlotRef { kind: PoolKind::Bool, index: 0 }]);
        assert_eq!(plan.bindings[3].slots, vec![SlotRef { kind: PoolKind::Point2D, index: 0 }]);
        assert_eq!(plan.bindings[4].slots, vec![SlotRef { kind: PoolKind::Color, index: 0 }]);
        assert_eq!(plan.bindings[5].slots, vec![SlotRef { kind: PoolKind::Angle, index: 0 }]);
        // Second Float lands on the next kind-local index.
        assert_eq!(plan.bindings[6].slots, vec![SlotRef { kind: PoolKind::Float, index: 1 }]);
    }

    #[test]
    fn namespace_conflicts_reject_before_allocation() {
        let decls = [decl("speed", ShaderParamType::Float), decl("speed", ShaderParamType::Int)];
        assert!(matches!(build_fresh(&decls), Err(BindingError::Declarations(_))));
    }

    fn decl_aliased(id: &str, ty: ShaderParamType, alias: &str) -> ParamDeclaration {
        ParamDeclaration {
            id: ParamId::new(id).unwrap(),
            ty,
            aliases: vec![ParamId::new(alias).unwrap()],
            ui: Default::default(),
        }
    }

    /// Reordered declarations keep their slots: keyframes follow the ID,
    /// not the declaration position (ADR-0013 §2).
    #[test]
    fn same_ids_keep_slots_across_reorder() {
        let v1 = [decl("a", ShaderParamType::Float), decl("b", ShaderParamType::Float)];
        let plan1 = build_fresh(&v1).unwrap();
        let v2 = [decl("b", ShaderParamType::Float), decl("a", ShaderParamType::Float)];
        let plan2 = build_with_reuse(&v2, &plan1).unwrap();
        // b still owns Float 1, a still owns Float 0, despite the swap.
        assert_eq!(plan2.bindings[0].slots, vec![SlotRef { kind: PoolKind::Float, index: 1 }]);
        assert_eq!(plan2.bindings[1].slots, vec![SlotRef { kind: PoolKind::Float, index: 0 }]);
    }

    /// Rename with alias inherits the old slot; rename without one
    /// allocates fresh (visible, intended — ADR-0013 §2).
    #[test]
    fn alias_inherits_and_plain_rename_does_not() {
        let v1 = [decl("speed", ShaderParamType::Float)];
        let plan1 = build_fresh(&v1).unwrap();

        let renamed = [decl_aliased("velocity", ShaderParamType::Float, "speed")];
        let plan2 = build_with_reuse(&renamed, &plan1).unwrap();
        assert_eq!(plan2.bindings[0].slots, plan1.bindings[0].slots);

        let no_alias = [decl("velocity2", ShaderParamType::Float)];
        let plan3 = build_with_reuse(&no_alias, &plan1).unwrap();
        // Exact-ID matching wins first, alias second, neither applies here:
        // the old slot 0 stays orphaned and the new parameter takes the next
        // free index.
        assert_eq!(plan3.bindings[0].slots, vec![SlotRef { kind: PoolKind::Float, index: 0 }]);
        // (slot 0 is free again because `speed` no longer exists — the freed
        // slot is reused by ascending order; keyframes on it belonged to a
        // parameter that is gone either way.)
    }

    /// A kind change is a different parameter: no inheritance, new slot.
    #[test]
    fn kind_change_reallocates() {
        let v1 = [decl("x", ShaderParamType::Float), decl("y", ShaderParamType::Float)];
        let plan1 = build_fresh(&v1).unwrap();
        let v2 = [decl("x", ShaderParamType::Int), decl("y", ShaderParamType::Float)];
        let plan2 = build_with_reuse(&v2, &plan1).unwrap();
        assert_eq!(plan2.bindings[0].slots, vec![SlotRef { kind: PoolKind::Integer, index: 0 }]);
        // y keeps its Float slot 1 untouched.
        assert_eq!(plan2.bindings[1].slots, vec![SlotRef { kind: PoolKind::Float, index: 1 }]);
    }

    /// Inherited slots leave holes; new declarations fill around them with
    /// explicit tables, never contiguity assumptions (ADR-0013 §5).
    #[test]
    fn new_declarations_fill_around_inherited_holes() {
        let v1 = [
            decl("a", ShaderParamType::Float),
            decl("b", ShaderParamType::Float),
            decl("c", ShaderParamType::Float),
        ];
        let plan1 = build_fresh(&v1).unwrap();
        // Keep only c (slot 2), add two new floats: they take 0 and 1.
        let v2 = [
            decl("c", ShaderParamType::Float),
            decl("d", ShaderParamType::Float),
            decl("e", ShaderParamType::Float),
        ];
        let plan2 = build_with_reuse(&v2, &plan1).unwrap();
        assert_eq!(plan2.bindings[0].slots, vec![SlotRef { kind: PoolKind::Float, index: 2 }]);
        assert_eq!(plan2.bindings[1].slots, vec![SlotRef { kind: PoolKind::Float, index: 0 }]);
        assert_eq!(plan2.bindings[2].slots, vec![SlotRef { kind: PoolKind::Float, index: 1 }]);
    }

    /// vec4 inherits both paired slots together or not at all.
    #[test]
    fn vec4_pairing_inherits_atomically() {
        let v1 = [decl("tint", ShaderParamType::Vec4Color)];
        let plan1 = build_fresh(&v1).unwrap();
        let v2 = [decl_aliased("overlay", ShaderParamType::Vec4Color, "tint")];
        let plan2 = build_with_reuse(&v2, &plan1).unwrap();
        assert_eq!(plan2.bindings[0].slots, plan1.bindings[0].slots);

        // Changing to vec3 drops the pairing: fresh Color slot, alias or not.
        let v3 = [decl_aliased("overlay", ShaderParamType::Vec3Color, "tint")];
        let plan3 = build_with_reuse(&v3, &plan1).unwrap();
        assert_eq!(plan3.bindings[0].slots, vec![SlotRef { kind: PoolKind::Color, index: 0 }]);
    }

    /// Capacity is validated over inherited + fresh occupancy atomically.
    #[test]
    fn reuse_overflow_rejects_atomically() {
        let v1: Vec<_> =
            (0..48).map(|i| decl(&format!("f{i}"), ShaderParamType::Float)).collect();
        let plan1 = build_fresh(&v1).unwrap();
        // Keep all 48 by ID and add one more float.
        let mut v2 = v1.clone();
        v2.push(decl("extra", ShaderParamType::Float));
        assert_eq!(
            build_with_reuse(&v2, &plan1),
            Err(BindingError::PoolOverflow {
                kind: PoolKind::Float,
                capacity: 48,
                required: 49
            })
        );
    }
}
