use lukiwiki_parser::parse;

fn main() {
    let output = parse("COLOR(success): This is a success message");
    println!("Output: {}", output);
    println!("Contains text-success: {}", output.contains("text-success"));
    println!("Contains bg-success: {}", output.contains("bg-success"));
}
