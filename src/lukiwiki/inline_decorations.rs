//! Inline decoration functions for LukiWiki
//!
//! Provides inline formatting functions:
//! - &color(fg,bg){text};
//! - &size(rem){text};
//! - &sup(text); (superscript)
//! - &sub(text); (subscript)
//! - &lang(locale){text};
//! - &abbr(text){description};

use once_cell::sync::Lazy;
use regex::Regex;

static INLINE_COLOR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&color\(([^,)]*?)(?:,([^)]*?))?\)\{([^}]+?)\};").unwrap());

static INLINE_SIZE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&size\(([^)]+?)\)\{([^}]+?)\};").unwrap());

static INLINE_SUP: Lazy<Regex> = Lazy::new(|| Regex::new(r"&sup\(([^)]+?)\);").unwrap());

static INLINE_SUB: Lazy<Regex> = Lazy::new(|| Regex::new(r"&sub\(([^)]+?)\);").unwrap());

static INLINE_LANG: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&lang\(([^)]+?)\)\{([^}]+?)\};").unwrap());

static INLINE_ABBR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&abbr\(([^)]+?)\)\{([^}]+?)\};").unwrap());

/// Apply inline decoration functions to HTML
///
/// # Arguments
///
/// * `html` - The HTML content to process
///
/// # Returns
///
/// HTML with inline decorations applied
pub fn apply_inline_decorations(html: &str) -> String {
    let mut result = html.to_string();

    // Apply &color(fg,bg){text};
    result = INLINE_COLOR
        .replace_all(&result, |caps: &regex::Captures| {
            let fg = caps.get(1).map_or("", |m| m.as_str().trim());
            let bg = caps.get(2).map_or("", |m| m.as_str().trim());
            let text = caps.get(3).map_or("", |m| m.as_str());

            let mut styles = Vec::new();
            if !fg.is_empty() && fg != "inherit" {
                styles.push(format!("color: {}", fg));
            }
            if !bg.is_empty() && bg != "inherit" {
                styles.push(format!("background-color: {}", bg));
            }

            if styles.is_empty() {
                text.to_string()
            } else {
                format!("<span style=\"{}\">{}</span>", styles.join("; "), text)
            }
        })
        .to_string();

    // Apply &size(rem){text};
    result = INLINE_SIZE
        .replace_all(&result, |caps: &regex::Captures| {
            let size = caps.get(1).map_or("", |m| m.as_str());
            let text = caps.get(2).map_or("", |m| m.as_str());
            format!("<span style=\"font-size: {}rem\">{}</span>", size, text)
        })
        .to_string();

    // Apply &sup(text);
    result = INLINE_SUP
        .replace_all(&result, "<sup>$1</sup>;")
        .to_string();

    // Apply &sub(text);
    result = INLINE_SUB
        .replace_all(&result, "<sub>$1</sub>;")
        .to_string();

    // Apply &lang(locale){text};
    result = INLINE_LANG
        .replace_all(&result, "<span lang=\"$1\">$2</span>;")
        .to_string();

    // Apply &abbr(text){description};
    result = INLINE_ABBR
        .replace_all(&result, "<abbr title=\"$2\">$1</abbr>;")
        .to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_color_foreground() {
        let input = "This is &color(red){red text};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<span style=\"color: red\">red text</span>"));
    }

    #[test]
    fn test_inline_color_background() {
        let input = "&color(,yellow){yellow bg};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<span style=\"background-color: yellow\">yellow bg</span>"));
    }

    #[test]
    fn test_inline_color_both() {
        let input = "&color(white,black){white on black};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("color: white"));
        assert!(output.contains("background-color: black"));
    }

    #[test]
    fn test_inline_size() {
        let input = "&size(1.5){larger};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<span style=\"font-size: 1.5rem\">larger</span>"));
    }

    #[test]
    fn test_inline_sup() {
        let input = "x&sup(2);";
        let output = apply_inline_decorations(input);
        assert_eq!(output, "x<sup>2</sup>;");
    }

    #[test]
    fn test_inline_sub() {
        let input = "H&sub(2);O";
        let output = apply_inline_decorations(input);
        assert_eq!(output, "H<sub>2</sub>;O");
    }

    #[test]
    fn test_inline_lang() {
        let input = "&lang(en){Hello};";
        let output = apply_inline_decorations(input);
        assert_eq!(output, "<span lang=\"en\">Hello</span>;");
    }

    #[test]
    fn test_inline_abbr() {
        let input = "&abbr(HTML){HyperText Markup Language};";
        let output = apply_inline_decorations(input);
        assert_eq!(
            output,
            "<abbr title=\"HyperText Markup Language\">HTML</abbr>;"
        );
    }

    #[test]
    fn test_multiple_inline_decorations() {
        let input = "&color(red){Red}; and &size(2){Big}; and &sup(superscript);";
        let output = apply_inline_decorations(input);
        assert!(output.contains("color: red"));
        assert!(output.contains("font-size: 2rem"));
        assert!(output.contains("<sup>superscript</sup>"));
    }
}
