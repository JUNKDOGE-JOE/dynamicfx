//! Identity hashing (ADR-0017): BLAKE3-256 with length-prefixed ASCII domain
//! tags and canonical little-endian serialization.
//!
//! M3 consumes the `dfx:token:v1` domain (StateToken payload and snapshot
//! fingerprint). The module/artifact/graph/definition domains land with
//! their cache-layer consumers (M4) on the same `Canonical` builder; the
//! encoding rules are pinned by the golden tests below either way.

use crate::binding::BindingPlan;
use crate::frontend::LanguageId;

/// Canonical hash-input builder: every field is written in declared order
/// with fixed-width little-endian integers and u32 length prefixes.
pub struct Canonical {
    hasher: blake3::Hasher,
}

impl Canonical {
    pub fn new(domain: &str) -> Self {
        let mut c = Self { hasher: blake3::Hasher::new() };
        c.bytes(domain.as_bytes());
        c
    }

    pub fn u32(&mut self, v: u32) -> &mut Self {
        self.hasher.update(&v.to_le_bytes());
        self
    }

    pub fn bytes(&mut self, v: &[u8]) -> &mut Self {
        self.hasher.update(&(v.len() as u32).to_le_bytes());
        self.hasher.update(v);
        self
    }

    pub fn finish(&self) -> [u8; 32] {
        *self.hasher.finalize().as_bytes()
    }
}

/// `dfx:token:v1`: the 51-bit fingerprint carried by the StateToken payload
/// and the snapshot (ADR-0015/0016). Truncated from BLAKE3-256 so the bits
/// inherit strong mixing; zero maps to 1 (zero means "none").
pub fn token_fingerprint(language: LanguageId, source: &str) -> u64 {
    let mut c = Canonical::new("dfx:token:v1");
    c.u32(language.0).bytes(source.as_bytes());
    let digest = c.finish();
    let word = u64::from_le_bytes(digest[..8].try_into().expect("8 bytes"));
    let truncated = word & ((1u64 << 51) - 1);
    if truncated == 0 { 1 } else { truncated }
}

/// `dfx:plan:v1` (ADR-0038 §1): the session-local identity of an ordered
/// ParamId → slot mapping, over `BindingPlan::mapping` so the registry's
/// equality check and this digest can never disagree on what the mapping
/// is. Slot kinds use the persistent snapshot codes, not enum order. The
/// compile-transient `inherited` flag is deliberately excluded. Truncated
/// to 51 bits with zero mapped to 1, exactly like the token fingerprint, so
/// the value travels unchanged through the plan-token stream (ADR-0038 §7)
/// as an exact f64 integer. Never persisted in the snapshot.
pub fn plan_identity(plan: &BindingPlan) -> u64 {
    let mut c = Canonical::new("dfx:plan:v1");
    for (id, slots) in plan.mapping() {
        c.bytes(id.as_str().as_bytes()).u32(slots.len() as u32);
        for slot in slots {
            c.u32(crate::persistence::kind_byte(slot.kind) as u32)
                .u32(u32::try_from(slot.index).expect("binding slot index fits u32"));
        }
    }
    let digest = c.finish();
    let word = u64::from_le_bytes(digest[..8].try_into().expect("8 bytes"));
    let truncated = word & ((1u64 << 51) - 1);
    if truncated == 0 { 1 } else { truncated }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::{ParamBinding, PoolKind, SlotRef};
    use crate::definition::param::ParamId;

    fn sample_plan() -> BindingPlan {
        BindingPlan {
            bindings: vec![
                ParamBinding {
                    id: ParamId::new("gain").unwrap(),
                    slots: vec![SlotRef { kind: PoolKind::Float, index: 2 }],
                    inherited: false,
                },
                ParamBinding {
                    id: ParamId::new("tint").unwrap(),
                    slots: vec![
                        SlotRef { kind: PoolKind::Color, index: 0 },
                        SlotRef { kind: PoolKind::Float, index: 7 },
                    ],
                    inherited: true,
                },
            ],
        }
    }

    /// Golden vector: this value may only change through a superseding ADR
    /// (or a recorded pre-release amendment). If this test goes red, a
    /// format decision is being made — stop and treat it as one.
    #[test]
    fn token_fingerprint_golden_vector() {
        let fp = token_fingerprint(LanguageId::GLSL, "void main() {}");
        assert_eq!(fp, 0x0007_a4ec_182d_6429, "golden fingerprint moved: {fp:#018x}");
        assert!(fp < (1u64 << 51));
        assert_ne!(fp, 0);
    }

    #[test]
    fn domains_separate_identical_payloads() {
        let mut a = Canonical::new("dfx:token:v1");
        a.u32(1).bytes(b"same");
        let mut b = Canonical::new("dfx:module:v1");
        b.u32(1).bytes(b"same");
        assert_ne!(a.finish(), b.finish());
    }

    #[test]
    fn length_prefixing_prevents_field_bleed() {
        // ("ab", "c") must not collide with ("a", "bc").
        let mut a = Canonical::new("dfx:token:v1");
        a.bytes(b"ab").bytes(b"c");
        let mut b = Canonical::new("dfx:token:v1");
        b.bytes(b"a").bytes(b"bc");
        assert_ne!(a.finish(), b.finish());
    }

    #[test]
    fn language_and_source_both_move_the_fingerprint() {
        let base = token_fingerprint(LanguageId::GLSL, "src");
        assert_ne!(base, token_fingerprint(LanguageId(2), "src"));
        assert_ne!(base, token_fingerprint(LanguageId::GLSL, "src "));
    }

    /// This identity is session-local and never persisted, but its encoding
    /// may still change only through an ADR amendment.
    #[test]
    fn plan_identity_golden_vector() {
        let id = plan_identity(&sample_plan());
        assert_eq!(
            id, 0x0002_1b8b_7c86_e167,
            "golden plan identity moved: {id:#018x}"
        );
        assert!(id < (1u64 << 51));
        assert_ne!(id, 0);
    }

    #[test]
    fn plan_identity_excludes_inherited_and_covers_the_mapping() {
        let base = sample_plan();
        let base_id = plan_identity(&base);

        let mut inherited = base.clone();
        for binding in &mut inherited.bindings {
            binding.inherited = !binding.inherited;
        }
        assert_eq!(plan_identity(&inherited), base_id);

        let mut changed_id = base.clone();
        changed_id.bindings[0].id = ParamId::new("level").unwrap();
        assert_ne!(plan_identity(&changed_id), base_id);

        let mut changed_slot = base.clone();
        changed_slot.bindings[0].slots[0].index += 1;
        assert_ne!(plan_identity(&changed_slot), base_id);

        let mut changed_kind = base.clone();
        changed_kind.bindings[0].slots[0].kind = PoolKind::Angle;
        assert_ne!(plan_identity(&changed_kind), base_id);

        let mut changed_order = base.clone();
        changed_order.bindings.swap(0, 1);
        assert_ne!(plan_identity(&changed_order), base_id);
    }
}
