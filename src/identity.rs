//! Identity hashing (ADR-0017): BLAKE3-256 with length-prefixed ASCII domain
//! tags and canonical little-endian serialization.
//!
//! M3 consumes the `dfx:token:v1` domain (StateToken payload and snapshot
//! fingerprint). The module/artifact/graph/definition domains land with
//! their cache-layer consumers (M4) on the same `Canonical` builder; the
//! encoding rules are pinned by the golden tests below either way.

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
