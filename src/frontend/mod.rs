//! Language identity and the frontend registry (ADR-0010).
//!
//! `LanguageId` is a permanent, append-only u32 registry. The AE popup is a
//! derived UI surface: each build carries a fixed position→ID table, and the
//! persisted snapshot ID is the restore authority over the popup stream.

pub mod annotation;
pub mod envelope;
pub mod glsl;
pub mod grammar;

use crate::definition::param::ParamDeclaration;

/// Stable numeric language identity (ADR-0010). Wire-stable forever: values
/// are never reused, reordered, or repurposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LanguageId(pub u32);

impl LanguageId {
    /// Reserved: invalid/unknown. Never a selectable language.
    pub const INVALID: Self = Self(0);
    /// GLSL, the default language.
    pub const GLSL: Self = Self(1);
    /// WGSL, registered by ADR-0010 but not implemented in Phase 1.
    pub const WGSL: Self = Self(2);
}

struct LanguageEntry {
    id: LanguageId,
    /// Presentation only; never identity, persistence, or hashing input.
    display_name: &'static str,
    implemented: bool,
}

/// The permanent registry. Rows may only be appended, with ascending IDs.
const REGISTRY: &[LanguageEntry] = &[
    LanguageEntry { id: LanguageId::GLSL, display_name: "GLSL", implemented: true },
    LanguageEntry { id: LanguageId::WGSL, display_name: "WGSL", implemented: false },
];

pub fn default_language() -> LanguageId {
    LanguageId::GLSL
}

pub fn is_implemented(id: LanguageId) -> bool {
    REGISTRY.iter().any(|e| e.id == id && e.implemented)
}

/// Menu labels for the Language popup: implemented languages only, in
/// registry order (ADR-0010 §2). v1 is exactly `["GLSL"]`; across builds this
/// list may only append.
pub fn popup_menu() -> Vec<&'static str> {
    REGISTRY.iter().filter(|e| e.implemented).map(|e| e.display_name).collect()
}

/// Map a committed 1-based popup position to its `LanguageId`. Positions
/// beyond the menu are unknown, not clamped.
pub fn language_from_popup_position(position_1_based: u32) -> Option<LanguageId> {
    let implemented: Vec<&LanguageEntry> = REGISTRY.iter().filter(|e| e.implemented).collect();
    let index = usize::try_from(position_1_based.checked_sub(1)?).ok()?;
    implemented.get(index).map(|e| e.id)
}

/// Inverse mapping for correcting a popup that disagrees with the snapshot
/// ID (snapshot wins, ADR-0010 §3). `None` for unknown or unimplemented IDs:
/// those keep the Invalid state instead of being clamped.
pub fn popup_position_for(id: LanguageId) -> Option<u32> {
    REGISTRY
        .iter()
        .filter(|e| e.implemented)
        .position(|e| e.id == id)
        .and_then(|index| u32::try_from(index + 1).ok())
}

/// One validated pass module: language-neutral IR (naga is the shared IR all
/// frontends lower into), the user parameter declarations reflected from it
/// (ADR-0013 types), the std140 layout the GPU path uploads against, and the
/// extra input bindings the module declares (set 0, bindings 3+ — the
/// ADR-0011 reserved space consumed by ADR-0018 multi-input).
#[derive(Debug, Clone)]
pub struct PassModule {
    pub module: naga::Module,
    pub params: Vec<ParamDeclaration>,
    pub layout: UniformBlockLayout,
    /// Sorted declared bindings in 3..=15; binding 2+i feeds manifest
    /// input i.
    pub extra_input_bindings: Vec<u32>,
}

/// Reflected `FxUniforms` layout. `entries` parallels `PassModule::params`;
/// the 16-byte builtin head (ADR-0011 §4) precedes every entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniformBlockLayout {
    /// std140 span of the whole block, ≥ 16.
    pub block_size: usize,
    pub entries: Vec<UniformEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UniformEntry {
    /// Byte offset inside the block.
    pub offset: usize,
    /// 32-bit words this member occupies (1, 2, 3, or 4).
    pub words: usize,
    /// Written as i32 (int and bool-as-i32 members, ADR-0011 §4).
    pub int: bool,
}

/// Frontend failure classes. Code values belong to the M3 diagnostic
/// registry; the classes themselves are testable now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontendError {
    /// Source failed language parse/validation.
    Parse(String),
    /// Module violates the Shader ABI v1 interface (ADR-0011): missing
    /// required elements, wrong FxUniforms head, or reserved bindings.
    Abi(String),
    /// A reflected user parameter is invalid (ParamId grammar/reserved name)
    /// or has a type outside the v1 set.
    Param(String),
    /// `hint:canvas` names a reflected member outside the scalar Float kind.
    CanvasWrongKind(String),
}

/// A language implementation: parses and validates one pass module of raw
/// source against the Shader ABI (ADR-0011) and reflects user parameter
/// declarations. Frontends are selected by stable `LanguageId` (ADR-0002).
/// `annotations` is the whole committed text's `@param` map (parsed once —
/// ADR-0018 §6); `allowed_inputs` is the pass's manifest input count, which
/// bounds the extra bindings the module may declare (`E18` past it).
pub trait LanguageFrontend: Sync {
    fn language(&self) -> LanguageId;
    fn parse_module(
        &self,
        source: &str,
        annotations: &std::collections::HashMap<String, annotation::Annotation>,
        allowed_inputs: usize,
    ) -> Result<PassModule, FrontendError>;
}

/// Registry lookup. Only implemented languages resolve; unknown or
/// unimplemented IDs return `None` and the caller keeps the Invalid state
/// (never clamps, ADR-0010 §4).
pub fn frontend_for(id: LanguageId) -> Option<&'static dyn LanguageFrontend> {
    if id == LanguageId::GLSL {
        Some(&glsl::GlslFrontend)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Append-only registry guard: ascending unique IDs, no ID zero, no
    /// blank labels. A failure here means someone edited history instead of
    /// appending (ADR-0010 §1).
    #[test]
    fn registry_is_append_only_shaped() {
        let mut previous = 0u32;
        for entry in REGISTRY {
            assert!(entry.id.0 > previous, "IDs must be strictly ascending");
            assert!(entry.id != LanguageId::INVALID, "ID 0 is never registrable");
            assert!(!entry.display_name.is_empty());
            previous = entry.id.0;
        }
    }

    #[test]
    fn v1_menu_is_exactly_glsl() {
        assert_eq!(popup_menu(), vec!["GLSL"]);
        assert_eq!(default_language(), LanguageId::GLSL);
    }

    #[test]
    fn position_and_id_map_both_ways() {
        assert_eq!(language_from_popup_position(1), Some(LanguageId::GLSL));
        assert_eq!(popup_position_for(LanguageId::GLSL), Some(1));
    }

    #[test]
    fn unknown_positions_and_ids_are_rejected_not_clamped() {
        assert_eq!(language_from_popup_position(0), None);
        assert_eq!(language_from_popup_position(2), None);
        assert_eq!(popup_position_for(LanguageId::INVALID), None);
        // WGSL is registered but unimplemented: no menu position, not implemented.
        assert_eq!(popup_position_for(LanguageId::WGSL), None);
        assert!(!is_implemented(LanguageId::WGSL));
        assert!(!is_implemented(LanguageId(999)));
        assert!(is_implemented(LanguageId::GLSL));
    }
}
