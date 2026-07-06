//! Plugin syntax marker processing
//!
//! This module handles the conversion of plugin syntax into safe markers
//! that won't be affected by Markdown parsing.

use base64::{Engine as _, engine::general_purpose};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

/// HTML entities that should NOT be treated as plugins
fn html_entities() -> HashSet<&'static str> {
    [
        "lt", "gt", "amp", "nbsp", "quot", "apos", "ndash", "mdash", "hellip", "copy", "reg",
        "trade", "times", "divide", "plusmn", "le", "ge", "ne", "asymp", "equiv", "forall",
        "exist", "empty", "nabla", "isin", "notin", "ni", "prod", "sum", "minus", "lowast",
        "radic", "prop", "infin", "ang", "and", "or", "cap", "cup", "int", "there4", "sim", "cong",
        "sub", "sup", "nsub", "sube", "supe", "oplus", "otimes", "perp", "sdot", "lceil", "rceil",
        "lfloor", "rfloor", "lang", "rang", "loz", "spades", "clubs", "hearts", "diams", "alpha",
        "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa", "lambda",
        "mu", "nu", "xi", "omicron", "pi", "rho", "sigma", "tau", "upsilon", "phi", "chi", "psi",
        "omega", "Iuml", "iuml", "Uuml", "uuml", "Auml", "auml", "Ouml", "ouml", "Euml", "euml",
        "Aring", "aring", "AElig", "aelig", "Ccedil", "ccedil", "Eth", "eth", "Ntilde", "ntilde",
        "Oslash", "oslash", "Thorn", "thorn", "szlig", "yuml", "Agrave", "agrave", "Aacute",
        "aacute", "Acirc", "acirc", "Atilde", "atilde", "Egrave", "egrave", "Eacute", "eacute",
        "Ecirc", "ecirc", "Igrave", "igrave", "Iacute", "iacute", "Icirc", "icirc", "Ograve",
        "ograve", "Oacute", "oacute", "Ocirc", "ocirc", "Otilde", "otilde", "Ugrave", "ugrave",
        "Uacute", "uacute", "Ucirc", "ucirc", "Yacute", "yacute", "cent", "pound", "curren", "yen",
        "brvbar", "sect", "uml", "ordf", "laquo", "not", "shy", "macr", "deg", "sup2", "sup3",
        "acute", "micro", "para", "middot", "cedil", "sup1", "ordm", "raquo", "frac14", "frac12",
        "frac34", "iquest", "ensp", "emsp", "thinsp", "zwnj", "zwj", "lrm", "rlm",
    ]
    .iter()
    .copied()
    .collect()
}

