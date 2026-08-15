//! Stable parameter identity (ADR-0013): the ParamId grammar, reserved
//! names, alias namespace rules, and the shader-type → pool-slot mapping
//! consumed by `BindingPlan`.

use crate::binding::PoolKind;

/// Maximum ParamId length in bytes (ADR-0013 §1).
pub const MAX_PARAM_ID_BYTES: usize = 64;

/// Builtin uniform-head names (ADR-0011) are never user ParamIds.
const RESERVED_EXACT: &[&str] = &["u_resolution", "u_time", "u_frame"];

/// Runtime-reserved namespace (ADR-0013 §1).
const RESERVED_PREFIX: &str = "dfx_";

/// A validated stable parameter identity: `[A-Za-z_][A-Za-z0-9_]*`, 1-64
/// bytes, case-sensitive — the shared identifier subset of GLSL/WGSL, so a
/// uniform member name is always a well-formed initial ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParamId(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamIdError {
    Empty,
    TooLong { bytes: usize },
    BadStart { c: char },
    BadChar { c: char },
    ReservedBuiltinName,
    ReservedPrefix,
}

impl ParamId {
    pub fn new(raw: &str) -> Result<Self, ParamIdError> {
        let mut chars = raw.chars();
        let first = chars.next().ok_or(ParamIdError::Empty)?;
        if raw.len() > MAX_PARAM_ID_BYTES {
            return Err(ParamIdError::TooLong { bytes: raw.len() });
        }
        if !(first.is_ascii_alphabetic() || first == '_') {
            return Err(ParamIdError::BadStart { c: first });
        }
        if let Some(c) = chars.find(|&c| !(c.is_ascii_alphanumeric() || c == '_')) {
            return Err(ParamIdError::BadChar { c });
        }
        if RESERVED_EXACT.contains(&raw) {
            return Err(ParamIdError::ReservedBuiltinName);
        }
        if raw.starts_with(RESERVED_PREFIX) {
            return Err(ParamIdError::ReservedPrefix);
        }
        Ok(Self(raw.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Shader-side parameter types bindable in v1. The type set is ADR-0011's;
/// the pool mapping is ADR-0013 §3-§4. `Vec3Color` is the vec3 default
/// (ADR-0026); `Point3D` is the same GLSL type reached through `hint:point3d`
/// (ADR-0034 §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderParamType {
    Float,
    Int,
    Bool,
    Vec2,
    Vec3Color,
    Vec4Color,
    /// `float` with the `angle` annotation hint.
    AngleFloat,
    /// ADR-0030: an AE layer selector feeding a texture binding. Unlike every
    /// other variant this one has no `FxUniforms` member and contributes no
    /// words to the uniform buffer — it is declared by annotation alone.
    Layer,
    /// ADR-0031/0032: a gradient baked into a 1D LUT texture. Same shape as
    /// `Layer` — annotation-declared, texture-bound, no block storage.
    Gradient,
    /// ADR-0034: `vec3` with the `point3d` annotation hint — a spatial value
    /// on the AE 3D point widget, not a colour.
    Point3D,
    /// ADR-0035: an AE mask delivered as an `N x 2` vertex texture. Same shape
    /// as `Layer`/`Gradient` — annotation-declared, texture-bound, no block
    /// storage.
    Path,
}

impl ShaderParamType {
    /// Pool slots this type consumes. `Vec4Color` is the only multi-slot
    /// mapping — Color (RGB) plus Float (alpha) — and allocates atomically:
    /// if either pool is full the whole definition is rejected.
    pub fn slot_requirements(self) -> &'static [PoolKind] {
        match self {
            Self::Float => &[PoolKind::Float],
            Self::Int => &[PoolKind::Integer],
            Self::Bool => &[PoolKind::Bool],
            Self::Vec2 => &[PoolKind::Point2D],
            Self::Vec3Color => &[PoolKind::Color],
            Self::Vec4Color => &[PoolKind::Color, PoolKind::Float],
            Self::AngleFloat => &[PoolKind::Angle],
            Self::Layer => &[PoolKind::Layer],
            Self::Gradient => &[PoolKind::Gradient],
            Self::Point3D => &[PoolKind::Point3D],
            Self::Path => &[PoolKind::Path],
        }
    }

    /// True when the parameter is a texture binding rather than uniform-block
    /// storage. Such parameters are skipped by uniform packing and by the
    /// float-budget accounting.
    pub fn is_texture_binding(self) -> bool {
        matches!(self, Self::Layer | Self::Gradient | Self::Path)
    }
}

/// UI metadata from the `@param` annotation. Presentation only: none of it
/// enters identity or hashing (ADR-0010 §6 discipline applies here too).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParamUiMeta {
    pub label: Option<String>,
    pub min: Option<f32>,
    pub max: Option<f32>,
    /// Component count matches the type's word count (validated at merge).
    pub default: Option<Vec<f32>>,
}

/// One declared user parameter. Aliases are prior IDs for slot inheritance
/// (single-generation, resolved against the previous BindingPlan).
#[derive(Debug, Clone)]
pub struct ParamDeclaration {
    pub id: ParamId,
    pub ty: ShaderParamType,
    pub aliases: Vec<ParamId>,
    pub ui: ParamUiMeta,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclarationError {
    /// ADR-0013 §2: current IDs and aliases share one namespace; any
    /// duplicate rejects the whole definition atomically.
    DuplicateName { name: String },
}

/// Validate the shared ID/alias namespace of a whole definition.
pub fn validate_declarations(decls: &[ParamDeclaration]) -> Result<(), DeclarationError> {
    let mut seen = std::collections::HashSet::new();
    for decl in decls {
        for name in std::iter::once(&decl.id).chain(decl.aliases.iter()) {
            if !seen.insert(name.as_str().to_owned()) {
                return Err(DeclarationError::DuplicateName { name: name.as_str().to_owned() });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_accepts_identifier_forms() {
        for ok in ["a", "_ok", "speed", "Speed2", "a_b_c", "A"] {
            assert!(ParamId::new(ok).is_ok(), "{ok:?} should be valid");
        }
        let max = "a".repeat(MAX_PARAM_ID_BYTES);
        assert!(ParamId::new(&max).is_ok());
    }

    #[test]
    fn grammar_rejects_bad_forms() {
        assert_eq!(ParamId::new(""), Err(ParamIdError::Empty));
        let over = "a".repeat(MAX_PARAM_ID_BYTES + 1);
        assert!(matches!(ParamId::new(&over), Err(ParamIdError::TooLong { .. })));
        assert!(matches!(ParamId::new("9abc"), Err(ParamIdError::BadStart { .. })));
        assert!(matches!(ParamId::new("a-b"), Err(ParamIdError::BadChar { .. })));
        assert!(matches!(ParamId::new("速度"), Err(ParamIdError::BadStart { .. })));
        assert!(matches!(ParamId::new("a b"), Err(ParamIdError::BadChar { .. })));
    }

    #[test]
    fn reserved_names_and_prefix_are_rejected() {
        for builtin in ["u_resolution", "u_time", "u_frame"] {
            assert_eq!(ParamId::new(builtin), Err(ParamIdError::ReservedBuiltinName));
        }
        assert_eq!(ParamId::new("dfx_internal"), Err(ParamIdError::ReservedPrefix));
        // Ordinary `u_` names stay allowed; only the exact builtins are reserved.
        assert!(ParamId::new("u_speed").is_ok());
    }

    #[test]
    fn id_and_alias_namespace_is_shared() {
        let dup_id = vec![
            ParamDeclaration {
                id: ParamId::new("speed").unwrap(),
                ty: ShaderParamType::Float,
                aliases: vec![],
                ui: Default::default(),
            },
            ParamDeclaration {
                id: ParamId::new("speed").unwrap(),
                ty: ShaderParamType::Float,
                aliases: vec![],
                ui: Default::default(),
            },
        ];
        assert!(validate_declarations(&dup_id).is_err());

        let alias_hits_id = vec![
            ParamDeclaration {
                id: ParamId::new("speed").unwrap(),
                ty: ShaderParamType::Float,
                aliases: vec![],
                ui: Default::default(),
            },
            ParamDeclaration {
                id: ParamId::new("velocity").unwrap(),
                ty: ShaderParamType::Float,
                aliases: vec![ParamId::new("speed").unwrap()],
                ui: Default::default(),
            },
        ];
        assert!(validate_declarations(&alias_hits_id).is_err());

        let clean = vec![
            ParamDeclaration {
                id: ParamId::new("speed").unwrap(),
                ty: ShaderParamType::Float,
                aliases: vec![ParamId::new("rate").unwrap()],
                ui: Default::default(),
            },
            ParamDeclaration {
                id: ParamId::new("tint").unwrap(),
                ty: ShaderParamType::Vec3Color,
                aliases: vec![],
                ui: Default::default(),
            },
        ];
        assert!(validate_declarations(&clean).is_ok());
    }
}
