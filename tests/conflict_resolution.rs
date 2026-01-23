//! Integration tests for syntax conflict resolution

use lukiwiki_parser::lukiwiki::conflict_resolver::detect_ambiguous_syntax;
use lukiwiki_parser::parse;

#[test]
fn test_lukiwiki_blockquote() {
    let input = "> This is a LukiWiki-style blockquote <";
    let output = parse(input);
    assert!(output.contains(r#"<blockquote class="lukiwiki">"#));
    assert!(output.contains("This is a LukiWiki-style blockquote"));
}

#[test]
fn test_emphasis_coexistence() {
    let input = "**Markdown bold** and LukiWiki bold";
    let output = parse(input);
    assert!(output.contains("<strong>Markdown bold</strong>"));
}

#[test]
fn test_italic_coexistence() {
    let input = "*Markdown italic* and LukiWiki italic";
    let output = parse(input);
    assert!(output.contains("<em>Markdown italic</em>"));
}

#[test]
fn test_triple_emphasis_no_conflict() {
    let input = "***Bold and italic*** vs Visual italic";
    let output = parse(input);
    // Markdown triple emphasis should produce nested tags
    assert!(output.contains("<strong>") || output.contains("<em>"));
}

#[test]
fn test_list_markers() {
    let input = "- Markdown unordered\n* Also Markdown unordered\n1. Markdown ordered";
    let output = parse(input);
    assert!(output.contains("<ul>"));
    assert!(output.contains("<ol>"));
}

#[test]
fn test_horizontal_rules() {
    let input = "Text before\n\n----\n\nText after\n\n***\n\nMore text";
    let output = parse(input);
    let hr_count = output.matches("<hr").count();
    assert!(
        hr_count >= 2,
        "Expected at least 2 <hr> tags, found {}",
        hr_count
    );
}

#[test]
fn test_color_decoration() {
    let input = "COLOR(red): Red text";
    let output = parse(input);
    assert!(output.contains("color: red"));
}

#[test]
fn test_plugin_syntax_preserved() {
    let input = "@toc(){{ }}";
    let output = parse(input);
    assert!(output.contains(r#"class="plugin-toc""#));
}

#[test]
fn test_code_blocks_not_processed() {
    // LukiWiki syntax inside code blocks should NOT be converted
    let input = "```\ntest code\n```";
    let output = parse(input);
    assert!(output.contains("<code>"));
}

#[test]
fn test_inline_code_not_processed() {
    let input = "Use `test` for code";
    let output = parse(input);
    assert!(output.contains("<code>"));
}

#[test]
fn test_no_false_positive_warnings() {
    let input = "# Heading\n\n**Bold** text";
    let warnings = detect_ambiguous_syntax(input);
    assert!(warnings.is_empty());
}

#[test]
fn test_complex_nesting() {
    let input = "**Markdown *with nested* emphasis**";
    let output = parse(input);
    assert!(output.contains("<strong>"));
    assert!(output.contains("<em>"));
}

#[test]
fn test_multiline_content() {
    let input = "# Heading\n\n> LukiWiki blockquote <\n\nParagraph\n\nCOLOR(blue): Blue paragraph";
    let output = parse(input);
    assert!(output.contains("<h1"));
    assert!(output.contains(r#"<blockquote class="lukiwiki">"#));
    assert!(output.contains("color: blue"));
}
