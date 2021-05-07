mod common;

mod lexer;
mod parser;
mod ir;
mod blueprint;

fn main() {
    let source = std::fs::read_to_string("test.c8r").expect("failed to read file");
    let results = crate::parser::parse(&source);
    
    let mut ir_mod = ir::build_ir(results);
    ir_mod.select_colors();
    ir_mod.select_symbols();
    ir_mod.place_nodes();

    ir_mod.print();

    let bp = blueprint::read_blueprint("0eNqVkU1uhDAMhe/iZRUq/kaUHGK2XVQjFMDtWIIEJQYNQty9DkhtpS6qZmf7+fOzs0E7zDh5sgx6A+qcDaDfNgj0Yc0Qc7xOCBqIcQQF1owxMp74PiJTl3RubMkadh52BWR7fIDO9psCtExMeAKPYG3sPLboRfAHSsHkgnQ7Gz0IMcmL+vmiYAVdlLVM6sljdwpKBWKcvRuaFu9mIQFI1ze5kXJ/0EIsvJMP3PxacSHPs2S+rJ2K5DUuFjAyIiiwiddKsjRNFbgJvTltwJO0upmn+R/wK+zy5FrHgfWP/1CwoA8HOX/JyqrOq6KqLlla7vsnCIWVwA==");

    let bp_obj = ir_mod.to_blueprint();

    let bp_string = blueprint::write_blueprint(bp_obj);
    println!("{}",bp_string);

    /*blueprint::write_blueprint(r#"{
        "blueprint":{
            "entities":[{"entity_number":1,"name":"constant-combinator","position":{"x":-239.5,"y":347.5},"direction":4,"control_behavior":{"filters":[{"signal":{"type":"virtual","name":"signal-O"},"count":9,"index":1}]},"connections":{"1":{"red":[{"entity_id":3,"circuit_id":1}]}}},{"entity_number":2,"name":"arithmetic-combinator","position":{"x":-238.5,"y":350},"direction":4,"control_behavior":{"arithmetic_conditions":{"second_constant":0,"operation":"*"}},"connections":{"1":{"green":[{"entity_id":3,"circuit_id":2}]},"2":{"red":[{"entity_id":3,"circuit_id":2}]}}},{"entity_number":3,"name":"arithmetic-combinator","position":{"x":-239.5,"y":350},"direction":4,"control_behavior":{"arithmetic_conditions":{"first_signal":{"type":"virtual","name":"signal-1"},"second_constant":50,"operation":"+","output_signal":{"type":"virtual","name":"signal-O"}}},"connections":{"1":{"red":[{"entity_id":1}]},"2":{"red":[{"entity_id":2,"circuit_id":2}],"green":[{"entity_id":2,"circuit_id":1}]}}}]}}"#)
*/
}
