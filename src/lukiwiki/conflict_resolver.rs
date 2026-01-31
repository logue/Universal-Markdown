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

/// Convert comma-separated args to JSON array
///
/// # Arguments
///
/// * `args` - Comma-separated argument string
///
/// # Returns
///
/// JSON array string
fn args_to_json(args: &str) -> String {
    if args.is_empty() {
        return "[]".to_string();
    }

    let parts: Vec<String> = args
        .split(',')
        .map(|s| {
            let trimmed = s.trim();
            // Escape quotes and backslashes in the string
            let escaped = trimmed.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{}\"", escaped)
        })
        .collect();

    format!("[{}]", parts.join(","))
}

/// Map font size value to Bootstrap class or inline style
fn map_font_size_value(value: &str) -> (bool, String) {
    // Check if value has unit (rem, em, px, etc.)
    if value.contains("rem") || value.contains("em") || value.contains("px") {
        return (false, value.to_string()); // Return as inline style
    }

    // Map to Bootstrap fs-* classes (unitless values)
    let class = match value {
        "2.5" => "fs-1",
        "2" | "2.0" => "fs-2",
        "1.75" => "fs-3",
        "1.5" => "fs-4",
        "1.25" => "fs-5",
        "0.875" => "fs-6",
        _ => return (false, format!("{}rem", value)), // Custom value as inline style
    };

    (true, class.to_string())
}

