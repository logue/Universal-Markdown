//! Plugin system for LukiWiki
//!
//! Provides basic plugin syntax support:
//! - @function(args){{ content }}
//!
//! Note: This only parses plugin syntax and outputs placeholder HTML.
//! Actual plugin execution is handled by JavaScript/frontend layer.

use once_cell::sync::Lazy;
use regex::Regex;

static BLOCK_PLUGIN: Lazy<Regex> = Lazy::new(|| {
    // Match @function(args){{ content }} using non-greedy match
    Regex::new(r"@(\w+)\(([^)]*)\)\{\{([\s\S]*?)\}\}").unwrap()
});

/// Apply plugin syntax transformation
///
/// Converts plugin syntax to HTML containers that can be processed by JavaScript.
/// The parser only detects and preserves plugin metadata; actual execution happens
/// in the frontend.
///
/// # Arguments
///
/// * `html` - The HTML content to process
///
/// # Returns
///
/// HTML with plugin syntax converted to containers
///
/// # Examples
///
/// ```
/// use lukiwiki_parser::lukiwiki::plugins::apply_plugin_syntax;
///
/// let input = "@toc(2){{ }}";
/// let output = apply_plugin_syntax(input);
/// assert!(output.contains("class=\"plugin-toc\""));
/// assert!(output.contains("data-args=\"2\""));
/// ```
pub fn apply_plugin_syntax(html: &str) -> String {
    BLOCK_PLUGIN.replace_all(html, |caps: &regex::Captures| {
        let function = caps.get(1).map_or("", |m| m.as_str());
        let args = caps.get(2).map_or("", |m| m.as_str());
        let _content = caps.get(3).map_or("", |m| m.as_str());
        
        // Output a placeholder div that JavaScript can process
        format!(
            "<div class=\"plugin-{}\" data-args=\"{}\">@{}</div>",
            function,
            html_escape::encode_double_quoted_attribute(args),
            function
        )
    }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_plugin() {
        let input = "@toc(2){{ }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"plugin-toc\""));
        assert!(output.contains("data-args=\"2\""));
        assert!(output.contains("@toc"));
    }

    #[test]
    fn test_plugin_with_complex_args() {
        let input = "@calendar(2024,1,true){{ }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("plugin-calendar"));
        assert!(output.contains("data-args=\"2024,1,true\""));
    }

    #[test]
    fn test_plugin_no_args() {
        let input = "@timestamp(){{ }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("plugin-timestamp"));
        assert!(output.contains("data-args=\"\""));
    }

    #[test]
    fn test_plugin_with_content() {
        let input = "@code(rust){{ fn main() {} }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("plugin-code"));
        assert!(output.contains("data-args=\"rust\""));
    }

    #[test]
    fn test_multiple_plugins() {
        let input = "@toc(2){{ }} and @timestamp(){{ }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("plugin-toc"));
        assert!(output.contains("plugin-timestamp"));
    }

    #[test]
    fn test_no_plugin() {
        let input = "This is normal text with @mention but not @plugin()";
        let output = apply_plugin_syntax(input);
        // Should not match without {{ }}
        assert_eq!(output, input);
    }
}
