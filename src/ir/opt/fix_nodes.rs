// This is not really an optimization pass, but it happens during optimization.
// It expands Gates and makes sure some requirements for conversion to combinators are satasfied.

use crate::{common::BinOp, ir::WireColor};

use super::super::{IRModule, IRNode, IRArg};

impl IRModule {
    pub fn fix_nodes(&mut self) {
        for i in 0..self.nodes.len() {
            let node = &self.nodes[i];
            match node {
                IRNode::BinOp(lhs,op,rhs) => {
                    if lhs.is_link() && lhs == rhs {
                        // 1. fix same-arg binops
                        self.nodes[i] = IRNode::BinOpSame(lhs.clone(),op.clone());
                    } else if op.is_compare() {
                        // 2. fix comparisons (lhs cannot be constant)
                        if lhs.is_link() {
                            // fine as-is
                        } else if rhs.is_link() {
                            self.nodes[i] = IRNode::BinOp(rhs.clone(),op.flip(),lhs.clone());
                        } else if let IRArg::Constant(n) = lhs {
                            let op = op.clone();
                            let rhs = rhs.clone();
                            let n = *n;

                            let lhs_const = self.add_node_at(i,IRNode::Constant(n));
                            self.nodes[i] = IRNode::BinOp(lhs_const,op,rhs);
                        }
                    }
                },
                // 3. fix multi-driver (constant args not permitted)
                IRNode::MultiDriver(args) => {
                    let mut args = args.clone();
                    for j in 0..args.len() {
                        if let IRArg::Constant(x) = args[j] {
                            args[j] = self.add_node_at(i, IRNode::Constant(x))
                        }
                    }
                    self.nodes[i] = IRNode::MultiDriver(args);
                }
                _ => ()
            }
        }

        // 4. expand gates
        for i in 0..self.nodes.len() {
            let node = &self.nodes[i];
            match node {
                IRNode::Gate(cond,check,gated) => {
                    let check = *check;
                    let cond = cond.clone();
                    let gated = gated.clone();

                    let fixed_gated = if let IRArg::Constant(x) = gated {
                        self.add_node_at(i, IRNode::Constant(x))
                    } else {
                        gated
                    };

                    // If cond is a comparison, we can re-use it.
                    if let IRArg::Link(cond_id,_) = cond {
                        let cond_node = &self.nodes[cond_id as usize];
                        if let IRNode::BinOp(lhs,op,IRArg::Constant(rhs)) = cond_node {
                            assert!(lhs.is_link()); // should always be true (stage 2)
                            if op.is_compare() {

                                let new_node = if check {
                                    IRNode::BinOpCmpGate(lhs.clone(), op.clone(), *rhs, fixed_gated)
                                } else {
                                    IRNode::BinOpCmpGate(lhs.clone(), op.invert(), *rhs, fixed_gated)
                                };
                                self.nodes[i] = new_node;

                                continue;
                            }
                        }
                    }

                    // LHS and gated must be signals.
                    let fixed_cond = if let IRArg::Constant(x) = cond {
                        self.add_node_at(i, IRNode::Constant(x))
                    } else {
                        cond
                    };

                    let new_node = if check {
                        IRNode::BinOpCmpGate(fixed_cond, BinOp::CmpNeq, 0, fixed_gated)
                    } else {
                        IRNode::BinOpCmpGate(fixed_cond, BinOp::CmpEq, 0, fixed_gated)
                    };
                    self.nodes[i] = new_node;
                },
                _ => ()
            }
        }

        // 5. check for short-cycles, IE
        // let a = b;
        // let b = a;
        for i in 0..self.nodes.len() {
            let node = &self.nodes[i];
            if let IRNode::MultiDriver(args) = node {
                for arg in args {
                    if self.detect_short_cycles(i,arg) {
                        panic!("Short cycle detected. TODO add more useful information.");
                    }
                }
            }
        }
    }

    fn detect_short_cycles(&self, base_index: usize, arg: &IRArg) -> bool {
        if let IRArg::Link(target_index,_) = arg {
            let target_index = *target_index as usize;
            if target_index == base_index {
                return true;
            }
            let node = &self.nodes[target_index];
            if let IRNode::MultiDriver(args) = node {
                for arg in args {
                    if self.detect_short_cycles(base_index,arg) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// IIRC the point of this is to add nodes close to a specific index, to prevent spaghetti
    fn add_node_at(&mut self, i: usize, node: IRNode) -> IRArg {
        let mut offset = 0;
        while offset >= i && i + offset < self.nodes.len() {
            if let Some(IRNode::Removed) = self.nodes.get(i + offset) {
                self.nodes[i + offset] = node;
                return IRArg::Link((i + offset) as u32,WireColor::None);
            }
            if offset >= i {
                if let Some(IRNode::Removed) = self.nodes.get(i - offset) {
                    self.nodes[i - offset] = node;
                    return IRArg::Link((i - offset) as u32,WireColor::None);
                }
            }
            offset += 1;
        }
        self.add_node(node,None)
    }
}
