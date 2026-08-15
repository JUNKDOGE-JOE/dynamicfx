//! Sequence schema v1 (ADR-0016): the only persistent definition carrier.
//! `DFXS` magic, schema u16, flags u16, body_len u32, CRC-32/ISO-HDLC over
//! the whole envelope (crc field zeroed), then the body: LanguageId,
//! fingerprint, exact source, ParamId→slot map. Little-endian throughout.

use crate::binding::{BindingPlan, ParamBinding, PoolKind, SlotRef};
use crate::definition::param::ParamId;
use crate::frontend::envelope::{MAX_COMMITTED_SOURCE_BYTES, MAX_SNAPSHOT_BYTES};
use crate::frontend::LanguageId;

const MAGIC: &[u8; 4] = b"DFXS";
const SCHEMA_V1: u16 = 1;
const HEADER_LEN: usize = 4 + 2 + 2 + 4 + 4;
const MAX_MAP_ENTRIES: usize = 256;

/// One decoded snapshot. `map` is the saved BindingPlan; on restore it seeds
/// `build_with_reuse` so keyframed streams stay aligned (ADR-0016 §4).
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub language: LanguageId,
    pub fingerprint: u64,
    pub source: String,
    pub map: Vec<(String, Vec<SlotRef>)>,
}

impl Snapshot {
    pub fn from_state(
        language: LanguageId,
        fingerprint: u64,
        source: &str,
        plan: &BindingPlan,
    ) -> Self {
        Self {
            language,
            fingerprint,
            source: source.to_owned(),
            map: plan
                .bindings
                .iter()
                .map(|b| (b.id.as_str().to_owned(), b.slots.clone()))
                .collect(),
        }
    }

