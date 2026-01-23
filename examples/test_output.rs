use lukiwiki_parser::parse;

fn main() {
    let test_cases = vec![
        (
            "> This is a LukiWiki-style blockquote <",
            "LukiWiki blockquote",
        ),
        (
            "> This is a Markdown blockquote\n> With multiple lines",
            "Markdown blockquote",
        ),
        ("COLOR(red): Red text", "COLOR decoration"),
        (
            "**Bold** with &color(red){colored}; and *italic*",
            "Inline decorations",
        ),
    ];

    for (input, label) in test_cases {
        println!("=== {} ===", label);
        println!("Input: {}", input);
        let output = parse(input);
        println!("Output: {}", output);
        println!();
    }
}
