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
    pub fn select_colors(&mut self) {
        let mut out_color_counts: Vec<ColorCounts> = Vec::new();
        out_color_counts.resize(self.nodes.len(), Default::default());

        for node in &mut self.nodes {
            match node {
                IRNode::Input(_) => (),
                IRNode::Constant(_) => (),
                IRNode::Removed => (),
                IRNode::BinOp(lhs,_,rhs) |
                IRNode::BinOpCmpGate(lhs,_,_,rhs) => {
                    let forbid_color = update_color_for_arg(lhs,WireColor::None, &mut out_color_counts);
                    update_color_for_arg(rhs , forbid_color, &mut out_color_counts);
                },
                IRNode::BinOpSame(arg,_) => {
                    update_color_for_arg(arg,WireColor::None, &mut out_color_counts);
                },
                IRNode::Output(_,arg) => {
                    update_color_for_arg(arg , WireColor::None, &mut out_color_counts);
                },
                IRNode::MultiDriver(_) => (), // use colors determined by downstream nodes
                _ => panic!("Node {:?} is not supported at this stage.",node)
            }
        }
    }
}
