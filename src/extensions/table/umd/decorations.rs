//! Cell decoration support for UMD tables
//!
//! Provides support for:
//! - COLOR(fg,bg): Cell foreground and background colors
//! - SIZE(value): Font size adjustments
//! - Alignment prefixes: TOP:, MIDDLE:, BOTTOM:, CENTER:, etc.

use super::parser::Cell;
use regex::Regex;

/// Parse cell content for decorations and markers
pub fn parse_cell_content(cell: &mut Cell) {
    let content = &cell.content;

    // Check for colspan marker: |> - mark but don't remove yet
    if content == "|>" || content.ends_with(" |>") {
        // Keep the marker for process_cell_spanning
        return;
    }

    // Check for rowspan marker: |^
    if content == "|^" {
        // Keep the marker for process_cell_spanning
        return;
    }

    // Parse decoration prefixes
    let mut remaining = content.clone();

    // Check for header marker: ~
    if remaining.starts_with('~') {
        cell.is_header = true;
        remaining = remaining.strip_prefix('~').unwrap().trim().to_string();
    }

    // Parse COLOR(fg,bg):
    let color_pattern = Regex::new(r"^COLOR\(([^)]*)\):\s*(.*)$").unwrap();
    if let Some(caps) = color_pattern.captures(&remaining) {
        let args = caps[1].to_string();
        remaining = caps[2].to_string();

        let parts: Vec<&str> = args.split(',').collect();
        let fg = parts.get(0).map_or("", |s| s.trim());
        let bg = parts.get(1).map_or("", |s| s.trim());

        if !fg.is_empty() && fg != "inherit" {
            if is_bootstrap_color(fg) {
                cell.classes.push(format!("text-{}", fg));
            } else {
                cell.styles.push(format!("color: {}", fg));
            }
        }

        if !bg.is_empty() && bg != "inherit" {
            if is_bootstrap_color(bg) {
                cell.classes.push(format!("bg-{}", bg));
            } else {
                cell.styles.push(format!("background-color: {}", bg));
            }
        }
    }

    // Parse SIZE(value):
    let size_pattern = Regex::new(r"^SIZE\(([^)]+)\):\s*(.*)$").unwrap();
    if let Some(caps) = size_pattern.captures(&remaining) {
        let value = caps[1].to_string();
        remaining = caps[2].to_string();

        // Check if it's a Bootstrap size
        if let Some(bs_class) = get_bootstrap_size_class(&value) {
            cell.classes.push(bs_class);
        } else {
            // Use inline style
            let size_value =
                if value.contains("rem") || value.contains("em") || value.contains("px") {
                    value.to_string()
                } else {
                    format!("{}rem", value)
                };
            cell.styles.push(format!("font-size: {}", size_value));
        }
    }

    // Parse alignment prefixes
    for (prefix, class) in &[
        ("TOP:", "align-top"),
        ("MIDDLE:", "align-middle"),
        ("BOTTOM:", "align-bottom"),
        ("BASELINE:", "align-baseline"),
        ("RIGHT:", "text-end"),
        ("CENTER:", "text-center"),
        ("LEFT:", "text-start"),
        ("JUSTIFY:", "text-justify"),
    ] {
        if remaining.starts_with(prefix) {
            cell.classes.push(class.to_string());
            remaining = remaining.strip_prefix(prefix).unwrap().trim().to_string();
        }
    }

    // Check for header marker after decoration prefixes: ~
    if remaining.starts_with('~') {
        cell.is_header = true;
        remaining = remaining.strip_prefix('~').unwrap().trim().to_string();
    }

    cell.content = remaining;
}

/// Check if a color is a Bootstrap color name
fn is_bootstrap_color(color: &str) -> bool {
    matches!(
        color,
        "blue"
            | "indigo"
            | "purple"
            | "pink"
            | "red"
            | "orange"
            | "yellow"
            | "green"
            | "teal"
            | "cyan"
    )
}

/// Get Bootstrap size class for a given value
fn get_bootstrap_size_class(value: &str) -> Option<String> {
    let val: f32 = value.parse().ok()?;

    let class = if val >= 2.5 {
        "fs-1"
    } else if val >= 2.0 {
        "fs-2"
    } else if val >= 1.75 {
        "fs-3"
    } else if val >= 1.5 {
        "fs-4"
    } else if val >= 1.25 {
        "fs-5"
    } else if val >= 0.875 {
        "fs-6"
    } else {
        return None;
    };

    Some(class.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_decoration() {
        let mut cell = Cell::new("COLOR(red,blue): Text".to_string(), false);
        parse_cell_content(&mut cell);

        assert_eq!(cell.content, "Text");
        assert!(cell.classes.contains(&"text-red".to_string()));
        assert!(cell.classes.contains(&"bg-blue".to_string()));
        assert!(!cell.is_header);
    }

    #[test]
    fn test_header_marker() {
        let mut cell = Cell::new("~Header Text".to_string(), false);
        parse_cell_content(&mut cell);

        assert_eq!(cell.content, "Header Text");
        assert!(cell.is_header);
    }

    #[test]
    fn test_header_marker_with_decoration() {
        let mut cell = Cell::new("COLOR(red): ~Header".to_string(), false);
        parse_cell_content(&mut cell);

        assert_eq!(cell.content, "Header");
        assert!(cell.is_header);
        assert!(cell.classes.contains(&"text-red".to_string()));
    }

    #[test]
    fn test_size_decoration() {
        let mut cell = Cell::new("SIZE(1.5): Text".to_string(), false);
        parse_cell_content(&mut cell);

        assert_eq!(cell.content, "Text");
        assert!(cell.classes.contains(&"fs-4".to_string()));
    }

    #[test]
    fn test_alignment_decoration() {
        let mut cell = Cell::new("CENTER: Text".to_string(), false);
        parse_cell_content(&mut cell);

        assert_eq!(cell.content, "Text");
        assert!(cell.classes.contains(&"text-center".to_string()));
    }

    #[test]
    fn test_bootstrap_color_check() {
        assert!(is_bootstrap_color("blue"));
        assert!(is_bootstrap_color("cyan"));
        assert!(!is_bootstrap_color("primary"));
        assert!(!is_bootstrap_color("custom-color"));
    }
}
