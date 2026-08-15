//! The `@param` annotation grammar (M2, authorized by ADR-0013: identity
//! semantics live in the ADR; this surface syntax is fixed by these parser
//! tests). Language-agnostic: annotations live in `//` line comments, so any
//! frontend whose language has them can share this parser.
//!
//! ```text
//! // @param <id> [entry ...]
//! entry := label:"quoted text" | label:word
//!        | min:<number> | max:<number>
//!        | default:<number>[,<number>]*      (1-4 components)
//!        | default:#RRGGBB[AA]               (hint:color only, ADR-0026)
//!        | alias:<id>[,<id>]*
//!        | hint:angle | hint:color | hint:layer | hint:gradient
//!        | hint:point3d | hint:path
//! ```
//!
//! Error policy (fail closed without punishing leftovers): a malformed entry
//! on any `@param` line is a definition-rejecting error — silently ignoring
//! a typo like `mim:0` would leave the user wondering why their range never
//! applied. A well-formed line whose id matches no reflected uniform is
//! ignored (commented-out uniforms leave stale annotations behind).

use crate::definition::param::{ParamId, ParamIdError};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hint {
    Angle,
    /// ADR-0030: not a `FxUniforms` member at all. The id names a graph
    /// resource fed by an AE Layer parameter, so unlike every other hint it
    /// declares a parameter that reflection will never find.
    Layer,
    /// ADR-0031/0032: like `Layer`, declares a graph resource rather than a
    /// block member — the value is baked into a 1D LUT texture.
    Gradient,
    /// std140 rejects `bool` members (naga: non-host-shareable), so a bool
    /// parameter is an `int` member carrying `hint:bool` — exactly ADR-0011's
    /// "bool as i32".
    Bool,
    Color,
    /// ADR-0035: like `Layer` and `Gradient`, declares a graph resource rather
    /// than a block member — the value is an AE mask's vertices in a texture.
    Path,
    /// ADR-0034: makes a `vec3` a spatial Point 3D instead of the colour it
    /// would otherwise be. The default is deliberately unchanged, so this hint
    /// is the only way to reach the kind.
    Point3D,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Annotation {
    pub label: Option<String>,
    pub min: Option<f32>,
    pub max: Option<f32>,
    /// 1-4 components; arity is validated against the member type when the
    /// annotation is merged into a declaration, not here.
    pub default: Option<Vec<f32>>,
    pub aliases: Vec<ParamId>,
    pub hint: Option<Hint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationError {
    /// 1-based source line.
    pub line: usize,
    pub message: String,
}

fn err(line: usize, message: impl Into<String>) -> AnnotationError {
    AnnotationError { line, message: message.into() }
}

/// ADR-0025 §3 bounds: `// @window <n>` with 1..=64; default 16 when the
/// source declares none.
pub const WINDOW_DEFAULT: u32 = 16;
pub const WINDOW_MAX: u32 = 64;

/// Parse the optional `// @window <n>` annotation (ADR-0025). Returns
/// `None` when absent; malformed, duplicate, or out-of-range values are
/// definition-rejecting errors (never a silent clamp).
pub fn parse_window(source: &str) -> Result<Option<u32>, AnnotationError> {
    let mut found: Option<u32> = None;
    for (line_index, raw_line) in source.lines().enumerate() {
        let line_no = line_index + 1;
        let trimmed = raw_line.trim();
        let Some(comment) = trimmed.strip_prefix("//") else { continue };
        let Some(rest) = comment.trim_start().strip_prefix("@window") else { continue };
        let value = rest.trim();
        let n: u32 = value
            .parse()
            .map_err(|_| err(line_no, format!("@window needs an integer, got `{value}`")))?;
        if n < 1 || n > WINDOW_MAX {
            return Err(err(line_no, format!("@window must be 1..={WINDOW_MAX}, got {n}")));
        }
        if found.is_some() {
            return Err(err(line_no, "duplicate @window"));
        }
        found = Some(n);
    }
    Ok(found)
}

/// Parse every `@param` line. Returns id → annotation; duplicate ids and any
/// malformed entry are errors.
pub fn parse_annotations(source: &str) -> Result<HashMap<String, Annotation>, AnnotationError> {
    let mut out: HashMap<String, Annotation> = HashMap::new();
    for (line_index, raw_line) in source.lines().enumerate() {
        let line_no = line_index + 1;
        let trimmed = raw_line.trim();
        let Some(comment) = trimmed.strip_prefix("//") else { continue };
        let Some(rest) = comment.trim_start().strip_prefix("@param") else { continue };
        let rest = rest.trim();

        let mut tokens = tokenize(rest, line_no)?;
        if tokens.is_empty() {
            return Err(err(line_no, "@param needs an id"));
        }
        let id_token = tokens.remove(0);
        let id = ParamId::new(&id_token)
            .map_err(|e: ParamIdError| err(line_no, format!("bad id `{id_token}`: {e:?}")))?;

        let mut annotation = Annotation::default();
        let mut default_was_hex = false;
        for token in tokens {
            let Some((key, value)) = token.split_once(':') else {
                return Err(err(line_no, format!("`{token}` is not a key:value entry")));
            };
            match key {
                "label" => {
                    set_once(line_no, "label", &mut annotation.label, value.to_string())?;
                }
                "min" => {
                    let v = parse_number(line_no, "min", value)?;
                    set_once(line_no, "min", &mut annotation.min, v)?;
                }
                "max" => {
                    let v = parse_number(line_no, "max", value)?;
                    set_once(line_no, "max", &mut annotation.max, v)?;
                }
                "default" => {
                    // ADR-0026: `#RRGGBB[AA]` color literals decode to
                    // normalized components (6 digits imply alpha 1.0);
                    // malformed hex is rejected, never guessed.
                    let components: Vec<f32> = if let Some(hex) = value.strip_prefix('#') {
                        default_was_hex = true;
                        parse_hex_color(line_no, hex)?
                    } else {
                        value
                            .split(',')
                            .map(|c| parse_number(line_no, "default", c))
                            .collect::<Result<_, _>>()?
                    };
                    if components.is_empty() || components.len() > 4 {
                        return Err(err(line_no, "default takes 1-4 components"));
                    }
                    set_once(line_no, "default", &mut annotation.default, components)?;
                }
                "alias" => {
                    if !annotation.aliases.is_empty() {
                        return Err(err(line_no, "duplicate alias entry"));
                    }
                    for alias in value.split(',') {
                        let alias = ParamId::new(alias).map_err(|e| {
                            err(line_no, format!("bad alias `{alias}`: {e:?}"))
                        })?;
                        annotation.aliases.push(alias);
                    }
                }
                "hint" => {
                    let hint = match value {
                        "angle" => Hint::Angle,
                        "layer" => Hint::Layer,
                        "gradient" => Hint::Gradient,
                        "bool" => Hint::Bool,
                        "color" => Hint::Color,
                        "point3d" => Hint::Point3D,
                        "path" => Hint::Path,
                        other => return Err(err(line_no, format!("unknown hint `{other}`"))),
                    };
                    set_once(line_no, "hint", &mut annotation.hint, hint)?;
                }
                other => {
                    return Err(err(line_no, format!("unknown entry `{other}`")));
                }
            }
        }

        // ADR-0026 combination rules: hex literals are color-only, and a
        // color default never combines with min/max.
        if default_was_hex && annotation.hint != Some(Hint::Color) {
            return Err(err(line_no, "hex default requires hint:color"));
        }
        if annotation.hint == Some(Hint::Color)
            && annotation.default.is_some()
            && (annotation.min.is_some() || annotation.max.is_some())
        {
            return Err(err(line_no, "color default does not combine with min/max"));
        }

        if out.insert(id.as_str().to_string(), annotation).is_some() {
            return Err(err(line_no, format!("duplicate @param for `{}`", id.as_str())));
        }
    }
    Ok(out)
}

fn set_once<T>(line: usize, key: &str, slot: &mut Option<T>, value: T) -> Result<(), AnnotationError> {
    if slot.is_some() {
        return Err(err(line, format!("duplicate {key} entry")));
    }
    *slot = Some(value);
    Ok(())
}

fn parse_number(line: usize, key: &str, value: &str) -> Result<f32, AnnotationError> {
    let parsed = value.trim().parse::<f32>();
    match parsed {
        Ok(v) if v.is_finite() => Ok(v),
        _ => Err(err(line, format!("{key} needs a finite number, got `{value}`"))),
    }
}

/// ADR-0026 hex color literal (without the `#`): exactly 6 or 8 hex digits,
/// sRGB-8 channels normalized to 0..=1; 6 digits imply alpha 1.0.
fn parse_hex_color(line: usize, hex: &str) -> Result<Vec<f32>, AnnotationError> {
    if hex.len() != 6 && hex.len() != 8 {
        return Err(err(line, format!("hex default needs 6 or 8 digits, got `{hex}`")));
    }
    let mut components = Vec::with_capacity(4);
    for pair in 0..hex.len() / 2 {
        let byte = u8::from_str_radix(&hex[pair * 2..pair * 2 + 2], 16)
            .map_err(|_| err(line, format!("bad hex digits in default `#{hex}`")))?;
        components.push(byte as f32 / 255.0);
    }
    if components.len() == 3 {
        components.push(1.0);
    }
    Ok(components)
}

/// Split on whitespace, keeping `label:"..."` quoted spans intact.
fn tokenize(rest: &str, line: usize) -> Result<Vec<String>, AnnotationError> {
    let mut tokens = Vec::new();
    let mut chars = rest.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        let mut token = String::new();
        let mut in_quotes = false;
        while let Some(&c) = chars.peek() {
            if c == '"' {
                in_quotes = !in_quotes;
                chars.next();
                continue; // quotes delimit, they are not part of the value
            }
            if c.is_whitespace() && !in_quotes {
                break;
            }
            token.push(c);
            chars.next();
        }
        if in_quotes {
            return Err(err(line, "unterminated quote"));
        }
        tokens.push(token);
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ADR-0025 §3: default 16, cap 64, reject rather than clamp.
    #[test]
    fn window_annotation() {
        assert_eq!(parse_window("void main() {}").unwrap(), None);
        assert_eq!(parse_window("// @window 8\nvoid main() {}").unwrap(), Some(8));
        assert_eq!(parse_window("//@window 64").unwrap(), Some(64));
        assert!(parse_window("// @window 0").unwrap_err().message.contains("1..=64"));
        assert!(parse_window("// @window 65").unwrap_err().message.contains("1..=64"));
        assert!(parse_window("// @window lots").unwrap_err().message.contains("integer"));
        assert!(parse_window("// @window 4\n// @window 4").unwrap_err().message.contains("duplicate"));
        assert_eq!(WINDOW_DEFAULT, 16);
    }

    // ADR-0026: hex color defaults — exact decode, alpha rules, rejections.
    #[test]
    fn color_hex_defaults() {
        let ok = parse_annotations("// @param tint hint:color default:#1A6BFF").unwrap();
        let d = ok["tint"].default.as_ref().unwrap();
        assert_eq!(d.len(), 4);
        assert!((d[0] - 26.0 / 255.0).abs() < 1e-6);
        assert!((d[1] - 107.0 / 255.0).abs() < 1e-6);
        assert!((d[2] - 255.0 / 255.0).abs() < 1e-6);
        assert_eq!(d[3], 1.0);

        let with_alpha = parse_annotations("// @param t hint:color default:#00FF8080").unwrap();
        let d = with_alpha["t"].default.as_ref().unwrap();
        assert_eq!(d[0], 0.0);
        assert_eq!(d[1], 1.0);
        assert!((d[3] - 128.0 / 255.0).abs() < 1e-6);

        // Case-insensitive digits.
        assert!(parse_annotations("// @param t hint:color default:#a0b1c2").is_ok());
        // Wrong length, bad digits, hex without hint:color, min/max combo.
        assert!(parse_annotations("// @param t hint:color default:#12345")
            .unwrap_err()
            .message
            .contains("6 or 8 digits"));
        assert!(parse_annotations("// @param t hint:color default:#GGGGGG")
            .unwrap_err()
            .message
            .contains("bad hex digits"));
        assert!(parse_annotations("// @param t default:#112233")
            .unwrap_err()
            .message
            .contains("requires hint:color"));
        assert!(parse_annotations("// @param t hint:color min:0 max:1 default:#112233")
            .unwrap_err()
            .message
            .contains("does not combine"));
    }

    #[test]
    fn full_entry_set_parses() {
        let src = r#"
// @param gain label:"Master Gain" min:0 max:2 default:0.5 alias:level,volume
// @param sweep hint:angle default:90
uniform stuff;
"#;
        let map = parse_annotations(src).unwrap();
        let gain = &map["gain"];
        assert_eq!(gain.label.as_deref(), Some("Master Gain"));
        assert_eq!(gain.min, Some(0.0));
        assert_eq!(gain.max, Some(2.0));
        assert_eq!(gain.default, Some(vec![0.5]));
        let aliases: Vec<&str> = gain.aliases.iter().map(|a| a.as_str()).collect();
        assert_eq!(aliases, vec!["level", "volume"]);
        let sweep = &map["sweep"];
        assert_eq!(sweep.hint, Some(Hint::Angle));
        assert_eq!(sweep.default, Some(vec![90.0]));
    }

    #[test]
    fn unquoted_label_and_multi_component_default() {
        let map =
            parse_annotations("// @param tint label:Tint default:1,0.5,0.25 hint:color").unwrap();
        let tint = &map["tint"];
        assert_eq!(tint.label.as_deref(), Some("Tint"));
        assert_eq!(tint.default, Some(vec![1.0, 0.5, 0.25]));
        assert_eq!(tint.hint, Some(Hint::Color));
    }

    #[test]
    fn non_param_comments_and_code_are_ignored() {
        let map = parse_annotations("// plain comment\nfloat x; // @paramish nothing").unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn malformed_entries_are_errors_with_line_numbers() {
        for (src, needle) in [
            ("// @param", "needs an id"),
            ("// @param 9bad min:0", "bad id"),
            ("// @param x stray", "not a key:value"),
            ("// @param x mim:0", "unknown entry"),
            ("// @param x min:abc", "finite number"),
            ("// @param x min:0 min:1", "duplicate min"),
            ("// @param x default:1,2,3,4,5", "1-4 components"),
            ("// @param x hint:sideways", "unknown hint"),
            ("// @param x alias:9bad", "bad alias"),
            ("// @param x label:\"open", "unterminated quote"),
            ("// @param x min:0\n// @param x max:1", "duplicate @param"),
        ] {
            let e = parse_annotations(src).expect_err(src);
            assert!(e.message.contains(needle), "{src}: {e:?}");
        }
        let two_lines = parse_annotations("//ok\n// @param x mim:0").unwrap_err();
        assert_eq!(two_lines.line, 2);
    }
}

/// Ids annotated `hint:gradient` (ADR-0031), sorted.
pub fn gradient_param_names(source: &str) -> Vec<String> {
    hinted_names(source, Hint::Gradient)
}

/// Ids annotated `hint:path` (ADR-0035), in source order.
pub fn path_param_names(source: &str) -> Vec<String> {
    hinted_names(source, Hint::Path)
}

/// Ids annotated `hint:layer` (ADR-0030), in source order.
///
/// The graph grammar needs these *before* any frontend runs: a layer name is
/// a legal pass input with no writer, which the writer rules would otherwise
/// reject as `E6`. Parse errors are swallowed here on purpose — this is a
/// pre-pass, and the real `parse_annotations` call reports them with a line
/// number once the pass body is compiled.
pub fn layer_param_names(source: &str) -> Vec<String> {
    hinted_names(source, Hint::Layer)
}

/// Ids carrying one hint. Both graph-resource hints share this: the grammar
/// needs their names before any frontend runs, because such a name is a legal
/// pass input that no pass writes.
fn hinted_names(source: &str, hint: Hint) -> Vec<String> {
    let Ok(annotations) = parse_annotations(source) else {
        return Vec::new();
    };
    let mut names: Vec<String> = annotations
        .iter()
        .filter(|(_, a)| a.hint == Some(hint))
        .map(|(name, _)| name.clone())
        .collect();
    names.sort();
    names
}
