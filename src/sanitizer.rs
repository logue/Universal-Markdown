//! Markdown-source sanitization module
//!
//! Raw-HTML injection safety is *not* handled here — it's delegated to
//! comrak's own `render.unsafe = false` (see parser.rs), which recognizes
//! and neutralizes actual HTML constructs during rendering (replacing them
//! with an HTML comment) while leaving legitimate CommonMark syntax like
//! `>` blockquotes untouched. An earlier version of this module pre-escaped
//! `<`/`>`/`&` in the raw source before comrak ever ran, which duplicated
//! that protection and, as a side effect, broke blockquote parsing (every
//! line-leading `>` became `&gt;` before comrak could see it as a
//! blockquote marker) — removed for that reason.
//!
//! What's left here: stripping invisible/disallowed characters (zero-width
//! chars, BOM, BiDi control characters) from the source, and blocking
//! dangerous URL schemes.

use std::borrow::Cow;

/// Sanitizes a URL by blocking dangerous schemes
///
/// # Arguments
///
/// * `url` - The URL to sanitize
///
/// # Returns
///
/// A sanitized URL or `#blocked-url` if the scheme is blocked
///
/// # Blocked Schemes
///
/// - `javascript:` - JavaScript execution XSS
/// - `data:` - Base64 encoded script injection XSS
/// - `vbscript:` - VBScript execution XSS (IE legacy)
/// - `file:` - Local file system access (information leakage)
///
/// Note: `file:` scheme is blocked by default for security reasons, but may be
/// useful in specific use cases such as:
/// - Standalone software offline help systems
/// - Local document management applications
/// - Electron/Tauri apps with local resource access
///
/// Future enhancement: Consider adding a configuration option to allow `file:`
/// scheme when explicitly enabled by the application developer (see planned-features.md).
///
/// # Behavior
///
/// When a dangerous scheme is detected:
/// - In explicit autolinks (`<url>`): the URL is rendered as plain text (not linked)
/// - In inline links (`[text](url)`): the link is replaced with `#blocked-url` for safety
///
/// Allowed schemes include:
/// - Standard protocols: `http:`, `https:`, `mailto:`, `tel:`, `ftp:`
/// - Custom app schemes: `spotify:`, `discord:`, `vscode:`, `steam:`, etc.
/// - Relative paths: `/path`, `./path`, `#anchor`
///
/// # Examples
///
/// ```
/// use umd::sanitizer::sanitize_url;
///
/// assert_eq!(sanitize_url("https://example.com"), "https://example.com");
/// assert_eq!(sanitize_url("javascript:alert(1)"), "#blocked-url");
/// assert_eq!(sanitize_url("data:text/html,<script>alert(1)</script>"), "#blocked-url");
/// assert_eq!(sanitize_url("spotify:track:123"), "spotify:track:123"); // Custom app schemes allowed
/// ```
pub fn sanitize_url(url: &str) -> Cow<'_, str> {
    let normalized = remove_disallowed_blank_chars(url);
    let url_lower = normalized.trim().to_lowercase();

    // Check for dangerous schemes (case-insensitive)
    // TODO: Consider adding ParserOptions.allow_file_scheme configuration
    // to conditionally allow file:// in trusted environments (see planned-features.md)
    if url_lower.starts_with("javascript:")
        || url_lower.starts_with("data:")
        || url_lower.starts_with("vbscript:")
        || url_lower.starts_with("file:")
    {
        return Cow::Borrowed("#blocked-url");
    }

    normalized
}

/// Sanitizes input text by removing disallowed invisible blank-like
/// characters (zero-width chars, BOM, BiDi control characters).
///
/// Does *not* escape `<`/`>`/`&` — raw-HTML safety is handled downstream by
/// comrak's `render.unsafe = false` at render time, which preserves
/// legitimate CommonMark syntax (e.g. `>` blockquotes) that a blind
/// pre-escape would otherwise corrupt. See the module docs for why.
///
/// # Arguments
///
/// * `input` - The raw input text to sanitize
///
/// # Returns
///
/// The input with disallowed invisible characters removed.
///
/// # Examples
///
/// ```
/// use umd::sanitizer::sanitize;
///
/// let input = "Hello\u{200B}World"; // zero-width space
/// assert_eq!(sanitize(input), "HelloWorld");
///
/// // Markdown/HTML-shaped text passes through untouched — comrak handles it.
/// let input = "> a blockquote";
/// assert_eq!(sanitize(input), "> a blockquote");
/// ```
pub fn sanitize(input: &str) -> Cow<'_, str> {
    sanitize_opts(input, false)
}

