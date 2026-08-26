//! v1 parameter pools and BindingPlan allocation (ADR-0013).
//!
//! Main-pool and per-pass-bank allocation (ADR-0013, ADR-0040). Plans carry
//! explicit kind-local slot indexes, so physical declaration order remains a
//! host concern and inherited holes remain safe.

use crate::definition::param::{
    validate_declarations, DeclarationError, ParamDeclaration, ParamId,
};

/// AE pool kinds declared in v1 (ADR-0013 §3). Popup is deliberately absent
/// (TR-M0-006: menus are immutable after PARAMS_SETUP).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoolKind {
    Float,
    Integer,
    Bool,
    Color,
    Point2D,
    Angle,
    /// ADR-0030: an AE Layer selector feeding a texture binding. Carries no
    /// uniform-block storage and no float budget.
    Layer,
    /// ADR-0031: an arbitrary-data gradient baked into a 1D LUT texture.
    /// Carries no uniform-block storage and no float budget.
    Gradient,
    /// ADR-0035: an AE mask selector feeding an `N x 2` vertex texture. Like
    /// `Layer` and `Gradient` it carries no uniform-block storage.
    Path,
    /// ADR-0034: a three-component *spatial* value — the AE 3D point widget.
    /// Distinct from `Color`, which is what an un-annotated `vec3` still maps
    /// to (ADR-0026); this kind is reached only through `hint:point3d`.
    Point3D,
}

/// The pool table released in 0.0.1: 104 stable Main-slot identities.
pub const V1_POOLS: &[(PoolKind, usize)] = &[
    (PoolKind::Float, 48),
    (PoolKind::Integer, 8),
    (PoolKind::Bool, 16),
    (PoolKind::Color, 12),
    (PoolKind::Point2D, 12),
    (PoolKind::Angle, 8),
];

/// Main-only pools added after the v1 table shipped (ADR-0030–ADR-0035).
/// Their separate table preserves each released `Pool(kind, index)` identity
/// while ADR-0040 is free to place those keys inside `Main`.
pub const GROWTH_POOLS: &[(PoolKind, usize)] = &[
    (PoolKind::Layer, 4),
    // ADR-0033 §2: two gradients, because each now costs 26 declared
    // parameters rather than one arbitrary row. The slot itself is the
    // preview/canvas; the stops are declared separately in `declaration_order`.
    (PoolKind::Gradient, 2),
    // ADR-0034 §1: closes the Point 3D kind ADR-0013 §3 left reserved. Eight,
    // matching the Integer and Angle pools — a spatial vec3 is a
    // several-per-shader parameter, not a dozens-per-shader one.
    (PoolKind::Point3D, 8),
    // ADR-0035 §1: two, because each bound path costs a checkout and a vertex
    // walk every frame, and a shader wanting more masks than that is better
    // served by a layer input.
    (PoolKind::Path, 2),
];

/// Fixed pass-bank topology. Main-only kinds deliberately have no row here.
pub const BANK_GROUPS: usize = 12;
pub const BANK_POOLS: &[(PoolKind, usize)] = &[
    (PoolKind::Float, 8),
    (PoolKind::Integer, 2),
    (PoolKind::Bool, 2),
    (PoolKind::Color, 3),
    (PoolKind::Point2D, 2),
    (PoolKind::Angle, 1),
];

/// Every pool that can be allocated, in allocation order. Physical
/// declaration order is `host::params::declaration_order`'s business, and it
/// is deliberately not the same sequence.
pub fn all_pools() -> impl Iterator<Item = (PoolKind, usize)> {
    V1_POOLS
        .iter()
        .chain(GROWTH_POOLS.iter())
        .map(|(kind, _)| (*kind, pool_capacity(*kind)))
}

pub fn main_pool_capacity(kind: PoolKind) -> usize {
    V1_POOLS
        .iter()
        .chain(GROWTH_POOLS.iter())
        .find(|(k, _)| *k == kind)
        .map(|(_, capacity)| *capacity)
        .unwrap_or(0)
}

pub fn bank_capacity(kind: PoolKind) -> usize {
    BANK_POOLS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, capacity)| *capacity)
        .unwrap_or(0)
}

