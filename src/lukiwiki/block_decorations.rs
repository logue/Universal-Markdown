//! Block decoration syntax for LukiWiki with Bootstrap 5 class support
//!
//! Provides line-prefix decorations with compound syntax support:
//! - COLOR(fg,bg): Bootstrap color classes or inherit
//! - SIZE(value): Bootstrap fs-* classes or inline rem
//! - TRUNCATE: Bootstrap text-truncate class
//! - JUSTIFY/RIGHT/CENTER/LEFT: Bootstrap text alignment classes
//!
//! Multiple prefixes can be combined:
//! - SIZE(1.5): COLOR(primary): CENTER: Text
//! - TRUNCATE: RIGHT: Text

use once_cell::sync::Lazy;
use regex::Regex;

/// Block decoration attributes
#[derive(Default, Debug)]
struct BlockDecoration {
    // Color classes or inline styles
    fg_color: Option<String>,
    bg_color: Option<String>,
    // Font size class or inline style
    font_size: Option<String>,
    // Text alignment class
    text_align: Option<String>,
    // Truncate flag
    truncate: bool,
    // Vertical alignment (for table cells)
    vertical_align: Option<String>,
}

impl BlockDecoration {
    /// Convert to HTML class and style attributes
    fn to_html_attrs(&self) -> (Option<String>, Option<String>) {
        let mut classes = Vec::new();
        let mut styles = Vec::new();

        // Text alignment
        if let Some(ref align) = self.text_align {
            classes.push(align.clone());
        }

        // Truncate
        if self.truncate {
            classes.push("text-truncate".to_string());
        }

        // Vertical alignment
        if let Some(ref valign) = self.vertical_align {
            classes.push(valign.clone());
        }

        // Font size (class or inline)
        if let Some(ref size) = self.font_size {
            if size.starts_with("fs-") {
                classes.push(size.clone());
            } else {
                styles.push(format!("font-size: {}", size));
            }
        }

        // Foreground color (class or inline)
        if let Some(ref fg) = self.fg_color {
            if fg.starts_with("text-") {
                classes.push(fg.clone());
            } else {
                styles.push(format!("color: {}", fg));
            }
        }

        // Background color (class or inline)
        if let Some(ref bg) = self.bg_color {
            if bg.starts_with("bg-") {
                classes.push(bg.clone());
            } else {
                styles.push(format!("background-color: {}", bg));
            }
        }

        let class_attr = if classes.is_empty() {
            None
        } else {
            Some(format!("class=\"{}\"", classes.join(" ")))
        };

        let style_attr = if styles.is_empty() {
            None
        } else {
            Some(format!("style=\"{}\"", styles.join("; ")))
        };

        (class_attr, style_attr)
    }
}

// Compound prefix pattern: captures all decoration prefixes in one line (reserved for future use)
#[allow(dead_code)]
static COMPOUND_PREFIX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?m)^(?:(?:SIZE\(([^)]+)\)|COLOR\(([^,)]*?)(?:,([^)]*?))?\)|(TRUNCATE)|(TOP|MIDDLE|BOTTOM|BASELINE)|(JUSTIFY|RIGHT|CENTER|LEFT)):\s*)+(.+)$"
    )
    .unwrap()
});

// Individual pattern extractors
static SIZE_EXTRACT: Lazy<Regex> = Lazy::new(|| Regex::new(r"SIZE\(([^)]+)\):").unwrap());
static COLOR_EXTRACT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"COLOR\(([^,)]*?)(?:,([^)]*?))?\):").unwrap());
static TRUNCATE_EXTRACT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(TRUNCATE):").unwrap());
static VALIGN_EXTRACT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(TOP|MIDDLE|BOTTOM|BASELINE):").unwrap());
static ALIGN_EXTRACT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(JUSTIFY|RIGHT|CENTER|LEFT):").unwrap());

