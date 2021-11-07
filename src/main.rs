use std::{collections::HashMap, path::Path, rc::Rc};

use clap::{Parser as CmdParser};

mod common;

mod lexer;
mod parser;
mod ir;
mod blueprint;
mod disjoint_set;
mod symbols;
mod assets;

#[derive(CmdParser)]
#[clap(version = "1.0.0", author = "cogg <adam@cogg.rocks>")]
struct CmdOptions {
    /// The source file to compile.
    filename: String,
    #[clap(default_value = "main")]
    /// The name of the top-level module to generate a blueprint for.
    mod_name: String,

    #[clap(long)]
    /// Disable all optimizations.
    no_opt: bool,
    #[clap(long)]
    /// Disable constant folding.
    no_fold: bool,
    #[clap(long)]
    /// Disable pruning unused combinators.
    no_prune: bool
}

#[derive(Debug)]
pub struct CompileSettings {
    fold_constants: bool,
    prune: bool,
    main_mod_name: String
}

fn main() {

    let options = CmdOptions::parse();

    // TODO allow users to override or add additional symbols
    let symbols_json = assets::get_asset_string("symbols.json").expect("failed to load symbol defintions");
    symbols::load_symbols(&symbols_json);

    let settings = Rc::new(CompileSettings{
        fold_constants: !(options.no_fold || options.no_opt),
        prune: !(options.no_prune || options.no_opt),
        main_mod_name: options.mod_name
    });

    let mut modules = HashMap::new();
    {
        let prelude_source = assets::get_asset_string("std/prelude.cdl").expect("failed to load prelude");
        let prelude_parsed = crate::parser::parse(&prelude_source);
        ir::build_ir(prelude_parsed, settings.clone(), &mut modules);
    };

    let source = std::fs::read_to_string(options.filename).expect("failed to read file");
    let parse_results = crate::parser::parse(&source);
    ir::build_ir(parse_results, settings.clone(), &mut modules);

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
