use std::collections::HashMap;

use super::{IRArg, IRModule, IRNode, WireColor};


#[derive(Default,Clone)]
struct ColorCounts {
    red: i32,
    green: i32
}

fn update_color_for_arg(arg: &mut IRArg, forbid_color: WireColor, out_color_counts: &mut Vec<ColorCounts>) -> WireColor {
    match arg {
        IRArg::Link(parent,color) => {
            assert_eq!(*color,WireColor::None);

            let counts = &mut out_color_counts[*parent as usize];

            let red_picked = match forbid_color {
                WireColor::Red => false,
                WireColor::Green => true,
                WireColor::None => counts.red >= counts.green
            };

            *color = if red_picked {
                counts.red += 1;
                WireColor::Red
            } else {
                counts.green += 1;
                WireColor::Green
            };
            
            *color
        },
        IRArg::Constant(_) => WireColor::None
    }
}

impl IRModule {
    /// Is the argument connected to a module input?
    fn is_input(&self, arg: &IRArg) -> bool {
        match arg {
            IRArg::Link(parent,_) => {
                let node = self.nodes.get(*parent as usize);
                if let IRNode::Input(_) = node {
                    true
                } else if let IRNode::MultiDriver(args) = node {
                    args.iter().any(|x| self.is_input(x))
                } else {
                    false
                }
            },
            IRArg::Constant(_) => false
        }
    }

    pub fn select_colors(&mut self) {
        let mut out_color_counts: Vec<ColorCounts> = Vec::new();
        out_color_counts.resize(self.nodes.len(), Default::default());

        let mut inputs = HashMap::new();

        for i in 0..self.nodes.len() {
            let node = self.nodes.get(i);
            match node {
                IRNode::BinOp(lhs,_,rhs) |
                IRNode::BinOpCmpGate(lhs,_,_,rhs) => {
                    let li = self.is_input(lhs);
                    let ri = self.is_input(rhs);

                    if li && ri {
                        panic!("CANNOT COLOR BOTH ARGS RED: {:?} {:?}",lhs,rhs);
                    }

                    if li {
                        inputs.insert(i, 0);
                    }

                    if ri {
                        inputs.insert(i, 1);
                    }
                },
                IRNode::BinOpSame(arg,_) => {
                    if self.is_input(arg) {
                        inputs.insert(i, 0);
                    }
                },
                _ => ()
            }
        }

        for i in 0..self.nodes.len() {
            let node = self.nodes.get_mut(i);
            match node {
                IRNode::Input(_) => (),
                IRNode::Constant(_) => (),
                IRNode::Removed => (),
                IRNode::BinOp(lhs,_,rhs) |
                IRNode::BinOpCmpGate(lhs,_,_,rhs) => {
                    let input_side = inputs.get(&i);
                    let forbid_color = match input_side {
                        Some(0) => WireColor::Green,
                        Some(1) => WireColor::Red,
                        _ => WireColor::None
                    };
                    let forbid_color = update_color_for_arg(lhs, forbid_color, &mut out_color_counts);
                    update_color_for_arg(rhs , forbid_color, &mut out_color_counts);
                },
                IRNode::BinOpSame(arg,_) => {
                    let input_side = inputs.get(&i);
                    let forbid_color = if input_side.is_some() { WireColor::Green } else { WireColor::None };
                    update_color_for_arg(arg, forbid_color, &mut out_color_counts);
                },
                IRNode::Output(_,arg) => {
                    // Force red outputs
                    update_color_for_arg(arg , WireColor::Green, &mut out_color_counts);
                },
                IRNode::MultiDriver(_) => (), // use colors determined by downstream nodes
                _ => panic!("Node {:?} is not supported at this stage.",node)
            }
        }
    }
}
