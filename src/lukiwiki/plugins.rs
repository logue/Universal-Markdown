//! Plugin system for LukiWiki
//!
//! Provides plugin syntax support:
//! - Inline plugins: &function(args){content};
//! - Block plugins (multiline): @function(args){{ content }}
//! - Block plugins (single line): @function(args){content}
//!
//! Note: This only parses plugin syntax and outputs placeholder HTML.
//! Actual plugin execution is handled by JavaScript/frontend layer.
//! Content within plugins may contain nested plugins or other Wiki syntax.

use once_cell::sync::Lazy;
use regex::Regex;

// Block plugin patterns
static BLOCK_PLUGIN_MULTILINE: Lazy<Regex> = Lazy::new(|| {
    // Match @function(args){{ content }} using non-greedy match
    Regex::new(r"@(\w+)\(([^)]*)\)\{\{([\s\S]*?)\}\}").unwrap()
});

static BLOCK_PLUGIN_SINGLELINE: Lazy<Regex> = Lazy::new(|| {
    // Match @function(args){content} (single braces)
    Regex::new(r"@(\w+)\(([^)]*)\)\{([^}]*)\}").unwrap()
});

// Inline plugin pattern
static INLINE_PLUGIN: Lazy<Regex> = Lazy::new(|| {
    // Match &function(args){content};
    // Content may contain nested braces for nested plugins
    Regex::new(r"&(\w+)\(([^)]*)\)\{((?:[^{}]|\{[^}]*\})*)\};").unwrap()
});

/// Apply plugin syntax transformation
///
/// Converts plugin syntax to HTML containers that can be processed by JavaScript.
/// The parser only detects and preserves plugin metadata; actual execution happens
/// in the frontend.
///
/// Supports three plugin patterns:
/// - Inline: `&function(args){content};`
/// - Block multiline: `@function(args){{ content }}`
/// - Block singleline: `@function(args){content}`
///
/// Content within plugins is preserved as-is and can contain nested plugins
/// or other Wiki syntax that will be processed by the plugin at runtime.
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
/// // Block plugin
/// let input = "@toc(2){{ }}";
/// let output = apply_plugin_syntax(input);
/// assert!(output.contains("class=\"plugin-toc\""));
/// assert!(output.contains("data-args=\"2\""));
///
/// // Inline plugin
/// let input = "&highlight(yellow){important text};";
/// let output = apply_plugin_syntax(input);
/// assert!(output.contains("class=\"plugin-highlight\""));
/// ```
pub fn apply_plugin_syntax(html: &str) -> String {
    let mut result = html.to_string();

    // Process block plugins (multiline) first - @function(args){{ content }}
    result = BLOCK_PLUGIN_MULTILINE
        .replace_all(&result, |caps: &regex::Captures| {
            let function = caps.get(1).map_or("", |m| m.as_str());
            let args = caps.get(2).map_or("", |m| m.as_str());
            let content = caps.get(3).map_or("", |m| m.as_str());

            let escaped_content = content.replace('<', "&lt;").replace('>', "&gt;");
            format!(
                "\n<div class=\"plugin-{}\" data-args=\"{}\">{}\n</div>\n",
                function,
                html_escape::encode_double_quoted_attribute(args),
                escaped_content
            )
        })
        .to_string();

    // Process block plugins (singleline) - @function(args){content}
    result = BLOCK_PLUGIN_SINGLELINE
        .replace_all(&result, |caps: &regex::Captures| {
            let function = caps.get(1).map_or("", |m| m.as_str());
            let args = caps.get(2).map_or("", |m| m.as_str());
            let content = caps.get(3).map_or("", |m| m.as_str());

            let escaped_content = content.replace('<', "&lt;").replace('>', "&gt;");
            format!(
                "\n<div class=\"plugin-{}\" data-args=\"{}\">{}\n</div>\n",
                function,
                html_escape::encode_double_quoted_attribute(args),
                escaped_content
            )
        })
        .to_string();

    // Process inline plugins - &function(args){content};
    result = INLINE_PLUGIN
        .replace_all(&result, |caps: &regex::Captures| {
            let function = caps.get(1).map_or("", |m| m.as_str());
            let args = caps.get(2).map_or("", |m| m.as_str());
            let content = caps.get(3).map_or("", |m| m.as_str());

            let escaped_content = content.replace('<', "&lt;").replace('>', "&gt;");
            format!(
                "<span class=\"plugin-{}\" data-args=\"{}\">{}</span>",
                function,
                html_escape::encode_double_quoted_attribute(args),
                escaped_content
            )
        })
        .to_string();

    result
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
        assert!(output.contains("fn main()"));
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

    #[test]
    fn test_inline_plugin() {
        let input = "&highlight(yellow){important text};";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"plugin-highlight\""));
        assert!(output.contains("data-args=\"yellow\""));
        assert!(output.contains("important text"));
        assert!(output.contains("<span"));
    }

    #[test]
    fn test_block_plugin_singleline() {
        let input = "@include(file.txt){default content}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"plugin-include\""));
        assert!(output.contains("data-args=\"file.txt\""));
        assert!(output.contains("default content"));
    }

    #[test]
    fn test_nested_plugins() {
        let input = "&outer(arg1){text &inner(arg2){nested}; more};";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"plugin-outer\""));
        // Content should preserve the nested plugin syntax (& not escaped)
        assert!(output.contains("&inner"));
    }

    #[test]
    fn test_plugin_with_wiki_syntax() {
        let input = "@box(){{ **bold** and text }}";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("class=\"plugin-box\""));
        // Content should preserve wiki syntax for JS processing
        assert!(output.contains("**bold**"));
    }

    #[test]
    fn test_mixed_plugin_types() {
        let input = "@block(){{ content }} and &inline(arg){text}; mixed";
        let output = apply_plugin_syntax(input);
        assert!(output.contains("plugin-block"));
        assert!(output.contains("plugin-inline"));
    }
}
