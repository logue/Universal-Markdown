//! Integration tests for `ParserOptions::allow_bidi_in_code_blocks`.

use umd::{parse_with_frontmatter_opts, parser::ParserOptions};

#[test]
fn bidi_control_chars_stripped_in_code_blocks_by_default() {
    let input = "```\nlet a = 1;\u{202E}\n```\n";
    let options = ParserOptions::default();
    let result = parse_with_frontmatter_opts(input, &options);

    assert!(!result.html.contains('\u{202E}'));
}

#[test]
fn bidi_control_chars_preserved_in_code_blocks_when_enabled() {
    let input = "```\nlet a = 1;\u{202E}\n```\n";
    let mut options = ParserOptions::default();
    options.allow_bidi_in_code_blocks = true;
    let result = parse_with_frontmatter_opts(input, &options);

    assert!(result.html.contains('\u{202E}'));
}

#[test]
fn bidi_control_chars_still_stripped_outside_code_blocks_when_enabled() {
    let input = "plain text\u{202E}here\n";
    let mut options = ParserOptions::default();
    options.allow_bidi_in_code_blocks = true;
    let result = parse_with_frontmatter_opts(input, &options);

    assert!(!result.html.contains('\u{202E}'));
}
