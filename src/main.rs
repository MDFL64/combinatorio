mod lexer;
mod parser;

fn main() {
    let source = std::fs::read_to_string("test.c8r").expect("failed to read file");
    let results = crate::parser::parse(&source);
    println!("{:#?}",results);
}
