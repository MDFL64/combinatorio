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

    fn clone_arg(&self, arg: &IRArg) -> IRNode {
        if let IRArg::Link(id,_) = arg {
            self.nodes[*id as usize].clone()
        } else if let IRArg::Constant(n) = arg {
            IRNode::Constant(*n)
        } else {
            panic!();
        }
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
                    IRNode::Gate(cond,check,gated) => {
                        let cond = self.fix_const(cond);
                        let gated = self.fix_const(gated);

                        // If gated == 0, this gate has no effect.
                        if let IRArg::Constant(const_gated) = gated {
                            if const_gated == 0 {
                                self.nodes[index] = IRNode::Constant(0);
                                changes += 1;
                                continue;
                            }
                        }

                        // If cond is constant, evaluate to gated value or 0.
                        if let IRArg::Constant(const_cond) = cond {
                            let cond_bool = const_cond != 0;
                            self.nodes[index] = if cond_bool == *check {
                                self.clone_arg(&gated)
                            } else {
                                IRNode::Constant(0)
                            };
                            changes += 1;
                            continue;
                        }

                        self.nodes[index] = IRNode::Gate(cond,*check,gated);
                    },
                    IRNode::MultiDriver(args) => {
                        let mut const_sum: i32 = 0;
                        let mut filtered_args: Vec<IRArg> = args.clone();
                        filtered_args.retain(|arg| {
                            let arg = self.fix_const(arg);

                            if let IRArg::Constant(n) = arg {
                                const_sum = const_sum.wrapping_add(n);
                                false
                            } else {
                                true
                            }
                        });

                        if const_sum != 0 || filtered_args.len() != args.len() {
                            if filtered_args.len() == 0 {
                                self.nodes[index] = IRNode::Constant(const_sum);
                            } else {
                                filtered_args.push(IRArg::Constant(const_sum));
                                self.nodes[index] = IRNode::MultiDriver(filtered_args);
                            }
                            changes += 1;
                        }
                    },
                    _ => panic!("fold {:?}",node)
                }
            }
            println!("fold changed {}",changes);
            if changes == 0 {
                break;
            }
        }
        println!("=> {:?}",self.nodes);
    }
}