/// Sanitizes input text, same as [`sanitize`], but allows configuring whether
/// BiDi control characters are preserved inside fenced code blocks.
///
/// # Arguments
///
/// * `input` - The raw input text to sanitize
/// * `allow_bidi_in_code_blocks` - When `true`, BiDi control characters
///   (`U+202A`-`U+202E`, `U+2066`-`U+2069`) are left untouched inside fenced
///   code blocks (` ``` ` / `~~~`), so RTL-heavy code samples or BiDi-attack
///   demonstrations can be shown verbatim. All other disallowed invisible
///   characters (zero-width chars, BOM) are still stripped everywhere, and
///   BiDi characters outside code blocks are always removed regardless of
///   this flag. Defaults to `false` (disabled) via [`sanitize`].
pub fn sanitize_opts(input: &str, allow_bidi_in_code_blocks: bool) -> Cow<'_, str> {
    remove_disallowed_blank_chars_opts(input, allow_bidi_in_code_blocks)
}

fn remove_disallowed_blank_chars(input: &str) -> Cow<'_, str> {
    if !input.chars().any(is_disallowed_blank_char) {
        return Cow::Borrowed(input);
    }

    let filtered: String = input
        .chars()
        .filter(|&ch| !is_disallowed_blank_char(ch))
        .collect();

    Cow::Owned(filtered)
}

/// Same as [`remove_disallowed_blank_chars`], but when `allow_bidi_in_code_blocks`
/// is `true`, BiDi control characters inside fenced code blocks (` ``` ` / `~~~`)
/// are preserved instead of stripped. Other disallowed invisible characters are
/// still removed everywhere, and code fence detection mirrors
/// [`remove_ascii_control_chars_from_markup`].
fn remove_disallowed_blank_chars_opts(
    input: &str,
    allow_bidi_in_code_blocks: bool,
) -> Cow<'_, str> {
    if !allow_bidi_in_code_blocks {
        return remove_disallowed_blank_chars(input);
    }

    if !input.chars().any(is_disallowed_blank_char) {
        return Cow::Borrowed(input);
    }

    let ends_with_newline = input.ends_with('\n');
    let mut result = String::with_capacity(input.len());
    let mut in_code_block = false;
    let mut code_fence_char = '`';

    for line in input.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let fence_char = if trimmed.starts_with("```") { '`' } else { '~' };
            if !in_code_block {
                in_code_block = true;
                code_fence_char = fence_char;
            } else if fence_char == code_fence_char {
                in_code_block = false;
            }
            result.push_str(line);
            result.push('\n');
            continue;
        }

        for c in line.chars() {
            let skip = if in_code_block {
                is_disallowed_blank_char(c) && !is_bidi_control_char(c)
            } else {
                is_disallowed_blank_char(c)
            };
            if !skip {
                result.push(c);
            }
        }
        result.push('\n');
    }

    if !ends_with_newline && result.ends_with('\n') {
        result.pop();
    }

    Cow::Owned(result)
}

/// Returns true for BiDi control characters that can be used to visually
/// spoof text direction (Trojan Source style attacks): LRE, RLE, PDF, LRO,
/// RLO (`U+202A`-`U+202E`) and LRI, RLI, FSI, PDI (`U+2066`-`U+2069`).
fn is_bidi_control_char(ch: char) -> bool {
    ('\u{202A}'..='\u{202E}').contains(&ch) || ('\u{2066}'..='\u{2069}').contains(&ch)
}

fn is_disallowed_blank_char(ch: char) -> bool {
    matches!(
        ch,
        '\u{200B}' // Zero Width Space
            | '\u{200C}' // Zero Width Non-Joiner
            | '\u{200D}' // Zero Width Joiner
            | '\u{FEFF}' // Zero Width No-Break Space / BOM
            | '\u{3164}' // Hangul Filler
    ) || is_bidi_control_char(ch)
}

/// Returns true for ASCII C0 control characters (except TAB, LF, CR) and DEL.
///
/// Removed:
/// - U+0000–U+0008: NUL, SOH, STX, ETX, EOT, ENQ, ACK, BEL, BS
/// - U+000B: VT (vertical tab)
/// - U+000C: FF (form feed)
/// - U+000E–U+001F: SO through US
/// - U+007F: DEL
///
/// Preserved:
/// - U+0009 (TAB), U+000A (LF), U+000D (CR) — required for Markdown formatting
fn is_ascii_control_char(ch: char) -> bool {
    let c = ch as u32;
    matches!(c, 0x00..=0x08 | 0x0B | 0x0C | 0x0E..=0x1F | 0x7F)
}

