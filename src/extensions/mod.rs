//! Extended syntax for Universal Markdown
//!
//! This module provides extended syntax support including Bootstrap 5 integration,
//! semantic HTML elements, definition lists, and LukiWiki legacy compatibility.

pub mod block_decorations;
pub mod code_block;
pub mod conflict_resolver;
pub mod emphasis;
pub mod inline_decorations;
pub mod media;
pub mod nested_blocks;
pub mod plugin_markers;
pub mod plugins;
pub mod preprocessor;
pub mod table;

/// Apply extended syntax transformations to HTML output
///
/// This function processes the HTML output from the Markdown parser and applies
/// extended syntax transformations including Bootstrap integration.
///
/// # Arguments
///
/// * `html` - The HTML output from the Markdown parser
///
/// # Returns
///
/// Transformed HTML with extended syntax applied
pub fn apply_extensions(html: &str) -> String {
    let header_map = conflict_resolver::HeaderIdMap::new();
    let options = crate::parser::ParserOptions::default();
    apply_extensions_with_headers(html, &header_map, &options)
}

/// Apply extended syntax transformations with custom header IDs
///
/// # Arguments
///
/// * `html` - The HTML output from the Markdown parser
/// * `header_map` - Map of custom header IDs
///
/// # Returns
///
/// Transformed HTML with extended syntax and custom header IDs applied
pub fn apply_extensions_with_headers(
    html: &str,
    header_map: &conflict_resolver::HeaderIdMap,
    options: &crate::parser::ParserOptions,
) -> String {
    let mut result = html.to_string();

    // Protect code blocks and inline code from transformation
    let (protected, placeholders) = protect_code_sections(&result);
    result = protected;

    // Apply transformations in order
    // Note: Plugins are handled in conflict_resolver::postprocess_conflicts
    result = media::transform_images_to_media(
        &result,
        &options.icons,
        options.allow_fragment_extension_hint,
    );
    result = conflict_resolver::postprocess_conflicts_with_options(
        &result,
        header_map,
        options.allow_hex_colors,
        &options.icons,
    );
    result = emphasis::apply_umd_emphasis(&result);
    result = block_decorations::apply_block_placement(&result); // Apply block placement first
    result =
        block_decorations::apply_block_decorations_with_options(&result, options.allow_hex_colors);
    result = inline_decorations::apply_inline_decorations_with_limit_and_options(
        &result,
        options.max_inline_nesting.map(usize::from),
        options.allow_hex_colors,
    );

    // Apply base URL resolution to links
    if let Some(base_url) = &options.base_url {
        result = conflict_resolver::apply_base_url_to_links(&result, base_url);
    }

    // Restore protected code sections
    restore_code_sections(&result, &placeholders, &options.icons.color_swatch)
}

/// Protect code blocks and inline code from transformation
///
/// Returns the HTML with code sections replaced by placeholders,
/// and a vector of the original code sections.
fn protect_code_sections(html: &str) -> (String, Vec<String>) {
    use regex::Regex;

    let mut placeholders = Vec::new();
    let mut result = html.to_string();

    // Protect <pre><code>...</code></pre> blocks
    let code_block_re = Regex::new(r"<pre><code[^>]*>[\s\S]*?</code></pre>").unwrap();
    result = code_block_re
        .replace_all(&result, |caps: &regex::Captures| {
            let index = placeholders.len();
            placeholders.push(caps[0].to_string());
            format!("<!--CODE_BLOCK_{}-->", index)
        })
        .to_string();

    // Protect <code>...</code> inline
    let inline_code_re = Regex::new(r"<code[^>]*>[^<]*</code>").unwrap();
    result = inline_code_re
        .replace_all(&result, |caps: &regex::Captures| {
            let index = placeholders.len();
            placeholders.push(caps[0].to_string());
            format!("<!--INLINE_CODE_{}-->", index)
        })
        .to_string();

    (result, placeholders)
}

/// Restore protected code sections
fn restore_code_sections(
    html: &str,
    placeholders: &[String],
    color_swatch_icon_html: &str,
) -> String {
    use regex::Regex;

    let mut result = html.to_string();

    // Restore code blocks
    let placeholder_re = Regex::new(r"<!--(CODE_BLOCK|INLINE_CODE)_(\d+)-->").unwrap();
    result = placeholder_re
        .replace_all(&result, |caps: &regex::Captures| {
            let section_type = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let index: usize = caps[2].parse().unwrap();
            let original = placeholders.get(index).map(|s| s.as_str()).unwrap_or("");

            if section_type == "INLINE_CODE" {
                enhance_inline_code_color_sample(original, color_swatch_icon_html)
            } else {
                original.to_string()
            }
        })
        .to_string();

    // Apply code block enhancements (syntax highlighting, Mermaid, filenames)
    result = code_block::process_code_blocks(&result);

    result
}