/// Protect inline plugin syntax by converting to markers
///
/// Converts various inline plugin patterns into safe markers:
/// - `&function{content};` → marker with content
/// - `&function(args){content};` → marker with args and content
/// - `&function(args);` → marker with args
/// - `&function;` → marker (excluding HTML entities)
pub fn protect_inline_plugins(input: &str) -> String {
    let mut result = input.to_string();

    // Protect inline plugins with content but no args: &function{content};
    let inline_plugin_noargs_content = Regex::new(r"&(\w+)\{((?:[^{}]|\{[^}]*\})*)\};").unwrap();
    result = inline_plugin_noargs_content
        .replace_all(&result, |caps: &regex::Captures| {
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
    let inline_plugin = Regex::new(r"&(\w+)\(([^)]*)\)\{((?:[^{}]|\{[^}]*\})*)\};").unwrap();
    result = inline_plugin
        .replace_all(&result, |caps: &regex::Captures| {
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
    // Function name must start with a letter to avoid conflicts with HTML entities
    let inline_plugin_noargs = Regex::new(r"&([a-zA-Z]\w*);").unwrap();
    let entities = html_entities();

    result = inline_plugin_noargs
        .replace_all(&result, |caps: &regex::Captures| {
            let function = &caps[1];

            // Skip HTML entities
            if entities.contains(function) {
                return caps[0].to_string();
            }

            format!(
                "{{{{INLINE_PLUGIN_NOARGS:{}:INLINE_PLUGIN_NOARGS}}}}",
                function
            )
        })
        .to_string();

    result
}

/// Protect block plugin syntax by converting to markers
///
/// Converts various block plugin patterns into safe markers:
/// - `@function(args){{ content }}` → marker with content
/// - `@function(args){content}` → marker with content
/// - `@function(args)` → marker with args
pub fn protect_block_plugins(input: &str) -> String {
    let mut result = input.to_string();

    // Protect block plugins multiline: @function(args){{ content }}
    let block_plugin_multi = Regex::new(r"@(\w+)\(([^)]*)\)\{\{([\s\S]*?)\}\}").unwrap();
    result = block_plugin_multi
        .replace_all(&result, |caps: &regex::Captures| {
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
        .replace_all(&result, |caps: &regex::Captures| {
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
    let block_plugin_argsonly = Regex::new(r"@(\w+)\(([^)]*)\)").unwrap();
    result = block_plugin_argsonly
        .replace_all(&result, |caps: &regex::Captures| {
            let function = &caps[1];
            let args = &caps[2];
            let encoded_args = general_purpose::STANDARD.encode(args.as_bytes());
            format!(
                "{{{{BLOCK_PLUGIN_ARGSONLY:{}:{}:BLOCK_PLUGIN_ARGSONLY}}}}",
                function, encoded_args
            )
        })
        .to_string();

    result
}

/// Protect block-type plugin syntax written with the `::: 記法` notation
///
/// Converts:
///
/// ```text
/// :::function args
/// content
/// :::
/// ```
///
/// into a safe marker carrying the function name, raw args (the remainder
/// of the opening line), and base64-encoded content.
///
/// A block is closed by the *first* line that consists of exactly `:::`
/// (optionally trailing whitespace). Nesting is not supported: any `:::`
/// block, or `@`/`&` plugin syntax, found between the opening and closing
/// line is captured verbatim as opaque literal content rather than being
/// parsed as a nested plugin.
pub fn protect_colon_block_plugins(input: &str) -> String {
    static START_LINE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?m)^:::(\w+)(?:[ \t]+([^\r\n]*))?[ \t]*\r?\n").unwrap());
    static CLOSE_LINE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?m)^:::[ \t]*(?:\r?\n|$)").unwrap());

    let mut result = String::with_capacity(input.len());
    let mut pos = 0usize;

    while let Some(start_caps) = START_LINE.captures_at(input, pos) {
        let start_match = start_caps.get(0).unwrap();

        // Preserve any text before the opening marker unchanged
        result.push_str(&input[pos..start_match.start()]);

        let function = start_caps.get(1).map_or("", |m| m.as_str());
        let args = start_caps.get(2).map_or("", |m| m.as_str()).trim();
        let content_start = start_match.end();

        match CLOSE_LINE.captures_at(input, content_start) {
            Some(close_caps) => {
                let close_match = close_caps.get(0).unwrap();
                let raw_content = &input[content_start..close_match.start()];
                // The newline terminating the last content line belongs to
                // the closing fence, not the content itself.
                let content = raw_content
                    .strip_suffix("\r\n")
                    .or_else(|| raw_content.strip_suffix('\n'))
                    .unwrap_or(raw_content);
                let encoded_content = general_purpose::STANDARD.encode(content.as_bytes());
                result.push_str(&format!(
                    "{{{{COLON_BLOCK_PLUGIN:{}:{}:{}:COLON_BLOCK_PLUGIN}}}}",
                    function, args, encoded_content
                ));
                pos = close_match.end();
            }
            None => {
                // No closing marker found; leave the opening line untouched
                // and keep scanning for other blocks after it.
                result.push_str(&input[start_match.start()..start_match.end()]);
                pos = start_match.end();
            }
        }
    }

    result.push_str(&input[pos..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protect_inline_plugin_with_content() {
        let input = "&test{content};";
        let output = protect_inline_plugins(input);
        assert!(output.contains("INLINE_PLUGIN:test::"));
        assert!(!output.contains("&test"));
    }

    #[test]
    fn test_protect_inline_plugin_with_args_and_content() {
        let input = "&test(arg1,arg2){content};";
        let output = protect_inline_plugins(input);
        assert!(output.contains("INLINE_PLUGIN:test:arg1,arg2:"));
    }

    #[test]
    fn test_skip_html_entities() {
        let input = "&lt; &gt; &amp;";
        let output = protect_inline_plugins(input);
        assert_eq!(input, output); // Should remain unchanged
    }

    #[test]
    fn test_protect_block_plugin_multiline() {
        let input = "@test(args){{ content }}";
        let output = protect_block_plugins(input);
        assert!(output.contains("BLOCK_PLUGIN:test:args:"));
    }

    #[test]
    fn test_protect_block_plugin_single_line() {
        let input = "@test(args){content}";
        let output = protect_block_plugins(input);
        assert!(output.contains("BLOCK_PLUGIN:test:args:"));
    }

    #[test]
    fn test_protect_block_plugin_args_only() {
        let input = "@test(args)";
        let output = protect_block_plugins(input);
        assert!(output.contains("BLOCK_PLUGIN_ARGSONLY:test:"));
    }

    #[test]
    fn test_protect_colon_block_plugin_basic() {
        let input = ":::alert warning\nSomething went wrong\n:::";
        let output = protect_colon_block_plugins(input);
        assert!(output.contains("COLON_BLOCK_PLUGIN:alert:warning:"));
        assert!(!output.contains(":::"));
    }

    #[test]
    fn test_protect_colon_block_plugin_no_args() {
        let input = ":::toc\nignored\n:::";
        let output = protect_colon_block_plugins(input);
        assert!(output.contains("COLON_BLOCK_PLUGIN:toc::"));
    }

    #[test]
    fn test_protect_colon_block_plugin_empty_content() {
        let input = ":::clear\n:::";
        let output = protect_colon_block_plugins(input);
        assert!(output.contains("COLON_BLOCK_PLUGIN:clear::"));
        assert!(output.contains("COLON_BLOCK_PLUGIN}}"));
    }

    #[test]
    fn test_protect_colon_block_plugin_multiline_content() {
        let input = ":::box\nline one\nline two\n:::";
        let output = protect_colon_block_plugins(input);

        // Decode the base64 content back out and verify roundtrip
        let marker_re = Regex::new(r"COLON_BLOCK_PLUGIN:box::([^:]+):COLON_BLOCK_PLUGIN").unwrap();
        let caps = marker_re
            .captures(&output)
            .expect("marker should be present");
        let decoded = general_purpose::STANDARD.decode(&caps[1]).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "line one\nline two");
    }

    #[test]
    fn test_protect_colon_block_plugin_unclosed_left_untouched() {
        let input = ":::alert warning\nSomething went wrong";
        let output = protect_colon_block_plugins(input);
        // No closing marker present, so the input is left unchanged
        assert_eq!(output, input);
    }

    #[test]
    fn test_protect_colon_block_plugin_nested_not_supported() {
        // The outer block closes at the FIRST bare `:::` line, which
        // belongs to the inner (nested) block. The nested start marker
        // is therefore captured as opaque literal content, and the
        // outer's real closing `:::` is left over as plain text.
        let input = ":::outer args\n:::inner args2\ncontent\n:::\n:::";
        let output = protect_colon_block_plugins(input);
        assert!(output.contains("COLON_BLOCK_PLUGIN:outer:args:"));
        // The leftover closing line for the (never-matched) outer block remains
        assert!(output.trim_end().ends_with(":::"));
    }
}
