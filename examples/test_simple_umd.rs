use umd::parse;

fn main() {
    let lukiwiki_table = "| head1 | head2 | head3 |\n\
                          | content1 | content2 | content3 |";

    println!("=== Input ===");
    println!("{}", lukiwiki_table);

    println!("\n=== Output ===");
    let html = parse(lukiwiki_table);
    println!("{}", html);
}