/// Remove ASCII control characters from markup source while preserving content
/// inside fenced code blocks (` ``` ` / `~~~`).
///
/// Plugin content is already base64-encoded by the conflict resolver before
/// this function is called, so plugin markers are safe without special handling.
///
/// # Arguments
///
/// * `input` - Preprocessed Markdown source (after conflict resolution)
///
/// # Returns
///
/// Source with control characters removed from non-code-block regions.
///
/// # Examples
///
/// ```
/// use umd::sanitizer::remove_ascii_control_chars_from_markup;
///
/// let input = "hello\x01world";
/// assert_eq!(remove_ascii_control_chars_from_markup(input), "helloworld");
///
/// // Content inside code blocks is preserved
/// let with_fence = "text\n```\nhello\x01world\n```\n";
/// let result = remove_ascii_control_chars_from_markup(with_fence);
/// assert!(result.contains("hello\x01world"));
/// ```
pub fn remove_ascii_control_chars_from_markup(input: &str) -> std::borrow::Cow<'_, str> {
    // Fast path: no control chars present
    if !input.chars().any(is_ascii_control_char) {
        return std::borrow::Cow::Borrowed(input);
    }

    let ends_with_newline = input.ends_with('\n');
    let mut result = String::with_capacity(input.len());
    let mut in_code_block = false;
    let mut code_fence_char = '`';

    for line in input.lines() {
        let trimmed = line.trim_start();

        // Detect fenced code block boundaries (``` or ~~~)
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let fence_char = if trimmed.starts_with("```") { '`' } else { '~' };
            if !in_code_block {
                in_code_block = true;
                code_fence_char = fence_char;
            } else if fence_char == code_fence_char {
                in_code_block = false;
            }
            // Fence lines are always preserved as-is
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if in_code_block {
            // Inside code block: preserve everything including control chars
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // Outside code block: strip control chars
        for c in line.chars() {
            if !is_ascii_control_char(c) {
                result.push(c);
            }
        }
        result.push('\n');
    }

    // Restore trailing-newline state (lines() strips it)
    if !ends_with_newline && result.ends_with('\n') {
        result.pop();
    }

    std::borrow::Cow::Owned(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_html() {
        let input = "Hello World";
        assert_eq!(sanitize(input), "Hello World");
    }

    #[test]
    fn test_remove_disallowed_blank_like_chars() {
        let input = "A\u{200B}B\u{200C}C\u{200D}D\u{FEFF}E\u{3164}F";
        assert_eq!(sanitize(input), "ABCDEF");
    }

    #[test]
    fn test_remove_bidi_control_chars() {
        let input = "A\u{202A}B\u{202E}C\u{2066}D\u{2069}E";
        assert_eq!(sanitize(input), "ABCDE");
    }

    #[test]
    fn test_bidi_still_removed_in_code_blocks_by_default() {
        let input = "```\nA\u{202E}B\n```\n";
        assert_eq!(sanitize(input), "```\nAB\n```\n");
    }

    #[test]
    fn test_bidi_preserved_in_code_blocks_when_opted_in() {
        let input = "```\nA\u{202E}B\n```\n";
        assert_eq!(sanitize_opts(input, true), "```\nA\u{202E}B\n```\n");
    }

    #[test]
    fn test_bidi_still_removed_outside_code_blocks_when_opted_in() {
        let input = "A\u{202E}B\n```\nC\u{202E}D\n```\nE\u{2066}F";
        assert_eq!(sanitize_opts(input, true), "AB\n```\nC\u{202E}D\n```\nEF");
    }

    #[test]
    fn test_other_invisible_chars_still_removed_in_code_blocks_when_opted_in() {
        let input = "```\nA\u{200B}B\n```\n";
        assert_eq!(sanitize_opts(input, true), "```\nAB\n```\n");
    }

    #[test]
    fn test_preserve_allowed_spaces_only() {
        let input = "A B　C";
        assert_eq!(sanitize(input), "A B　C");
    }

    #[test]
    fn test_html_and_entities_pass_through_unescaped() {
        // sanitize() no longer escapes <, >, & — comrak's render.unsafe = false
        // handles raw-HTML safety at render time instead (see module docs).
        // This is what lets legitimate CommonMark syntax like `>` blockquotes
        // survive; actual XSS-safety is verified at the parse() level in lib.rs.
        assert_eq!(
            sanitize("<script>alert('xss')</script>"),
            "<script>alert('xss')</script>"
        );
        assert_eq!(sanitize("Hello&nbsp;World"), "Hello&nbsp;World");
        assert_eq!(sanitize("A & B"), "A & B");
        assert_eq!(sanitize("> a blockquote"), "> a blockquote");
    }

    #[test]
    fn test_sanitize_url_safe_schemes() {
        assert_eq!(sanitize_url("https://example.com"), "https://example.com");
        assert_eq!(sanitize_url("http://example.com"), "http://example.com");
        assert_eq!(
            sanitize_url("mailto:user@example.com"),
            "mailto:user@example.com"
        );
        assert_eq!(sanitize_url("ftp://example.com"), "ftp://example.com");
        assert_eq!(sanitize_url("/relative/path"), "/relative/path");
        assert_eq!(sanitize_url("./relative"), "./relative");
        assert_eq!(sanitize_url("#anchor"), "#anchor");
    }

    #[test]
    fn test_sanitize_url_custom_app_schemes() {
        assert_eq!(sanitize_url("spotify:track:123"), "spotify:track:123");
        assert_eq!(sanitize_url("steam://open/game"), "steam://open/game");
        assert_eq!(sanitize_url("discord://invite/123"), "discord://invite/123");
        assert_eq!(
            sanitize_url("slack://channel?id=123"),
            "slack://channel?id=123"
        );
        assert_eq!(sanitize_url("zoom:meeting:123"), "zoom:meeting:123");
        assert_eq!(sanitize_url("vscode://file/path"), "vscode://file/path");
    }

    #[test]
    fn test_sanitize_url_blocked_schemes() {
        assert_eq!(sanitize_url("javascript:alert(1)"), "#blocked-url");
        assert_eq!(sanitize_url("JavaScript:alert(1)"), "#blocked-url");
        assert_eq!(sanitize_url("JAVASCRIPT:alert(1)"), "#blocked-url");
        assert_eq!(
            sanitize_url("data:text/html,<script>alert(1)</script>"),
            "#blocked-url"
        );
        assert_eq!(sanitize_url("Data:text/html,test"), "#blocked-url");
        assert_eq!(sanitize_url("vbscript:msgbox(1)"), "#blocked-url");
        assert_eq!(sanitize_url("VBScript:msgbox(1)"), "#blocked-url");
        assert_eq!(sanitize_url("file:///etc/passwd"), "#blocked-url");
        assert_eq!(sanitize_url("FILE:///C:/Windows"), "#blocked-url");
    }

    #[test]
    fn test_sanitize_url_with_whitespace() {
        assert_eq!(sanitize_url("  javascript:alert(1)  "), "#blocked-url");
        assert_eq!(sanitize_url("\tdata:text/html,test\n"), "#blocked-url");
        assert_eq!(
            sanitize_url("  https://example.com  "),
            "  https://example.com  "
        );
    }

    #[test]
    fn test_sanitize_url_removes_disallowed_blank_like_chars() {
        assert_eq!(
            sanitize_url("https://exa\u{200B}mple.com/\u{3164}path"),
            "https://example.com/path"
        );
    }

    #[test]
    fn test_sanitize_url_blocks_scheme_after_normalization() {
        assert_eq!(sanitize_url("java\u{200B}script:alert(1)"), "#blocked-url");
        assert_eq!(sanitize_url("data:\u{FEFF}text/html,test"), "#blocked-url");
        assert_eq!(sanitize_url("java\u{202E}script:alert(1)"), "#blocked-url");
    }

    // --- remove_ascii_control_chars_from_markup ---

    #[test]
    fn test_ascii_control_chars_removed_from_text() {
        // NUL, SOH, BEL, BS, VT, FF, SO, DEL
        let input = "hello\x00\x01\x07\x08\x0B\x0C\x0E\x7Fworld";
        assert_eq!(remove_ascii_control_chars_from_markup(input), "helloworld");
    }

    #[test]
    fn test_ascii_control_chars_preserved_tab_lf_cr() {
        // TAB, LF, CR must be preserved
        let input = "col1\tcol2\nline2\r\nline3";
        let result = remove_ascii_control_chars_from_markup(input);
        assert!(result.contains('\t'));
        assert!(result.contains('\n'));
    }

    #[test]
    fn test_ascii_control_chars_preserved_inside_code_fence() {
        let input = "text\n```\nhello\x01world\n```\nafter";
        let result = remove_ascii_control_chars_from_markup(input);
        // Control char inside code block must survive
        assert!(result.contains("hello\x01world"));
        // Regular text outside is cleaned
        assert!(result.contains("text"));
        assert!(result.contains("after"));
    }

    #[test]
    fn test_ascii_control_chars_removed_outside_code_fence() {
        let input = "be\x01fore\n```\nclean\n```\naf\x01ter";
        let result = remove_ascii_control_chars_from_markup(input);
        assert!(result.contains("before"));
        assert!(result.contains("after"));
    }

    #[test]
    fn test_ascii_control_fast_path_no_change() {
        let input = "hello world\n\ttab here";
        // Should return Borrowed (no allocation)
        assert_eq!(remove_ascii_control_chars_from_markup(input), input);
    }

    #[test]
    fn test_tilde_fence_also_protected() {
        let input = "~~~\nhello\x01world\n~~~\n";
        let result = remove_ascii_control_chars_from_markup(input);
        assert!(result.contains("hello\x01world"));
    }
}
