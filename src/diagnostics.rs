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
    /// Appended by ADR-0030 §6: a `hint:layer` input in a graph that also
    /// reads `prev`. Windowed re-simulation would have to check the
    /// referenced layer out once per iterated frame; rather than silently
    /// reuse the requested frame's pixels, the combination fails closed.
    LayerInTemporalGraph = 7,
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
    /// Appended within the runtime/transport family (ADR-0015 §4 pre-
    /// partitions 48-63 for exactly this): a well-formed committed source
    /// exists, but no definition has been published for it, so render clones
    /// cannot resolve one. Without this code the state is byte-identical to a
    /// never-authored instance from the render side — token 0, no snapshot,
    /// and a Source slider that reads 0 whether or not the `…`;0 expression
    /// is there — so pass-through was indistinguishable from "nothing here".
    PublicationPending = 53,
    /// Appended by ADR-0031 §3: a stored gradient value that is empty,
    /// unsorted, out of range, or over the 8-stop cap. Repairing it silently
    /// would hide corruption and make the format's guarantees untestable, so
    /// the resource binds transparent black and says why.
    GradientMalformed = 54,
    /// One effect may expose only one canvas authority. This is checked after
    /// cross-pass parameter merging so repeated uses of one ParamId stay legal.
    /// ADR-0039 §1.
    CanvasDuplicate = 55,
    /// A canvas authority must retain the ordinary scalar Float pool kind;
    /// other parameter kinds fail closed under their own stable code.
    /// ADR-0039 §1.
    CanvasWrongKind = 56,
    /// A canvas dimension would exceed the device texture limit; the canvas
    /// fell back to the layer frame and the frame rendered under the released
    /// contract. Degradation, never a crash. ADR-0039 §6.
    CanvasTooLarge = 57,
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
    Diag::LayerInTemporalGraph,
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
    Diag::PublicationPending,
    Diag::GradientMalformed,
    Diag::CanvasDuplicate,
    Diag::CanvasWrongKind,
    Diag::CanvasTooLarge,
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
                        | Diag::LayerInTemporalGraph
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
                        | Diag::PublicationPending
                        | Diag::GradientMalformed
                        | Diag::CanvasDuplicate
                        | Diag::CanvasWrongKind
                        | Diag::CanvasTooLarge
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
