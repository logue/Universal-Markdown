use umd::parse;

fn main() {
    // LukiWiki形式のテーブル
    let lukiwiki = r#"| head1 | head2 | head3 |
| content1 | content2 | content3 |"#;

    println!("=== LukiWiki形式 ===");
    println!("Input:\n{}\n", lukiwiki);
    println!("Output:\n{}\n", parse(lukiwiki));

    // GFM形式のテーブル
    let gfm = r#"| head1 | head2 | head3 |
|-------|-------|-------|
| content1 | content2 | content3 |"#;

    println!("=== GFM形式 ===");
    println!("Input:\n{}\n", gfm);
    println!("Output:\n{}\n", parse(gfm));

    // LukiWiki形式でセル連結
    let lukiwiki_with_colspan = r#"| Header1 |> | Header3 |
| Cell1 | Cell2 | Cell3 |"#;

    println!("=== LukiWiki形式（セル連結） ===");
    println!("Input:\n{}\n", lukiwiki_with_colspan);
    println!("Output:\n{}\n", parse(lukiwiki_with_colspan));

    // LukiWiki形式で色指定
    let lukiwiki_with_color = r#"| COLOR(primary): Header1 | Header2 |
| Cell1 | Cell2 |"#;

    println!("=== LukiWiki形式（色指定） ===");
    println!("Input:\n{}\n", lukiwiki_with_color);
    println!("Output:\n{}\n", parse(lukiwiki_with_color));
}
