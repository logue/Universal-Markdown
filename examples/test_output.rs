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
            "**Bold** with inline text and *italic*",
            "Markdown emphasis",
        ),
        ("@toc(2){{ }}", "Block plugin multiline"),
        (
            "@include(file.txt){default content}",
            "Block plugin singleline",
        ),
        ("&highlight(yellow){important text};", "Inline plugin"),
        (
            "&outer(arg1){text &inner(arg2){nested}; more};",
            "Nested plugins",
        ),
        ("@box(){{ **bold** and text }}", "Plugin with wiki syntax"),
    ];

    for (input, label) in test_cases {
        println!("=== {} ===", label);
        println!("Input: {}", input);
        let output = parse(input);
        println!("Output: {}", output);
        println!();
    }
}
