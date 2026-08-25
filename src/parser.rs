//! Parser module for Universal Markdown
//!
//! This module provides the core parsing functionality using comrak as the base
//! Markdown parser, with extensions for Universal Markdown-specific syntax.

use comrak::options::{ListStyleType, Plugins};
use comrak::{Arena, Options, format_html_with_plugins, parse_document};

/// Icon markup configuration
///
/// Values can include raw HTML (e.g. `<span class="bi bi-camera-video-fill"></span>`).
/// These values are inserted directly into the HTML output without sanitization —
/// only use trusted, developer-supplied values here.
#[derive(Debug, Clone)]
pub struct Icons {
    /// Icon for video fallback download links.
    /// Default: `<span class="bi bi-camera-video-fill" aria-hidden="true"></span>`
    pub video: String,
    /// Icon for audio fallback download links.
    /// Default: `<span class="bi bi-music-note-beamed" aria-hidden="true"></span>`
    pub audio: String,
    /// Icon for downloadable file links.
    /// Default: `<span class="bi bi-file-earmark-arrow-down-fill" aria-hidden="true"></span>`
    pub download: String,
    /// Icon markup rendered inside the inline color swatch.
    /// Default: `<span class="bi bi-palette-fill" aria-hidden="true"></span>`
    pub color_swatch: String,
}

impl Default for Icons {
    fn default() -> Self {
        Self {
            video: r#"<span class="bi bi-camera-video-fill" aria-hidden="true"></span>"#
                .to_string(),
            audio: r#"<span class="bi bi-music-note-beamed" aria-hidden="true"></span>"#
                .to_string(),
            download:
                r#"<span class="bi bi-file-earmark-arrow-down-fill" aria-hidden="true"></span>"#
                    .to_string(),
            color_swatch: r#"<span class="bi bi-palette-fill" aria-hidden="true"></span>"#
                .to_string(),
        }
    }
}

/// Parser configuration for Universal Markdown
#[derive(Debug, Clone)]
pub struct ParserOptions {
    /// Enable GitHub Flavored Markdown extensions
    pub gfm_extensions: bool,
    /// Enable Universal Markdown-specific extensions
    pub umd_extensions: bool,
    /// Maximum heading level (1-5 for Universal Markdown, 1-6 for standard Markdown)
    pub max_heading_level: u8,
    /// Base URL for resolving absolute paths (e.g., "/umd-core", "https://example.com/app")
    /// If set, absolute paths (starting with "/") will be prefixed with this base URL
    pub base_url: Option<String>,
    /// Allow media type detection from fragment extension hints like `#.png`.
    ///
    /// Disabled by default for safer behavior; enable only when you trust
    /// the source content and need extension-less URL fallback.
    pub allow_fragment_extension_hint: bool,
    /// Allow hex color values like `#FF5733` in UMD color syntax.
    ///
    /// Disabled by default to keep semantic-color syntax opt-in and avoid
    /// unintended CSS output from raw color literals.
    pub allow_hex_colors: bool,
    /// Maximum allowed nesting depth for inline UMD decoration functions.
    ///
    /// When exceeded, inline decoration expansion is skipped as a fail-safe.
    /// Use `None` to disable this limit.
    pub max_inline_nesting: Option<u8>,
    /// Icon configuration (media fallback links and inline code enhancements)
    pub icons: Icons,
    /// Allow BiDi control characters (`U+202A`-`U+202E`, `U+2066`-`U+2069`)
    /// to survive inside fenced code blocks (` ``` ` / `~~~`).
    ///
    /// Disabled by default for safer behavior: BiDi control characters are
    /// stripped everywhere, including inside code blocks, to prevent
    /// Trojan Source-style visual spoofing attacks. Enable only when you
    /// trust the source content and need to display RTL code samples or
    /// BiDi-attack demonstrations verbatim.
    pub allow_bidi_in_code_blocks: bool,
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self {
            gfm_extensions: true,
            umd_extensions: true,
            max_heading_level: 5,
            base_url: None,
            allow_fragment_extension_hint: false,
            allow_hex_colors: false,
            max_inline_nesting: Some(5),
            icons: Icons::default(),
            allow_bidi_in_code_blocks: false,
        }
    }
}

