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
//!        | alias:<id>[,<id>]*
//!        | hint:angle | hint:color
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
    /// std140 rejects `bool` members (naga: non-host-shareable), so a bool
    /// parameter is an `int` member carrying `hint:bool` — exactly ADR-0011's
    /// "bool as i32".
    Bool,
    Color,
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
                    let components: Vec<f32> = value
                        .split(',')
                        .map(|c| parse_number(line_no, "default", c))
                        .collect::<Result<_, _>>()?;
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
                        "bool" => Hint::Bool,
                        "color" => Hint::Color,
                        other => return Err(err(line_no, format!("unknown hint `{other}`"))),
                    };
                    set_once(line_no, "hint", &mut annotation.hint, hint)?;
                }
                other => {
                    return Err(err(line_no, format!("unknown entry `{other}`")));
                }
            }
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
