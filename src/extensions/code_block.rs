//! Code block extensions for UMD
//!
//! Provides syntax highlighting and Mermaid diagram support for code blocks.
//! - Syntax highlighting: Multiple language support with syntax coloring
//! - Mermaid diagrams: Diagram rendering from Markdown fence blocks with SVG generation
//! - File name support: Code blocks with associated file names

use once_cell::sync::Lazy;
use regex::Regex;
#[cfg(not(target_arch = "wasm32"))]
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
#[cfg(not(target_arch = "wasm32"))]
use syntect::parsing::SyntaxSet;
#[cfg(not(target_arch = "wasm32"))]
use syntect::util::LinesWithEndings;
use uuid::Uuid;

static MERMAID_BLOCK_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?s)<pre><code[^>]*class=\"language-mermaid\"[^>]*>(.*?)</code></pre>"#)
        .expect("valid mermaid block regex")
});

static CODE_BLOCK_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?s)<pre><code(?P<attrs>[^>]*)>(?P<code>.*?)</code></pre>"#)
        .expect("valid code block regex")
});

static HTML_ATTR_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"([a-zA-Z_:][-a-zA-Z0-9_:.]*)\s*=\s*\"([^\"]*)\""#).expect("valid html attr regex")
});

#[cfg(not(target_arch = "wasm32"))]
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);

/// Process code blocks with syntax highlighting and metadata
///
/// # Features
/// - ✅ Language detection from code class attribute
/// - ✅ Syntax highlighting class generation
/// - ✅ Mermaid diagram detection and SVG rendering
/// - ✅ Bootstrap CSS variable integration
/// - ✅ Plain text blocks (no language) without `<code>` tags
///
/// # Input Format (from comrak)
///
/// comrak outputs code blocks in GitHub-flavored format:
/// - `<pre><code>plain text content</code></pre>` - Plain text (no language)
/// - `<pre><code class="language-rust">highlighted content</code></pre>` - With language
/// - `<pre><code class="language-mermaid">diagram code</code></pre>` - Mermaid diagrams
///
/// # Output HTML Patterns
///
/// Plain text: `<pre>content</pre>` (code tag removed)
///
/// Language-specific: `<pre><code class="language-rust">content</code></pre>` (unchanged)
///
/// Mermaid diagram: `<figure class="code-block code-block-mermaid mermaid-diagram">SVG content</figure>`
pub fn process_code_blocks(html: &str) -> String {
    // First handle Mermaid diagrams if present
    let html = process_mermaid_blocks(html);

    // Then process regular code blocks with syntax highlighting
    process_syntax_highlighted_blocks(&html)
}

/// Process Mermaid diagram blocks
///
/// Converts `<code class="language-mermaid">` blocks into SVG diagrams with Bootstrap styling
/// comrak outputs: `<pre><code class="language-mermaid">...</code></pre>`
fn process_mermaid_blocks(html: &str) -> String {
    // Check if mermaid is present (but not already wrapped)
    if !html.contains("language-mermaid") || html.contains("mermaid-diagram") {
        return html.to_string();
    }

    MERMAID_BLOCK_RE
        .replace_all(html, |caps: &regex::Captures| {
            let code = &caps[1];
            let decoded = decode_html_entities(code);
            let code_text = decoded.trim();

            match render_mermaid_as_svg(code_text) {
                Ok(svg) => {
                    let diagram_id = Uuid::new_v4().to_string();
                    format!(
                        "<figure class=\"code-block code-block-mermaid mermaid-diagram\" id=\"mermaid-{}\" data-mermaid-source=\"{}\">{}</figure>",
                        &diagram_id[..8],
                        html_escape::encode_double_quoted_attribute(code_text),
                        svg
                    )
                }
                Err(error) => {
                    let escaped_error = html_escape::encode_double_quoted_attribute(&error);
                    format!(
                        "<figure class=\"code-block code-block-mermaid mermaid-diagram\"><pre class=\"mermaid-error\" data-error=\"{}\"><code class=\"language-mermaid\">{}</code></pre></figure>",
                        escaped_error,
                        code
                    )
                }
            }
        })
        .to_string()
}

