use std::rc::Rc;

mod common;

mod lexer;
mod parser;
mod ir;
mod blueprint;

pub struct CompileSettings {
    fold_constants: bool
}

fn main() {
    let source = std::fs::read_to_string("test.c8r").expect("failed to read file");
    let parse_results = crate::parser::parse(&source);
    
    let settings = Rc::new(CompileSettings{
        fold_constants: true
    });

    let mut ir_mod = ir::build_ir(parse_results, settings);
    ir_mod.select_colors();
    ir_mod.select_symbols();
    ir_mod.layout_nodes();

    //ir_mod.print();

    //blueprint::read_blueprint("0eNqVkN0KwjAMRt/lu+5k6+ZfX0WGuBk0sKaj68Qx+u6u80YQBC8TknOSb0bTjdR7lgAzg1snA8xpxsA3uXSpF6aeYMCBLBTkYlNl6cqjzaijNnhus951hKjAcqUnTBFrBZLAgenNW4vpLKNtyC8Dv0kKvRuWZSfpggWY6fK42SpMMGW122wXlRDf7o0bfRLoOqovif5fkv+SFHV6aw3CfOSm8CA/rBh9KKr9Ue/LQ5XvqjzGF/UAc5w=");

    let bp_obj = ir_mod.to_blueprint();

    let bp_string = blueprint::write_blueprint(bp_obj);
    println!();
    println!("{}",bp_string);

    /*blueprint::write_blueprint(r#"{
        "blueprint":{
            "entities":[{"entity_number":1,"name":"constant-combinator","position":{"x":-239.5,"y":347.5},"direction":4,"control_behavior":{"filters":[{"signal":{"type":"virtual","name":"signal-O"},"count":9,"index":1}]},"connections":{"1":{"red":[{"entity_id":3,"circuit_id":1}]}}},{"entity_number":2,"name":"arithmetic-combinator","position":{"x":-238.5,"y":350},"direction":4,"control_behavior":{"arithmetic_conditions":{"second_constant":0,"operation":"*"}},"connections":{"1":{"green":[{"entity_id":3,"circuit_id":2}]},"2":{"red":[{"entity_id":3,"circuit_id":2}]}}},{"entity_number":3,"name":"arithmetic-combinator","position":{"x":-239.5,"y":350},"direction":4,"control_behavior":{"arithmetic_conditions":{"first_signal":{"type":"virtual","name":"signal-1"},"second_constant":50,"operation":"+","output_signal":{"type":"virtual","name":"signal-O"}}},"connections":{"1":{"red":[{"entity_id":1}]},"2":{"red":[{"entity_id":2,"circuit_id":2}],"green":[{"entity_id":2,"circuit_id":1}]}}}]}}"#)
*/
}
