//! LukiWiki emphasis syntax
//!
//! Provides support for LukiWiki-style emphasis using '' and '''
//! - ''text'' → <b>text</b> (visual bold)
//! - '''text''' → <i>text</i> (visual italic)

use once_cell::sync::Lazy;
use regex::Regex;

static LUKIWIKI_BOLD: Lazy<Regex> = Lazy::new(|| {
    // Match ''text'' but not '''text''' (at least 2 non-quote chars)
    Regex::new(r"''([^']{2,})''").unwrap()
});

static LUKIWIKI_ITALIC: Lazy<Regex> = Lazy::new(|| {
    // Match '''text''' with at least one non-quote char
    Regex::new(r"'''([^']+)'''").unwrap()
});

/// Apply LukiWiki emphasis syntax to HTML
///
/// Converts LukiWiki-style emphasis markers to HTML tags.
/// Note: '''text''' must be processed before ''text'' to avoid conflicts.
///
/// # Arguments
///
/// * `html` - The HTML content to process
///
/// # Returns
///
/// HTML with LukiWiki emphasis applied
///
/// # Examples
///
/// ```
/// use lukiwiki_parser::lukiwiki::emphasis::apply_lukiwiki_emphasis;
///
/// let input = "This is ''bold'' and '''italic'''";
/// let output = apply_lukiwiki_emphasis(input);
/// assert!(output.contains("<b>bold</b>"));
/// assert!(output.contains("<i>italic</i>"));
/// ```
pub fn apply_lukiwiki_emphasis(html: &str) -> String {
    // Process '''text''' first (italic) to avoid conflicts with ''text''
    let result = LUKIWIKI_ITALIC.replace_all(html, "<i>$1</i>");

    // Then process ''text'' (bold)
    let result = LUKIWIKI_BOLD.replace_all(&result, "<b>$1</b>");

    result.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lukiwiki_bold() {
        let input = "This is ''bold'' text.";
        let output = apply_lukiwiki_emphasis(input);
        assert_eq!(output, "This is <b>bold</b> text.");
    }

    #[test]
    fn test_lukiwiki_italic() {
        let input = "This is '''italic''' text.";
        let output = apply_lukiwiki_emphasis(input);
        assert_eq!(output, "This is <i>italic</i> text.");
    }

    #[test]
    fn test_lukiwiki_both() {
        let input = "''bold'' and '''italic'''";
        let output = apply_lukiwiki_emphasis(input);
        assert!(output.contains("<b>bold</b>"));
        assert!(output.contains("<i>italic</i>"));
    }

    #[test]
    fn test_lukiwiki_mixed_with_markdown() {
        // Should work alongside Markdown emphasis
        let input = "<p>**Markdown bold** and ''LukiWiki bold''</p>";
        let output = apply_lukiwiki_emphasis(input);
        assert!(output.contains("**Markdown bold**")); // Unchanged
        assert!(output.contains("<b>LukiWiki bold</b>"));
    }

    #[test]
    fn test_no_false_matches() {
        let input = "Don't match this: 'single' or four";
        let output = apply_lukiwiki_emphasis(input);
        // Should not match single quotes
        assert_eq!(output, input);
    }
}
