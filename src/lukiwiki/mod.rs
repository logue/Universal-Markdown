//! LukiWiki-specific syntax extensions
//!
//! This module provides parsing support for LukiWiki legacy syntax that extends
//! standard Markdown with additional formatting and layout capabilities.

pub mod block_decorations;
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

    // Apply transformations in order
    result = emphasis::apply_lukiwiki_emphasis(&result);
    result = block_decorations::apply_block_decorations(&result);
    result = inline_decorations::apply_inline_decorations(&result);
    result = plugins::apply_plugin_syntax(&result);

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
