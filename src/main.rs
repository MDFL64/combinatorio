use std::{collections::HashMap, rc::Rc};

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

    // Load prelude
    {
        let prelude_source = assets::get_asset_string("std/prelude.cdl").expect("failed to load prelude");
        let prelude_parsed = crate::parser::parse(&prelude_source);
        ir::build_ir(prelude_parsed, settings.clone(), &mut modules);
    }

    // Load main source file
    {
        let source = std::fs::read_to_string(options.filename).expect("failed to read file");
        let parse_results = crate::parser::parse(&source);
        ir::build_ir(parse_results, settings.clone(), &mut modules);
    }

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
}
