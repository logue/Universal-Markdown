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

// Block placement pattern for tables and plugins (must start on new line)
static BLOCK_PLACEMENT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?m)^(LEFT|CENTER|RIGHT|JUSTIFY):\n((?:\|[^\n]*\|(?:\n|$))+|@\w+(?:\([^)]*\))?\{[^}]*\})",
    )
    .unwrap()
});

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
    map_color_with_options(value, is_background, false)
}

fn map_color_with_options(
    value: &str,
    is_background: bool,
    allow_hex_colors: bool,
) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "inherit" {
        return None;
    }

    let colors = [
        "blue", "indigo", "purple", "pink", "red", "orange", "yellow", "green", "teal", "cyan",
    ];
    for color in colors {
        if trimmed == color {
            let prefix = if is_background { "bg" } else { "text" };
            return Some(format!("{}-{}", prefix, trimmed));
        }
    }

    if allow_hex_colors
        && trimmed.starts_with('#')
        && (trimmed.len() == 4 || trimmed.len() == 7)
        && trimmed[1..].chars().all(|c| c.is_ascii_hexdigit())
    {
        return Some(trimmed.to_string());
    }

    None
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
fn parse_prefixes_with_options(line: &str, allow_hex_colors: bool) -> (BlockDecoration, String) {
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
        decoration.fg_color = map_color_with_options(
            fg,
            false,
            allow_hex_colors,
        );
        decoration.bg_color = map_color_with_options(
            bg,
            true,
            allow_hex_colors,
        );
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

fn parse_prefixes(line: &str) -> (BlockDecoration, String) {
    parse_prefixes_with_options(line, false)
}

pub fn apply_block_decorations_with_options(
    html: &str,
    allow_hex_colors: bool,
) -> String {
    let mut result = String::new();

    for line in html.lines() {
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
            let (decoration, content) = parse_prefixes_with_options(line, allow_hex_colors);
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

pub fn apply_block_decorations(html: &str) -> String {
    apply_block_decorations_with_options(html, false)
}

/// Apply block placement prefixes to tables and block plugins
///
/// Handles LEFT:/CENTER:/RIGHT:/JUSTIFY: prefixes followed by newline
/// for UMD tables and block plugins (@function).
///
/// # Arguments
///
/// * `html` - The HTML content to process
///
/// # Returns
///
/// HTML with block placement applied (Bootstrap utility classes)
pub fn apply_block_placement(html: &str) -> String {
    fn merge_class_attr(tag_html: &str, extra_classes: &str) -> String {
        let class_re = Regex::new(r#"class=\"([^\"]*)\""#).unwrap();

        if let Some(caps) = class_re.captures(tag_html) {
            let existing = caps.get(1).map_or("", |m| m.as_str());
            let mut merged: Vec<String> = if existing.trim().is_empty() {
                Vec::new()
            } else {
                existing.split_whitespace().map(|s| s.to_string()).collect()
            };

            for class_name in extra_classes.split_whitespace() {
                if !merged.iter().any(|c| c == class_name) {
                    merged.push(class_name.to_string());
                }
            }

            class_re
                .replace(tag_html, format!(r#"class=\"{}\""#, merged.join(" ")))
                .to_string()
        } else {
            tag_html.replacen('>', &format!(r#" class=\"{}\">"#, extra_classes), 1)
        }
    }

    fn placement_class_for_block(placement: &str) -> &'static str {
        match placement {
            "LEFT" => "w-auto",
            "CENTER" => "w-auto mx-auto",
            "RIGHT" => "w-auto ms-auto me-0",
            "JUSTIFY" => "w-100",
            _ => "",
        }
    }

    let media_block_placement = Regex::new(
        r#"(?s)<p>\s*(LEFT|CENTER|RIGHT|JUSTIFY):\s*\n\s*(<picture[\s\S]*?</picture>|<video[\s\S]*?</video>|<audio[\s\S]*?</audio>|<a href="[^"]+" download class="download-link[^"]*"[^>]*>[\s\S]*?</a>)\s*</p>"#,
    )
    .unwrap();

    let with_media_placement = media_block_placement
        .replace_all(html, |caps: &regex::Captures| {
            let placement = &caps[1];
            let media = &caps[2];

            let wrapper_class = match placement {
                "LEFT" => "ms-0 me-auto",
                "CENTER" => "mx-auto",
                "RIGHT" => "ms-auto me-0",
                "JUSTIFY" => "w-100",
                _ => "",
            };

            if wrapper_class.is_empty() {
                format!("<figure>\n{}\n</figure>", media)
            } else {
                format!("<figure class=\"{}\">\n{}\n</figure>", wrapper_class, media)
            }
        })
        .to_string();

    let table_and_plugin_placement_in_paragraph = Regex::new(
        r#"(?s)<p>\s*(LEFT|CENTER|RIGHT|JUSTIFY):\s*\n\s*(<(?:table|template)\b[^>]*>[\s\S]*?</(?:table|template)>)\s*</p>"#,
    )
    .unwrap();

    let with_table_and_plugin_placement_in_paragraph = table_and_plugin_placement_in_paragraph
        .replace_all(&with_media_placement, |caps: &regex::Captures| {
            let placement = &caps[1];
            let block = &caps[2];
            let placement_class = placement_class_for_block(placement);

            if block.starts_with("<table") {
                return merge_class_attr(block, placement_class);
            }

            if block.starts_with("<template") && block.contains("umd-plugin") {
                return merge_class_attr(block, placement_class);
            }

            block.to_string()
        })
        .to_string();

    let table_and_plugin_placement = Regex::new(
        r#"(?s)<p>\s*(LEFT|CENTER|RIGHT|JUSTIFY):\s*</p>\s*(<(?:table|template)\b[^>]*>[\s\S]*?</(?:table|template)>)"#,
    )
    .unwrap();

    let with_table_and_plugin_placement = table_and_plugin_placement
        .replace_all(
            &with_table_and_plugin_placement_in_paragraph,
            |caps: &regex::Captures| {
                let placement = &caps[1];
                let block = &caps[2];
                let placement_class = placement_class_for_block(placement);

                if block.starts_with("<table") {
                    return merge_class_attr(block, placement_class);
                }

                if block.starts_with("<template") && block.contains("umd-plugin") {
                    return merge_class_attr(block, placement_class);
                }

                block.to_string()
            },
        )
        .to_string();

    BLOCK_PLACEMENT
        .replace_all(
            &with_table_and_plugin_placement,
            |caps: &regex::Captures| {
                let placement = &caps[1];
                let content = &caps[2];

                let wrapper_class = placement_class_for_block(placement);

                // Wrap table or plugin in div with appropriate class
                if content.starts_with('|') {
                    // UMD table
                    format!("<div class=\"{}\">\n{}</div>", wrapper_class, content)
                } else if content.starts_with('@') {
                    // Block plugin
                    format!("<div class=\"{}\">\n{}</div>", wrapper_class, content)
                } else {
                    content.to_string()
                }
            },
        )
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_bootstrap_class() {
        let input = "COLOR(blue): Blue text";
        let output = apply_block_decorations_with_options(input, false);
        assert!(output.contains("class=\"text-blue\""));
        assert!(output.contains("Blue text"));
    }

    #[test]
    fn test_color_custom_value() {
        let input = "COLOR(#FF0000): Custom red";
        let output = apply_block_decorations_with_options(input, true);
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
        let input = "SIZE(1.5): COLOR(blue): CENTER: Styled text";
        let output = apply_block_decorations_with_options(input, false);
        assert!(output.contains("fs-4"));
        assert!(output.contains("text-blue"));
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

    #[test]
    fn test_block_placement_left() {
        let input = "LEFT:\n|Header|\n|Cell|";
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<div class="w-auto">"#));
        assert!(output.contains("|Header|"));
    }

    #[test]
    fn test_block_placement_center() {
        let input = "CENTER:\n|Header|\n|Cell|";
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<div class="w-auto mx-auto">"#));
    }

    #[test]
    fn test_block_placement_right() {
        let input = "RIGHT:\n|Header|\n|Cell|";
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<div class="w-auto ms-auto me-0">"#));
    }

    #[test]
    fn test_block_placement_justify() {
        let input = "JUSTIFY:\n|Header|\n|Cell|";
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<div class="w-100">"#));
    }

    #[test]
    fn test_block_placement_plugin() {
        let input = "CENTER:\n@youtube{video_id}";
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<div class="w-auto mx-auto">"#));
        assert!(output.contains("@youtube"));
    }

    #[test]
    fn test_block_placement_right_media() {
        let input = r#"<p>RIGHT:
<picture>
  <img src="image.png" alt="alt" title="Title" />
</picture></p>"#;
        let output = apply_block_placement(input);
        assert!(output.contains(r#"<figure class="ms-auto me-0">"#));
        assert!(output.contains("<picture>"));
        assert!(!output.contains("RIGHT:"));
    }
}
