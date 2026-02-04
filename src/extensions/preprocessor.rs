//! Preprocessor utilities for conflict resolution
//!
//! This module handles early-stage text processing before Markdown parsing.

/// Remove comment syntax from input
///
/// Removes single-line comments (`//`) and multi-line comments (`/* ... */`)
/// while preserving comments inside code blocks and inline code.
///
/// # Arguments
///
/// * `input` - The raw markup input
///
/// # Returns
///
/// String with comments removed
pub fn remove_comments(input: &str) -> String {
    let ends_with_newline = input.ends_with('\n');
    let mut result = String::new();
    let mut in_code_block = false;
    let mut code_fence_marker = "";
    let mut in_multiline_comment = false;

    for line in input.lines() {
        // Detect code block start/end
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            if !in_code_block {
                in_code_block = true;
                code_fence_marker = if trimmed.starts_with("```") {
                    "```"
                } else {
                    "~~~"
                };
            } else if trimmed.contains(code_fence_marker) {
                in_code_block = false;
            }
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // Inside code block: preserve everything
        if in_code_block {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // Process line outside code blocks
        let mut processed_line = String::new();
        let mut chars = line.chars().peekable();
        let mut in_inline_code = false;
        let mut prev_ch = '\0';

        while let Some(ch) = chars.next() {
            // Detect inline code
            if ch == '`' {
                in_inline_code = !in_inline_code;
                processed_line.push(ch);
                prev_ch = ch;
                continue;
            }

            // Inside inline code: preserve everything
            if in_inline_code {
                processed_line.push(ch);
                prev_ch = ch;
                continue;
            }

            // Multi-line comment start: /*
            if !in_multiline_comment && ch == '/' && chars.peek() == Some(&'*') {
                in_multiline_comment = true;
                chars.next(); // consume '*'
                prev_ch = '*';
                continue;
            }

            // Multi-line comment end: */
            if in_multiline_comment && ch == '*' && chars.peek() == Some(&'/') {
                in_multiline_comment = false;
                chars.next(); // consume '/'
                prev_ch = '/';
                continue;
            }

            // Single-line comment start: //
            // But NOT if preceded by ':' (URL scheme like https://)
            if !in_multiline_comment && ch == '/' && chars.peek() == Some(&'/') && prev_ch != ':' {
                // Skip rest of line
                break;
            }

            // Normal character (not in comment)
            if !in_multiline_comment {
                processed_line.push(ch);
                prev_ch = ch;
            }
        }

        // Add processed line if not empty or if we're still in multiline comment
        if !processed_line.trim().is_empty() {
            result.push_str(&processed_line);
            result.push('\n');
        } else if !in_multiline_comment {
            // Preserve empty lines (important for Markdown structure)
            result.push('\n');
        }
    }

    // Remove trailing newline if input didn't have one
    if !ends_with_newline && result.ends_with('\n') {
        result.pop();
    }

    result
}

/// Process definition lists (:term|definition syntax)
///
/// Converts consecutive lines starting with `:term|definition` into
/// marker placeholders that will be converted to HTML later.
pub fn process_definition_lists(input: &str) -> String {
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
                let items_json = serde_json::to_string(&dl_items).unwrap();
                result.push(format!(
                    "{{{{DEFINITION_LIST:{}:DEFINITION_LIST}}}}",
                    items_json
                ));
            }
        } else {
            result.push(line.to_string());
        }
    }

    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_single_line_comment() {
        let input = "text // comment\nmore text";
        let output = remove_comments(input);
        assert!(!output.contains("comment"));
        assert!(output.contains("text"));
        assert!(output.contains("more text"));
    }

    #[test]
    fn test_remove_multiline_comment() {
        let input = "text /* comment */ more";
        let output = remove_comments(input);
        assert!(!output.contains("comment"));
        assert!(output.contains("text"));
        assert!(output.contains("more"));
    }

    #[test]
    fn test_preserve_url_slashes() {
        let input = "https://example.com";
        let output = remove_comments(input);
        assert!(output.contains("https://example.com"));
    }

    #[test]
    fn test_preserve_comments_in_code_block() {
        let input = "```\n// code comment\n```";
        let output = remove_comments(input);
        assert!(output.contains("// code comment"));
    }

    #[test]
    fn test_definition_list() {
        let input = ":term1|definition1\n:term2|definition2\nregular text";
        let output = process_definition_lists(input);
        assert!(output.contains("{{DEFINITION_LIST:"));
        assert!(output.contains("DEFINITION_LIST}}"));
        assert!(output.contains("regular text"));
    }
}
