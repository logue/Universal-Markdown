//! Inline decoration functions for LukiWiki
//!
//! Provides inline formatting functions:
//! - &color(fg,bg){text};
//! - &size(rem){text};
//! - &sup(text); (superscript)
//! - &sub(text); (subscript)
//! - &lang(locale){text};
//! - &abbr(text){description};
//! - &ruby(reading){text}; (furigana)
//! - Semantic HTML elements: dfn, kbd, samp, var, cite, q, small, u
//! - &time(datetime){text};
//! - &data(value){text};
//! - &bdi(text); &bdo(dir){text};
//! - &wbr; (word break opportunity)
//! - %%text%% → <s>text</s> (strikethrough)

use once_cell::sync::Lazy;
use regex::Regex;

// Badge pattern with optional link support
static INLINE_BADGE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&badge\(([^)]+?)\)\{([^}]+?)\};").unwrap());

// Link pattern for detecting [text](url) inside badge content
static MARKDOWN_LINK: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap());

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

static INLINE_RUBY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&ruby\(([^)]+?)\)\{([^}]+?)\};").unwrap());

// Semantic HTML elements - simple wrapper tags
static INLINE_DFN: Lazy<Regex> = Lazy::new(|| Regex::new(r"&dfn\(([^)]+?)\);").unwrap());
static INLINE_KBD: Lazy<Regex> = Lazy::new(|| Regex::new(r"&kbd\(([^)]+?)\);").unwrap());
static INLINE_SAMP: Lazy<Regex> = Lazy::new(|| Regex::new(r"&samp\(([^)]+?)\);").unwrap());
static INLINE_VAR: Lazy<Regex> = Lazy::new(|| Regex::new(r"&var\(([^)]+?)\);").unwrap());
static INLINE_CITE: Lazy<Regex> = Lazy::new(|| Regex::new(r"&cite\(([^)]+?)\);").unwrap());
static INLINE_Q: Lazy<Regex> = Lazy::new(|| Regex::new(r"&q\(([^)]+?)\);").unwrap());
static INLINE_SMALL: Lazy<Regex> = Lazy::new(|| Regex::new(r"&small\(([^)]+?)\);").unwrap());
static INLINE_U: Lazy<Regex> = Lazy::new(|| Regex::new(r"&u\(([^)]+?)\);").unwrap());

// Elements with attributes
static INLINE_TIME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&time\(([^)]+?)\)\{([^}]+?)\};").unwrap());
static INLINE_DATA: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&data\(([^)]+?)\)\{([^}]+?)\};").unwrap());

// Bidirectional text
static INLINE_BDI: Lazy<Regex> = Lazy::new(|| Regex::new(r"&bdi\(([^)]+?)\);").unwrap());
static INLINE_BDO: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&bdo\(([^)]+?)\)\{([^}]+?)\};").unwrap());

// Word break opportunity (self-closing)
static INLINE_WBR: Lazy<Regex> = Lazy::new(|| Regex::new(r"&wbr;").unwrap());

/// Regex for LukiWiki strikethrough: %%text%% → <s>text</s>
static LUKIWIKI_STRIKETHROUGH: Lazy<Regex> = Lazy::new(|| Regex::new(r"%%([^%]+)%%").unwrap());

/// Map font size value to Bootstrap class or inline style
fn map_font_size(value: &str) -> (bool, String) {
    // Check if value has unit (rem, em, px, etc.)
    if value.contains("rem") || value.contains("em") || value.contains("px") {
        return (false, value.to_string()); // Return as inline style
    }

    // Map to Bootstrap fs-* classes (unitless values)
    let class = match value {
        "2.5" => "fs-1",
        "2" | "2.0" => "fs-2",
        "1.75" => "fs-3",
        "1.5" => "fs-4",
        "1.25" => "fs-5",
        "0.875" => "fs-6",
        _ => return (false, format!("{}rem", value)), // Custom value as inline style
    };

    (true, class.to_string())
}

/// Map color value to Bootstrap class or inline style
fn map_color(value: &str, is_background: bool) -> (bool, String) {
    let trimmed = value.trim();

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
            return (true, format!("{}-{}", prefix, trimmed));
        }
    }

    // Otherwise, return as inline style value
    (false, trimmed.to_string())
}

