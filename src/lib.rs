//! Universal Markdown Parser
//!
//! A post-Markdown superset parser with Bootstrap 5 integration and extensible syntax.
//! This parser aims for reasonable CommonMark compliance (75%+) while
//! providing powerful extensions including Bootstrap styling, semantic HTML, and plugin support.
//!
//! # Features
//!
//! - CommonMark-compliant Markdown parsing
//! - Bootstrap 5 integration (Core UI compatible)
//! - Extended syntax (definition lists, decorations, semantic HTML)
//! - LukiWiki legacy syntax support for backward compatibility
//! - HTML sanitization (direct HTML input is forbidden)
//! - Safe HTML output generation
//! - Plugin system support (output only, execution handled externally)
//!
//! # Security
//!
//! All user input is sanitized to prevent XSS attacks. HTML entities are
//! preserved, but raw HTML tags are escaped. Plugin output is the only
//! exception, as plugins are considered trusted code.
//!
//! # Example
//!
//! ```
//! use umd::parse;
//!
//! let input = "# Hello World\n\nThis is **bold** text.";
//! let html = parse(input);
//! ```
//!
//! # WASM Usage
//!
//! This library can be compiled to WebAssembly for use in browsers:
//!
//! ```javascript
//! import init, { parse } from './umd.js';
//!
//! await init();
//! const html = parse('# Hello World');
//! ```

use serde::Deserialize;
use wasm_bindgen::prelude::*;

pub mod extensions;
pub mod frontmatter;
pub mod parser;
pub mod sanitizer;

/// Parse result with optional frontmatter and footnotes
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// The rendered HTML content (body only, footnotes are separate)
    pub html: String,
    /// Optional frontmatter data
    pub frontmatter: Option<frontmatter::Frontmatter>,
    /// Footnotes HTML (if any footnotes are present)
    pub footnotes: Option<String>,
}