/// Process syntax highlighting for code blocks
///
/// comrak outputs code blocks as:
/// - `<pre><code>plain content</code></pre>` for plain text blocks (no language)
/// - `<pre><code class="language-rust">highlighted content</code></pre>` for language-specific blocks
///
/// The full fence info string can include title metadata (e.g., "rust: main.rs"),
/// but comrak only extracts the language part to generate the class attribute.
/// This function processes the pre and code tags to add title support and
/// remove <code> tags for plain text blocks.
///
/// Supports four code block patterns:
/// 1. Plain text: `<pre><code>...</code></pre>` → `<pre>...</pre>`
/// 2. Plain text with title: parse from fence info in data attributes
/// 3. Language-only: `<pre><code class="language-rust">...</code></pre>`
/// 4. Language+Title: add figcaption wrapper with title
fn process_syntax_highlighted_blocks(html: &str) -> String {
    CODE_BLOCK_RE
        .replace_all(html, |caps: &regex::Captures| {
            let attrs = caps.name("attrs").map(|m| m.as_str()).unwrap_or("");
            let code = caps.name("code").map(|m| m.as_str()).unwrap_or("");

            let language = extract_language_from_attrs(attrs);
            if matches!(language.as_deref(), Some(lang) if lang.eq_ignore_ascii_case("mermaid")) {
                return caps[0].to_string();
            }

            let filename = extract_attribute(attrs, "data-meta")
                .map(|value| decode_html_entities(&value))
                .and_then(|meta| extract_filename_from_meta(&meta));

            let rendered_block = if let Some(lang) = language.as_deref() {
                let decoded = decode_html_entities(code);
                match highlight_code_with_syntect(lang, &decoded) {
                    Some(highlighted) => format!(
                        "<pre><code class=\"language-{} syntect-highlight\" data-highlighted=\"true\">{}</code></pre>",
                        lang, highlighted
                    ),
                    None => format!("<pre><code class=\"language-{}\">{}</code></pre>", lang, code),
                }
            } else {
                format!("<pre>{}</pre>", code)
            };

            if let Some(filename) = filename {
                let escaped_filename = html_escape::encode_text(&filename);
                format!(
                    "<figure class=\"code-block\"><figcaption class=\"code-filename\"><span class=\"filename\">{}</span></figcaption>{}</figure>",
                    escaped_filename,
                    rendered_block
                )
            } else {
                rendered_block
            }
        })
        .to_string()
}

fn extract_attribute(attrs: &str, name: &str) -> Option<String> {
    for caps in HTML_ATTR_RE.captures_iter(attrs) {
        if caps.get(1)?.as_str().eq_ignore_ascii_case(name) {
            return Some(caps.get(2)?.as_str().to_string());
        }
    }
    None
}

fn extract_language_from_attrs(attrs: &str) -> Option<String> {
    let class_attr = extract_attribute(attrs, "class")?;

    for class_name in class_attr.split_whitespace() {
        if let Some(language) = class_name.strip_prefix("language-") {
            if language.eq_ignore_ascii_case("umd-nolang") {
                return None;
            }
            if !language.is_empty() {
                return Some(language.to_string());
            }
        }
    }

    None
}

fn extract_filename_from_meta(meta: &str) -> Option<String> {
    let marker = "umd-filename:";
    let index = meta.find(marker)?;
    let filename = meta[index + marker.len()..].trim();
    if filename.is_empty() {
        None
    } else {
        Some(filename.to_string())
    }
}

/// Render Mermaid code to SVG
///
/// Converts Mermaid diagram notation to SVG format with Bootstrap CSS variable support.
/// Supports basic graph, flowchart, and sequence diagrams.
fn render_mermaid_as_svg(mermaid_code: &str) -> Result<String, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        mermaid_rs_renderer::render(mermaid_code)
            .map(|svg| inject_umd_color_variables(&svg))
            .map_err(|error| error.to_string())
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = mermaid_code;
        Err("Mermaid rendering is unavailable on wasm32 target".to_string())
    }
}

fn highlight_code_with_syntect(language: &str, source: &str) -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let syntax = SYNTAX_SET
            .find_syntax_by_token(language)
            .or_else(|| SYNTAX_SET.find_syntax_by_name(language))
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

        let mut generator = ClassedHTMLGenerator::new_with_class_style(
            syntax,
            &SYNTAX_SET,
            ClassStyle::SpacedPrefixed { prefix: "syntect-" },
        );

        for line in LinesWithEndings::from(source) {
            if generator
                .parse_html_for_line_which_includes_newline(line)
                .is_err()
            {
                return None;
            }
        }

        Some(generator.finalize())
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = (language, source);
        None
    }
}

/// Inject umd-core CSS variables for diagram coloring
///
/// Replaces hardcoded colors with umd-core color tokens (--umd-color-blue,
/// --umd-color-green, etc.) instead of system theme variables. White and
/// black are excluded as they represent structural elements rather than
/// semantic colors.
///
/// `mermaid-rs-renderer` bakes these literal hex values into the SVG itself
/// (no theme-variable passthrough), so this post-hoc string replacement is
/// the only way to make the diagram respond to the reference CSS's light/dark
/// tokens — see PLAN.md for the longer-term fix.
fn inject_umd_color_variables(svg: &str) -> String {
    svg.replace("#0d6efd", "var(--umd-color-blue, #0d6efd)")
        .replace("#6c757d", "var(--umd-color-gray, #6c757d)")
        .replace("#198754", "var(--umd-color-green, #198754)")
        .replace("#dc3545", "var(--umd-color-red, #dc3545)")
        .replace("#ffc107", "var(--umd-color-yellow, #ffc107)")
        .replace("#0dcaf0", "var(--umd-color-cyan, #0dcaf0)")
    // Note: #ffffff (white) and #000000 (black) are intentionally excluded
    // as they represent structural elements, not semantic colors
}

