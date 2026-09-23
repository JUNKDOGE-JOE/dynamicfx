use crate::{binding::PoolKind, persistence::{self, DecodeError, EncodeError, Snapshot}};
use std::cell::Cell;
use std::sync::{atomic::{AtomicU64, Ordering}, OnceLock};

pub const MAX_CERTIFICATE: u64 = (1 << 48) - 1;
const PROVISIONAL: u64 = 1 << 63;

pub fn valid_certificate(value: u64) -> bool { (4..=MAX_CERTIFICATE).contains(&value) }

#[derive(Default)]
pub struct Permit(AtomicU64);

impl Permit {
    pub fn restored(certificate: u64) -> Self {
        Self(AtomicU64::new(if valid_certificate(certificate) { certificate | PROVISIONAL } else { 0 }))
    }
    pub fn certificate(&self) -> u64 {
        let value = self.0.load(Ordering::Acquire);
        if valid_certificate(value) { value } else { 0 }
    }
    pub fn revoke(&self) { self.0.store(0, Ordering::Release); }
    pub fn resetup(&self, render_only: bool) {
        let value = self.0.load(Ordering::Acquire);
        self.0.store(if render_only && value & PROVISIONAL != 0 { value & !PROVISIONAL } else { 0 }, Ordering::Release);
    }
    pub fn grant(&self) -> u64 {
        let current = self.certificate();
        if current != 0 { return current; }
        static NEXT: OnceLock<AtomicU64> = OnceLock::new();
        let counter = NEXT.get_or_init(|| {
            let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                .map(|time| time.as_nanos() as u64).unwrap_or(0);
            AtomicU64::new(time ^ (u64::from(std::process::id()) << 17))
        });
        loop {
            let certificate = counter.fetch_add(1, Ordering::Relaxed) & MAX_CERTIFICATE;
            if valid_certificate(certificate) {
                self.0.store(certificate, Ordering::Release);
                return certificate;
            }
        }
    }
}

thread_local! { static PUBLISHING: Cell<bool> = const { Cell::new(false) }; }
pub struct Publication(bool);
impl Publication {
    pub fn enter() -> Self { Self(PUBLISHING.with(|value| value.replace(true))) }
    pub fn active() -> bool { PUBLISHING.with(Cell::get) }
}
impl Drop for Publication {
    fn drop(&mut self) { PUBLISHING.with(|value| value.set(self.0)); }
}

fn uses_coverage(snapshot: &Snapshot) -> bool {
    snapshot.map.iter().any(|(_, slots)| slots.iter().any(|slot| slot.kind == PoolKind::Coverage))
}

pub fn encode(snapshot: &Snapshot, certificate: u64) -> Result<(u16, Vec<u8>), EncodeError> {
    let bytes = persistence::encode(snapshot)?;
    if !uses_coverage(snapshot) { return Ok((1, bytes)); }
    if certificate != 0 && !valid_certificate(certificate) { return Err(EncodeError::FieldLimit("coverage certificate")); }
    let total = bytes.len() + 16;
    if total > crate::frontend::envelope::MAX_SNAPSHOT_BYTES { return Err(EncodeError::BudgetExceeded { bytes: total }); }
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(b"DFXC");
    out.extend_from_slice(&certificate.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&bytes);
    let crc = persistence::crc32(&out);
    out[12..16].copy_from_slice(&crc.to_le_bytes());
    Ok((2, out))
}

pub fn decode(version: u16, bytes: &[u8]) -> Result<(Snapshot, u64), DecodeError> {
    match version {
        1 => persistence::decode(bytes).map(|snapshot| (snapshot, 0)),
        2 => {
            if bytes.len() < 16 || &bytes[..4] != b"DFXC" { return Err(DecodeError::Corrupt("coverage transport header")); }
            if bytes.len() > crate::frontend::envelope::MAX_SNAPSHOT_BYTES { return Err(DecodeError::Corrupt("coverage transport budget")); }
            let certificate = u64::from_le_bytes(bytes[4..12].try_into().unwrap());
            let crc = u32::from_le_bytes(bytes[12..16].try_into().unwrap());
            let mut check = bytes.to_vec();
            check[12..16].fill(0);
            if persistence::crc32(&check) != crc { return Err(DecodeError::Corrupt("coverage transport crc")); }
            if certificate != 0 && !valid_certificate(certificate) { return Err(DecodeError::Corrupt("coverage certificate")); }
            let snapshot = persistence::decode(&bytes[16..])?;
            if !uses_coverage(&snapshot) { return Err(DecodeError::Corrupt("coverage transport without coverage")); }
            Ok((snapshot, certificate))
        }
        _ => Err(DecodeError::SchemaUnknown { schema: version, flags: 0 }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn snapshot(coverage: bool) -> Snapshot {
        Snapshot { language: crate::frontend::LanguageId::GLSL, fingerprint: 123, source: "test source".into(),
            map: vec![("input_map".into(), vec![crate::binding::SlotRef { kind: if coverage { PoolKind::Coverage } else { PoolKind::Layer }, index: 0 }])] }
    }
    #[test]
    fn restored_certificates_need_render_only_resetup_and_ui_copies_revoke() {
        let source = Permit::default();let certificate = source.grant();
        let ui = Permit::restored(certificate);let render = Permit::restored(certificate);
        assert_eq!(ui.certificate(), 0);assert_eq!(render.certificate(), 0);
        ui.resetup(false);render.resetup(true);
        assert_eq!(ui.certificate(), 0);assert_eq!(render.certificate(), certificate);
        assert_ne!(ui.grant(), certificate);
        render.revoke();assert_eq!(render.certificate(), 0);
    }
    #[test]
    fn transport_keeps_legacy_bytes_and_binding_maps() {
        let legacy = snapshot(false);
        assert_eq!(encode(&legacy, 0).unwrap(), (1, persistence::encode(&legacy).unwrap()));
        let coverage = snapshot(true);
        let certificate = Permit::default().grant();
        let (version, bytes) = encode(&coverage, certificate).unwrap();
        assert_eq!(version, 2);assert_eq!(decode(version, &bytes).unwrap(), (coverage.clone(), certificate));
        assert_eq!(decode(1, &persistence::encode(&coverage).unwrap()).unwrap(), (coverage, 0));
    }
    #[test]
    fn transport_corruption_and_unknown_versions_cannot_grant_permission() {
        let (_, bytes) = encode(&snapshot(true), 42).unwrap();
        for index in 0..bytes.len() {
            let mut corrupt = bytes.clone();corrupt[index] ^= 1;
            assert!(decode(2, &corrupt).is_err(), "offset {index}");
        }
        assert!(matches!(decode(3, &bytes), Err(DecodeError::SchemaUnknown { .. })));
        assert!(encode(&snapshot(true), PROVISIONAL | 42).is_err());
        let (_, pending) = encode(&snapshot(true), 0).unwrap();
        assert_eq!(decode(2, &pending).unwrap().1, 0);
    }
    #[test]
    fn publication_scope_is_nested_and_restored_on_error() {
        assert!(!Publication::active());
        { let _outer = Publication::enter();{let _inner = Publication::enter();assert!(Publication::active());}assert!(Publication::active()); }
        assert!(!Publication::active());
    }
}
