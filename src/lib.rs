//! LukiWiki Parser
//!
//! A Markdown superset wiki markup parser with LukiWiki syntax support.
//! This parser aims for reasonable CommonMark compliance (75%+) while
//! maintaining compatibility with legacy LukiWiki syntax.
//!
//! # Features
//!
//! - CommonMark-compliant Markdown parsing
//! - LukiWiki legacy syntax support (tables, definition lists, etc.)
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
//! use lukiwiki_parser::parse;
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
//! import init, { parse_wiki } from './lukiwiki_parser.js';
//!
//! await init();
//! const html = parse_wiki('# Hello World');
//! ```

use wasm_bindgen::prelude::*;

pub mod lukiwiki;
pub mod parser;
pub mod sanitizer;

/// Parse LukiWiki markup and convert to HTML
///
/// # Arguments
///
/// * `input` - The LukiWiki markup source text
///
/// # Returns
///
/// HTML string
///
/// # Examples
///
/// ```
/// use lukiwiki_parser::parse;
///
/// let input = "# Heading\n\n**Bold** and *italic*";
/// let html = parse(input);
/// assert!(html.contains("<h1>"));
/// assert!(html.contains("<strong>"));
/// ```
pub fn parse(input: &str) -> String {
    // Step 1: Pre-process to resolve syntax conflicts (before sanitization)
    let preprocessed = lukiwiki::conflict_resolver::preprocess_conflicts(input);

    // Step 2: Sanitize input
    let sanitized = sanitizer::sanitize(&preprocessed);

    // Step 3: Parse with comrak-based parser
    let options = parser::ParserOptions::default();
    let html = parser::parse_to_html(&sanitized, &options);

    // Step 4: Apply LukiWiki-specific syntax (includes post-processing)
    lukiwiki::apply_lukiwiki_syntax(&html)
}

/// WASM-exposed API for parsing LukiWiki markup
///
/// This is the main entry point when using the library from JavaScript/WebAssembly.
///
/// # Arguments
///
/// * `input` - The LukiWiki markup source text
///
/// # Returns
///
/// HTML string
///
/// # JavaScript Example
///
/// ```javascript
/// import init, { parse_wiki } from './lukiwiki_parser.js';
///
/// await init();
/// const html = parse_wiki('# Hello World\n\nThis is **bold** text.');
/// console.log(html);
/// ```
#[wasm_bindgen]
pub fn parse_wiki(input: &str) -> String {
    parse(input)
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
}