/// Map font size value to Bootstrap class or inline style
fn map_font_size(value: &str) -> String {
    // Check if value has unit (rem, em, px, etc.)
    if value.contains("rem") || value.contains("em") || value.contains("px") {
        return value.to_string(); // Return as inline style
    }

    // Map to Bootstrap fs-* classes (unitless values)
    match value {
        "2.5" => "fs-1".to_string(),       // 2.5rem
        "2" | "2.0" => "fs-2".to_string(), // 2rem
        "1.75" => "fs-3".to_string(),      // 1.75rem
        "1.5" => "fs-4".to_string(),       // 1.5rem
        "1.25" => "fs-5".to_string(),      // 1.25rem
        "0.875" => "fs-6".to_string(),     // 0.875rem
        _ => format!("{}rem", value),      // Custom value as inline style
    }
}

/// Map color value to Bootstrap class or inline style
fn map_color(value: &str, is_background: bool) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "inherit" {
        return None;
    }

    // Bootstrap theme colors
    let bootstrap_colors = [
        "primary",
        "secondary",
        "success",
        "danger",
        "warning",
        "info",
        "light",
        "dark",
        "body",
        "body-secondary",
        "body-tertiary",
        "body-emphasis",
        // With suffixes
        "primary-subtle",
        "secondary-subtle",
        "success-subtle",
        "danger-subtle",
        "warning-subtle",
        "info-subtle",
        "light-subtle",
        "dark-subtle",
        "primary-emphasis",
        "secondary-emphasis",
        "success-emphasis",
        "danger-emphasis",
        "warning-emphasis",
        "info-emphasis",
        "light-emphasis",
        "dark-emphasis",
    ];

    // Check if it's a Bootstrap color
    for color in &bootstrap_colors {
        if trimmed == *color || trimmed.starts_with(&format!("{}-", color)) {
            let prefix = if is_background { "bg" } else { "text" };
            return Some(format!("{}-{}", prefix, trimmed));
        }
    }

    // Otherwise, return as inline style value
    Some(trimmed.to_string())
}

/// Map alignment to Bootstrap class
fn map_text_align(value: &str) -> String {
    match value.to_uppercase().as_str() {
        "RIGHT" => "text-end".to_string(),
        "CENTER" => "text-center".to_string(),
        "LEFT" => "text-start".to_string(),
        "JUSTIFY" => "text-justify".to_string(),
        _ => "text-start".to_string(),
    }
}

/// Map vertical alignment to Bootstrap class
fn map_vertical_align(value: &str) -> String {
    match value.to_uppercase().as_str() {
        "TOP" => "align-top".to_string(),
        "MIDDLE" => "align-middle".to_string(),
        "BOTTOM" => "align-bottom".to_string(),
        "BASELINE" => "align-baseline".to_string(),
        _ => "align-baseline".to_string(),
    }
}