/// Parse Universal Markdown and convert to HTML
///
/// This function extracts frontmatter (if present) and parses the content.
/// Returns merged HTML of body and footnotes (if any).
///
/// # Arguments
///
/// * `input` - The Universal Markdown source text
///
/// # Returns
///
/// HTML string (frontmatter is removed from output, footnotes are appended)
///
/// # Examples
///
/// ```
/// use umd::parse;
///
/// let input = "# Heading\n\n**Bold** and *italic*[^1]\n\n[^1]: Footnote";
/// let html = parse(input);
/// assert!(html.contains("<h1>"));
/// assert!(html.contains("<strong>"));
/// assert!(html.contains("Footnote"));
/// ```
pub fn parse(input: &str) -> String {
    let result = parse_with_frontmatter(input);
    if let Some(footnotes) = result.footnotes {
        format!("{}\n{}", result.html, footnotes)
    } else {
        result.html
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmIconsOptions {
    video: Option<String>,
    audio: Option<String>,
    download: Option<String>,
    color_swatch: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmParseOptions {
    gfm_extensions: Option<bool>,
    umd_extensions: Option<bool>,
    max_heading_level: Option<u8>,
    max_inline_nesting: Option<u8>,
    base_url: Option<String>,
    allow_fragment_extension_hint: Option<bool>,
    icons: Option<WasmIconsOptions>,
}

fn parse_with_options_json(input: &str, options_json: Option<&str>) -> String {
    let mut options = parser::ParserOptions::default();

    if let Some(raw_json) = options_json {
        let trimmed = raw_json.trim();
        if !trimmed.is_empty() {
            if let Ok(raw) = serde_json::from_str::<WasmParseOptions>(trimmed) {
                if let Some(value) = raw.gfm_extensions {
                    options.gfm_extensions = value;
                }
                if let Some(value) = raw.umd_extensions {
                    options.umd_extensions = value;
                }
                if let Some(value) = raw.max_heading_level {
                    options.max_heading_level = value;
                }
                if let Some(value) = raw.max_inline_nesting {
                    options.max_inline_nesting = Some(value);
                }
                if let Some(value) = raw.base_url {
                    options.base_url = Some(value);
                }
                if let Some(value) = raw.allow_fragment_extension_hint {
                    options.allow_fragment_extension_hint = value;
                }
                if let Some(icons) = raw.icons {
                    if let Some(value) = icons.video {
                        options.icons.video = value;
                    }
                    if let Some(value) = icons.audio {
                        options.icons.audio = value;
                    }
                    if let Some(value) = icons.download {
                        options.icons.download = value;
                    }
                    if let Some(value) = icons.color_swatch {
                        options.icons.color_swatch = value;
                    }
                }
            }
        }
    }

    let result = parse_with_frontmatter_opts(input, &options);
    if let Some(footnotes) = result.footnotes {
        format!("{}\n{}", result.html, footnotes)
    } else {
        result.html
    }
}

/// Parse Universal Markdown and return HTML with frontmatter
///
/// This function extracts frontmatter and returns it separately from the HTML content.
///
/// # Arguments
///
/// * `input` - The Universal Markdown source text
///
/// # Returns
///
/// ParseResult containing HTML and optional frontmatter
///
/// # Examples
///
/// ```
/// use umd::parse_with_frontmatter;
///
/// let input = "---\ntitle: Test\n---\n\n# Content";
/// let result = parse_with_frontmatter(input);
/// assert!(result.frontmatter.is_some());
/// assert!(result.html.contains("<h1>"));
/// ```
pub fn parse_with_frontmatter(input: &str) -> ParseResult {
    let options = parser::ParserOptions::default();
    parse_with_frontmatter_opts(input, &options)
}

/// Parse Universal Markdown and return HTML with frontmatter and custom options
///
/// This function extracts frontmatter and returns it separately from the HTML content,
/// with support for custom parser options (e.g., base_url for URL resolution).
///
/// # Arguments
///
/// * `input` - The Universal Markdown source text
/// * `options` - Parser configuration options
///
/// # Returns
///
/// ParseResult containing HTML and optional frontmatter
///
/// # Examples
///
/// ```
/// use umd::{parse_with_frontmatter_opts, parser::ParserOptions};
///
/// let input = "[link](/docs)";
/// let mut opts = ParserOptions::default();
/// opts.base_url = Some("/app".to_string());
/// let result = parse_with_frontmatter_opts(input, &opts);
/// assert!(result.html.contains(r#"href="/app/docs""#));
/// ```
pub fn parse_with_frontmatter_opts(input: &str, options: &parser::ParserOptions) -> ParseResult {
    // Step 0: Extract frontmatter
    let (frontmatter_data, content) = frontmatter::extract_frontmatter(input);

    // Step 1: Pre-process list items to allow nested block elements
    let content = extensions::nested_blocks::preprocess_nested_blocks(&content);

    // Step 2: Pre-process indeterminate task list markers
    let content = extensions::preprocessor::preprocess_tasklist_indeterminate(&content);

    // Step 3: Pre-process Discord-style underline (__text__) to prevent CommonMark conversion
    let content = extensions::preprocessor::preprocess_discord_underline(&content);

    // Step 3.5: Normalize fenced code block filename syntax (```lang:file)
    let content = extensions::preprocessor::preprocess_code_block_filenames(&content);

    // Step 4: Pre-process to resolve syntax conflicts and extract custom header IDs
    let (preprocessed, header_map) = extensions::conflict_resolver::preprocess_conflicts(&content);

    // Step 4.5: Remove ASCII control characters (U+0000-U+001F except TAB/LF/CR, and U+007F)
    // from non-code-block regions. Plugin content is already base64-encoded by Step 4,
    // so plugin markers are inherently safe. Plugin authors are responsible for any
    // further sanitization of their plugin's content.
    let preprocessed = sanitizer::remove_ascii_control_chars_from_markup(&preprocessed);

    // Step 5: Sanitize input
    let sanitized = sanitizer::sanitize_opts(&preprocessed, options.allow_bidi_in_code_blocks);

    // Step 6: Parse with comrak-based parser
    let html = parser::parse_to_html(&sanitized, options);

    // Step 7: Restore Discord-style underline placeholders to <u> tags
    let html = extensions::preprocessor::postprocess_discord_underline(&html);

    // Step 8: Apply extended syntax and custom header IDs (includes post-processing)
    let final_html = extensions::apply_extensions_with_headers(&html, &header_map, options);

    // Step 9: Extract footnotes from HTML
    let (body_html, footnotes_html) = extract_footnotes(&final_html);

    ParseResult {
        html: body_html,
        frontmatter: frontmatter_data,
        footnotes: footnotes_html,
    }
}

/// Extract footnotes section from HTML
///
/// Comrak generates footnotes as a `<section class="footnotes">` element.
/// This function separates the footnotes from the main content.
///
/// # Arguments
///
/// * `html` - The complete HTML with potential footnotes
///
/// # Returns
///
/// A tuple of (body HTML, optional footnotes HTML)
fn extract_footnotes(html: &str) -> (String, Option<String>) {
    use regex::Regex;

    // Match the footnotes section generated by comrak
    let footnote_pattern =
        Regex::new(r#"(?s)<section class="footnotes"[^>]*>.*?</section>"#).unwrap();

    if let Some(matched) = footnote_pattern.find(html) {
        let footnotes = matched.as_str().to_string();
        let body = footnote_pattern.replace(html, "").to_string();
        (body, Some(footnotes))
    } else {
        (html.to_string(), None)
    }
}

/// WASM-exposed API for parsing Universal Markdown
///
/// This is the main entry point when using the library from JavaScript/WebAssembly.
///
/// The second argument is optional JSON options in camelCase.
///
/// Supported options:
/// - `gfmExtensions`: boolean
/// - `umdExtensions`: boolean
/// - `maxHeadingLevel`: number
/// - `maxInlineNesting`: number (recommended: 3-5)
/// - `baseUrl`: string
/// - `allowFragmentExtensionHint`: boolean
/// - `icons`: object with `video`, `audio`, `download`, `colorSwatch`
///
/// # Arguments
///
/// * `input` - The Universal Markdown source text
///
/// # Returns
///
/// HTML string
///
/// # JavaScript Example
///
/// ```javascript
/// import init, { parse } from './umd.js';
///
/// await init();
/// const html = parse('# Hello World\n\nThis is **bold** text.');
/// const html2 = parse('# Link\n\n[docs](/docs)', JSON.stringify({
///   baseUrl: '/app',
///   maxInlineNesting: 4,
///   icons: {
///     colorSwatch: '<span class="bi bi-eyedropper" aria-hidden="true"></span>'
///   }
/// }));
/// console.log(html);
/// ```
#[wasm_bindgen(js_name = parse)]
pub fn parse_wasm(input: &str, options_json: Option<String>) -> String {
    parse_with_options_json(input, options_json.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parse() {
        let input = "Hello World";
        let output = parse(input);
        assert!(output.contains("Hello World"));
    }

    #[test]
    fn test_html_escaping() {
        let input = "<script>alert('xss')</script>";
        let output = parse(input);
        assert!(!output.contains("<script>"));
        assert!(output.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_parse_with_options_json_base_url() {
        let input = "[docs](/guide)";
        let output = parse_with_options_json(input, Some(r#"{"baseUrl":"/app"}"#));
        assert!(output.contains(r#"href="/app/guide""#));
    }

    #[test]
    fn test_parse_with_options_json_icon_override() {
        let input = "`#ff0000`";
        let output = parse_with_options_json(
            input,
            Some(
                r#"{"icons":{"colorSwatch":"<span class=\"my-icon\" aria-hidden=\"true\"></span>"}}"#,
            ),
        );
        assert!(output.contains(r#"<span class="my-icon" aria-hidden="true"></span>"#));
    }

    #[test]
    fn test_parse_with_options_json_inline_nesting_limit() {
        let input = "&color(blue){&abbr(t){x};};";
        let output_from_json = parse_with_options_json(input, Some(r#"{"maxInlineNesting":1}"#));

        let mut options = parser::ParserOptions::default();
        options.max_inline_nesting = Some(1);
        let expected = parse_with_frontmatter_opts(input, &options);
        let expected_html = if let Some(footnotes) = expected.footnotes {
            format!("{}\n{}", expected.html, footnotes)
        } else {
            expected.html
        };

        assert_eq!(output_from_json, expected_html);
    }
}
