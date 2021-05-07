use std::collections::HashMap;

use crate::blueprint::{ArithmeticConditions, Blueprint, ControlBehavior, Entity, Filter, Position, Signal};

use super::{IRArg, IRModule, IRNode};

struct BlueprintBuilder {
    entities: Vec<Entity>
}

fn make_pos(arg: (f32,f32)) -> Position {
    Position{x: arg.0, y: arg.1}
}

// TODO bidirectional mapping
fn symbol_to_signal(symbol: u32) -> Signal {
    match symbol {
        0 => Signal{cat:"virtual".to_owned(),name:"signal-A".to_owned()},
        1 => Signal{cat:"virtual".to_owned(),name:"signal-B".to_owned()},
        2 => Signal{cat:"virtual".to_owned(),name:"signal-C".to_owned()},
        _ => panic!("can't get symbol for {}",symbol)
    }
}

enum SymbolOrConstant {
    Symbol(u32),
    Constant(i32)
}

impl SymbolOrConstant {
    fn unpack(&self) -> (Option<Signal>,Option<i32>) {
        match self {
            Self::Symbol(x) => (Some(symbol_to_signal(*x)),None),
            Self::Constant(x) => (None,Some(*x)),
        }
    }
}

impl BlueprintBuilder{
    fn new() -> Self {
        Self{
            entities: Vec::new()
        }
    }

    fn add_constant(&mut self, pos: (f32,f32), symbol: u32, count: i32) -> usize {
        let id = self.entities.len();
        let signal = symbol_to_signal(symbol);
        self.entities.push(Entity{
            entity_number: id as u32,
            name: "constant-combinator".to_owned(),
            position: make_pos(pos),
            direction: 4,

            connections: Some(HashMap::new()),
            control_behavior: ControlBehavior{
                arithmetic_conditions: None,
                filters: Some(vec!(Filter{index:1,count,signal}))
            }
        });
        id
    }

    fn add_arithmetic(&mut self, pos: (f32,f32), operation: String, lhs: SymbolOrConstant, rhs: SymbolOrConstant, out_symbol: u32) -> usize {
        let id = self.entities.len();

        let (first_signal,first_constant) = lhs.unpack();
        let (second_signal,second_constant) = rhs.unpack();
        let output_signal = Some(symbol_to_signal(out_symbol));

        self.entities.push(Entity{
            entity_number: id as u32,
            name: "arithmetic-combinator".to_owned(),
            position: make_pos(pos),
            direction: 4,

            connections: Some(HashMap::new()),
            control_behavior: ControlBehavior{
                arithmetic_conditions: Some(ArithmeticConditions{
                    operation,
                    first_constant,
                    second_constant,
                    first_signal,
                    second_signal,
                    output_signal
                }),
                filters: None
            }
        });
        id
    }

    fn finish(self) -> Blueprint {
        Blueprint{
            entities: self.entities
        }
    }
}

impl IRModule {
    fn get_arg_symbol(&self, arg: &IRArg) -> u32 {
        match arg {
            IRArg::Link(id,_) => {
                self.out_symbols[*id as usize]
            },
            IRArg::Constant(_) => 0
        }
    }

    fn get_arg_symbol_or_const(&self, arg: &IRArg) -> SymbolOrConstant {
        match arg {
            IRArg::Link(id,_) => {
                SymbolOrConstant::Symbol(self.out_symbols[*id as usize])
            },
            IRArg::Constant(n) => SymbolOrConstant::Constant(*n)
        }
    }

    pub fn to_blueprint(&self) -> Blueprint {
        let mut builder = BlueprintBuilder::new();
        for (id, node) in self.nodes.iter().enumerate() {
            match node {
                IRNode::Input(_) => {
                    let pos = self.get_true_pos(id as u32);
                    let symbol = self.out_symbols[id];
                    builder.add_constant(pos,symbol,0);
                },
                IRNode::Output(_,arg) => {
                    let pos = self.get_true_pos(id as u32);
                    let symbol = self.get_arg_symbol(arg);
                    let const_val = if let IRArg::Constant(n) = arg { *n } else { 0 };
                    builder.add_constant(pos,symbol,const_val);
                },
                IRNode::BinOp(lhs,op,rhs) => {
                    let pos = self.get_true_pos(id as u32);
                    builder.add_arithmetic(pos,op.to_str().to_owned(), 
                        self.get_arg_symbol_or_const(lhs),
                        self.get_arg_symbol_or_const(rhs),
                        self.out_symbols[id]
                    );
                }
                _ => println!("todo build {:?}",node)
            }
        }
        builder.finish()
    }
}
