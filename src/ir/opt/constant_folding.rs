use std::collections::{HashMap, HashSet};

use super::super::{IRModule, IRNode, IRArg, WireColor};

impl IRModule {
    // Attempts to covert constant nodes to constant args.
    fn fix_const(&self, arg: &IRArg) -> IRArg {
        if let IRArg::Link(id,_) = arg {
            let node = &self.nodes[*id as usize];
            if let IRNode::Constant(n) = node {
                return IRArg::Constant(*n);
            }
        }
        arg.clone()
    }

    pub fn fold_constants(&mut self) {
        // Doing this iteratively might be kinda dumb, but because of the
        // order of our nodes, we should usually finish in only a couple passes.
        loop {
            let mut changes = 0;
            for index in 0..self.nodes.len() {
                let node = &self.nodes[index];
                match node {
                    IRNode::Input(..) | IRNode::Constant(..) => (),
                    IRNode::Output(id,arg) => {
                        self.nodes[index] = IRNode::Output(*id,self.fix_const(arg));
                    },
                    IRNode::BinOp(lhs,op,rhs) |
                    IRNode::BinOpCmp(lhs,op,rhs) => {
                        let lhs = self.fix_const(lhs);
                        let rhs = self.fix_const(rhs);
                        
                        if let IRArg::Constant(const_l) = lhs {
                            if let IRArg::Constant(const_r) = rhs {
                                self.nodes[index] = IRNode::Constant(op.fold(const_l,const_r));
                                changes += 1;
                                continue;
                            }
                        }
    
                        self.nodes[index] = IRNode::BinOp(lhs,op.clone(),rhs);
                    },
                    _ => println!("fold {:?}",node)
                }
            }
            println!("fold changed {}",changes);
            if changes == 0 {
                break;
            }
        }
    }
}
