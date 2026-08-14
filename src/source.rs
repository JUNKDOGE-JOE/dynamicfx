//! Extraction of shader source from an AE expression.
//!
//! Convention: the `Source` parameter carries an expression of the form
//!
//! ```js
//! `<arbitrary multi-line shader code>`;0
//! ```
//!
//! The backtick template literal lets users paste multi-line code verbatim,
//! and the trailing `;0` makes the expression evaluate to a number so AE
//! does not flag a type-mismatch error on the numeric parameter.

/// Extract the shader source between the first and the last backtick.
/// Returns `None` when the expression does not follow the convention.
pub fn extract_source(expression: &str) -> Option<String> {
    let start = expression.find('`')?;
    let end = expression.rfind('`')?;
    if end <= start {
        return None;
    }
    Some(expression[start + 1..end].to_string())
}

/// Wrap shader source into a valid AE expression. Write-side helper for the
/// M1 host harness (JSX scenarios build expressions with the same escaping).
#[allow(dead_code)]
pub fn wrap_source(source: &str) -> String {
    // Escape any backticks or ${ in user code would break the template
    // literal; for now assume shader code rarely contains them, but guard.
    let escaped = source.replace('`', "\\`").replace("${", "\\${");
    format!("`{}`;0", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_between_backticks() {
        let expr = "`float4 main() { return 0; }`;0";
        assert_eq!(
            extract_source(expr).as_deref(),
            Some("float4 main() { return 0; }")
        );
    }

    #[test]
    fn rejects_plain_text() {
        assert_eq!(extract_source("1 + 2"), None);
    }
}