/// Parse Universal Markdown and convert to HTML
///
/// # Arguments
///
/// * `input` - The sanitized Universal Markdown source text
/// * `options` - Parser configuration options
///
/// # Returns
///
/// HTML string
///
/// # Examples
///
/// ```
/// use umd::parser::{parse_to_html, ParserOptions};
///
/// let input = "# Hello World\n\nThis is **bold** text.";
/// let html = parse_to_html(input, &ParserOptions::default());
/// assert!(html.contains("<h1>"));
/// assert!(html.contains("<strong>"));
/// ```
pub fn parse_to_html(input: &str, options: &ParserOptions) -> String {
    // Configure comrak options
    let mut comrak_options = Options::default();

    // Enable extensions
    if options.gfm_extensions {
        comrak_options.extension.strikethrough = true;
        comrak_options.extension.tagfilter = true; // Disallow dangerous HTML tags
        comrak_options.extension.table = true;
        comrak_options.extension.autolink = true;
        comrak_options.extension.tasklist = true;
        comrak_options.extension.footnotes = true; // Enable footnotes
        comrak_options.extension.header_id_prefix = None; // Disable automatic IDs, we'll add them ourselves
    }

    // Render options
    comrak_options.render.hardbreaks = false;
    comrak_options.render.github_pre_lang = false; // Keep language on <code class="language-*"> (no lang on <pre>)
    comrak_options.render.full_info_string = true;
    comrak_options.render.width = 0;
    comrak_options.render.r#unsafe = false; // Don't render raw HTML
    comrak_options.render.escape = false;
    comrak_options.render.list_style = ListStyleType::Dash;

    // Create arena for AST nodes
    let arena = Arena::new();

    // Parse markdown to AST
    let root = parse_document(&arena, input, &comrak_options);

    // Render to HTML
    let mut html = String::new();
    format_html_with_plugins(root, &comrak_options, &mut html, &Plugins::default())
        .expect("Failed to render HTML");

    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_heading() {
        let input = "# Heading 1\n## Heading 2";
        let html = parse_to_html(input, &ParserOptions::default());
        assert!(html.contains("<h1>"));
        assert!(html.contains("Heading 1"));
        assert!(html.contains("<h2>"));
        assert!(html.contains("Heading 2"));
    }

    #[test]
    fn test_paragraph() {
        let input = "This is a paragraph.";
        let html = parse_to_html(input, &ParserOptions::default());
        assert!(html.contains("<p>"));
        assert!(html.contains("This is a paragraph."));
    }

    #[test]
    fn test_bold_italic() {
        let input = "**bold** and *italic*";
        let html = parse_to_html(input, &ParserOptions::default());
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    #[test]
    fn test_unordered_list() {
        let input = "- Item 1\n- Item 2\n- Item 3";
        let html = parse_to_html(input, &ParserOptions::default());
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>Item 1</li>"));
        assert!(html.contains("<li>Item 2</li>"));
    }

    #[test]
    fn test_ordered_list() {
        let input = "1. First\n2. Second\n3. Third";
        let html = parse_to_html(input, &ParserOptions::default());
        assert!(html.contains("<ol>"));
        assert!(html.contains("<li>First</li>"));
        assert!(html.contains("<li>Second</li>"));
    }

    #[test]
    fn test_code_block() {
        let input = "```rust\nfn main() {}\n```";
        let html = parse_to_html(input, &ParserOptions::default());
        println!("HTML output: {}", html);
        // comrak wraps code blocks in <pre><code> tags
        assert!(html.contains("<code") || html.contains("fn main() {}"));
        assert!(html.contains("rust") || html.contains("language-rust"));
    }

    #[test]
    fn test_inline_code() {
        let input = "This is `inline code` example.";
        let html = parse_to_html(input, &ParserOptions::default());
        assert!(html.contains("<code>inline code</code>"));
    }

    #[test]
    fn test_link() {
        let input = "[Link text](https://example.com)";
        let html = parse_to_html(input, &ParserOptions::default());
        assert!(html.contains("<a href=\"https://example.com\">"));
        assert!(html.contains("Link text"));
    }

    #[test]
    fn test_image() {
        let input = "![Alt text](https://example.com/image.png)";
        let html = crate::parse(input);
        // Now images are wrapped in <picture> tags
        assert!(html.contains("<picture"));
        assert!(html.contains("src=\"https://example.com/image.png\""));
        assert!(html.contains("alt=\"Alt text\""));
    }

    #[test]
    fn test_gfm_strikethrough() {
        let input = "~~strikethrough~~";
        let html = parse_to_html(input, &ParserOptions::default());
        assert!(html.contains("<del>strikethrough</del>"));
    }

    #[test]
    fn test_gfm_table() {
        let input = "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |";
        let html = parse_to_html(input, &ParserOptions::default());
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>Header 1</th>"));
        assert!(html.contains("<td>Cell 1</td>"));
    }

    #[test]
    fn test_task_list() {
        let input = "- [ ] Unchecked\n- [x] Checked";
        let html = parse_to_html(input, &ParserOptions::default());
        assert!(html.contains("type=\"checkbox\""));
        assert!(html.contains("disabled"));
    }

    #[test]
    fn test_video_media() {
        let input = "![Demo video](https://example.com/video.mp4)";
        let html = crate::parse(input);
        println!("Video HTML output: {}", html);
        assert!(html.contains("<video controls"));
        assert!(html.contains("src=\"https://example.com/video.mp4\""));
        assert!(html.contains("type=\"video/mp4\""));
        assert!(html.contains("<track kind=\"captions\" label=\"Demo video\""));
    }

    #[test]
    fn test_audio_media() {
        let input = "![Background music](https://example.com/audio.mp3)";
        let html = crate::parse(input);
        assert!(html.contains("<audio controls"));
        assert!(html.contains("src=\"https://example.com/audio.mp3\""));
        assert!(html.contains("type=\"audio/mpeg\""));
    }

    #[test]
    fn test_image_with_title() {
        let input = "![Logo](https://example.com/logo.png \"Company Logo\")";
        let html = crate::parse(input);
        assert!(html.contains("<picture"));
        assert!(html.contains("title=\"Company Logo\""));
        assert!(html.contains("alt=\"Logo\""));
    }

    #[test]
    fn test_video_with_title() {
        let input = "![Product demo](video.mp4 \"Our new product\")";
        let html = crate::parse(input);
        assert!(html.contains("<video controls"));
        assert!(html.contains("title=\"Our new product\""));
    }

    #[test]
    fn test_jxl_image() {
        let input = "![Modern image](image.jxl \"JPEG XL format\")";
        let html = crate::parse(input);
        assert!(html.contains("<picture"));
        assert!(html.contains("type=\"image/jxl\""));
        assert!(html.contains("title=\"JPEG XL format\""));
    }
}
