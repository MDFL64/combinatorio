use std::collections::HashMap;

use crate::blueprint::{ArithmeticConditions, Blueprint, Connection, ControlBehavior, Entity, Filter, Position, Signal};

use super::{IRArg, IRModule, IRNode, WireColor};
use crate::common::ConnectType;

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

fn get_circuit_id(ent_type: &str, connect_type: ConnectType) -> u32 {
    match ent_type {
        "constant-combinator" | "medium-electric-pole" => 1,
        "arithmetic-combinator" => match connect_type {
            ConnectType::In => 1,
            ConnectType::Out => 2
        },
        _ => panic!("can't get circuit id for: {}",ent_type)
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
        let id = self.entities.len()+1;
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

    fn add_pole(&mut self, pos: (f32,f32)) -> usize {
        let id = self.entities.len()+1;
        self.entities.push(Entity{
            entity_number: id as u32,
            name: "medium-electric-pole".to_owned(),
            position: make_pos(pos),
            direction: 4,

            connections: Some(HashMap::new()),
            control_behavior: ControlBehavior{
                arithmetic_conditions: None,
                filters: None
            }
        });
        id
    }

    fn add_arithmetic(&mut self, pos: (f32,f32), operation: String, lhs: SymbolOrConstant, rhs: SymbolOrConstant, out_symbol: u32) -> usize {
        let id = self.entities.len()+1;

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

    fn add_link(&mut self, color: WireColor, a: (usize,ConnectType), b: (usize,ConnectType)) {
        let a_circuit_id = get_circuit_id(&self.entities[a.0-1].name, a.1);
        let b_circuit_id = get_circuit_id(&self.entities[b.0-1].name, b.1);

        // TODO clean this up a bit -- add a getter for Connections that returns the list for a color
        if color == WireColor::Red {
            self.entities[a.0-1].connections.as_mut().unwrap()
                .entry(a_circuit_id).or_default()
                .red.get_or_insert_with(|| Vec::new())
                .push(Connection{entity_id: b.0 as u32, circuit_id: Some(b_circuit_id)});

            self.entities[b.0-1].connections.as_mut().unwrap()
                .entry(b_circuit_id).or_default()
                .red.get_or_insert_with(|| Vec::new())
                .push(Connection{entity_id: a.0 as u32, circuit_id: Some(a_circuit_id)});
        } else if color == WireColor::Green {
            self.entities[a.0-1].connections.as_mut().unwrap()
                .entry(a_circuit_id).or_default()
                .green.get_or_insert_with(|| Vec::new())
                .push(Connection{entity_id: b.0 as u32, circuit_id: Some(b_circuit_id)});

            self.entities[b.0-1].connections.as_mut().unwrap()
                .entry(b_circuit_id).or_default()
                .green.get_or_insert_with(|| Vec::new())
                .push(Connection{entity_id: a.0 as u32, circuit_id: Some(a_circuit_id)});
        } else {
            panic!("no wire color in blueprint gen");
        }
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
        let mut ent_ids = Vec::new();
        ent_ids.resize(self.nodes.len(), 0);
        for (id, node) in self.nodes.iter().enumerate() {
            match node {
                IRNode::Input(_) => {
                    let pos = self.get_true_pos(id as u32);
                    let symbol = self.out_symbols[id];
                    ent_ids[id] = builder.add_constant(pos,symbol,0);
                },
                IRNode::Output(_,arg) => {
                    let pos = self.get_true_pos(id as u32);
                    if let IRArg::Constant(n) = arg {
                        ent_ids[id] = builder.add_constant(pos,0,*n);
                    } else {
                        ent_ids[id] = builder.add_pole(pos);
                    }
                },
                IRNode::BinOp(lhs,op,rhs) => {
                    let pos = self.get_true_pos(id as u32);
                    ent_ids[id] = builder.add_arithmetic(pos,
                        op.to_str().to_owned(), 
                        self.get_arg_symbol_or_const(lhs),
                        self.get_arg_symbol_or_const(rhs),
                        self.out_symbols[id]
                    );
                }
                _ => println!("todo build {:?}",node)
            }
        }

        for link in &self.links {
            let (a_id,a_ty) = link.a.clone();
            let (b_id,b_ty) = link.b.clone();

            builder.add_link(link.color, 
                (ent_ids[a_id as usize],a_ty),
                (ent_ids[b_id as usize],b_ty)
            );

            println!("{:?}",link);
        }

        builder.finish()
    }
}
