use umd::parse;

#[test]
fn test_semantic_html_integration() {
    // Test dfn
    let html = parse("&dfn{API};");
    println!("DFN output: {}", html);
    assert!(
        html.contains("<dfn>API</dfn>"),
        "Expected dfn tag, got: {}",
        html
    );

    // Test kbd
    let html = parse("&kbd{Ctrl};");
    println!("KBD output: {}", html);
    assert!(
        html.contains("<kbd>Ctrl</kbd>"),
        "Expected kbd tag, got: {}",
        html
    );

    // Test existing color (red is a named palette color, so should use class)
    let html = parse("&color(red){text};");
    println!("COLOR output: {}", html);
    assert!(
        html.contains(r#"class="umd-color-red""#),
        "Expected umd-color-red class, got: {}",
        html
    );
}