/// Map color value to Bootstrap class or inline style
fn map_color_value(value: &str, is_background: bool) -> (bool, String) {
    let trimmed = value.trim();

    // Bootstrap theme colors
    let bootstrap_colors = [
        "primary",
        "secondary",
        "success",
        "danger",
        "warning",
        "info",
        "light",
        "dark",
        "body",
        "body-secondary",
        "body-tertiary",
        "body-emphasis",
        "primary-subtle",
        "secondary-subtle",
        "success-subtle",
        "danger-subtle",
        "warning-subtle",
        "info-subtle",
        "light-subtle",
        "dark-subtle",
        "primary-emphasis",
        "secondary-emphasis",
        "success-emphasis",
        "danger-emphasis",
        "warning-emphasis",
        "info-emphasis",
        "light-emphasis",
        "dark-emphasis",
    ];

    // Check if it's a Bootstrap color
    for color in &bootstrap_colors {
        if trimmed == *color || trimmed.starts_with(&format!("{}-", color)) {
            let prefix = if is_background { "bg" } else { "text" };
            return (true, format!("{}-{}", prefix, trimmed));
        }
    }

    // Otherwise, return as inline style value
    (false, trimmed.to_string())
}

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

    // Protect inline plugins with content but no args: &function{content};
    // Use base64 encoding to safely preserve content with special characters
    let inline_plugin_noargs_content = Regex::new(r"&(\w+)\{((?:[^{}]|\{[^}]*\})*)\};").unwrap();
    result = inline_plugin_noargs_content
        .replace_all(&result, |caps: &regex::Captures| {
            use base64::{Engine as _, engine::general_purpose};
            let function = &caps[1];
            let content = &caps[2];
            let encoded_content = general_purpose::STANDARD.encode(content.as_bytes());
            format!(
                "{{{{INLINE_PLUGIN:{}::{}:INLINE_PLUGIN}}}}",
                function, encoded_content
            )
        })
        .to_string();

    // Protect inline plugins: &function(args){content};
    // Use base64 encoding to safely preserve content with special characters
    // Note: Inline decoration functions ARE protected here and processed in postprocess
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

    // Protect inline plugins (args only): &function(args);
    let inline_plugin_argsonly = Regex::new(r"&(\w+)\(([^)]*)\);").unwrap();
    result = inline_plugin_argsonly
        .replace_all(&result, |caps: &regex::Captures| {
            let function = &caps[1];
            let args = &caps[2];
            format!(
                "{{{{INLINE_PLUGIN_ARGSONLY:{}:{}:INLINE_PLUGIN_ARGSONLY}}}}",
                function, args
            )
        })
        .to_string();

    // Protect inline plugins (no args): &function;
    let inline_plugin_noargs = Regex::new(r"&(\w+);").unwrap();
    result = inline_plugin_noargs
        .replace_all(&result, |caps: &regex::Captures| {
            let function = &caps[1];
            format!(
                "{{{{INLINE_PLUGIN_NOARGS:{}:INLINE_PLUGIN_NOARGS}}}}",
                function
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

    // Protect block plugins (args only, no content): @function(args)
    // This should be processed AFTER patterns with { and {{
    let block_plugin_argsonly = Regex::new(r"@(\w+)\(([^)]*)\)").unwrap();
    result = block_plugin_argsonly
        .replace_all(&result, |caps: &regex::Captures| {
            use base64::{Engine as _, engine::general_purpose};
            let function = &caps[1];
            let args = &caps[2];
            // Encode args to prevent Markdown parser from converting URLs
            let encoded_args = general_purpose::STANDARD.encode(args.as_bytes());
            format!(
                "{{{{BLOCK_PLUGIN_ARGSONLY:{}:{}:BLOCK_PLUGIN_ARGSONLY}}}}",
                function, encoded_args
            )
        })
        .to_string();

    // Process definition lists: :term|definition
    result = process_definition_lists(&result);

    (result, header_map)
}

/// Process definition lists (:term|definition syntax)
///
/// Converts consecutive lines starting with `:term|definition` into
/// a single `<dl>` element with `<dt>` and `<dd>` tags.
fn process_definition_lists(input: &str) -> String {
    let mut result = Vec::new();
    let mut lines = input.lines().peekable();

    while let Some(line) = lines.next() {
        // Check if this line starts a definition list
        if line.trim_start().starts_with(':') && line.contains('|') {
            let mut dl_items = Vec::new();

            // Collect consecutive definition list items
            let mut current_line = line;
            loop {
                if let Some(stripped) = current_line.trim_start().strip_prefix(':') {
                    if let Some((term, definition)) = stripped.split_once('|') {
                        dl_items.push((term.trim().to_string(), definition.trim().to_string()));
                    }
                }

                // Check if next line is also a definition list item
                match lines.peek() {
                    Some(next_line)
                        if next_line.trim_start().starts_with(':') && next_line.contains('|') =>
                    {
                        current_line = lines.next().unwrap();
                    }
                    _ => break,
                }
            }

            // Create marker for the definition list
            if !dl_items.is_empty() {
                use base64::{Engine as _, engine::general_purpose};
                let items_json = serde_json::to_string(&dl_items).unwrap();
                let encoded_items = general_purpose::STANDARD.encode(items_json.as_bytes());
                result.push(format!(
                    "{{{{DEFINITION_LIST:{}:DEFINITION_LIST}}}}",
                    encoded_items
                ));
            }
        } else {
            result.push(line.to_string());
        }
    }

    result.join("\n")
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
/// Convert inline decoration function to HTML
/// Returns None if not a decoration function
fn convert_inline_decoration_to_html(function: &str, args: &str, content: &str) -> Option<String> {
    match function {
        // Simple wrapper tags without content
        "dfn" => Some(format!("<dfn>{}</dfn>", content)),
        "kbd" => Some(format!("<kbd>{}</kbd>", content)),
        "samp" => Some(format!("<samp>{}</samp>", content)),
        "var" => Some(format!("<var>{}</var>", content)),
        "cite" => Some(format!("<cite>{}</cite>", content)),
        "q" => Some(format!("<q>{}</q>", content)),
        "small" => Some(format!("<small>{}</small>", content)),
        "u" => Some(format!("<u>{}</u>", content)),
        "bdi" => Some(format!("<bdi>{}</bdi>", content)),

        // Tags with attributes
        "ruby" => {
            // &ruby(reading){text}; → <ruby>text<rp>(</rp><rt>reading</rt><rp>)</rp></ruby>
            Some(format!(
                "<ruby>{}<rp>(</rp><rt>{}</rt><rp>)</rp></ruby>",
                content, args
            ))
        }
        "time" => {
            // &time(datetime){text}; → <time datetime="datetime">text</time>
            Some(format!("<time datetime=\"{}\">{}</time>", args, content))
        }
        "data" => {
            // &data(value){text}; → <data value="value">text</data>
            Some(format!("<data value=\"{}\">{}</data>", args, content))
        }
        "bdo" => {
            // &bdo(dir){text}; → <bdo dir="dir">text</bdo>
            Some(format!("<bdo dir=\"{}\">{}</bdo>", args, content))
        }
        "lang" => {
            // &lang(locale){text}; → <span lang="locale">text</span>
            Some(format!("<span lang=\"{}\">{}</span>", args, content))
        }
        "abbr" => {
            // &abbr(text){description}; → <abbr title="description">text</abbr>
            Some(format!("<abbr title=\"{}\">{}</abbr>", content, args))
        }
        "sup" => {
            // &sup(text); → <sup>text</sup>
            Some(format!("<sup>{}</sup>", args))
        }
        "sub" => {
            // &sub(text); → <sub>text</sub>
            Some(format!("<sub>{}</sub>", args))
        }
        "badge" => {
            // &badge(type){content}; → <span class="badge bg-type">content</span>
            // Support for badge-pill variants and links
            let badge_class = if args.ends_with("-pill") {
                let color = args.trim_end_matches("-pill");
                format!("badge rounded-pill bg-{}", color)
            } else {
                format!("badge bg-{}", args)
            };

            // Check if content contains a Markdown link: [text](url)
            let link_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
            if let Some(link_caps) = link_regex.captures(content) {
                let text = link_caps.get(1).map_or("", |m| m.as_str());
                let url = link_caps.get(2).map_or("", |m| m.as_str());
                Some(format!(
                    "<a href=\"{}\" class=\"{}\">{}</a>",
                    url, badge_class, text
                ))
            } else {
                Some(format!(
                    "<span class=\"{}\">{}</span>",
                    badge_class, content
                ))
            }
        }
        "color" => {
            // &color(fg,bg){text}; with Bootstrap support
            let parts: Vec<&str> = args.split(',').collect();
            let fg = parts.get(0).map_or("", |m| m.trim());
            let bg = parts.get(1).map_or("", |m| m.trim());

            let mut classes = Vec::new();
            let mut styles = Vec::new();

            if !fg.is_empty() && fg != "inherit" {
                let (is_class, value) = map_color_value(fg, false);
                if is_class {
                    classes.push(value);
                } else {
                    styles.push(format!("color: {}", value));
                }
            }

            if !bg.is_empty() && bg != "inherit" {
                let (is_class, value) = map_color_value(bg, true);
                if is_class {
                    classes.push(value);
                } else {
                    styles.push(format!("background-color: {}", value));
                }
            }

            if classes.is_empty() && styles.is_empty() {
                Some(content.to_string())
            } else {
                let mut attrs = Vec::new();
                if !classes.is_empty() {
                    attrs.push(format!("class=\"{}\"", classes.join(" ")));
                }
                if !styles.is_empty() {
                    attrs.push(format!("style=\"{}\"", styles.join("; ")));
                }
                Some(format!("<span {}>{}</span>", attrs.join(" "), content))
            }
        }
        "size" => {
            // &size(value){text}; with Bootstrap support
            let (is_class, value) = map_font_size_value(args);
            if is_class {
                Some(format!("<span class=\"{}\">{}</span>", value, content))
            } else {
                Some(format!(
                    "<span style=\"font-size: {}\">{}</span>",
                    value, content
                ))
            }
        }
        _ => None,
    }
}

/// Convert args-only inline decoration function to HTML
fn convert_inline_decoration_argsonly_to_html(function: &str, args: &str) -> Option<String> {
    match function {
        "sup" => Some(format!("<sup>{}</sup>", args)),
        "sub" => Some(format!("<sub>{}</sub>", args)),
        _ => None,
    }
}

/// Convert no-args inline decoration function to HTML
fn convert_inline_decoration_noargs_to_html(function: &str) -> Option<String> {
    match function {
        "wbr" => Some("<wbr />".to_string()),
        "br" => Some("<br />".to_string()),
        _ => None,
    }
}

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
                // Add 'h-' prefix to custom IDs to avoid conflicts with system IDs
                format!("h-{}", custom_id)
            } else {
                // Auto-numbered IDs also use 'h-' prefix for consistency
                format!("h-{}", heading_counter)
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

            // Try to convert as inline decoration function
            if let Some(html) = convert_inline_decoration_to_html(function, args, &content) {
                return html;
            }

            // Otherwise, convert to plugin HTML
            // Escape HTML entities in content while preserving & for nested plugins
            let escaped_content = content.replace('<', "&lt;").replace('>', "&gt;");

            // Convert args to JSON array
            let json_args = args_to_json(args);

            format!(
                "<span class=\"plugin-{}\" data-args='{}'>{}</span>",
                function, json_args, escaped_content
            )
        })
        .to_string();

    // Restore inline plugins (args only)
    let inline_plugin_argsonly_marker =
        Regex::new(r"\{\{INLINE_PLUGIN_ARGSONLY:(\w+):([^:]*):INLINE_PLUGIN_ARGSONLY\}\}").unwrap();
    result = inline_plugin_argsonly_marker
        .replace_all(&result, |caps: &Captures| {
            let function = &caps[1];
            let args = &caps[2];

            // Try to convert as inline decoration function
            if let Some(html) = convert_inline_decoration_argsonly_to_html(function, args) {
                return html;
            }

            // Otherwise, convert to plugin HTML
            let json_args = args_to_json(args);

            format!(
                "<span class=\"plugin-{}\" data-args='{}' />",
                function, json_args
            )
        })
        .to_string();

    // Restore inline plugins (no args)
    let inline_plugin_noargs_marker =
        Regex::new(r"\{\{INLINE_PLUGIN_NOARGS:(\w+):INLINE_PLUGIN_NOARGS\}\}").unwrap();
    result = inline_plugin_noargs_marker
        .replace_all(&result, |caps: &Captures| {
            let function = &caps[1];

            // Try to convert as inline decoration function
            if let Some(html) = convert_inline_decoration_noargs_to_html(function) {
                return html;
            }

            // Otherwise, convert to plugin HTML
            format!("<span class=\"plugin-{}\" data-args='[]' />", function)
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

            // Convert args to JSON array
            let json_args = args_to_json(args);

            format!(
                "<div class=\"plugin-{}\" data-args='{}'>{}</div>",
                function, json_args, escaped_content
            )
        })
        .to_string();

    // Restore block plugins (args only, no content)
    let block_plugin_argsonly_marker =
        Regex::new(r"\{\{BLOCK_PLUGIN_ARGSONLY:(\w+):([^:]*):BLOCK_PLUGIN_ARGSONLY\}\}").unwrap();
    result = block_plugin_argsonly_marker
        .replace_all(&result, |caps: &Captures| {
            use base64::{Engine as _, engine::general_purpose};
            let function = &caps[1];
            let encoded_args = &caps[2];
            // Decode base64 to get original args
            let args = general_purpose::STANDARD
                .decode(encoded_args.as_bytes())
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_else(|| encoded_args.to_string());
            let json_args = args_to_json(&args);

            format!(
                "<div class=\"plugin-{}\" data-args='{}' />",
                function, json_args
            )
        })
        .to_string();

    // Remove wrapping <p> tags around block plugins
    let wrapped_plugin =
        Regex::new(r#"<p>\s*(<div class="plugin-[^"]+"[^>]*>.*?</div>)\s*</p>"#).unwrap();
    result = wrapped_plugin.replace_all(&result, "$1").to_string();

    // Remove wrapping <p> tags around self-closing block plugins
    let wrapped_plugin_self =
        Regex::new(r#"<p>\s*(<div class="plugin-[^"]+"[^>]*/>\s*)\s*</p>"#).unwrap();
    result = wrapped_plugin_self.replace_all(&result, "$1").to_string();

    // Restore definition lists
    let definition_list_marker =
        Regex::new(r"\{\{DEFINITION_LIST:([^:]+):DEFINITION_LIST\}\}").unwrap();
    result = definition_list_marker
        .replace_all(&result, |caps: &Captures| {
            use base64::{Engine as _, engine::general_purpose};
            let encoded_items = &caps[1];

            // Decode base64 to get JSON
            let items_json = general_purpose::STANDARD
                .decode(encoded_items.as_bytes())
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_else(|| "[]".to_string());

            // Parse JSON to get items
            let items: Vec<(String, String)> =
                serde_json::from_str(&items_json).unwrap_or_default();

            if items.is_empty() {
                return String::new();
            }

            let mut dl_html = String::from("<dl>");
            for (term, definition) in items {
                dl_html.push_str(&format!("<dt>{}</dt><dd>{}</dd>", term, definition));
            }
            dl_html.push_str("</dl>");
            dl_html
        })
        .to_string();

    // Remove wrapping <p> tags around definition lists
    let wrapped_dl = Regex::new(r"<p>\s*(<dl>.*?</dl>)\s*</p>").unwrap();
    result = wrapped_dl.replace_all(&result, "$1").to_string();

    // Apply Bootstrap default classes, GFM alerts, and table cell alignment
    result = apply_bootstrap_enhancements(&result);

    result
}

/// Apply Bootstrap 5 enhancements to HTML
///
/// - Add default `table` class to all <table> elements
/// - Add default `blockquote` class to all <blockquote> elements (except LukiWiki-style)
/// - Convert GFM alerts ([!NOTE], etc.) to Bootstrap alert components
/// - Add JUSTIFY support for tables (w-100 class)
fn apply_bootstrap_enhancements(html: &str) -> String {
    let mut result = html.to_string();

    // Add default class to tables
    let table_pattern = Regex::new(r"<table>").unwrap();
    result = table_pattern
        .replace_all(&result, "<table class=\"table\">")
        .to_string();

    // Add default class to blockquotes (check if it doesn't already have class="lukiwiki")
    let blockquote_pattern = Regex::new(r#"<blockquote>"#).unwrap();
    result = blockquote_pattern
        .replace_all(&result, "<blockquote class=\"blockquote\">")
        .to_string();

    // LukiWiki blockquotes already have class="lukiwiki", so they remain unchanged

    // Handle GFM alerts: > [!NOTE] etc.
    // These are rendered as <blockquote class="blockquote"><p>[!NOTE] ...</p></blockquote>
    let gfm_alert_pattern = Regex::new(
        r#"<blockquote class="blockquote">\s*<p>\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]\s*(.*?)</p>\s*</blockquote>"#
    ).unwrap();

    result = gfm_alert_pattern
        .replace_all(&result, |caps: &Captures| {
            let alert_type = &caps[1];
            let content = &caps[2];

            let (alert_class, icon_text) = match alert_type {
                "NOTE" => ("alert-info", "Note"),
                "TIP" => ("alert-success", "Tip"),
                "IMPORTANT" => ("alert-primary", "Important"),
                "WARNING" => ("alert-warning", "Warning"),
                "CAUTION" => ("alert-danger", "Caution"),
                _ => ("alert-info", "Note"),
            };

            format!(
                r#"<div class="alert {}" role="alert"><strong>{}:</strong> {}</div>"#,
                alert_class, icon_text, content
            )
        })
        .to_string();

    // Process table cell vertical alignment prefixes
    result = process_table_cell_alignment(&result);

    result
}

/// Process table cell alignment prefixes (TOP:, MIDDLE:, BOTTOM:, BASELINE:)
///
/// Detects alignment prefixes in table cells and adds Bootstrap alignment classes.
fn process_table_cell_alignment(html: &str) -> String {
    let mut result = html.to_string();

    // Process <td> tags
    let td_pattern = Regex::new(r"<td([^>]*)>(.*?)</td>").unwrap();
    result = td_pattern
        .replace_all(&result, |caps: &Captures| {
            let existing_attrs = &caps[1];
            let content = &caps[2];
            process_cell_content("td", existing_attrs, content)
        })
        .to_string();

    // Process <th> tags
    let th_pattern = Regex::new(r"<th([^>]*)>(.*?)</th>").unwrap();
    result = th_pattern
        .replace_all(&result, |caps: &Captures| {
            let existing_attrs = &caps[1];
            let content = &caps[2];
            process_cell_content("th", existing_attrs, content)
        })
        .to_string();

    result
}

/// Process individual cell content for alignment
fn process_cell_content(tag: &str, existing_attrs: &str, content: &str) -> String {
    // Check for vertical alignment prefixes
    let (align_class, remaining_content) =
        if let Some(stripped) = content.trim_start().strip_prefix("TOP:") {
            ("align-top", stripped.trim_start())
        } else if let Some(stripped) = content.trim_start().strip_prefix("MIDDLE:") {
            ("align-middle", stripped.trim_start())
        } else if let Some(stripped) = content.trim_start().strip_prefix("BOTTOM:") {
            ("align-bottom", stripped.trim_start())
        } else if let Some(stripped) = content.trim_start().strip_prefix("BASELINE:") {
            ("align-baseline", stripped.trim_start())
        } else {
            ("", content)
        };

    if align_class.is_empty() {
        // No alignment prefix, return original
        format!("<{}{}>{}</{}>", tag, existing_attrs, content, tag)
    } else {
        // Add alignment class
        if existing_attrs.contains("class=") {
            // Append to existing class attribute
            let new_attrs =
                existing_attrs.replace("class=\"", &format!("class=\"{} ", align_class));
            format!("<{}{}>{}</{}>", tag, new_attrs, remaining_content, tag)
        } else {
            // Add new class attribute
            format!(
                r#"<{} class="{}"{}>{}</{}>"#,
                tag, align_class, existing_attrs, remaining_content, tag
            )
        }
    }
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

        assert!(output.contains("id=\"h-my-custom-id\""));
        assert!(output.contains("href=\"#h-my-custom-id\""));
        assert!(!output.contains("heading-1"));
    }

    #[test]
    fn test_sequential_header_ids() {
        let header_map = HeaderIdMap::new();
        let html = "<h1>First</h1><h2>Second</h2>";
        let output = postprocess_conflicts(html, &header_map);

        assert!(output.contains("id=\"h-1\""));
        assert!(output.contains("id=\"h-2\""));
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

    #[test]
    fn test_bootstrap_table_class() {
        let header_map = HeaderIdMap::new();
        let input = "<table><tr><td>Cell</td></tr></table>";
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains(r#"<table class="table">"#));
    }

    #[test]
    fn test_bootstrap_blockquote_class() {
        let header_map = HeaderIdMap::new();
        let input = "<blockquote><p>Quote</p></blockquote>";
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains(r#"<blockquote class="blockquote">"#));
    }

    #[test]
    fn test_gfm_alert_note() {
        let header_map = HeaderIdMap::new();
        let input = r#"<blockquote class="blockquote"><p>[!NOTE] This is a note</p></blockquote>"#;
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains(r#"<div class="alert alert-info" role="alert">"#));
        assert!(output.contains("<strong>Note:</strong>"));
        assert!(output.contains("This is a note"));
    }

    #[test]
    fn test_gfm_alert_warning() {
        let header_map = HeaderIdMap::new();
        let input = r#"<blockquote class="blockquote"><p>[!WARNING] Be careful</p></blockquote>"#;
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains(r#"<div class="alert alert-warning" role="alert">"#));
        assert!(output.contains("<strong>Warning:</strong>"));
    }

    #[test]
    fn test_lukiwiki_blockquote_no_bootstrap_class() {
        let header_map = HeaderIdMap::new();
        let input = "{{LUKIWIKI_BLOCKQUOTE:Test content:LUKIWIKI_BLOCKQUOTE}}";
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains(r#"<blockquote class="lukiwiki">"#));
        assert!(!output.contains(r#"class="blockquote""#));
    }

    #[test]
    fn test_definition_list() {
        let input = ":Term 1|Definition 1\n:Term 2|Definition 2";
        let (preprocessed, _) = preprocess_conflicts(input);
        assert!(preprocessed.contains("{{DEFINITION_LIST:"));
    }

    #[test]
    fn test_definition_list_html_output() {
        let header_map = HeaderIdMap::new();
        let input = ":HTML|HyperText Markup Language\n:CSS|Cascading Style Sheets";
        let (preprocessed, _) = preprocess_conflicts(input);
        let output = postprocess_conflicts(&preprocessed, &header_map);
        assert!(output.contains("<dl>"));
        assert!(output.contains("<dt>HTML</dt>"));
        assert!(output.contains("<dd>HyperText Markup Language</dd>"));
        assert!(output.contains("<dt>CSS</dt>"));
        assert!(output.contains("<dd>Cascading Style Sheets</dd>"));
        assert!(output.contains("</dl>"));
    }

    #[test]
    fn test_table_cell_vertical_alignment() {
        let header_map = HeaderIdMap::new();
        let input =
            r#"<table class="table"><tr><td>TOP: Cell1</td><td>MIDDLE: Cell2</td></tr></table>"#;
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains(r#"class="align-top""#));
        assert!(output.contains("Cell1"));
        assert!(output.contains(r#"class="align-middle""#));
        assert!(output.contains("Cell2"));
    }

    #[test]
    fn test_table_cell_multiple_alignments() {
        let header_map = HeaderIdMap::new();
        let input = r#"<table><tr><th>BASELINE: Header</th><td>BOTTOM: Data</td></tr></table>"#;
        let output = postprocess_conflicts(input, &header_map);
        assert!(output.contains(r#"class="align-baseline""#));
        assert!(output.contains(r#"class="align-bottom""#));
    }
}
