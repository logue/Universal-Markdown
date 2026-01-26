use lukiwiki_parser::parse;

fn main() {
    println!("=== Testing Header ID Generation ===\n");

    // Test 1: Sequential IDs (no custom IDs)
    println!("--- Test 1: Sequential IDs ---");
    let input1 = r#"# First Header

# Second Header

## Third Header"#;

    let output1 = parse(input1);
    println!("Input:\n{}\n", input1);
    println!("Output:\n{}\n", output1);

    // Test 2: Custom IDs
    println!("\n--- Test 2: Custom IDs ---");
    let input2 = r#"# Introduction {#intro}

This is the introduction section.

# Main Content {#main}

## Subsection {#sub}

# Conclusion"#;

    let output2 = parse(input2);
    println!("Input:\n{}\n", input2);
    println!("Output:\n{}\n", output2);

    // Test 3: Mixed Japanese and English (sequential IDs)
    println!("\n--- Test 3: Multibyte Characters (Sequential IDs) ---");
    let input3 = r#"# 日本語のヘッダー

# English Header

# 混合 Mixed ヘッダー"#;

    let output3 = parse(input3);
    println!("Input:\n{}\n", input3);
    println!("Output:\n{}\n", output3);

    // Verify custom IDs are working
    if output2.contains(r#"id="intro""#) && output2.contains(r#"id="main""#) {
        println!("\n✓ Custom header IDs are working correctly");
    } else {
        println!("\n✗ Custom header IDs not found");
    }

    // Verify sequential IDs for headers without custom IDs
    if output1.contains(r#"id="heading-1""#) && output1.contains(r#"id="heading-2""#) {
        println!("✓ Sequential header IDs are working correctly");
    } else {
        println!("✗ Sequential header IDs not found");
    }

    // Verify Japanese headers use sequential IDs (not the text itself)
    if output3.contains(r#"id="heading-"#) && !output3.contains(r#"id="日本語"#) {
        println!("✓ Multibyte characters are handled safely with sequential IDs");
    } else {
        println!("✗ Multibyte handling issue detected");
    }
}
