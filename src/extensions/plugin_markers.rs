//! Plugin syntax marker processing
//!
//! This module handles the conversion of plugin syntax into safe markers
//! that won't be affected by Markdown parsing.

use base64::{Engine as _, engine::general_purpose};
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
}
