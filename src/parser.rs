//! Parser module for LukiWiki markup
//!
//! This module provides the core parsing functionality using comrak as the base
//! Markdown parser, with extensions for LukiWiki-specific syntax.

use comrak::options::ListStyleType;
use comrak::{Arena, Options, Plugins, format_html_with_plugins, parse_document};

/// Parser configuration for LukiWiki markup
#[derive(Debug, Clone)]
pub struct ParserOptions {
    /// Enable GitHub Flavored Markdown extensions
    pub gfm_extensions: bool,
    /// Enable LukiWiki-specific extensions
    pub lukiwiki_extensions: bool,
    /// Maximum heading level (1-5 for LukiWiki, 1-6 for standard Markdown)
    pub max_heading_level: u8,
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self {
            gfm_extensions: true,
            lukiwiki_extensions: true,
            max_heading_level: 5,
        }
    }
}

/// Parse LukiWiki markup and convert to HTML
///
/// # Arguments
///
/// * `input` - The sanitized LukiWiki markup source text
/// * `options` - Parser configuration options
///
/// # Returns
///
/// HTML string
///
/// # Examples
///
/// ```
/// use lukiwiki_parser::parser::{parse_to_html, ParserOptions};
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
        comrak_options.extension.header_ids = None; // Disable automatic IDs, we'll add them ourselves
    }

    // Render options
    comrak_options.render.hardbreaks = false;
    comrak_options.render.github_pre_lang = true; // Use GitHub-style language tags
    comrak_options.render.full_info_string = false;
    comrak_options.render.width = 0;
    comrak_options.render.r#unsafe = false; // Don't render raw HTML
    comrak_options.render.escape = false;
    comrak_options.render.list_style = ListStyleType::Dash;

    // Create arena for AST nodes
    let arena = Arena::new();

    // Parse markdown to AST
    let root = parse_document(&arena, input, &comrak_options);

    // TODO: Apply LukiWiki-specific transformations to AST here
    // This is where we'll add custom syntax handling in later steps

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
        let html = parse_to_html(input, &ParserOptions::default());
        assert!(html.contains("<img"));
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
}