/// Parse all prefixes from a line and extract decoration attributes
fn parse_prefixes(line: &str) -> (BlockDecoration, String) {
    let mut decoration = BlockDecoration::default();
    let mut remaining = line;

    // Extract SIZE
    if let Some(caps) = SIZE_EXTRACT.captures(remaining) {
        let value = caps.get(1).map_or("", |m| m.as_str());
        decoration.font_size = Some(map_font_size(value));
        remaining = &remaining[caps.get(0).unwrap().end()..];
    }

    // Extract COLOR
    if let Some(caps) = COLOR_EXTRACT.captures(remaining) {
        let fg = caps.get(1).map_or("", |m| m.as_str());
        let bg = caps.get(2).map_or("", |m| m.as_str());
        decoration.fg_color = map_color(fg, false);
        decoration.bg_color = map_color(bg, true);
        remaining = &remaining[caps.get(0).unwrap().end()..];
    }

    // Extract TRUNCATE
    if TRUNCATE_EXTRACT.is_match(remaining) {
        decoration.truncate = true;
        remaining = TRUNCATE_EXTRACT.replace(remaining, "").to_string().leak();
    }

    // Extract vertical alignment
    if let Some(caps) = VALIGN_EXTRACT.captures(remaining) {
        let value = caps.get(1).map_or("", |m| m.as_str());
        decoration.vertical_align = Some(map_vertical_align(value));
        remaining = &remaining[caps.get(0).unwrap().end()..];
    }

    // Extract text alignment (must be last as it contains the content)
    if let Some(caps) = ALIGN_EXTRACT.captures(remaining) {
        let value = caps.get(1).map_or("", |m| m.as_str());
        decoration.text_align = Some(map_text_align(value));
        remaining = &remaining[caps.get(0).unwrap().end()..];
    }

    (decoration, remaining.trim().to_string())
}

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
    let mut result = String::new();

    for line in html.lines() {
        // Check if line starts with any decoration prefix
        if line.starts_with("SIZE(")
            || line.starts_with("COLOR(")
            || line.starts_with("TRUNCATE:")
            || line.starts_with("TOP:")
            || line.starts_with("MIDDLE:")
            || line.starts_with("BOTTOM:")
            || line.starts_with("BASELINE:")
            || line.starts_with("JUSTIFY:")
            || line.starts_with("RIGHT:")
            || line.starts_with("CENTER:")
            || line.starts_with("LEFT:")
        {
            let (decoration, content) = parse_prefixes(line);
            let (class_attr, style_attr) = decoration.to_html_attrs();

            let mut attrs = Vec::new();
            if let Some(class) = class_attr {
                attrs.push(class);
            }
            if let Some(style) = style_attr {
                attrs.push(style);
            }

            if attrs.is_empty() {
                result.push_str(&format!("<p>{}</p>\n", content));
            } else {
                result.push_str(&format!("<p {}>{}</p>\n", attrs.join(" "), content));
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    result.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_bootstrap_class() {
        let input = "COLOR(primary): Primary text";
        let output = apply_block_decorations(input);
        assert!(output.contains("class=\"text-primary\""));
        assert!(output.contains("Primary text"));
    }

    #[test]
    fn test_color_custom_value() {
        let input = "COLOR(#FF0000): Custom red";
        let output = apply_block_decorations(input);
        assert!(output.contains("style=\"color: #FF0000\""));
    }

    #[test]
    fn test_size_bootstrap_class() {
        let input = "SIZE(1.5): Medium text";
        let output = apply_block_decorations(input);
        assert!(output.contains("class=\"fs-4\""));
    }

    #[test]
    fn test_size_custom_value() {
        let input = "SIZE(3rem): Custom size";
        let output = apply_block_decorations(input);
        assert!(output.contains("style=\"font-size: 3rem\""));
    }

    #[test]
    fn test_text_align() {
        let input = "CENTER: Centered text";
        let output = apply_block_decorations(input);
        assert!(output.contains("class=\"text-center\""));
    }

    #[test]
    fn test_truncate() {
        let input = "TRUNCATE: Long text that will be truncated";
        let output = apply_block_decorations(input);
        assert!(output.contains("class=\"text-truncate\""));
    }

    #[test]
    fn test_compound_decorations() {
        let input = "SIZE(1.5): COLOR(primary): CENTER: Styled text";
        let output = apply_block_decorations(input);
        assert!(output.contains("fs-4"));
        assert!(output.contains("text-primary"));
        assert!(output.contains("text-center"));
        assert!(output.contains("Styled text"));
    }

    #[test]
    fn test_vertical_align() {
        let input = "TOP: Top aligned";
        let output = apply_block_decorations(input);
        assert!(output.contains("class=\"align-top\""));
    }

    #[test]
    fn test_compound_with_truncate() {
        let input = "TRUNCATE: RIGHT: Truncated right text";
        let output = apply_block_decorations(input);
        assert!(output.contains("text-truncate"));
        assert!(output.contains("text-end"));
    }
}
