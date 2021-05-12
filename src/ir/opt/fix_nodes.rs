// This is not really an optimization pass, but it happens during optimization.
// It expands Gates and makes sure some requirements for conversion to combinators are satasfied.

use crate::ir::WireColor;

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
                            // okay
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
                }
                _ => ()
            }
        }


        // 3. expand gates


        // 4. add constant nodes
    }

    fn add_node_at(&mut self, i: usize, node: IRNode) -> IRArg {
        let mut offset = 0;
        while offset < self.nodes.len() {
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
        self.add_node(node)
    }
}