/// Map badge type to Bootstrap badge classes
fn map_badge_type(badge_type: &str) -> String {
    // Check if it's a pill badge
    if badge_type.ends_with("-pill") {
        let color = badge_type.trim_end_matches("-pill");
        format!("badge rounded-pill bg-{}", color)
    } else {
        // Regular badge
        format!("badge bg-{}", badge_type)
    }
}

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

    // Apply %%text%% → <s>text</s> (LukiWiki strikethrough)
    result = LUKIWIKI_STRIKETHROUGH
        .replace_all(&result, "<s>$1</s>")
        .to_string();

    // Apply &badge(type){text}; with optional link support
    result = INLINE_BADGE
        .replace_all(&result, |caps: &regex::Captures| {
            let badge_type = caps.get(1).map_or("", |m| m.as_str());
            let content = caps.get(2).map_or("", |m| m.as_str());
            let badge_class = map_badge_type(badge_type);

            // Check if content contains a Markdown link: [text](url)
            if let Some(link_caps) = MARKDOWN_LINK.captures(content) {
                let text = link_caps.get(1).map_or("", |m| m.as_str());
                let url = link_caps.get(2).map_or("", |m| m.as_str());
                format!("<a href=\"{}\" class=\"{}\">{}</a>", url, badge_class, text)
            } else {
                format!("<span class=\"{}\">{}</span>", badge_class, content)
            }
        })
        .to_string();

    // Apply &color(fg,bg){text}; with Bootstrap support
    result = INLINE_COLOR
        .replace_all(&result, |caps: &regex::Captures| {
            let fg = caps.get(1).map_or("", |m| m.as_str().trim());
            let bg = caps.get(2).map_or("", |m| m.as_str().trim());
            let text = caps.get(3).map_or("", |m| m.as_str());

            let mut classes = Vec::new();
            let mut styles = Vec::new();

            if !fg.is_empty() && fg != "inherit" {
                let (is_class, value) = map_color(fg, false);
                if is_class {
                    classes.push(value);
                } else {
                    styles.push(format!("color: {}", value));
                }
            }

            if !bg.is_empty() && bg != "inherit" {
                let (is_class, value) = map_color(bg, true);
                if is_class {
                    classes.push(value);
                } else {
                    styles.push(format!("background-color: {}", value));
                }
            }

            if classes.is_empty() && styles.is_empty() {
                text.to_string()
            } else {
                let mut attrs = Vec::new();
                if !classes.is_empty() {
                    attrs.push(format!("class=\"{}\"", classes.join(" ")));
                }
                if !styles.is_empty() {
                    attrs.push(format!("style=\"{}\"", styles.join("; ")));
                }
                format!("<span {}>{}</span>", attrs.join(" "), text)
            }
        })
        .to_string();

    // Apply &size(value){text}; with Bootstrap support
    result = INLINE_SIZE
        .replace_all(&result, |caps: &regex::Captures| {
            let size = caps.get(1).map_or("", |m| m.as_str());
            let text = caps.get(2).map_or("", |m| m.as_str());

            let (is_class, value) = map_font_size(size);
            if is_class {
                format!("<span class=\"{}\">{}</span>", value, text)
            } else {
                format!("<span style=\"font-size: {}\">{}</span>", value, text)
            }
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

    // Apply &ruby(reading){text};
    result = INLINE_RUBY
        .replace_all(&result, "<ruby>$2<rp>(</rp><rt>$1</rt><rp>)</rp></ruby>;")
        .to_string();

    // Semantic HTML elements - simple wrappers
    result = INLINE_DFN
        .replace_all(&result, "<dfn>$1</dfn>;")
        .to_string();
    result = INLINE_KBD
        .replace_all(&result, "<kbd>$1</kbd>;")
        .to_string();
    result = INLINE_SAMP
        .replace_all(&result, "<samp>$1</samp>;")
        .to_string();
    result = INLINE_VAR
        .replace_all(&result, "<var>$1</var>;")
        .to_string();
    result = INLINE_CITE
        .replace_all(&result, "<cite>$1</cite>;")
        .to_string();
    result = INLINE_Q.replace_all(&result, "<q>$1</q>;").to_string();
    result = INLINE_SMALL
        .replace_all(&result, "<small>$1</small>;")
        .to_string();
    result = INLINE_U.replace_all(&result, "<u>$1</u>;").to_string();

    // Elements with attributes
    result = INLINE_TIME
        .replace_all(&result, "<time datetime=\"$1\">$2</time>;")
        .to_string();
    result = INLINE_DATA
        .replace_all(&result, "<data value=\"$1\">$2</data>;")
        .to_string();

    // Bidirectional text
    result = INLINE_BDI
        .replace_all(&result, "<bdi>$1</bdi>;")
        .to_string();
    result = INLINE_BDO
        .replace_all(&result, "<bdo dir=\"$1\">$2</bdo>;")
        .to_string();

    // Word break opportunity
    result = INLINE_WBR.replace_all(&result, "<wbr />").to_string();

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
        // 1.5 maps to Bootstrap fs-4 class
        assert!(output.contains("<span class=\"fs-4\">larger</span>"));
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
        // 2 maps to Bootstrap fs-2 class
        assert!(output.contains("fs-2"));
        assert!(output.contains("<sup>superscript</sup>"));
    }

    #[test]
    fn test_lukiwiki_strikethrough() {
        let input = "This is %%strikethrough%% text.";
        let output = apply_inline_decorations(input);
        assert_eq!(output, "This is <s>strikethrough</s> text.");
    }

    #[test]
    fn test_lukiwiki_strikethrough_multiple() {
        let input = "%%first%% and %%second%%";
        let output = apply_inline_decorations(input);
        assert_eq!(output, "<s>first</s> and <s>second</s>");
    }

    #[test]
    fn test_inline_ruby() {
        let input = "&ruby(Ashita){明日};";
        let output = apply_inline_decorations(input);
        assert_eq!(
            output,
            "<ruby>明日<rp>(</rp><rt>Ashita</rt><rp>)</rp></ruby>;"
        );
    }

    #[test]
    fn test_semantic_elements() {
        let input = "&dfn(term); &kbd(Ctrl); &samp(output); &var(x);";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<dfn>term</dfn>"));
        assert!(output.contains("<kbd>Ctrl</kbd>"));
        assert!(output.contains("<samp>output</samp>"));
        assert!(output.contains("<var>x</var>"));
    }

    #[test]
    fn test_cite_q_small_u() {
        let input = "&cite(Book Title); &q(quote); &small(note); &u(underline);";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<cite>Book Title</cite>"));
        assert!(output.contains("<q>quote</q>"));
        assert!(output.contains("<small>note</small>"));
        assert!(output.contains("<u>underline</u>"));
    }

    #[test]
    fn test_time_and_data() {
        let input = "&time(2026-01-26){today}; &data(12345){value};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<time datetime=\"2026-01-26\">today</time>"));
        assert!(output.contains("<data value=\"12345\">value</data>"));
    }

    #[test]
    fn test_bidirectional_text() {
        let input = "&bdi(مرحبا); &bdo(rtl){right-to-left};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<bdi>مرحبا</bdi>"));
        assert!(output.contains("<bdo dir=\"rtl\">right-to-left</bdo>"));
    }

    #[test]
    fn test_wbr() {
        let input = "Very&wbr;Long&wbr;Word";
        let output = apply_inline_decorations(input);
        assert_eq!(output, "Very<wbr />Long<wbr />Word");
    }

    #[test]
    fn test_badge_basic() {
        let input = "&badge(primary){New};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<span class=\"badge bg-primary\">New</span>"));
    }

    #[test]
    fn test_badge_pill() {
        let input = "&badge(success-pill){Active};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("badge rounded-pill"));
        assert!(output.contains("bg-success"));
    }

    #[test]
    fn test_badge_with_link() {
        let input = "&badge(danger){[Error](/error)};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<a href=\"/error\" class=\"badge bg-danger\">Error</a>"));
    }

    #[test]
    fn test_color_bootstrap_class() {
        let input = "&color(primary){Primary text};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("class=\"text-primary\""));
    }

    #[test]
    fn test_color_custom_value() {
        let input = "&color(#FF0000){Custom red};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("style=\"color: #FF0000\""));
    }

    #[test]
    fn test_size_bootstrap_class() {
        let input = "&size(1.5){Medium text};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("class=\"fs-4\""));
    }

    #[test]
    fn test_size_custom_value() {
        let input = "&size(3rem){Custom size};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("style=\"font-size: 3rem\""));
    }
}
