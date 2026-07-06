use umd::parse;

fn main() {
    println!("=== Table Cell Spanning Examples ===\n");

    // Example 1: Basic colspan
    let colspan_basic = r#"| Header1 |> | Header3 |
|---------|--|---------|
| Cell1   | Cell2   | Cell3   |"#;
    println!("Example 1: Basic colspan");
    println!("Input:\n{}\n", colspan_basic);
    println!("Output:\n{}\n", parse(colspan_basic));
    println!("Expected: Header1 spans 2 columns\n");

    // Example 2: Multiple colspan
    let colspan_multiple = r#"| Span 3 columns |> |> |
|----------------|--|--|
| A              | B   | C |"#;
    println!("Example 2: Multiple colspan");
    println!("Input:\n{}\n", colspan_multiple);
    println!("Output:\n{}\n", parse(colspan_multiple));
    println!("Expected: Header spans 3 columns\n");

    // Example 3: Basic rowspan
    let rowspan_basic = r#"| Header1 | Header2 |
|---------|---------|
| Cell1   | Cell2   |
| |^      | Cell4   |"#;
    println!("Example 3: Basic rowspan");
    println!("Input:\n{}\n", rowspan_basic);
    println!("Output:\n{}\n", parse(rowspan_basic));
    println!("Expected: Cell1 spans 2 rows\n");

    // Example 4: Combined colspan and rowspan
    let combined = r#"| Header1 |> | Header3 |
|---------|--|---------|
| Span2x2 |> | Cell3   |
| |^      |^ | Cell4   |
| Cell5   | Cell6 | Cell7 |"#;
    println!("Example 4: Combined colspan and rowspan");
    println!("Input:\n{}\n", combined);
    println!("Output:\n{}\n", parse(combined));
    println!("Expected: Cell 'Span2x2' spans 2 columns and 2 rows\n");

    // Example 5: With Bootstrap classes
    let with_classes = r#"| CENTER: Header1 |> | RIGHT: Header3 |
|-----------------|--|----------------|
| TOP: Cell1      |> | MIDDLE: Cell3  |
| BOTTOM: Cell4   | Cell5 | Cell6     |"#;
    println!("Example 5: With Bootstrap classes and alignment");
    println!("Input:\n{}\n", with_classes);
    println!("Output:\n{}\n", parse(with_classes));
    println!(
        "Expected: Headers centered/right aligned, cells with vertical alignment, and colspan\n"
    );

    // Example 6: Complex table with multiple spans
    let complex = r#"| Product |> |> | Q1 | Q2 | Q3 | Q4 |
|---------|--|--|----|----|----|----|
| Category A |> |> | 10 | 20 | 30 | 40 |
| Item 1  | Details |> | 5  | 10 | 15 | 20 |
| Item 2  | Details |> | 5  | 10 | 15 | 20 |
| Category B |> |> | 15 | 25 | 35 | 45 |"#;
    println!("Example 6: Complex table with multiple spans");
    println!("Input:\n{}\n", complex);
    println!("Output:\n{}\n", parse(complex));
    println!("Expected: Multiple colspan in headers and cells\n");
}
