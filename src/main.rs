mod common;

mod lexer;
mod parser;
mod ir;

fn main() {
    let source = std::fs::read_to_string("test.c8r").expect("failed to read file");
    let results = crate::parser::parse(&source);

    println!("{:#?}",results);
    
    ir::build_ir(results);
}
