//! Second-pass handling for standard inline plugins, plus LukiWiki-style
//! shorthand (strikethrough, spoiler) that isn't part of the `&function()`
//! syntax at all.
//!
//! The primary place standard inline plugins (`&color()`, `&size()`,
//! `&ruby()`, `&spoiler()`, etc.) are recognized is
//! `conflict_resolver::convert_standard_inline_plugin_to_html` and its
//! `argsonly`/`noargs` siblings, invoked while restoring the markers that
//! `plugin_markers::protect_inline_plugins` lays down before Markdown
//! parsing. That marker regex allows one level of `{...}` nesting in a
//! call's content, so a standard plugin nested inside *another* standard
//! plugin's content (e.g. `&color(blue){outer &abbr(t){d}; end};`) survives
//! the first pass as literal, unexpanded text embedded in the outer HTML.
//! This module's regexes are that second sweep — they run once more, after
//! the first pass, to catch exactly that leftover case.
//!
//! Because both passes must agree on what a given call produces, the
//! functions here that have real mapping logic (`&color()`'s palette,
//! `&size()`'s keyword restriction) call back into `conflict_resolver`'s
//! `map_color_value_with_options` / `map_font_size_value` rather than
//! re-deriving it — don't reintroduce a local copy.
//!
//! Functions handled by this second pass:
//! - &color(fg,bg){text};
//! - &size(xs|sm|lg|xl){text}; (or an arbitrary rem/px value, opt-in only)
//! - &sup(text); (superscript)
//! - &sub(text); (subscript)
//! - &lang(locale){text};
//! - &abbr(text){description};
//! - &ruby(reading){text}; (furigana)
//! - Semantic HTML elements: dfn, kbd, samp, var, cite, q, small
//! - &time(datetime){text};
//! - &data(value){text};
//! - &bdi(text); &bdo(dir){text};
//! - &wbr; (word break opportunity)
//! - &br; (manual line break)
//! - &spoiler(text); / &spoiler{text}; → <span class="umd-spoiler">
//!
//! Not standard plugins, and always live (not marker-protected, so there's
//! only ever one pass for these):
//! - %%text%% → <s>text</s> (strikethrough)
//! - ||text|| → <span class="umd-spoiler"> (Discord-style spoiler)
//!
//! Note: For underline, use Discord-style __text__ syntax instead

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

// Manual line break (self-closing) - mainly for table cells where trailing spaces don't work
static INLINE_BR: Lazy<Regex> = Lazy::new(|| Regex::new(r"&br;").unwrap());

/// Regex for LukiWiki strikethrough: %%text%% → <s>text</s>
static LUKIWIKI_STRIKETHROUGH: Lazy<Regex> = Lazy::new(|| Regex::new(r"%%([^%]+)%%").unwrap());

/// Regex for Discord-style spoiler: || text || → <span class="umd-spoiler">text</span>
static DISCORD_SPOILER: Lazy<Regex> = Lazy::new(|| Regex::new(r"\|\|([^|]+)\|\|").unwrap());

/// Regex for the standard `&spoiler()` plugin, second pass only (see module
/// docs): &spoiler(text); or &spoiler{text};
static INLINE_SPOILER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&spoiler(?:\(([^)]+?)\)|\{([^}]+?)\});").unwrap());

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
    apply_inline_decorations_with_limit_and_options(html, Some(5), false, false)
}

