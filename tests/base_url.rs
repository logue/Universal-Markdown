use umd::{parse_with_frontmatter_opts, parser::ParserOptions};

#[test]
fn test_base_url_with_links() {
    let input = "[docs](/docs)\n[api](/api/v1)";
    let mut opts = ParserOptions::default();
    opts.base_url = Some("/app".to_string());

    let result = parse_with_frontmatter_opts(input, &opts);
    assert!(result.html.contains(r#"href="/app/docs""#));
    assert!(result.html.contains(r#"href="/app/api/v1""#));
}

#[test]
fn test_base_url_with_media() {
    let input = "![logo](/logo.png)\n![banner](/images/banner.jpg)";
    let mut opts = ParserOptions::default();
    opts.base_url = Some("/assets".to_string());

    let result = parse_with_frontmatter_opts(input, &opts);
    assert!(result.html.contains(r#"src="/assets/logo.png""#));
    assert!(result.html.contains(r#"src="/assets/images/banner.jpg""#));
}

#[test]
fn test_base_url_with_trailing_slash() {
    let input = "[home](/home)";
    let mut opts = ParserOptions::default();
    opts.base_url = Some("/app/".to_string());

    let result = parse_with_frontmatter_opts(input, &opts);
    // Trailing slash should be removed
    assert!(result.html.contains(r#"href="/app/home""#));
}

#[test]
fn test_base_url_with_full_url() {
    let input = "[docs](/docs)";
    let mut opts = ParserOptions::default();
    opts.base_url = Some("https://example.com/app".to_string());

    let result = parse_with_frontmatter_opts(input, &opts);
    assert!(
        result
            .html
            .contains(r#"href="https://example.com/app/docs""#)
    );
}

#[test]
fn test_base_url_preserves_external_urls() {
    let input = "[external](https://example.com/page)\n[external2](http://example.com)";
    let mut opts = ParserOptions::default();
    opts.base_url = Some("/app".to_string());

    let result = parse_with_frontmatter_opts(input, &opts);
    // External URLs should not be modified
    assert!(result.html.contains(r#"href="https://example.com/page""#));
    assert!(result.html.contains(r#"href="http://example.com""#));
}

#[test]
fn test_base_url_with_single_quotes() {
    // apply_base_url_to_links handles single-quoted attributes defensively
    // (e.g. developer-supplied Icons HTML, or other raw-HTML postprocessing
    // inputs) — exercised directly here rather than through parse(), since
    // literal HTML in markdown *source* is correctly neutralized by comrak's
    // render.unsafe = false and never reaches this rewriter as real HTML.
    let html = r#"<a href='/docs'>Docs</a>"#;
    let result = umd::extensions::conflict_resolver::apply_base_url_to_links(html, "/app");
    assert!(result.contains(r#"href='/app/docs'"#));
}

#[test]
fn test_base_url_none() {
    let input = "[docs](/docs)";
    let opts = ParserOptions::default();

    let result = parse_with_frontmatter_opts(input, &opts);
    // Without base_url, paths should remain as-is
    assert!(result.html.contains(r#"href="/docs""#));
}

#[test]
fn test_base_url_with_mixed_links() {
    let input = "[home](/)\n[docs](/docs)\n[external](https://example.com)";
    let mut opts = ParserOptions::default();
    opts.base_url = Some("/app".to_string());

    let result = parse_with_frontmatter_opts(input, &opts);
    assert!(result.html.contains(r#"href="/app/""#));
    assert!(result.html.contains(r#"href="/app/docs""#));
    assert!(result.html.contains(r#"href="https://example.com""#));
}

#[test]
fn test_idn_external_link_adds_warning_marker() {
    let input = "[国際化ドメイン](https://日本.jp)";
    let opts = ParserOptions::default();

    let result = parse_with_frontmatter_opts(input, &opts);
    assert!(result.html.contains("国際化ドメイン"));
    assert!(result.html.contains(r#"class="umd-idn-warning-link""#));
    assert!(result.html.contains(r#"data-idn-warning="true""#));
    assert!(result.html.contains(r#"class="umd-idn-warning-icon""#));
}

#[test]
fn test_ascii_external_link_has_no_warning_marker() {
    let input = "[docs](https://example.com/docs)";
    let opts = ParserOptions::default();

    let result = parse_with_frontmatter_opts(input, &opts);
    assert!(!result.html.contains("umd-idn-warning-link"));
    assert!(!result.html.contains("data-idn-warning"));
}

#[test]
fn test_relative_link_has_no_warning_marker() {
    let input = "[docs](/docs)";
    let mut opts = ParserOptions::default();
    opts.base_url = Some("/app".to_string());

    let result = parse_with_frontmatter_opts(input, &opts);
    assert!(result.html.contains(r#"href="/app/docs""#));
    assert!(!result.html.contains("umd-idn-warning-link"));
    assert!(!result.html.contains("data-idn-warning"));
}
