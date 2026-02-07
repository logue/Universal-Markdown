//! Test Discord-style underline syntax
//!
//! This example demonstrates the Discord-style underline feature:
//! - __text__ → <u>text</u> (Discord-style underline)
//! - **text** → <strong>text</strong> (remains standard Markdown)
//! - &u(text); → <u>text</u> (UMD legacy syntax)

use universal_markdown::parse;

fn main() {
    // Test Discord-style underline
    let input = "This is __underlined__ text.";
    let output = parse(input);
    println!("Input: {}", input);
    println!("Output: {}", output);
    println!();

    // Test that **text** remains as <strong>
    let input = "This is **bold** text.";
    let output = parse(input);
    println!("Input: {}", input);
    println!("Output: {}", output);
    println!();

    // Test mixed emphasis
    let input = "Mix of **bold**, __underline__, *italic*, and ''visual bold''";
    let output = parse(input);
    println!("Input: {}", input);
    println!("Output: {}", output);
    println!();

    // Test UMD legacy syntax
    let input = "Legacy &u(underline); syntax still works.";
    let output = parse(input);
    println!("Input: {}", input);
    println!("Output: {}", output);
    println!();

    // Test multiple underlines
    let input = "First __underline__ and second __underline__ in same line.";
    let output = parse(input);
    println!("Input: {}", input);
    println!("Output: {}", output);
    println!();

    // Test that single underscore works for italic
    let input = "This is _italic_ text.";
    let output = parse(input);
    println!("Input: {}", input);
    println!("Output: {}", output);
}