fn enhance_inline_code_color_sample(
    inline_code_html: &str,
    color_swatch_icon_html: &str,
) -> String {
    use once_cell::sync::Lazy;
    use regex::Regex;

    static INLINE_CODE_TAG_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"^<code(?P<attrs>[^>]*)>(?P<content>[^<]*)</code>$"#)
            .expect("valid inline code tag regex")
    });
    static HEX_COLOR_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?i)^#(?:[0-9a-f]{3}|[0-9a-f]{4}|[0-9a-f]{6}|[0-9a-f]{8})$")
            .expect("valid hex color regex")
    });
    static RGB_COLOR_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?i)^rgb\(\s*(?:25[0-5]|2[0-4]\d|1?\d?\d)\s*,\s*(?:25[0-5]|2[0-4]\d|1?\d?\d)\s*,\s*(?:25[0-5]|2[0-4]\d|1?\d?\d)\s*\)$",
        )
        .expect("valid rgb color regex")
    });
    static RGBA_COLOR_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?i)^rgba\(\s*(?:25[0-5]|2[0-4]\d|1?\d?\d)\s*,\s*(?:25[0-5]|2[0-4]\d|1?\d?\d)\s*,\s*(?:25[0-5]|2[0-4]\d|1?\d?\d)\s*,\s*(?:0|1|0?\.\d+|1(?:\.0+)?)\s*\)$",
        )
        .expect("valid rgba color regex")
    });
    static HSL_COLOR_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?i)^hsl\(\s*(?:360|3[0-5]\d|[12]?\d?\d)\s*,\s*(?:100|[1-9]?\d)%\s*,\s*(?:100|[1-9]?\d)%\s*\)$",
        )
        .expect("valid hsl color regex")
    });
    static HSLA_COLOR_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?i)^hsla\(\s*(?:360|3[0-5]\d|[12]?\d?\d)\s*,\s*(?:100|[1-9]?\d)%\s*,\s*(?:100|[1-9]?\d)%\s*,\s*(?:0|1|0?\.\d+|1(?:\.0+)?)\s*\)$",
        )
        .expect("valid hsla color regex")
    });

    let Some(caps) = INLINE_CODE_TAG_RE.captures(inline_code_html) else {
        return inline_code_html.to_string();
    };

    let attrs = caps.name("attrs").map(|m| m.as_str()).unwrap_or("");
    let content = caps.name("content").map(|m| m.as_str()).unwrap_or("");
    let color_text = content.trim();

    let is_color_code = HEX_COLOR_RE.is_match(color_text)
        || RGB_COLOR_RE.is_match(color_text)
        || RGBA_COLOR_RE.is_match(color_text)
        || HSL_COLOR_RE.is_match(color_text)
        || HSLA_COLOR_RE.is_match(color_text);

    if !is_color_code {
        return inline_code_html.to_string();
    }

    format!(
        r#"<code{}><span class="inline-code-color" style="background-color: {};">{}</span>{}</code>"#,
        attrs, color_text, color_swatch_icon_html, content
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParserOptions;

    #[test]
    fn test_umd_syntax_integration() {
        let input = "<p>This is ''bold'' and '''italic'''</p>";
        let output = apply_extensions(input);
        assert!(output.contains("<b>bold</b>"));
        assert!(output.contains("<i>italic</i>"));
    }

    #[test]
    fn test_inline_code_color_swatch_is_rendered_before_color_text() {
        let input = "<p><code>#FF5733</code></p>";
        let output = apply_extensions(input);
        assert!(output.contains(
            r#"<code><span class="inline-code-color" style="background-color: #FF5733;"><iconify-icon icon="ic:baseline-palette" aria-hidden="true"></iconify-icon></span>#FF5733</code>"#
        ));
    }

    #[test]
    fn test_inline_code_color_swatch_icon_is_configurable() {
        let input = "<p><code>#FF5733</code></p>";
        let header_map = conflict_resolver::HeaderIdMap::new();
        let mut options = ParserOptions::default();
        options.icons.color_swatch =
            r#"<span class="my-swatch-icon" aria-hidden="true"></span>"#.to_string();

        let output = apply_extensions_with_headers(input, &header_map, &options);
        assert!(output.contains(r#"<span class="my-swatch-icon" aria-hidden="true"></span>"#));
    }
}
