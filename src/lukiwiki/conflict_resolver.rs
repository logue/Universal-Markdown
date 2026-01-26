//! Syntax conflict resolution for LukiWiki and Markdown
//!
//! This module handles cases where LukiWiki and Markdown syntax might conflict.
//! The general strategy is:
//! 1. Process input before Markdown parsing (pre-processing)
//! 2. Apply LukiWiki-specific transformations after Markdown rendering (post-processing)
//! 3. Use distinctive markers to avoid ambiguous patterns

use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::collections::HashMap;

// Patterns that need special handling

/// Regex to detect LukiWiki blockquote: > ... <
static LUKIWIKI_BLOCKQUOTE: Lazy<Regex> = Lazy::new(|| {
    // Match single line > content < pattern
    Regex::new(r"(?m)^>\s*(.+?)\s*<\s*$").unwrap()
});

/// Regex to detect Markdown-style emphasis that might conflict with LukiWiki
/// Detects ***text*** which could be confused with '''text'''
static TRIPLE_STAR_EMPHASIS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\*\*\*([^*]+)\*\*\*").unwrap());

/// Regex to detect custom header ID syntax: # Header {#custom-id}
static CUSTOM_HEADER_ID: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^(#{1,6})\s+(.+?)\s+\{#([a-zA-Z0-9_-]+)\}\s*$").unwrap());

/// Store custom header IDs during preprocessing
#[derive(Debug, Clone)]
pub struct HeaderIdMap {
    /// Maps heading number (1-based) to custom ID
    pub ids: HashMap<usize, String>,
}

impl HeaderIdMap {
    pub fn new() -> Self {
        Self {
            ids: HashMap::new(),
        }
    }
}

/// Pre-process input to resolve conflicts before Markdown parsing
///
/// This function escapes or transforms syntax that would otherwise create
/// ambiguous parsing situations. It also extracts custom header IDs.
///
/// # Arguments
///
/// * `input` - The raw wiki markup input
///
/// # Returns
///
/// A tuple of (pre-processed markup, header ID map)
///
/// # Examples
///
/// ```
/// use lukiwiki_parser::lukiwiki::conflict_resolver::preprocess_conflicts;
///
/// let input = "> quote <";
/// let (output, _) = preprocess_conflicts(input);
/// // LukiWiki blockquote is preserved
/// ```
pub fn preprocess_conflicts(input: &str) -> (String, HeaderIdMap) {
    let mut result = input.to_string();
    let mut header_map = HeaderIdMap::new();
    let mut heading_counter = 0;

    // Extract custom header IDs: # Header {#custom-id}
    result = CUSTOM_HEADER_ID
        .replace_all(&result, |caps: &Captures| {
            heading_counter += 1;
            let hashes = &caps[1];
            let title = &caps[2];
            let custom_id = &caps[3];

            // Store the custom ID for this heading
            header_map
                .ids
                .insert(heading_counter, custom_id.to_string());

            // Return the heading without the {#id} part
            format!("{} {}", hashes, title)
        })
        .to_string();

    // Handle LukiWiki blockquotes: > ... <
    // Use a safe marker that won't be affected by HTML escaping
    result = LUKIWIKI_BLOCKQUOTE
        .replace_all(&result, |caps: &Captures| {
            let content = &caps[1];
            format!(
                "{{{{LUKIWIKI_BLOCKQUOTE:{}:LUKIWIKI_BLOCKQUOTE}}}}",
                content
            )
        })
        .to_string();

    // Protect LukiWiki block decorations (COLOR, SIZE, alignment)
    // These will be applied in post-processing
    let color_prefix = Regex::new(r"(?m)^(COLOR\([^)]*\):\s*.+)$").unwrap();
    result = color_prefix
        .replace_all(&result, |caps: &Captures| {
            format!("{{{{BLOCK_DECORATION:{}:BLOCK_DECORATION}}}}", &caps[1])
        })
        .to_string();

    let size_prefix = Regex::new(r"(?m)^(SIZE\([^)]+\):\s*.+)$").unwrap();
    result = size_prefix
        .replace_all(&result, |caps: &Captures| {
            format!("{{{{BLOCK_DECORATION:{}:BLOCK_DECORATION}}}}", &caps[1])
        })
        .to_string();

    let align_prefix = Regex::new(r"(?m)^((RIGHT|CENTER|LEFT):\s*.+)$").unwrap();
    result = align_prefix
        .replace_all(&result, |caps: &regex::Captures| {
            format!("{{{{BLOCK_DECORATION:{}:BLOCK_DECORATION}}}}", &caps[1])
        })
        .to_string();

    // Protect inline plugins: &function(args){content};
    // Use base64 encoding to safely preserve content with special characters
    let inline_plugin = Regex::new(r"&(\w+)\(([^)]*)\)\{((?:[^{}]|\{[^}]*\})*)\};").unwrap();
    result = inline_plugin
        .replace_all(&result, |caps: &regex::Captures| {
            use base64::{Engine as _, engine::general_purpose};
            let function = &caps[1];
            let args = &caps[2];
            let content = &caps[3];
            let encoded_content = general_purpose::STANDARD.encode(content.as_bytes());
            format!(
                "{{{{INLINE_PLUGIN:{}:{}:{}:INLINE_PLUGIN}}}}",
                function, args, encoded_content
            )
        })
        .to_string();

    // Protect block plugins multiline: @function(args){{ content }}
    // Use base64 encoding and markers to preserve content
    let block_plugin_multi = Regex::new(r"@(\w+)\(([^)]*)\)\{\{([\s\S]*?)\}\}").unwrap();
    result = block_plugin_multi
        .replace_all(&result, |caps: &regex::Captures| {
            use base64::{Engine as _, engine::general_purpose};
            let function = &caps[1];
            let args = &caps[2];
            let content = &caps[3];
            let encoded_content = general_purpose::STANDARD.encode(content.as_bytes());
            format!(
                "{{{{BLOCK_PLUGIN:{}:{}:{}:BLOCK_PLUGIN}}}}",
                function, args, encoded_content
            )
        })
        .to_string();

    // Protect block plugins singleline: @function(args){content}
    let block_plugin_single = Regex::new(r"@(\w+)\(([^)]*)\)\{([^}]*)\}").unwrap();
    result = block_plugin_single
        .replace_all(&result, |caps: &Captures| {
            use base64::{Engine as _, engine::general_purpose};
            let function = &caps[1];
            let args = &caps[2];
            let content = &caps[3];
            let encoded_content = general_purpose::STANDARD.encode(content.as_bytes());
            format!(
                "{{{{BLOCK_PLUGIN:{}:{}:{}:BLOCK_PLUGIN}}}}",
                function, args, encoded_content
            )
        })
        .to_string();

    (result, header_map)
}

/// Post-process HTML to restore LukiWiki-specific syntax and apply custom header IDs
///
/// This function converts temporary markers back to their intended HTML output
/// and replaces sequential header IDs with custom IDs where specified.
///
/// # Arguments
///
/// * `html` - The HTML output from Markdown parser
/// * `header_map` - Map of custom header IDs
///
/// # Returns
///
/// HTML with LukiWiki blockquotes properly rendered and custom IDs applied
pub fn postprocess_conflicts(html: &str, header_map: &HeaderIdMap) -> String {
    use crate::lukiwiki::block_decorations;

    let mut result = html.to_string();

    // Add header IDs: <h1>Title</h1> -> <h1><a href="#id" id="id"></a>Title</h1>
    let mut heading_counter = 0;
    let header_regex = Regex::new(r"<h([1-6])>([^<]+)</h([1-6])>").unwrap();
    result = header_regex
        .replace_all(&result, |caps: &Captures| {
            heading_counter += 1;
            let level = &caps[1];
            let title = &caps[2];
            let close_level = &caps[3];

            let id = if let Some(custom_id) = header_map.ids.get(&heading_counter) {
                custom_id.clone()
            } else {
                format!("heading-{}", heading_counter)
            };

            format!(
                "<h{}><a href=\"#{}\" aria-hidden=\"true\" class=\"anchor\" id=\"{}\"></a>{}</h{}>",
                level, id, id, title, close_level
            )
        })
        .to_string();

    // Restore LukiWiki blockquotes
    let lukiwiki_blockquote_marker =
        Regex::new(r"\{\{LUKIWIKI_BLOCKQUOTE:(.+?):LUKIWIKI_BLOCKQUOTE\}\}").unwrap();

    result = lukiwiki_blockquote_marker
        .replace_all(&result, |caps: &Captures| {
            let content = &caps[1];
            format!("<blockquote class=\"lukiwiki\">{}</blockquote>", content)
        })
        .to_string();

    // Restore and apply block decorations
    let block_decoration_marker =
        Regex::new(r"<p>\{\{BLOCK_DECORATION:(.+?):BLOCK_DECORATION\}\}</p>").unwrap();

    result = block_decoration_marker
        .replace_all(&result, |caps: &Captures| {
            let decoration = &caps[1];
            // Apply block decoration logic
            block_decorations::apply_block_decorations(decoration)
        })
        .to_string();

    // Restore inline plugins
    let inline_plugin_marker =
        Regex::new(r"\{\{INLINE_PLUGIN:(\w+):([^:]*):([^:]*):INLINE_PLUGIN\}\}").unwrap();
    result = inline_plugin_marker
        .replace_all(&result, |caps: &Captures| {
            use base64::{Engine as _, engine::general_purpose};
            let function = &caps[1];
            let args = &caps[2];
            let encoded_content = &caps[3];
            // Decode base64 to get original content
            let content = general_purpose::STANDARD
                .decode(encoded_content.as_bytes())
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_else(|| encoded_content.to_string());

            // Escape HTML entities in content while preserving & for nested plugins
            let escaped_content = content.replace('<', "&lt;").replace('>', "&gt;");

            format!(
                "<span class=\"plugin-{}\" data-args=\"{}\">{}</span>",
                function,
                html_escape::encode_double_quoted_attribute(args),
                escaped_content
            )
        })
        .to_string();

    // Restore block plugins
    let block_plugin_marker =
        Regex::new(r"\{\{BLOCK_PLUGIN:(\w+):([^:]*):([^:]*):BLOCK_PLUGIN\}\}").unwrap();
    result = block_plugin_marker
        .replace_all(&result, |caps: &Captures| {
            use base64::{Engine as _, engine::general_purpose};
            let function = &caps[1];
            let args = &caps[2];
            let encoded_content = &caps[3];
            // Decode base64 to get original content
            let content = general_purpose::STANDARD
                .decode(encoded_content.as_bytes())
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_else(|| encoded_content.to_string());

            // Escape HTML entities in content while preserving & for nested plugins
            let escaped_content = content.replace('<', "&lt;").replace('>', "&gt;");

            format!(
                "<div class=\"plugin-{}\" data-args=\"{}\">{}</div>",
                function,
                html_escape::encode_double_quoted_attribute(args),
                escaped_content
            )
        })
        .to_string();

    // Remove wrapping <p> tags around block plugins
    let wrapped_plugin =
        Regex::new(r#"<p>\s*(<div class="plugin-[^"]+"[^>]*>.*?</div>)\s*</p>"#).unwrap();
    result = wrapped_plugin.replace_all(&result, "$1").to_string();

    result
}

/// Check if input contains potentially ambiguous syntax
///
/// Used for diagnostics and warnings. Returns descriptions of
/// detected conflicts.
///
/// # Arguments
///
/// * `input` - The raw wiki markup input
///
/// # Returns
///
/// Vector of warning messages for ambiguous patterns
pub fn detect_ambiguous_syntax(input: &str) -> Vec<String> {
    let mut warnings = Vec::new();

    // Check for ***text*** which could be confused with '''text'''
    if TRIPLE_STAR_EMPHASIS.is_match(input) && input.contains("'''") {
        warnings.push(
            "Detected both ***text*** (Markdown) and '''text''' (LukiWiki). \
             Consider using **text** for Markdown bold-italic."
                .to_string(),
        );
    }

    // Check for potential COLOR(): vs Markdown definition list conflict
    if input.contains("COLOR(") && input.contains("\n:") {
        warnings.push(
            "Detected COLOR() syntax near Markdown definition list. \
             Ensure proper spacing to avoid ambiguity."
                .to_string(),
        );
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lukiwiki_blockquote_preprocessing() {
        let input = "> This is a LukiWiki quote <";
        let (output, _) = preprocess_conflicts(input);
        assert!(output.contains("{{LUKIWIKI_BLOCKQUOTE:"));
        assert!(!output.starts_with(">"));
    }

    #[test]
    fn test_lukiwiki_blockquote_postprocessing() {
        let header_map = HeaderIdMap::new();
        let input = "{{LUKIWIKI_BLOCKQUOTE:Test content:LUKIWIKI_BLOCKQUOTE}}";
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains("<blockquote class=\"lukiwiki\">Test content</blockquote>"));
    }

    #[test]
    fn test_markdown_blockquote_unchanged() {
        let input = "> Standard Markdown quote\n> Second line";
        let (output, _) = preprocess_conflicts(input);
        // Should NOT be converted (no closing <)
        assert_eq!(output, input);
    }

    #[test]
    fn test_roundtrip_blockquote() {
        let header_map = HeaderIdMap::new();
        let input = "> LukiWiki style <";
        let (preprocessed, _) = preprocess_conflicts(input);
        let postprocessed = postprocess_conflicts(&preprocessed, &header_map);
        assert!(postprocessed.contains("<blockquote class=\"lukiwiki\">"));
    }

    #[test]
    fn test_custom_header_id() {
        let input = "# My Header {#custom-id}\n\nContent";
        let (output, header_map) = preprocess_conflicts(input);
        // Should extract the custom ID
        assert_eq!(header_map.ids.get(&1), Some(&"custom-id".to_string()));
        // Should remove {#custom-id} from the text
        assert!(!output.contains("{#custom-id}"));
        assert!(output.contains("# My Header"));
    }

    #[test]
    fn test_multiple_custom_header_ids() {
        let input = "# First {#first}\n\n## Second {#second}\n\n### Third";
        let (_output, header_map) = preprocess_conflicts(input);
        assert_eq!(header_map.ids.get(&1), Some(&"first".to_string()));
        assert_eq!(header_map.ids.get(&2), Some(&"second".to_string()));
        assert_eq!(header_map.ids.get(&3), None); // No custom ID for third
    }

    #[test]
    fn test_apply_custom_header_ids() {
        let mut header_map = HeaderIdMap::new();
        header_map.ids.insert(1, "my-custom-id".to_string());

        let html = "<h1>Header</h1>";
        let output = postprocess_conflicts(html, &header_map);

        assert!(output.contains("id=\"my-custom-id\""));
        assert!(output.contains("href=\"#my-custom-id\""));
        assert!(!output.contains("heading-1"));
    }

    #[test]
    fn test_sequential_header_ids() {
        let header_map = HeaderIdMap::new();
        let html = "<h1>First</h1><h2>Second</h2>";
        let output = postprocess_conflicts(html, &header_map);

        assert!(output.contains("id=\"heading-1\""));
        assert!(output.contains("id=\"heading-2\""));
    }

    #[test]
    fn test_detect_triple_emphasis_conflict() {
        let input = "***Markdown*** and '''LukiWiki'''";
        let warnings = detect_ambiguous_syntax(input);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("***text***"));
    }

    #[test]
    fn test_detect_color_definition_conflict() {
        let input = "COLOR(red): text\n: definition";
        let warnings = detect_ambiguous_syntax(input);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("COLOR()"));
    }

    #[test]
    fn test_no_warnings_for_clean_syntax() {
        let input = "# Heading\n\n**Bold** and ''LukiWiki bold''";
        let warnings = detect_ambiguous_syntax(input);
        assert!(warnings.is_empty());
    }
}
