//! UMD emphasis syntax
//!
//! Provides support for UMD-style emphasis using '' and '''
//! - ''text'' → <b>text</b> (visual bold)
//! - '''text''' → <i>text</i> (visual italic)
//! - __text__ → <u>text</u> (underline, Discord-style - handled in preprocessor)

use once_cell::sync::Lazy;
use regex::Regex;

static UMD_BOLD: Lazy<Regex> = Lazy::new(|| {
    // Match ''text'' but not '''text''' (at least 2 non-quote chars)
    Regex::new(r"''([^']{2,})''").unwrap()
});

static UMD_ITALIC: Lazy<Regex> = Lazy::new(|| {
    // Match '''text''' with at least one non-quote char
    Regex::new(r"'''([^']+)'''").unwrap()
});

/// Apply UMD emphasis syntax to HTML
///
/// Converts UMD-style emphasis markers to HTML tags.
/// Note: '''text''' must be processed before ''text'' to avoid conflicts.
///
/// # Arguments
///
/// * `html` - The HTML content to process
///
/// # Returns
///
/// HTML with UMD emphasis applied
///
/// # Examples
///
/// ```
/// use universal_markdown::extensions::emphasis::apply_umd_emphasis;
///
/// let input = "This is ''bold'' and '''italic'''";
/// let output = apply_umd_emphasis(input);
/// assert!(output.contains("<b>bold</b>"));
/// assert!(output.contains("<i>italic</i>"));
/// ```
pub fn apply_umd_emphasis(html: &str) -> String {
    // Process '''text''' (italic) first to avoid conflicts with ''text''
    let result = UMD_ITALIC.replace_all(html, "<i>$1</i>");

    // Then process ''text'' (bold)
    let result = UMD_BOLD.replace_all(&result, "<b>$1</b>");

    result.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_umd_bold() {
        let input = "This is ''bold'' text.";
        let output = apply_umd_emphasis(input);
        assert_eq!(output, "This is <b>bold</b> text.");
    }

    #[test]
    fn test_umd_italic() {
        let input = "This is '''italic''' text.";
        let output = apply_umd_emphasis(input);
        assert_eq!(output, "This is <i>italic</i> text.");
    }

    #[test]
    fn test_umd_both() {
        let input = "''bold'' and '''italic'''";
        let output = apply_umd_emphasis(input);
        assert!(output.contains("<b>bold</b>"));
        assert!(output.contains("<i>italic</i>"));
    }

    #[test]
    fn test_discord_underline() {
        // Note: __text__ is handled in preprocessor, not here
        // This test verifies that apply_umd_emphasis doesn't break placeholder markers
        let input = "This is {{UNDERLINE:underlined:UNDERLINE}} text.";
        let output = apply_umd_emphasis(input);
        assert!(output.contains("{{UNDERLINE:underlined:UNDERLINE}}"));
    }

    #[test]
    fn test_strong_remains_strong() {
        // **text** should remain as <strong>, not converted to <u>
        let input = "<strong>emphasis</strong>";
        let output = apply_umd_emphasis(input);
        assert_eq!(output, "<strong>emphasis</strong>");
    }

    #[test]
    fn test_all_emphasis_types() {
        let input = "''bold'' and '''italic'''";
        let output = apply_umd_emphasis(input);
        assert!(output.contains("<b>bold</b>"));
        assert!(output.contains("<i>italic</i>"));
    }

    #[test]
    fn test_no_false_matches() {
        let input = "Don't match this: 'single' or four";
        let output = apply_umd_emphasis(input);
        // Should not match single quotes
        assert_eq!(output, input);
    }
}