    /// The restore seed: every saved binding counts as inherited so defaults
    /// never overwrite restored streams (ADR-0013 discipline).
    pub fn to_previous_plan(&self) -> BindingPlan {
        BindingPlan {
            bindings: self
                .map
                .iter()
                .filter_map(|(id, slots)| {
                    Some(ParamBinding {
                        id: ParamId::new(id).ok()?,
                        slots: slots.clone(),
                        inherited: true,
                    })
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// Construction-bug guard: the assembled snapshot violated the ADR-0012
    /// budget or a field limit. Nothing is written.
    BudgetExceeded { bytes: usize },
    FieldLimit(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Diagnostic class SnapshotCorrupt: bad magic/length/CRC/field.
    Corrupt(&'static str),
    /// Diagnostic class SnapshotSchemaUnknown: schema or flags from a newer
    /// build. Rendering fails closed; re-binding needs an explicit Compile.
    SchemaUnknown { schema: u16, flags: u16 },
}

fn kind_byte(kind: PoolKind) -> u8 {
    match kind {
        PoolKind::Float => 0,
        PoolKind::Integer => 1,
        PoolKind::Bool => 2,
        PoolKind::Color => 3,
        PoolKind::Point2D => 4,
        PoolKind::Angle => 5,
        // ADR-0030. Codes are persistent and append-only, like the pools.
        PoolKind::Layer => 6,
        PoolKind::Gradient => 7,
        // ADR-0034.
        PoolKind::Point3D => 8,
        // ADR-0035.
        PoolKind::Path => 9,
    }
}

fn kind_from_byte(byte: u8) -> Option<PoolKind> {
    Some(match byte {
        0 => PoolKind::Float,
        1 => PoolKind::Integer,
        2 => PoolKind::Bool,
        3 => PoolKind::Color,
        4 => PoolKind::Point2D,
        5 => PoolKind::Angle,
        6 => PoolKind::Layer,
        7 => PoolKind::Gradient,
        8 => PoolKind::Point3D,
        9 => PoolKind::Path,
        _ => return None,
    })
}

/// CRC-32/ISO-HDLC (the ubiquitous zlib polynomial), table-driven.
fn crc32(data: &[u8]) -> u32 {
    static TABLE: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
    let table = TABLE.get_or_init(|| {
        let mut table = [0u32; 256];
        for (i, entry) in table.iter_mut().enumerate() {
            let mut c = i as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            }
            *entry = c;
        }
        table
    });
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc = table[((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

pub fn encode(snapshot: &Snapshot) -> Result<Vec<u8>, EncodeError> {
    if snapshot.source.len() > MAX_COMMITTED_SOURCE_BYTES {
        return Err(EncodeError::FieldLimit("source exceeds the 4 MiB cap"));
    }
    if snapshot.map.len() > MAX_MAP_ENTRIES {
        return Err(EncodeError::FieldLimit("map exceeds 256 entries"));
    }

    let mut body = Vec::with_capacity(snapshot.source.len() + 64);
    body.extend_from_slice(&snapshot.language.0.to_le_bytes());
    body.extend_from_slice(&snapshot.fingerprint.to_le_bytes());
    body.extend_from_slice(&(snapshot.source.len() as u32).to_le_bytes());
    body.extend_from_slice(snapshot.source.as_bytes());
    body.extend_from_slice(&(snapshot.map.len() as u16).to_le_bytes());
    for (id, slots) in &snapshot.map {
        if id.is_empty() || id.len() > 64 {
            return Err(EncodeError::FieldLimit("ParamId length"));
        }
        if slots.is_empty() || slots.len() > 2 {
            return Err(EncodeError::FieldLimit("slot count"));
        }
        body.push(id.len() as u8);
        body.extend_from_slice(id.as_bytes());
        body.push(slots.len() as u8);
        for slot in slots {
            body.push(kind_byte(slot.kind));
            let index = u16::try_from(slot.index)
                .map_err(|_| EncodeError::FieldLimit("slot index"))?;
            body.extend_from_slice(&index.to_le_bytes());
        }
    }

    let total = HEADER_LEN + body.len();
    if total > MAX_SNAPSHOT_BYTES {
        return Err(EncodeError::BudgetExceeded { bytes: total });
    }

    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&SCHEMA_V1.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // flags
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // crc placeholder
    out.extend_from_slice(&body);
    let crc = crc32(&out);
    out[12..16].copy_from_slice(&crc.to_le_bytes());
    Ok(out)
}

pub fn decode(data: &[u8]) -> Result<Snapshot, DecodeError> {
    if data.len() < HEADER_LEN {
        return Err(DecodeError::Corrupt("short header"));
    }
    if &data[0..4] != MAGIC {
        return Err(DecodeError::Corrupt("bad magic"));
    }
    let schema = u16::from_le_bytes(data[4..6].try_into().unwrap());
    let flags = u16::from_le_bytes(data[6..8].try_into().unwrap());
    let body_len = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
    let stored_crc = u32::from_le_bytes(data[12..16].try_into().unwrap());
    if data.len() != HEADER_LEN + body_len {
        return Err(DecodeError::Corrupt("length mismatch"));
    }
    let mut check = data.to_vec();
    check[12..16].copy_from_slice(&0u32.to_le_bytes());
    if crc32(&check) != stored_crc {
        return Err(DecodeError::Corrupt("crc mismatch"));
    }
    // CRC verified first: a valid-checksum envelope from a NEWER build is a
    // schema case, not corruption.
    if schema != SCHEMA_V1 || flags != 0 {
        return Err(DecodeError::SchemaUnknown { schema, flags });
    }

    let body = &data[HEADER_LEN..];
    let mut pos = 0usize;
    let take = |pos: &mut usize, n: usize| -> Result<&[u8], DecodeError> {
        let slice = body
            .get(*pos..*pos + n)
            .ok_or(DecodeError::Corrupt("truncated body"))?;
        *pos += n;
        Ok(slice)
    };

    let language = LanguageId(u32::from_le_bytes(take(&mut pos, 4)?.try_into().unwrap()));
    let fingerprint = u64::from_le_bytes(take(&mut pos, 8)?.try_into().unwrap());
    let source_len = u32::from_le_bytes(take(&mut pos, 4)?.try_into().unwrap()) as usize;
    if source_len > MAX_COMMITTED_SOURCE_BYTES {
        return Err(DecodeError::Corrupt("source length"));
    }
    let source = std::str::from_utf8(take(&mut pos, source_len)?)
        .map_err(|_| DecodeError::Corrupt("source utf-8"))?
        .to_owned();
    let map_count = u16::from_le_bytes(take(&mut pos, 2)?.try_into().unwrap()) as usize;
    if map_count > MAX_MAP_ENTRIES {
        return Err(DecodeError::Corrupt("map count"));
    }
    let mut map = Vec::with_capacity(map_count);
    for _ in 0..map_count {
        let id_len = take(&mut pos, 1)?[0] as usize;
        if id_len == 0 || id_len > 64 {
            return Err(DecodeError::Corrupt("id length"));
        }
        let id = std::str::from_utf8(take(&mut pos, id_len)?)
            .map_err(|_| DecodeError::Corrupt("id utf-8"))?
            .to_owned();
        let slot_count = take(&mut pos, 1)?[0] as usize;
        if slot_count == 0 || slot_count > 2 {
            return Err(DecodeError::Corrupt("slot count"));
        }
        let mut slots = Vec::with_capacity(slot_count);
        for _ in 0..slot_count {
            let kind =
                kind_from_byte(take(&mut pos, 1)?[0]).ok_or(DecodeError::Corrupt("slot kind"))?;
            let index = u16::from_le_bytes(take(&mut pos, 2)?.try_into().unwrap()) as usize;
            slots.push(SlotRef { kind, index });
        }
        map.push((id, slots));
    }
    if pos != body.len() {
        return Err(DecodeError::Corrupt("trailing bytes"));
    }
    Ok(Snapshot { language, fingerprint, source, map })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Snapshot {
        Snapshot {
            language: LanguageId::GLSL,
            fingerprint: 0x0004_2eec_2303_41ec,
            source: "void main() {}".to_string(),
            map: vec![
                (
                    "gain".to_string(),
                    vec![SlotRef { kind: PoolKind::Float, index: 2 }],
                ),
                (
                    "tint".to_string(),
                    vec![
                        SlotRef { kind: PoolKind::Color, index: 0 },
                        SlotRef { kind: PoolKind::Float, index: 7 },
                    ],
                ),
            ],
        }
    }

    #[test]
    fn round_trips_with_non_contiguous_slots() {
        let snapshot = sample();
        let bytes = encode(&snapshot).unwrap();
        assert_eq!(decode(&bytes).unwrap(), snapshot);
    }

    /// Golden bytes for the header of the sample snapshot: moving these is a
    /// format decision (superseding ADR or recorded pre-release amendment).
    #[test]
    fn golden_header_prefix() {
        let bytes = encode(&sample()).unwrap();
        assert_eq!(&bytes[0..4], b"DFXS");
        assert_eq!(u16::from_le_bytes(bytes[4..6].try_into().unwrap()), 1);
        assert_eq!(u16::from_le_bytes(bytes[6..8].try_into().unwrap()), 0);
        let body_len = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        assert_eq!(bytes.len(), HEADER_LEN + body_len);
    }

    #[test]
    fn every_flipped_byte_is_rejected() {
        let bytes = encode(&sample()).unwrap();
        for i in 0..bytes.len() {
            let mut bad = bytes.clone();
            bad[i] ^= 0x40;
            assert!(
                decode(&bad).is_err(),
                "byte {i} flip must not decode cleanly"
            );
        }
    }

    #[test]
    fn newer_schema_and_flags_are_schema_unknown_not_corrupt() {
        let mut bytes = encode(&sample()).unwrap();
        bytes[4..6].copy_from_slice(&2u16.to_le_bytes());
        // Re-seal the CRC so only the schema differs.
        bytes[12..16].copy_from_slice(&0u32.to_le_bytes());
        let crc = crc32(&bytes);
        bytes[12..16].copy_from_slice(&crc.to_le_bytes());
        assert_eq!(
            decode(&bytes),
            Err(DecodeError::SchemaUnknown { schema: 2, flags: 0 })
        );
    }

    #[test]
    fn truncation_is_corrupt() {
        let bytes = encode(&sample()).unwrap();
        assert!(matches!(
            decode(&bytes[..bytes.len() - 1]),
            Err(DecodeError::Corrupt(_))
        ));
        assert!(matches!(decode(&[]), Err(DecodeError::Corrupt(_))));
    }

    #[test]
    fn budget_and_field_limits_refuse_encoding() {
        let mut oversized = sample();
        oversized.source = "a".repeat(MAX_COMMITTED_SOURCE_BYTES + 1);
        assert!(matches!(encode(&oversized), Err(EncodeError::FieldLimit(_))));

        let mut too_many = sample();
        too_many.map = (0..257)
            .map(|i| {
                (format!("p{i}"), vec![SlotRef { kind: PoolKind::Float, index: 0 }])
            })
            .collect();
        assert!(matches!(encode(&too_many), Err(EncodeError::FieldLimit(_))));
    }

    #[test]
    fn restore_seed_marks_everything_inherited() {
        let plan = sample().to_previous_plan();
        assert_eq!(plan.bindings.len(), 2);
        assert!(plan.bindings.iter().all(|b| b.inherited));
        assert_eq!(plan.bindings[1].slots.len(), 2);
    }
}