pub fn pool_capacity(kind: PoolKind) -> usize {
    main_pool_capacity(kind) + BANK_GROUPS * bank_capacity(kind)
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

impl BindingPlan {
    pub(crate) fn mapping(&self) -> impl Iterator<Item = (&ParamId, &[SlotRef])> {
        self.bindings
            .iter()
            .map(|binding| (&binding.id, binding.slots.as_slice()))
    }
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
    build_fresh_counted(decls).map(|(plan, _)| plan)
}

pub(crate) fn build_fresh_counted(
    decls: &[ParamDeclaration],
) -> Result<(BindingPlan, usize), BindingError> {
    build_with_reuse_counted(decls, &BindingPlan { bindings: Vec::new() })
}

/// Allocation against a previous plan (ADR-0013 §2, architecture §11.1):
/// an exact current-ID match inherits its previous slots; on miss, an alias
/// matching a previous binding's ID inherits (single-generation, never a
/// chain). Inheritance requires the slot-kind requirements to be unchanged —
/// a kind change correctly reads as a different parameter and reallocates.
/// Unmatched declarations take ascending free slots in their assigned bank,
/// spilling to ascending Main holes when needed. Capacity is validated over
/// the complete plan before anything is returned (atomic rejection;
/// keyframed streams on inherited slots survive untouched).
pub fn build_with_reuse(
    decls: &[ParamDeclaration],
    previous: &BindingPlan,
) -> Result<BindingPlan, BindingError> {
    build_with_reuse_counted(decls, previous).map(|(plan, _)| plan)
}

pub(crate) fn build_with_reuse_counted(
    decls: &[ParamDeclaration],
    previous: &BindingPlan,
) -> Result<(BindingPlan, usize), BindingError> {
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

    // Pass 2: inheritance stays authoritative. Only fresh declarations consult
    // their lowering-assigned bank; a full bank spills the whole parameter to
    // Main so multi-slot values never split across panel groups (ADR-0040 §4).
    let mut bindings = Vec::with_capacity(decls.len());
    let mut bank_spills = 0;
    for (decl, inherited) in decls.iter().zip(inherited) {
        if let Some(slots) = inherited {
            bindings.push(ParamBinding { id: decl.id.clone(), slots, inherited: true });
            continue;
        }

        let requirements = decl.ty.slot_requirements();
        let bank = decl.bank.filter(|group| {
            *group < BANK_GROUPS && requirements.iter().all(|kind| bank_capacity(*kind) > 0)
        });
        let mut slots = bank.and_then(|group| {
            allocate_requirements(requirements, &taken, |kind| {
                let start = main_pool_capacity(kind) + group * bank_capacity(kind);
                start..start + bank_capacity(kind)
            })
        });
        if bank.is_some() && slots.is_none() {
            bank_spills += 1;
        }
        if slots.is_none() {
            slots = allocate_requirements(requirements, &taken, |kind| {
                0..main_pool_capacity(kind)
            });
        }
        let Some(slots) = slots else {
            let kind = requirements
                .iter()
                .copied()
                .find(|kind| {
                    (0..main_pool_capacity(*kind))
                        .all(|index| taken.contains(&SlotRef { kind: *kind, index }))
                })
                .unwrap_or(requirements[0]);
            return Err(BindingError::PoolOverflow {
                kind,
                capacity: main_pool_capacity(kind),
                required: required.get(&kind).copied().unwrap_or(0),
            });
        };
        taken.extend(slots.iter().copied());
        bindings.push(ParamBinding { id: decl.id.clone(), slots, inherited: false });
    }
    Ok((BindingPlan { bindings }, bank_spills))
}

fn allocate_requirements(
    requirements: &[PoolKind],
    taken: &std::collections::HashSet<SlotRef>,
    range_for: impl Fn(PoolKind) -> std::ops::Range<usize>,
) -> Option<Vec<SlotRef>> {
    let mut reserved = std::collections::HashSet::new();
    let mut slots = Vec::with_capacity(requirements.len());
    for kind in requirements {
        let slot = range_for(*kind)
            .map(|index| SlotRef { kind: *kind, index })
            .find(|slot| !taken.contains(slot) && !reserved.contains(slot))?;
        reserved.insert(slot);
        slots.push(slot);
    }
    Some(slots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::param::ShaderParamType;

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

    #[test]
    fn pool_table_matches_the_adr_totals() {
        let total: usize = V1_POOLS.iter().map(|(_, c)| c).sum();
        assert_eq!(total, 104, "ADR-0013 fixes 104 v1 slots");
        // Every kind appears exactly once: the table is the single source.
        let mut kinds: Vec<PoolKind> = V1_POOLS.iter().map(|(k, _)| *k).collect();
        kinds.dedup();
        assert_eq!(kinds.len(), V1_POOLS.len());
        assert_eq!(pool_capacity(PoolKind::Float), 48 + BANK_GROUPS * 8);
        assert_eq!(pool_capacity(PoolKind::Integer), 8 + BANK_GROUPS * 2);
        assert_eq!(pool_capacity(PoolKind::Bool), 16 + BANK_GROUPS * 2);
        assert_eq!(pool_capacity(PoolKind::Color), 12 + BANK_GROUPS * 3);
        assert_eq!(pool_capacity(PoolKind::Point2D), 12 + BANK_GROUPS * 2);
        assert_eq!(pool_capacity(PoolKind::Angle), 8 + BANK_GROUPS);
        for kind in [PoolKind::Layer, PoolKind::Gradient, PoolKind::Point3D, PoolKind::Path] {
            assert_eq!(pool_capacity(kind), main_pool_capacity(kind));
        }
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
    fn total_float_capacity_includes_every_bank() {
        let mut full: Vec<_> =
            (0..48).map(|i| decl(&format!("main{i}"), ShaderParamType::Float)).collect();
        for group in 0..BANK_GROUPS {
            for local in 0..bank_capacity(PoolKind::Float) {
                let mut item = decl(&format!("bank{group}_{local}"), ShaderParamType::Float);
                item.bank = Some(group);
                full.push(item);
            }
        }
        assert_eq!(full.len(), pool_capacity(PoolKind::Float));
        assert!(build_fresh(&full).is_ok());

        full.push(decl("overflow", ShaderParamType::Float));
        assert_eq!(
            build_fresh(&full),
            Err(BindingError::PoolOverflow {
                kind: PoolKind::Float,
                capacity: pool_capacity(PoolKind::Float),
                required: pool_capacity(PoolKind::Float) + 1,
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
            canvas: false,
            bank: None,
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
