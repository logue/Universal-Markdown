//! Bootstrap 5 integration tests
//!
//! Tests for Bootstrap class generation and styling features

use umd::parse;

#[test]
fn test_bootstrap_table_default_class() {
    let input = "| Header |\n|--------|\n| Cell   |";
    let output = parse(input);
    assert!(output.contains(r#"<table class="table">"#));
}

#[test]
fn test_bootstrap_blockquote_default_class() {
    let input = "> This is a quote";
    let output = parse(input);
    assert!(output.contains(r#"<blockquote class="blockquote">"#));
}

#[test]
fn test_gfm_alert_note() {
    let input = "> [!NOTE]\n> This is an informational note";
    let output = parse(input);
    assert!(output.contains(r#"<aside class="umd-note umd-note-note">"#));
    assert!(output.contains(r#"<p class="umd-note-title">Note</p>"#));
    assert!(output.contains("This is an informational note"));
}

#[test]
fn test_gfm_alert_warning() {
    let input = "> [!WARNING]\n> Be careful here";
    let output = parse(input);
    assert!(output.contains(r#"<aside class="umd-note umd-note-warning">"#));
    assert!(output.contains(r#"<p class="umd-note-title">Warning</p>"#));
}

#[test]
fn test_gfm_alert_tip() {
    let input = "> [!TIP]\n> Here's a helpful tip";
    let output = parse(input);
    assert!(output.contains(r#"<aside class="umd-note umd-note-tip">"#));
    assert!(output.contains(r#"<p class="umd-note-title">Tip</p>"#));
}

#[test]
fn test_badge_basic() {
    let input = "Check this &badge(blue){New};";
    let output = parse(input);
    assert!(output.contains(r#"<span class="badge bg-blue">New</span>"#));
}

#[test]
fn test_badge_with_link() {
    let input = "&badge(red){[Error](/error)};";
    let output = parse(input);
    assert!(output.contains(r#"<a href="/error" class="badge bg-red">Error</a>"#));
}

#[test]
fn test_color_bootstrap_class() {
    let input = "&color(blue){Blue text};";
    let output = parse(input);
    assert!(output.contains(r#"class="text-blue""#));
}

#[test]
fn test_hex_colors_are_disabled_by_default() {
    let input = "&color(#FF5733){Custom hex color};";
    let output = parse(input);
    assert_eq!(output, "<p>Custom hex color</p>");

    let mut options = umd::parser::ParserOptions::default();
    options.allow_hex_colors = true;
    let enabled = umd::parse_with_frontmatter_opts(input, &options).html;
    assert!(enabled.contains(r#"style="color: #FF5733""#));
}

#[test]
fn test_color_custom_bootstrap_colors() {
    // Test custom Bootstrap colors (blue, yellow, teal, etc.)
    // Should now use Bootstrap classes like text-blue, text-yellow, etc.
    let test_cases = vec![
        ("&color(blue){Blue text};", r#"class="text-blue""#),
        ("&color(yellow){Yellow text};", r#"class="text-yellow""#),
        ("&color(teal){Teal text};", r#"class="text-teal""#),
    ];

    for (input, expected) in test_cases {
        let output = parse(input);
        assert!(
            output.contains(expected),
            "Failed for input: {} - output: {}",
            input,
            output
        );
    }
}

#[test]
fn test_block_color_custom_bootstrap_colors() {
    // Test block-level custom colors
    let test_cases = vec![
        ("COLOR(blue): Blue block", "text-blue"),
        ("COLOR(yellow): Yellow block", "text-yellow"),
        ("COLOR(teal): Teal block", "text-teal"),
    ];

    for (input, expected_class) in test_cases {
        let output = parse(input);
        assert!(
            output.contains(&format!(r#"class="{}""#, expected_class)),
            "Failed for input: {} - output: {}",
            input,
            output
        );
    }
}

#[test]
fn test_color_background_custom_colors() {
    // Test background colors with Bootstrap custom colors
    let input = "&color(,blue){Text on blue background};";
    let output = parse(input);
    assert!(output.contains(r#"class="bg-blue""#));

    let input = "&color(cyan,yellow){Cyan text on yellow};";
    let output = parse(input);
    assert!(output.contains(r#"class="text-cyan bg-yellow""#));
}

#[test]
fn test_size_bootstrap_class() {
    let input = "&size(1.5){Medium text};";
    let output = parse(input);
    assert!(output.contains(r#"class="fs-4""#));
}

#[test]
fn test_size_custom_value() {
    let input = "&size(3rem){Large text};";
    let output = parse(input);
    assert!(output.contains(r#"style="font-size: 3rem""#));
}

#[test]
fn test_block_color_bootstrap() {
    let input = "COLOR(green): This is a green message";
    let output = parse(input);
    assert!(output.contains(r#"class="text-green""#));
}

#[test]
fn test_block_size_bootstrap() {
    let input = "SIZE(2): Large heading text";
    let output = parse(input);
    assert!(output.contains(r#"class="fs-2""#));
}

#[test]
fn test_block_alignment() {
    let input = "CENTER: Centered text";
    let output = parse(input);
    assert!(output.contains(r#"class="text-center""#));
}

#[test]
fn test_block_justify_alignment() {
    let input = "JUSTIFY: この文章は両端揃えです";
    let output = parse(input);
    assert!(output.contains(r#"class="text-justify""#));
    assert!(output.contains("この文章は両端揃えです"));
}

#[test]
fn test_block_truncate() {
    let input = "TRUNCATE: 長いテキストは省略表示されます";
    let output = parse(input);
    assert!(output.contains(r#"class="text-truncate""#));
    assert!(output.contains("長いテキストは省略表示されます"));
}

#[test]
fn test_block_placement_justify_for_umd_table() {
    let input = "JUSTIFY:\n| Header1 | Header2 |\n| Cell1 | Cell2 |";
    let output = parse(input);
    assert!(output.contains("<table"), "output: {}", output);
    assert!(output.contains("umd-table"), "output: {}", output);
    assert!(output.contains("w-100"), "output: {}", output);
    assert!(!output.contains("<p>JUSTIFY:</p>"));
}

#[test]
fn test_block_placement_center_for_block_plugin() {
    let input = "CENTER:\n@callout(info)";
    let output = parse(input);
    assert!(output.contains("umd-plugin-callout"), "output: {}", output);
    assert!(output.contains("w-auto"), "output: {}", output);
    assert!(output.contains("mx-auto"), "output: {}", output);
    assert!(!output.contains("CENTER:"));
}

#[test]
fn test_compound_prefixes() {
    let input = "SIZE(1.5): COLOR(blue): CENTER: Styled text";
    let output = parse(input);
    // Order may vary
    assert!(output.contains(r#"class="#));
    assert!(output.contains("fs-4"));
    assert!(output.contains("text-blue"));
    assert!(output.contains("text-center"));
}

#[test]
fn test_table_cell_vertical_alignment_top() {
    let input = "| TOP: Header |\n|-------------|\n| Cell        |";
    let output = parse(input);
    assert!(output.contains(r#"class="align-top""#));
}

#[test]
fn test_table_cell_vertical_alignment_middle() {
    let input = "| MIDDLE: Data |\n|-------------|\n| Cell         |";
    let output = parse(input);
    assert!(output.contains(r#"class="align-middle""#));
}

#[test]
fn test_definition_list() {
    let input = ":HTML|HyperText Markup Language\n:CSS|Cascading Style Sheets";
    let output = parse(input);
    assert!(output.contains("<dl>"));
    assert!(output.contains("<dt>HTML</dt>"));
    assert!(output.contains("<dd>HyperText Markup Language</dd>"));
    assert!(output.contains("<dt>CSS</dt>"));
    assert!(output.contains("<dd>Cascading Style Sheets</dd>"));
    assert!(output.contains("</dl>"));
}

#[test]
fn test_definition_list_single_item() {
    let input = ":JavaScript|A programming language for the web";
    let output = parse(input);
    assert!(output.contains("<dl>"));
    assert!(output.contains("<dt>JavaScript</dt>"));
    assert!(output.contains("<dd>A programming language for the web</dd>"));
    assert!(output.contains("</dl>"));
}

#[test]
fn test_mixed_bootstrap_features() {
    let input = r#"
# Heading

&badge(cyan){New}; This is &color(blue){important}; text.

| TOP: Header | MIDDLE: Data |
|-------------|--------------|
| Cell 1      | Cell 2       |

:Term|Definition of the term
"#;
    let output = parse(input);

    // Check all features are present
    assert!(output.contains(r#"class="badge bg-cyan""#));
    assert!(output.contains(r#"class="text-blue""#));
    // UMD table syntax (because of TOP: and MIDDLE: prefixes)
    assert!(output.contains(r#"class="table umd-table""#));
    assert!(output.contains(r#"class="align-top""#));
    assert!(output.contains(r#"class="align-middle""#));
    assert!(output.contains("<dl>"));
    assert!(output.contains("<dt>Term</dt>"));
}

#[test]
fn test_strikethrough_compatibility() {
    let input = "%%UMD strikethrough%% and ~~GFM strikethrough~~";
    let output = parse(input);
    assert!(output.contains("<s>UMD strikethrough</s>"));
    assert!(output.contains("<del>GFM strikethrough</del>"));
}

#[test]
fn test_media_line_start_treated_as_block() {
    let input = "![alt](image.png \"Title\")";
    let output = parse(input);
    assert!(output.contains(r#"<figure class="w-100">"#));
    assert!(output.contains("<picture"));
    assert!(output.contains("src=\"image.png\""));
}

#[test]
fn test_right_prefix_places_media_right() {
    let input = "RIGHT:\n![alt](image.png \"Title\")";
    let output = parse(input);
    assert!(output.contains(r#"<figure class="ms-auto me-0">"#));
    assert!(output.contains("<picture"));
    assert!(output.contains("src=\"image.png\""));
    assert!(!output.contains("RIGHT:"));
}

#[test]
fn test_mermaid_code_block_rendered_as_svg() {
    let input = "```mermaid\nflowchart TD\n  A[Start] --> B[End]\n```";
    let output = parse(input);
    assert!(output.contains("mermaid-diagram"));
    assert!(output.contains("<svg"));
    assert!(!output.contains("language-mermaid"));
}

#[test]
fn test_code_block_syntax_highlighted_with_syntect() {
    let input = "```rust\nfn main() {\n    println!(\"hello\");\n}\n```";
    let output = parse(input);
    assert!(output.contains("language-rust"));
    assert!(output.contains("syntect-highlight"));
    assert!(output.contains("data-highlighted=\"true\""));
    assert!(output.contains("syntect-"));
}

#[test]
fn test_code_block_with_filename_uses_figure_caption() {
    let input = "```rust:src/main.rs\nfn main() {\n    println!(\"hello\");\n}\n```";
    let output = parse(input);
    assert!(
        output.contains(r#"<figure class="code-block">"#),
        "output: {}",
        output
    );
    assert!(
        output.contains(r#"<figcaption class="code-filename"><span class="filename">src/main.rs</span></figcaption>"#),
        "output: {}",
        output
    );
    assert!(output.contains("language-rust"), "output: {}", output);
}

#[test]
fn test_code_block_with_filename_without_language() {
    let input = "```:config.yml\nkey: value\n```";
    let output = parse(input);
    assert!(
        output.contains(r#"<figure class="code-block">"#),
        "output: {}",
        output
    );
    assert!(output.contains("config.yml"), "output: {}", output);
    assert!(output.contains("<pre>key: value"), "output: {}", output);
    assert!(
        !output.contains("language-umd-nolang"),
        "output: {}",
        output
    );
}

#[test]
fn test_table_plugin_applies_bootstrap_variants() {
    let input = "@table(striped,hover){{\n| H1 | H2 |\n|----|----|\n| A  | B  |\n}}";
    let output = parse(input);

    assert!(output.contains("table-striped"), "output: {}", output);
    assert!(output.contains("table-hover"), "output: {}", output);
}

#[test]
fn test_table_plugin_responsive_wrapper() {
    let input = "@table(responsive){{\n| H1 | H2 |\n|----|----|\n| A  | B  |\n}}";
    let output = parse(input);

    assert!(
        output.contains(r#"<div class="table-responsive"><table"#),
        "output: {}",
        output
    );
}

#[test]
fn test_table_plugin_applies_to_first_table_only() {
    let input = "@table(hover){{\n| H1 | H2 |\n|----|----|\n| A  | B  |\n\n| X1 | X2 |\n|----|----|\n| Y1 | Y2 |\n}}";
    let output = parse(input);

    assert_eq!(
        output.matches("table-hover").count(),
        1,
        "output: {}",
        output
    );
    assert!(output.matches("<table").count() >= 2, "output: {}", output);
}

#[test]
fn test_table_plugin_without_table_falls_back_to_content() {
    let input = "@table(striped){{\nこれはテキストです\n}}";
    let output = parse(input);

    assert!(output.contains("これはテキストです"), "output: {}", output);
    assert!(!output.contains("umd-plugin-table"), "output: {}", output);
}

#[test]
fn test_math_formula_inline_support() {
    let input = "&math(\\frac{a}{b});";
    let output = parse(input);

    assert!(output.contains("<math"), "output: {}", output);
    assert!(
        output.contains("<mfrac>") || output.contains("<mrow>"),
        "output: {}",
        output
    );
}

#[test]
fn test_popover_inline_support() {
    let input = "テキスト&popover(詳細){短い補足説明です};テキスト";
    let output = parse(input);

    assert!(
        output.contains("command=\"show-popover\""),
        "output: {}",
        output
    );
    assert!(
        output.contains("commandfor=\"umd-popover-"),
        "output: {}",
        output
    );
    assert!(output.contains(" popover>"), "output: {}", output);
    assert!(output.contains("短い補足説明です"), "output: {}", output);
}

#[test]
fn test_popover_block_support() {
    let input = "@popover(詳細を表示){{\n**重要**\n\n- 項目1\n- 項目2\n}}";
    let output = parse(input);

    assert!(
        output.contains("command=\"show-popover\""),
        "output: {}",
        output
    );
    assert!(output.contains("詳細を表示</button>"), "output: {}", output);
    assert!(
        output.contains("<strong>重要</strong>"),
        "output: {}",
        output
    );
    assert!(output.contains("<ul>"), "output: {}", output);
}

#[test]
fn test_math_formula_block_argsonly_support() {
    let input = "@math(\\frac{a}{b})";
    let output = parse(input);

    assert!(output.contains("<math"), "output: {}", output);
    assert!(output.contains("display=\"block\""), "output: {}", output);
    assert!(
        output.contains("<mfrac>") || output.contains("<mrow>"),
        "output: {}",
        output
    );
    assert!(!output.contains("umd-plugin-math"), "output: {}", output);
}

#[test]
fn test_math_formula_block_with_content_support() {
    let input = "@math(){{\\sum_{i=1}^{n} i}}";
    let output = parse(input);

    assert!(output.contains("<math"), "output: {}", output);
    assert!(output.contains("display=\"block\""), "output: {}", output);
    assert!(!output.contains("umd-plugin-math"), "output: {}", output);
}

#[test]
fn test_inline_code_hex_color_adds_swatch() {
    let input = "`#ffce44`";
    let output = parse(input);
    assert!(
        output.contains(
            r#"<code><span class="inline-code-color" style="background-color: #ffce44;"><span class="bi bi-palette-fill" aria-hidden="true"></span></span>#ffce44</code>"#
        ),
        "output: {}",
        output
    );
}

#[test]
fn test_inline_code_function_color_adds_swatch() {
    let test_cases = vec![
        "rgb(255,0,0)",
        "rgba(0,255,0,0.4)",
        "hsl(100, 10%, 10%)",
        "hsla(100, 24%, 40%, 0.5)",
    ];

    for color in test_cases {
        let input = format!("`{}`", color);
        let output = parse(&input);
        let expected = format!(
            r#"<code><span class="inline-code-color" style="background-color: {};"><span class="bi bi-palette-fill" aria-hidden="true"></span></span>{}</code>"#,
            color, color
        );
        assert!(
            output.contains(&expected),
            "color: {} output: {}",
            color,
            output
        );
    }
}

#[test]
fn test_inline_code_non_color_does_not_add_swatch() {
    let input = "`not-a-color`";
    let output = parse(input);
    assert!(
        output.contains("<code>not-a-color</code>"),
        "output: {}",
        output
    );
    assert!(!output.contains("inline-code-color"), "output: {}", output);
}