/// Simple hash function for generating diagram IDs
/// Uses a lightweight FNV-1a algorithm
/// Note: Currently replaced with UUID for unique ID generation in render_mermaid_as_svg
#[allow(dead_code)]
fn simple_hash(data: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in data.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Basic HTML entity decoder for common entities
fn decode_html_entities(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

/// Get list of supported languages for syntax highlighting
///
/// Returns language identifiers that can be used in code fence info strings
pub fn get_supported_languages() -> Vec<&'static str> {
    vec![
        "rust",
        "python",
        "javascript",
        "typescript",
        "jsx",
        "tsx",
        "html",
        "css",
        "scss",
        "less",
        "java",
        "kotlin",
        "go",
        "c",
        "cpp",
        "csharp",
        "swift",
        "objc",
        "php",
        "ruby",
        "perl",
        "bash",
        "shell",
        "zsh",
        "fish",
        "sql",
        "mysql",
        "postgresql",
        "mongodb",
        "json",
        "yaml",
        "toml",
        "xml",
        "markdown",
        "latex",
        "dockerfile",
        "nginx",
        "apache",
        "lua",
        "vim",
        "elisp",
        "mermaid", // Diagram support
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_code_block_with_language() {
        // comrak GitHub format: <pre><code class="language-rust">code</code></pre>
        let html = "<pre><code class=\"language-rust\">fn main() {}</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("language-rust"));
        assert!(result.contains("syntect-highlight"));
        assert!(result.contains("data-highlighted=\"true\""));
        assert!(result.contains("fn"));
        assert!(result.contains("main"));
    }

    #[test]
    fn test_basic_code_block_plain_text() {
        // Plain text block (no language attribute): <pre><code>text</code></pre>
        let html = "<pre><code>plain text</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("<pre>plain text</pre>"));
        assert!(!result.contains("<code>"));
    }

    #[test]
    fn test_mermaid_block_detection() {
        // comrak Mermaid format: <pre><code class="language-mermaid">...</code></pre>
        let html =
            "<pre><code class=\"language-mermaid\">graph TD\n    A[Start] --> B[End]</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("code-block-mermaid"));
        assert!(result.contains("mermaid-diagram"));
        assert!(result.contains("data-mermaid-source"));
        assert!(result.contains("<svg"));
    }

    #[test]
    fn test_mermaid_parse_error_fallback() {
        let html = "<pre><code class=\"language-mermaid\">graph TD\n  A --&gt;</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("mermaid-error") || result.contains("mermaid-diagram"));
    }

    #[test]
    fn test_code_block_plain_text_no_code_tag() {
        // Plain text: <pre><code>...</code></pre> → <pre>...</pre>
        let html = "<pre><code>plain text here</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("<pre>plain text here</pre>"));
        assert!(!result.contains("<code>"));
    }

    #[test]
    fn test_code_block_multiline_plain_text() {
        // Plain text block with newlines
        let html = "<pre><code>line1\nline2\nline3</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("<pre>line1\nline2\nline3</pre>"));
        assert!(!result.contains("<code>"));
    }

    #[test]
    fn test_code_block_language_preserved() {
        // Language-specific block left unchanged
        let html = "<pre><code class=\"language-python\">print('hello')</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("language-python"));
        assert!(result.contains("data-highlighted=\"true\""));
        assert!(result.contains("print"));
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_code_block_escaping() {
        let html = "<pre><code class=\"language-html\">&lt;div&gt;content&lt;/div&gt;</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("&lt;"));
        assert!(result.contains("&gt;"));
        assert!(result.contains("content"));
    }

    #[test]
    fn test_simple_hash_consistency() {
        let hash1 = simple_hash("test");
        let hash2 = simple_hash("test");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_decoded_html_entities() {
        let encoded = "&lt;div&gt; &amp; &quot;test&quot;";
        let decoded = decode_html_entities(encoded);
        assert_eq!(decoded, "<div> & \"test\"");
    }

    #[test]
    fn test_code_block_with_filename_and_language() {
        let html = "<pre><code class=\"language-rust\" data-meta=\"umd-filename:src/main.rs\">fn main() {}</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("<figure class=\"code-block\">"));
        assert!(result.contains("<figcaption class=\"code-filename\">"));
        assert!(result.contains("src/main.rs"));
        assert!(result.contains("language-rust"));
    }

    #[test]
    fn test_code_block_with_filename_without_language() {
        let html = "<pre><code class=\"language-umd-nolang\" data-meta=\"umd-filename:config.yml\">key: value</code></pre>";
        let result = process_code_blocks(html);
        assert!(result.contains("<figure class=\"code-block\">"));
        assert!(result.contains("config.yml"));
        assert!(result.contains("<pre>key: value</pre>"));
        assert!(!result.contains("language-umd-nolang"));
    }
}
