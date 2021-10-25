use std::{path::Path, rc::Rc};

mod common;

mod lexer;
mod parser;
mod ir;
mod blueprint;
mod disjoint_set;
mod symbols;

#[derive(Debug)]
pub struct CompileSettings {
    fold_constants: bool,
    prune: bool,
    main_mod_name: String
}

fn main() {

    let main_mod_name = std::env::args().nth(1).unwrap_or_else(|| "main".into());

    // TODO: load relative to executable/build path instead of PWD?
    symbols::load_symbols(Path::new("symbols.json"));

    let source = std::fs::read_to_string("test.c8").expect("failed to read file");
    let parse_results = crate::parser::parse(&source);
    
    let settings = Rc::new(CompileSettings{
        fold_constants: true,
        prune: true,
        main_mod_name
    });


    let mut modules = ir::build_ir(parse_results, settings.clone());

    if let Some(ir_mod) = modules.get_mut(&settings.main_mod_name) {
        ir_mod.select_colors();
        ir_mod.select_symbols();
        ir_mod.layout_nodes();

        let bp_obj = ir_mod.to_blueprint();

        let bp_string = blueprint::write_blueprint(bp_obj);
        println!();
        println!("{}",bp_string);
    } else {
        panic!("Main module '{}' not found.",settings.main_mod_name);
    }

    //ir_mod.print();

    //blueprint::read_blueprint("0eNrFkt1qwzAMhd9F1+5o03RtTG92sXcYjGGcRN0EsR38UxaK331yMsagMNab7VKyzuejgy7QDglHTzaCvAB1zgaQzxcI9Gr1UHpxGhEkUEQDAqw2peqxox79qnOmJauj85AFkO3xHeQmvwhAGykSLrS5mJRNpkXPAz9xBIwusNTZ8jvjVpumvtsJmEBudw1/05PHbhmoBbDl6N2gWnzTZ2IAqz6xit/6GRVK90Q+RHW12Zl8TNz5MrVMrB7KSiWQqEs6h3WpzKj9bFPCkRUuxTHdwHxamOPE1pKN6uSdUWSZAfKkh4A5Z3GVV3VrXtt/yytgYfxe9LgE8gex8k3ONyy/nbyAM/owJ1MdNvW+qfbbQ72+r9c5fwCl4xEW");

    /*blueprint::write_blueprint(r#"{
        "blueprint":{
            "entities":[{"entity_number":1,"name":"constant-combinator","position":{"x":-239.5,"y":347.5},"direction":4,"control_behavior":{"filters":[{"signal":{"type":"virtual","name":"signal-O"},"count":9,"index":1}]},"connections":{"1":{"red":[{"entity_id":3,"circuit_id":1}]}}},{"entity_number":2,"name":"arithmetic-combinator","position":{"x":-238.5,"y":350},"direction":4,"control_behavior":{"arithmetic_conditions":{"second_constant":0,"operation":"*"}},"connections":{"1":{"green":[{"entity_id":3,"circuit_id":2}]},"2":{"red":[{"entity_id":3,"circuit_id":2}]}}},{"entity_number":3,"name":"arithmetic-combinator","position":{"x":-239.5,"y":350},"direction":4,"control_behavior":{"arithmetic_conditions":{"first_signal":{"type":"virtual","name":"signal-1"},"second_constant":50,"operation":"+","output_signal":{"type":"virtual","name":"signal-O"}}},"connections":{"1":{"red":[{"entity_id":1}]},"2":{"red":[{"entity_id":2,"circuit_id":2}],"green":[{"entity_id":2,"circuit_id":1}]}}}]}}"#)
*/
}
