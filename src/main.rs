mod common;

mod lexer;
mod parser;
mod ir;

fn main() {
    let source = std::fs::read_to_string("test.c8r").expect("failed to read file");
    let results = crate::parser::parse(&source);
    
    let mut ir_mod = ir::build_ir(results);
    ir_mod.select_colors();
    ir_mod.select_symbols();

    ir_mod.print();

    ir_mod.place_nodes();
}
