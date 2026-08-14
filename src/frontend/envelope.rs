//! Committed-source classification and size limits (ADR-0012).
//!
//! The runtime never modifies the committed text. Classification decides only
//! whether the text is raw single-pass source for the selected frontend or a
//! versioned envelope; until the M4 grammar ADR is Accepted every envelope
//! fails closed at the compile layer, so this module parses no further than
//! the marker line.

/// Hard cap on the committed source (raw or envelope), in UTF-8 bytes.
/// Checked before any parsing (ADR-0012 §5).
pub const MAX_COMMITTED_SOURCE_BYTES: usize = 4 * 1024 * 1024;

/// Budget for the serialized persisted definition snapshot, including header
/// and checksum overhead. The M3 sequence-schema codec is designed inside
/// this number (ADR-0012 §6).
pub const MAX_SNAPSHOT_BYTES: usize = 8 * 1024 * 1024;

/// The only token ADR-0012 reserves. Case-sensitive.
pub const ENVELOPE_PREFIX: &str = "@dynamicfx";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceClass {
    /// Hand the whole committed text, unmodified, to the selected frontend.
    Raw,
    /// Envelope input with a well-formed marker line. Version `1` is reserved
    /// for the M4 grammar; before that every version is rejected by the
    /// compile layer with the envelope-unsupported diagnostic class.
    Envelope { version: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceClassError {
    /// Source exceeds `MAX_COMMITTED_SOURCE_BYTES`. Diagnostic class:
    /// source-oversize.
    Oversize { bytes: usize },
    /// The reserved prefix matched but the marker line is not
    /// `@dynamicfx <version>`. Never falls back to Raw (ADR-0012 §3).
    /// Diagnostic class: envelope-malformed.
    EnvelopeMalformed,
}

/// Classify a committed source per ADR-0012 §2-§5: size gate, then skip an
/// optional UTF-8 BOM and ASCII whitespace, then match the reserved prefix.
pub fn classify(source: &str) -> Result<SourceClass, SourceClassError> {
    if source.len() > MAX_COMMITTED_SOURCE_BYTES {
        return Err(SourceClassError::Oversize { bytes: source.len() });
    }

    let body = source.strip_prefix('\u{feff}').unwrap_or(source);
    let body = body.trim_start_matches([' ', '\t', '\r', '\n']);

    let Some(after_prefix) = body.strip_prefix(ENVELOPE_PREFIX) else {
        return Ok(SourceClass::Raw);
    };
    // The prefix counts only when followed by whitespace or end-of-input;
    // `@dynamicfxx` is ordinary (raw) text.
    match after_prefix.bytes().next() {
        None => return Err(SourceClassError::EnvelopeMalformed),
        Some(b' ') | Some(b'\t') | Some(b'\r') | Some(b'\n') => {}
        Some(_) => return Ok(SourceClass::Raw),
    }

    let marker_line = after_prefix
        .split('\n')
        .next()
        .unwrap_or(after_prefix)
        .trim_end_matches('\r');
    parse_version(marker_line)
        .map(|version| SourceClass::Envelope { version })
        .ok_or(SourceClassError::EnvelopeMalformed)
}

/// Parse ` <version>` from the marker line remainder: a decimal u32 ≥ 1
/// without leading zeros, surrounded only by whitespace (ADR-0012 §4).
fn parse_version(rest: &str) -> Option<u32> {
    let mut tokens = rest.split_ascii_whitespace();
    let digits = tokens.next()?;
    if tokens.next().is_some() {
        return None;
    }
    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if digits.starts_with('0') {
        return None; // rejects `0` and leading zeros alike
    }
    digits.parse::<u32>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_glsl_is_raw() {
        assert_eq!(classify("void main() {}"), Ok(SourceClass::Raw));
        assert_eq!(classify(""), Ok(SourceClass::Raw));
    }

    #[test]
    fn marker_after_bom_and_whitespace_is_envelope() {
        assert_eq!(
            classify("\u{feff}\r\n\t @dynamicfx 1\n@graph"),
            Ok(SourceClass::Envelope { version: 1 })
        );
    }

    #[test]
    fn crlf_marker_line_is_tolerated() {
        assert_eq!(
            classify("@dynamicfx 1\r\npass body"),
            Ok(SourceClass::Envelope { version: 1 })
        );
        assert_eq!(
            classify("@dynamicfx  7  \r\nrest"),
            Ok(SourceClass::Envelope { version: 7 })
        );
    }

    #[test]
    fn prefix_match_is_case_sensitive_and_token_exact() {
        assert_eq!(classify("@Dynamicfx 1"), Ok(SourceClass::Raw));
        assert_eq!(classify("@dynamicfxx 1"), Ok(SourceClass::Raw));
    }

    #[test]
    fn comment_before_marker_keeps_it_raw() {
        // Only BOM and whitespace are skipped; anything else (like a leading
        // comment) means the text is raw source for the frontend.
        assert_eq!(classify("// note\n@dynamicfx 1"), Ok(SourceClass::Raw));
    }

    #[test]
    fn matched_prefix_never_falls_back_to_raw() {
        for bad in [
            "@dynamicfx",
            "@dynamicfx\nbody",
            "@dynamicfx 0",
            "@dynamicfx 01",
            "@dynamicfx one",
            "@dynamicfx 1 extra",
            "@dynamicfx 4294967296",
            "@dynamicfx -1",
        ] {
            assert_eq!(
                classify(bad),
                Err(SourceClassError::EnvelopeMalformed),
                "input {bad:?} must fail closed, not compile as raw"
            );
        }
    }

    #[test]
    fn unknown_versions_classify_for_diagnostics() {
        // Higher versions are well-formed envelopes; rejecting them (with the
        // version in the diagnostic) is the compile layer's job.
        assert_eq!(
            classify("@dynamicfx 4294967295"),
            Ok(SourceClass::Envelope { version: u32::MAX })
        );
    }

    #[test]
    fn oversize_boundary_is_exact_and_checked_first() {
        let at_cap = "a".repeat(MAX_COMMITTED_SOURCE_BYTES);
        assert_eq!(classify(&at_cap), Ok(SourceClass::Raw));

        let over = "a".repeat(MAX_COMMITTED_SOURCE_BYTES + 1);
        assert_eq!(
            classify(&over),
            Err(SourceClassError::Oversize { bytes: MAX_COMMITTED_SOURCE_BYTES + 1 })
        );

        // Oversize wins over envelope detection: the gate runs before parsing.
        let mut oversize_envelope = String::from("@dynamicfx 1\n");
        oversize_envelope.push_str(&"b".repeat(MAX_COMMITTED_SOURCE_BYTES));
        assert!(matches!(
            classify(&oversize_envelope),
            Err(SourceClassError::Oversize { .. })
        ));
    }

    #[test]
    fn snapshot_budget_exceeds_source_cap() {
        // The snapshot must hold the exact source plus metadata, so the
        // budget is strictly larger than the source cap by contract.
        assert!(MAX_SNAPSHOT_BYTES > MAX_COMMITTED_SOURCE_BYTES);
    }
}
