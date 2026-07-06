use comrak::{Options, markdown_to_html};

fn main() {
    let mut options = Options::default();

    // GFM table
    let gfm_table = "| Header1 | Header2 | Header3 |\n\
                     |---------|---------|--------|\n\
                     | Cell1   | Cell2   | Cell3   |";

    println!("=== comrakのデフォルト出力 ===");
    let html = markdown_to_html(gfm_table, &options);
    println!("{}", html);

    println!("\n=== GFM拡張有効 ===");
    options.extension.table = true;
    let html = markdown_to_html(gfm_table, &options);
    println!("{}", html);
}
