//! Block decoration syntax for LukiWiki
//!
//! Provides line-prefix decorations:
//! - COLOR(fg,bg): text
//! - SIZE(rem): text
//! - RIGHT:/CENTER:/LEFT: text alignment

use once_cell::sync::Lazy;
use regex::Regex;

static COLOR_PREFIX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^COLOR\(([^,)]*?)(?:,([^)]*?))?\):\s*(.+)$").unwrap());

static SIZE_PREFIX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^SIZE\(([^)]+?)\):\s*(.+)$").unwrap());

static ALIGN_PREFIX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^(RIGHT|CENTER|LEFT):\s*(.+)$").unwrap());

/// Apply block decoration prefixes to content
///
/// # Arguments
///
/// * `html` - The HTML content to process
///
/// # Returns
///
/// HTML with block decorations applied
pub fn apply_block_decorations(html: &str) -> String {
    let mut result = html.to_string();

    // Apply COLOR prefix
    result = COLOR_PREFIX
        .replace_all(&result, |caps: &regex::Captures| {
            let fg = caps.get(1).map_or("", |m| m.as_str().trim());
            let bg = caps.get(2).map_or("", |m| m.as_str().trim());
            let content = caps.get(3).map_or("", |m| m.as_str());

            let mut styles = Vec::new();
            if !fg.is_empty() && fg != "inherit" {
                styles.push(format!("color: {}", fg));
            }
            if !bg.is_empty() && bg != "inherit" {
                styles.push(format!("background-color: {}", bg));
            }

            if styles.is_empty() {
                content.to_string()
            } else {
                format!("<p style=\"{}\">{}</p>", styles.join("; "), content)
            }
        })
        .to_string();

    // Apply SIZE prefix
    result = SIZE_PREFIX
        .replace_all(&result, |caps: &regex::Captures| {
            let size = caps.get(1).map_or("", |m| m.as_str());
            let content = caps.get(2).map_or("", |m| m.as_str());
            format!("<p style=\"font-size: {}rem\">{}</p>", size, content)
        })
        .to_string();

    // Apply alignment prefixes
    result = ALIGN_PREFIX
        .replace_all(&result, |caps: &regex::Captures| {
            let align = caps.get(1).map_or("", |m| m.as_str()).to_lowercase();
            let content = caps.get(2).map_or("", |m| m.as_str());
            format!("<p style=\"text-align: {}\">{}</p>", align, content)
        })
        .to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_foreground_only() {
        let input = "COLOR(red): This is red text";
        let output = apply_block_decorations(input);
        assert!(output.contains("style=\"color: red\""));
        assert!(output.contains("This is red text"));
    }

    #[test]
    fn test_color_background_only() {
        let input = "COLOR(,yellow): Yellow background";
        let output = apply_block_decorations(input);
        assert!(output.contains("style=\"background-color: yellow\""));
        assert!(output.contains("Yellow background"));
    }

    #[test]
    fn test_color_both() {
        let input = "COLOR(white,black): White on black";
        let output = apply_block_decorations(input);
        assert!(output.contains("color: white"));
        assert!(output.contains("background-color: black"));
    }

    #[test]
    fn test_color_inherit() {
        let input = "COLOR(,inherit): No background";
        let output = apply_block_decorations(input);
        // inherit should be ignored, so no styles should be applied
        assert_eq!(output, "No background");
    }

    #[test]
    fn test_size() {
        let input = "SIZE(1.5): Larger text";
        let output = apply_block_decorations(input);
        assert!(output.contains("style=\"font-size: 1.5rem\""));
        assert!(output.contains("Larger text"));
    }

    #[test]
    fn test_alignment() {
        let cases = vec![
            ("RIGHT: Right aligned", "text-align: right"),
            ("CENTER: Centered", "text-align: center"),
            ("LEFT: Left aligned", "text-align: left"),
        ];

        for (input, expected_style) in cases {
            let output = apply_block_decorations(input);
            assert!(
                output.contains(expected_style),
                "Input: {}\nOutput: {}",
                input,
                output
            );
        }
    }

    #[test]
    fn test_multiple_decorations() {
        let input = "COLOR(blue): Blue text\nSIZE(2): Big text\nRIGHT: Right text";
        let output = apply_block_decorations(input);
        assert!(output.contains("color: blue"));
        assert!(output.contains("font-size: 2rem"));
        assert!(output.contains("text-align: right"));
    }
}
