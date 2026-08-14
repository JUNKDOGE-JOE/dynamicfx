//! The stable diagnostic code registry (ADR-0015 §4).
//!
//! Codes are u16, permanent and append-only from first release. Families are
//! pre-partitioned; a number is never reused or renumbered. Status text
//! renders as `E<code> <text>` — the 31-char PF name limit truncates text,
//! never the code.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Diag {
    Ok = 0,
    // 1-15: source/envelope
    SourceOversize = 1,
    EnvelopeMalformed = 2,
    EnvelopeUnsupported = 3,
    NotSourceBlock = 4,
    NoExpression = 5,
    /// Appended by ADR-0018: any envelope grammar/graph-rule violation,
    /// always reported with its 1-based source line.
    EnvelopeSyntax = 6,
    // 16-31: frontend
    LanguageUnknown = 16,
    GlslParse = 17,
    AbiViolation = 18,
    ParamRejected = 19,
    SpirvEmit = 20,
    // 32-47: binding
    PoolOverflow = 32,
    AliasConflict = 33,
    // 48-63: runtime/transport
    GpuUnavailable = 48,
    RegistryMiss = 49,
    SnapshotCorrupt = 50,
    SnapshotSchemaUnknown = 51,
    TokenCorrupt = 52,
}

/// The registry rows, in ascending code order. Append-only forever.
pub const REGISTRY: &[Diag] = &[
    Diag::Ok,
    Diag::SourceOversize,
    Diag::EnvelopeMalformed,
    Diag::EnvelopeUnsupported,
    Diag::NotSourceBlock,
    Diag::NoExpression,
    Diag::EnvelopeSyntax,
    Diag::LanguageUnknown,
    Diag::GlslParse,
    Diag::AbiViolation,
    Diag::ParamRejected,
    Diag::SpirvEmit,
    Diag::PoolOverflow,
    Diag::AliasConflict,
    Diag::GpuUnavailable,
    Diag::RegistryMiss,
    Diag::SnapshotCorrupt,
    Diag::SnapshotSchemaUnknown,
    Diag::TokenCorrupt,
];

impl Diag {
    pub fn code(self) -> u16 {
        self as u16
    }

    pub fn from_code(code: u16) -> Option<Self> {
        REGISTRY.iter().copied().find(|d| d.code() == code)
    }
}

/// Status line text: `E<code> <detail>` for failures, the plain detail for Ok.
pub fn status_text(diag: Diag, detail: &str) -> String {
    if diag == Diag::Ok {
        detail.to_string()
    } else {
        format!("E{} {detail}", diag.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Append-only registry guard: strictly ascending, unique, family-
    /// partitioned. A failure here means history was edited, not appended.
    #[test]
    fn registry_is_append_only_shaped() {
        let mut previous: Option<u16> = None;
        for diag in REGISTRY {
            let code = diag.code();
            if let Some(prev) = previous {
                assert!(code > prev, "codes must be strictly ascending");
            }
            previous = Some(code);
            let family_ok = match code {
                0 => true,
                1..=15 => matches!(
                    diag,
                    Diag::SourceOversize
                        | Diag::EnvelopeMalformed
                        | Diag::EnvelopeUnsupported
                        | Diag::NotSourceBlock
                        | Diag::NoExpression
                        | Diag::EnvelopeSyntax
                ),
                16..=31 => matches!(
                    diag,
                    Diag::LanguageUnknown
                        | Diag::GlslParse
                        | Diag::AbiViolation
                        | Diag::ParamRejected
                        | Diag::SpirvEmit
                ),
                32..=47 => matches!(diag, Diag::PoolOverflow | Diag::AliasConflict),
                48..=63 => matches!(
                    diag,
                    Diag::GpuUnavailable
                        | Diag::RegistryMiss
                        | Diag::SnapshotCorrupt
                        | Diag::SnapshotSchemaUnknown
                        | Diag::TokenCorrupt
                ),
                _ => false,
            };
            assert!(family_ok, "{diag:?} ({code}) sits outside its family partition");
        }
    }

    #[test]
    fn codes_round_trip() {
        for diag in REGISTRY {
            assert_eq!(Diag::from_code(diag.code()), Some(*diag));
        }
        assert_eq!(Diag::from_code(9999), None);
    }

    #[test]
    fn status_text_formats() {
        assert_eq!(status_text(Diag::Ok, "compiled: 1 pass"), "compiled: 1 pass");
        assert_eq!(status_text(Diag::GlslParse, "bad token"), "E17 bad token");
    }
}