pub fn apply_inline_decorations_with_limit_and_options(
    html: &str,
    max_inline_nesting: Option<usize>,
    allow_hex_colors: bool,
    allow_custom_font_size: bool,
) -> String {
    let mut result = html.to_string();

    // Decode HTML entities for UMD inline syntax
    // Comrak escapes & to &amp;, which prevents our regexes from matching
    // We need to convert &amp; back to & for UMD syntax only
    result = result.replace("&amp;color(", "&color(");
    result = result.replace("&amp;size(", "&size(");
    result = result.replace("&amp;sup(", "&sup(");
    result = result.replace("&amp;sub(", "&sub(");
    result = result.replace("&amp;lang(", "&lang(");
    result = result.replace("&amp;abbr(", "&abbr(");
    result = result.replace("&amp;ruby(", "&ruby(");
    result = result.replace("&amp;spoiler(", "&spoiler(");
    result = result.replace("&amp;spoiler{", "&spoiler{");
    result = result.replace("&amp;dfn(", "&dfn(");
    result = result.replace("&amp;kbd(", "&kbd(");
    result = result.replace("&amp;samp(", "&samp(");
    result = result.replace("&amp;var(", "&var(");
    result = result.replace("&amp;cite(", "&cite(");
    result = result.replace("&amp;q(", "&q(");
    result = result.replace("&amp;small(", "&small(");
    result = result.replace("&amp;time(", "&time(");
    result = result.replace("&amp;data(", "&data(");
    result = result.replace("&amp;bdi(", "&bdi(");
    result = result.replace("&amp;bdo(", "&bdo(");
    result = result.replace("&amp;wbr", "&wbr");
    result = result.replace("&amp;br", "&br");

    if let Some(limit) = max_inline_nesting.filter(|limit| *limit > 0) {
        result = neutralize_over_limit_inline_decorations(&result, limit);
    }

    // Apply %%text%% → <s>text</s> (LukiWiki strikethrough)
    result = LUKIWIKI_STRIKETHROUGH
        .replace_all(&result, "<s>$1</s>")
        .to_string();

    // Apply || text || → <span class="umd-spoiler">text</span> (Discord spoiler)
    result = DISCORD_SPOILER
        .replace_all(
            &result,
            r#"<span class="umd-spoiler" role="button" tabindex="0" aria-expanded="false">$1</span>"#,
        )
        .to_string();

    // Second pass for &spoiler(text); or &spoiler{text}; (see module docs)
    result = INLINE_SPOILER
        .replace_all(&result, |caps: &regex::Captures| {
            let text = caps.get(1).or_else(|| caps.get(2)).map_or("", |m| m.as_str());
            format!(r#"<span class="umd-spoiler" role="button" tabindex="0" aria-expanded="false">{}</span>"#, text)
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
                if let Some((is_class, value)) =
                    super::conflict_resolver::map_color_value_with_options(
                        fg,
                        false,
                        allow_hex_colors,
                    )
                {
                    if is_class {
                        classes.push(value);
                    } else {
                        styles.push(format!("color: {}", value));
                    }
                }
            }

            if !bg.is_empty() && bg != "inherit" {
                if let Some((is_class, value)) =
                    super::conflict_resolver::map_color_value_with_options(
                        bg,
                        true,
                        allow_hex_colors,
                    )
                {
                    if is_class {
                        classes.push(value);
                    } else {
                        styles.push(format!("background-color: {}", value));
                    }
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

    // Apply &size(xs|sm|lg|xl){text}; (or an arbitrary value, opt-in only)
    result = INLINE_SIZE
        .replace_all(&result, |caps: &regex::Captures| {
            let size = caps.get(1).map_or("", |m| m.as_str());
            let text = caps.get(2).map_or("", |m| m.as_str());

            match super::conflict_resolver::map_font_size_value(size, allow_custom_font_size) {
                Some((true, class)) => format!("<span class=\"{}\">{}</span>", class, text),
                Some((false, value)) => {
                    format!("<span style=\"font-size: {}\">{}</span>", value, text)
                }
                None => text.to_string(),
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

    // Manual line break (mainly for table cells)
    result = INLINE_BR.replace_all(&result, "<br />").to_string();

    result
}

pub fn apply_inline_decorations_with_limit(
    html: &str,
    max_inline_nesting: Option<usize>,
) -> String {
    apply_inline_decorations_with_limit_and_options(html, max_inline_nesting, false, false)
}

fn inline_decoration_nesting_depth(input: &str) -> usize {
    let bytes = input.as_bytes();
    let mut i = 0usize;
    let mut stack: Vec<bool> = Vec::new();
    let mut limited_depth = 0usize;
    let mut max_limited_depth = 0usize;

    while i < bytes.len() {
        if let Some((name, next_index)) = parse_inline_block_start(input, i) {
            let limited = is_limited_inline_name(name);
            stack.push(limited);
            if limited {
                limited_depth += 1;
                max_limited_depth = max_limited_depth.max(limited_depth);
            }
            i = next_index;
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'}' && bytes[i + 1] == b';' && !stack.is_empty() {
            if stack.pop().unwrap_or(false) {
                limited_depth = limited_depth.saturating_sub(1);
            }
            i += 2;
            continue;
        }

        i += 1;
    }

    max_limited_depth
}

fn neutralize_over_limit_inline_decorations(input: &str, max_inline_nesting: usize) -> String {
    let bytes = input.as_bytes();
    let mut i = 0usize;
    let mut stack: Vec<bool> = Vec::new();
    let mut limited_depth = 0usize;
    let mut output = String::with_capacity(input.len() + 16);

    while i < bytes.len() {
        if let Some((name, next_index)) = parse_inline_block_start(input, i) {
            let limited = is_limited_inline_name(name);
            let over_limit = limited && limited_depth >= max_inline_nesting;

            if over_limit {
                if let Some(end_index) = find_inline_block_end(input, i) {
                    let escaped = html_escape::encode_safe(&input[i..end_index])
                        .replace('{', "&#123;")
                        .replace('}', "&#125;");
                    output.push_str(r#"<span class="umd-error-deep-recursive">"#);
                    output.push_str(&escaped);
                    output.push_str("</span>");
                    i = end_index;
                    continue;
                }

                let escaped = html_escape::encode_safe("&");
                output.push_str(escaped.as_ref());
                i += 1;
                continue;
            }

            output.push_str(&input[i..next_index]);
            stack.push(limited);
            if limited {
                limited_depth += 1;
            }

            i = next_index;
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'}' && bytes[i + 1] == b';' && !stack.is_empty() {
            if stack.pop().unwrap_or(false) {
                limited_depth = limited_depth.saturating_sub(1);
            }
        }

        let ch = input[i..]
            .chars()
            .next()
            .expect("valid UTF-8 char boundary");
        output.push(ch);
        i += ch.len_utf8();
    }

    output
}

fn find_inline_block_end(input: &str, start: usize) -> Option<usize> {
    let mut i = parse_inline_block_start(input, start)?.1;
    let bytes = input.as_bytes();
    let mut depth = 1usize;

    while i < bytes.len() {
        if parse_inline_block_start(input, i).is_some() {
            depth += 1;
            i = parse_inline_block_start(input, i)?.1;
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'}' && bytes[i + 1] == b';' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return Some(i);
            }
            continue;
        }

        let ch = input[i..].chars().next()?;
        i += ch.len_utf8();
    }

    None
}

fn parse_inline_block_start(input: &str, start: usize) -> Option<(&str, usize)> {
    let bytes = input.as_bytes();
    if bytes.get(start).copied() != Some(b'&') {
        return None;
    }

    let mut j = start + 1;
    while j < bytes.len() && bytes[j].is_ascii_alphabetic() {
        j += 1;
    }

    if j <= start + 1 {
        return None;
    }

    let name = &input[start + 1..j];

    if bytes.get(j).copied() == Some(b'{') {
        return Some((name, j + 1));
    }

    if bytes.get(j).copied() != Some(b'(') {
        return None;
    }

    let mut k = j + 1;
    while k < bytes.len() && bytes[k] != b')' {
        k += 1;
    }

    if k < bytes.len() && bytes.get(k + 1).copied() == Some(b'{') {
        return Some((name, k + 2));
    }

    None
}

fn is_limited_inline_name(name: &str) -> bool {
    matches!(
        name,
        "color" | "size" | "lang" | "abbr" | "ruby" | "time" | "data" | "bdo" | "spoiler"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_color_blue() {
        let result =
            super::super::conflict_resolver::map_color_value_with_options("blue", false, false);
        assert!(
            result.is_some(),
            "blue should be recognized as a valid color"
        );
        let (is_class, class_or_style) = result.unwrap();
        assert!(is_class, "blue should be recognized as a umd-color-* class");
        assert_eq!(
            class_or_style, "umd-color-blue",
            "Expected umd-color-blue, got {}",
            class_or_style
        );
    }

    #[test]
    fn test_map_color_hex() {
        let result =
            super::super::conflict_resolver::map_color_value_with_options("#FF5733", false, true);
        assert!(
            result.is_some(),
            "#FF5733 should be recognized as a valid HEX color"
        );
        let (is_class, value) = result.unwrap();
        assert!(!is_class, "HEX color should not be a umd-color-* class");
        assert_eq!(value, "#FF5733", "Expected #FF5733, got {}", value);
    }

    #[test]
    fn test_map_color_invalid_html_name() {
        // HTML color names like "white" or "black" are not in the UMD
        // palette and should be rejected
        let result =
            super::super::conflict_resolver::map_color_value_with_options("white", false, false);
        assert!(
            result.is_none(),
            "HTML color name 'white' should be rejected"
        );

        let result =
            super::super::conflict_resolver::map_color_value_with_options("black", false, false);
        assert!(
            result.is_none(),
            "HTML color name 'black' should be rejected"
        );
    }

    #[test]
    fn test_inline_color_foreground() {
        let input = "This is &color(red){red text};";
        let output = apply_inline_decorations(input);
        assert!(output.contains(r#"<span class="umd-color-red">red text</span>"#));
    }

    #[test]
    fn test_inline_color_background() {
        let input = "&color(,yellow){yellow bg};";
        let output = apply_inline_decorations(input);
        assert!(output.contains(r#"<span class="umd-bg-yellow">yellow bg</span>"#));
    }

    #[test]
    fn test_inline_color_both() {
        let input = "&color(cyan,yellow){cyan on yellow};";
        let output = apply_inline_decorations(input);
        assert!(
            output.contains(r#"class="umd-color-cyan umd-bg-yellow""#),
            "Expected both colors as classes, got: {}",
            output
        );
    }

    #[test]
    fn test_inline_color_invalid() {
        // white and black are not in the UMD palette, so they should be rejected
        let input = "&color(white,black){white on black};";
        let output = apply_inline_decorations(input);
        // Invalid colors should be ignored, text remains as-is
        assert_eq!(
            output, "white on black",
            "Invalid colors should be ignored, got: {}",
            output
        );
    }

    #[test]
    fn test_inline_color_hex() {
        // Test with HEX color
        let input = "&color(#FF5733){Custom hex color};";
        let output = apply_inline_decorations_with_limit_and_options(input, Some(5), true, false);
        assert!(
            output.contains(r#"style="color: #FF5733""#),
            "Expected HEX color as inline style, got: {}",
            output
        );
    }

    #[test]
    fn test_inline_size_keyword() {
        let input = "&size(lg){larger};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<span class=\"umd-text-size-lg\">larger</span>"));
    }

    #[test]
    fn test_inline_size_custom_value_rejected_by_default() {
        let input = "&size(1.5){larger};";
        let output = apply_inline_decorations(input);
        assert_eq!(output, "larger");
    }

    #[test]
    fn test_inline_size_custom_value_allowed_when_enabled() {
        let input = "&size(1.5){larger};";
        let output = apply_inline_decorations_with_limit_and_options(input, Some(5), false, true);
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
        let input = "&color(red){Red}; and &size(lg){Big}; and &sup(superscript);";
        let output = apply_inline_decorations(input);
        assert!(output.contains("umd-color-red"));
        assert!(output.contains("umd-text-size-lg"));
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
    fn test_cite_q_small() {
        let input = "&cite(Book Title); &q(quote); &small(note);";
        let output = apply_inline_decorations(input);
        assert!(output.contains("<cite>Book Title</cite>"));
        assert!(output.contains("<q>quote</q>"));
        assert!(output.contains("<small>note</small>"));
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
    fn test_br() {
        let input = "Line 1&br;Line 2&br;Line 3";
        let output = apply_inline_decorations(input);
        assert_eq!(output, "Line 1<br />Line 2<br />Line 3");
    }

    #[test]
    fn test_inline_nesting_depth_detection() {
        let input = "&color(blue){&abbr(text){content};};";
        assert_eq!(inline_decoration_nesting_depth(input), 2);
    }

    #[test]
    fn test_inline_nesting_depth_ignores_plugin_names() {
        let input = "&color(blue){&plugin(arg){content};};";
        assert_eq!(inline_decoration_nesting_depth(input), 1);
    }

    #[test]
    fn test_inline_nesting_limit_blocks_expansion_when_exceeded() {
        let input = "&color(blue){&abbr(text){content};};";
        let output = apply_inline_decorations_with_limit(input, Some(1));
        assert!(output.contains(r#"<span class="umd-color-blue">"#));
        assert!(output.contains(
            r#"<span class="umd-error-deep-recursive">&amp;abbr(text)&#123;content&#125;;</span>"#
        ));
        assert!(!output.contains("<abbr"));
    }

    #[test]
    fn test_inline_nesting_limit_disables_only_exceeded_marker() {
        let input = "&color(red){ok}; and &color(blue){&abbr(text){content};};";
        let output = apply_inline_decorations_with_limit(input, Some(1));
        assert!(output.contains(r#"<span class="umd-color-red">ok</span>"#));
        assert!(output.contains(
            r#"<span class="umd-error-deep-recursive">&amp;abbr(text)&#123;content&#125;;</span>"#
        ));
        assert!(!output.contains("<abbr title="));
    }

    #[test]
    fn test_color_bootstrap_class() {
        let input = "&color(blue){Blue text};";
        let output = apply_inline_decorations_with_limit_and_options(input, Some(5), false, false);
        assert!(output.contains("class=\"umd-color-blue\""));
    }

    #[test]
    fn test_color_custom_value() {
        let input = "&color(#FF0000){Red text};";
        let output = apply_inline_decorations_with_limit_and_options(input, Some(5), true, false);
        assert!(output.contains("style=\"color: #FF0000\""));
    }

    #[test]
    fn test_spoiler_discord_syntax() {
        let input = "This is ||hidden text|| in a sentence.";
        let output = apply_inline_decorations(input);
        assert!(output.contains(r#"<span class="umd-spoiler" role="button" tabindex="0" aria-expanded="false">hidden text</span>"#));
    }

    #[test]
    fn test_spoiler_umd_function_parentheses() {
        let input = "This is &spoiler(hidden text); in a sentence.";
        let output = apply_inline_decorations(input);
        assert!(output.contains(r#"<span class="umd-spoiler" role="button" tabindex="0" aria-expanded="false">hidden text</span>"#));
    }

    #[test]
    fn test_spoiler_umd_function_braces() {
        let input = "This is &spoiler{hidden text}; in a sentence.";
        let output = apply_inline_decorations(input);
        assert!(output.contains(r#"<span class="umd-spoiler" role="button" tabindex="0" aria-expanded="false">hidden text</span>"#));
    }

    #[test]
    fn test_multiple_spoilers() {
        let input = "||spoiler1|| and ||spoiler2|| and &spoiler{spoiler3};";
        let output = apply_inline_decorations(input);
        let spoiler_count = output.matches(r#"<span class="umd-spoiler""#).count();
        assert_eq!(spoiler_count, 3);
    }

    #[test]
    fn test_size_keyword_class() {
        let input = "&size(sm){Medium text};";
        let output = apply_inline_decorations(input);
        assert!(output.contains("class=\"umd-text-size-sm\""));
    }

    #[test]
    fn test_size_custom_value_allowed_when_enabled() {
        let input = "&size(3rem){Custom size};";
        let output = apply_inline_decorations_with_limit_and_options(input, Some(5), false, true);
        assert!(output.contains("style=\"font-size: 3rem\""));
    }
}
