//! LukiWiki-specific syntax extensions
//!
//! This module provides parsing support for LukiWiki legacy syntax that extends
//! standard Markdown with additional formatting and layout capabilities.

pub mod block_decorations;
pub mod conflict_resolver;
pub mod emphasis;
pub mod inline_decorations;
pub mod plugins;

/// Apply LukiWiki-specific transformations to HTML output
///
/// This function processes the HTML output from the Markdown parser and applies
/// LukiWiki-specific syntax transformations.
///
/// # Arguments
///
/// * `html` - The HTML output from the Markdown parser
///
/// # Returns
///
/// Transformed HTML with LukiWiki syntax applied
pub fn apply_lukiwiki_syntax(html: &str) -> String {
    let mut result = html.to_string();

    // Protect code blocks and inline code from transformation
    let (protected, placeholders) = protect_code_sections(&result);
    result = protected;

    // Apply transformations in order
    // Note: Plugins are handled in conflict_resolver::postprocess_conflicts
    result = conflict_resolver::postprocess_conflicts(&result);
    result = emphasis::apply_lukiwiki_emphasis(&result);
    result = block_decorations::apply_block_decorations(&result);
    result = inline_decorations::apply_inline_decorations(&result);

    // Restore protected code sections
    restore_code_sections(&result, &placeholders)
}

/// Protect code blocks and inline code from transformation
///
/// Returns the HTML with code sections replaced by placeholders,
/// and a vector of the original code sections.
fn protect_code_sections(html: &str) -> (String, Vec<String>) {
    use regex::Regex;

    let mut placeholders = Vec::new();
    let mut result = html.to_string();

    // Protect <pre><code>...</code></pre> blocks
    let code_block_re = Regex::new(r"<pre><code[^>]*>[\s\S]*?</code></pre>").unwrap();
    result = code_block_re
        .replace_all(&result, |caps: &regex::Captures| {
            let index = placeholders.len();
            placeholders.push(caps[0].to_string());
            format!("<!--CODE_BLOCK_{}-->", index)
        })
        .to_string();

    // Protect <code>...</code> inline
    let inline_code_re = Regex::new(r"<code[^>]*>[^<]*</code>").unwrap();
    result = inline_code_re
        .replace_all(&result, |caps: &regex::Captures| {
            let index = placeholders.len();
            placeholders.push(caps[0].to_string());
            format!("<!--INLINE_CODE_{}-->", index)
        })
        .to_string();

    (result, placeholders)
}

/// Restore protected code sections
fn restore_code_sections(html: &str, placeholders: &[String]) -> String {
    use regex::Regex;

    let mut result = html.to_string();

    // Restore code blocks
    let placeholder_re = Regex::new(r"<!--(CODE_BLOCK|INLINE_CODE)_(\d+)-->").unwrap();
    result = placeholder_re
        .replace_all(&result, |caps: &regex::Captures| {
            let index: usize = caps[2].parse().unwrap();
            placeholders.get(index).map(|s| s.as_str()).unwrap_or("")
        })
        .to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lukiwiki_syntax_integration() {
        let input = "<p>This is ''bold'' and '''italic'''</p>";
        let output = apply_lukiwiki_syntax(input);
        assert!(output.contains("<b>bold</b>"));
        assert!(output.contains("<i>italic</i>"));
    }
}
