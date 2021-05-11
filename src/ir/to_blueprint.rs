use std::collections::HashMap;

use crate::blueprint::{ArithmeticConditions, Blueprint, Connection, ControlBehavior, DeciderConditions, Entity, Filter, Position, Signal};

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
        3 => Signal{cat:"virtual".to_owned(),name:"signal-D".to_owned()},
        4 => Signal{cat:"virtual".to_owned(),name:"signal-E".to_owned()},
        5 => Signal{cat:"virtual".to_owned(),name:"signal-F".to_owned()},
        6 => Signal{cat:"virtual".to_owned(),name:"signal-G".to_owned()},
        _ => panic!("can't get signal for {}, please add more signals",symbol)
    }
}

fn get_circuit_id(ent_type: &str, connect_type: ConnectType) -> u32 {
    match ent_type {
        "constant-combinator" | "medium-electric-pole" => 1,
        "arithmetic-combinator" | "decider-combinator" => match connect_type {
            ConnectType::In => 1,
            ConnectType::Out => 2
        },
        _ => panic!("can't get circuit id for: {}",ent_type)
    }
}

#[derive(Clone)]
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
                decider_conditions: None,
                filters: Some(vec!(Filter{index:1,count,signal}))
            }
        });
        id
    }

    fn add_pole(&mut self, pos: (f32,f32), substation: bool) -> usize {
        let id = self.entities.len()+1;
        self.entities.push(Entity{
            entity_number: id as u32,
            name: if substation { "substation" } else { "medium-electric-pole"}.to_owned(),
            position: make_pos(pos),
            direction: 4,

            connections: Some(HashMap::new()),
            control_behavior: ControlBehavior{
                arithmetic_conditions: None,
                decider_conditions: None,
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
                decider_conditions: None,
                filters: None
            }
        });
        id
    }

    fn add_decider(&mut self, pos: (f32,f32), comparator: String, lhs_symbol: u32, rhs: SymbolOrConstant, out_symbol: u32, copy_count_from_input: bool) -> usize {
        let id = self.entities.len()+1;

        let first_signal = symbol_to_signal(lhs_symbol);
        let (second_signal,constant) = rhs.unpack();
        let output_signal = Some(symbol_to_signal(out_symbol));

        self.entities.push(Entity{
            entity_number: id as u32,
            name: "decider-combinator".to_owned(),
            position: make_pos(pos),
            direction: 4,

            connections: Some(HashMap::new()),
            control_behavior: ControlBehavior{
                decider_conditions: Some(DeciderConditions{
                    comparator,
                    constant,
                    first_signal,
                    second_signal,
                    output_signal,
                    copy_count_from_input
                }),
                arithmetic_conditions: None,
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

    fn get_bounds(&self) -> [f32;4] {
        let mut max_x = f32::MIN;
        let mut min_x = f32::MAX;
        let mut max_y = max_x;
        let mut min_y = min_x;
        for ent in &self.entities {
            max_x = max_x.max(ent.position.x);
            min_x = min_x.min(ent.position.x);
            max_y = max_y.max(ent.position.y);
            min_y = min_y.min(ent.position.y);
        }
        [min_x,min_y,max_x,max_y]
    }

    fn finish(self) -> Blueprint {
        Blueprint{
            entities: self.entities
        }
    }
}

impl IRModule {

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
                IRNode::Constant(x) => {
                    let pos = self.get_true_pos(id as u32);
                    let symbol = self.out_symbols[id];
                    ent_ids[id] = builder.add_constant(pos,symbol,*x);
                },
                IRNode::Output(_,arg) => {
                    let pos = self.get_true_pos(id as u32);
                    if let IRArg::Constant(n) = arg {
                        ent_ids[id] = builder.add_constant(pos,0,*n);
                    } else {
                        ent_ids[id] = builder.add_pole(pos,false);
                    }
                },
                IRNode::BinOp(lhs,op,rhs) => {
                    let pos = self.get_true_pos(id as u32);
                    if !op.is_compare() {
                        ent_ids[id] = builder.add_arithmetic(pos,
                            op.to_str().to_owned(), 
                            self.get_arg_symbol_or_const(lhs),
                            self.get_arg_symbol_or_const(rhs),
                            self.out_symbols[id]
                        );
                    } else {
                        if let IRArg::Link(lhs_id,_) = lhs {
                            let lhs_symbol = self.out_symbols[*lhs_id as usize];
                            let pos = self.get_true_pos(id as u32);
                            ent_ids[id] = builder.add_decider(pos,
                                op.to_str().to_owned(),
                                lhs_symbol,
                                self.get_arg_symbol_or_const(rhs),
                                self.out_symbols[id],
                                false
                            );
                        } else {
                            panic!("Bad compare, constant on LHS.");
                        }
                    }
                },
                IRNode::BinOpSame(arg,op) => {
                    let pos = self.get_true_pos(id as u32);
                    let arg_val = self.get_arg_symbol_or_const(arg);
                    ent_ids[id] = builder.add_arithmetic(pos,
                        op.to_str().to_owned(), 
                        arg_val.clone(),
                        arg_val,
                        self.out_symbols[id]
                    );
                },
                IRNode::BinOpCmpGate(lhs,op,rhs,_gated) => {
                    if let IRArg::Link(lhs_id,_) = lhs {
                        let lhs_symbol = self.out_symbols[*lhs_id as usize];
                        let pos = self.get_true_pos(id as u32);
                        ent_ids[id] = builder.add_decider(pos,
                            op.to_str().to_owned(),
                            lhs_symbol,
                            SymbolOrConstant::Constant(*rhs),
                            self.out_symbols[id],
                            true
                        );
                    } else {
                        panic!("Bad compare, constant on LHS.");
                    }
                },
                IRNode::MultiDriver(_) => (), // virtual node, not built
                _ => panic!("Node {:?} is not supported at this stage.",node)
            }
        }

        for link in &self.links {
            let (a_id,a_ty) = link.a.clone();
            let (b_id,b_ty) = link.b.clone();

            builder.add_link(link.color, 
                (ent_ids[a_id as usize],a_ty),
                (ent_ids[b_id as usize],b_ty)
            );
        }

        let [x_min,_y_min,x_max,y_max] = builder.get_bounds();

        let mut x_pole_start = 0;
        let mut x_pole_end = 0;
        let y_pole_start = 0;
        let mut y_pole_end = 0;
        while 0.5 + x_pole_start as f32 * 18.0 - 9.0 > x_min {
            x_pole_start -= 1;
        }
        while 0.5 + x_pole_end as f32 * 18.0 + 9.0 < x_max {
            x_pole_end += 1;
        }
        while 0.5 + y_pole_end as f32 * 18.0 + 9.0 < y_max {
            y_pole_end += 1;
        }

        for y in y_pole_start..=y_pole_end {
            for x in x_pole_start..=x_pole_end {
                builder.add_pole((0.5+18.0*x as f32,0.5+18.0*y as f32), true);
            }
        }

        builder.finish()
    }
}
