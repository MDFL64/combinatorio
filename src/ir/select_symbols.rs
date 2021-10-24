
use crate::disjoint_set::DisjointSet;

use super::{IRModule, IRNode, IRArg};
use rand::Rng;

#[derive(Debug)]
enum SymbolConstraint {
    Equal(u32,u32),
    NotEqual(u32,u32)
}

impl IRModule {
    pub fn select_symbols(&mut self) {
        print!("Symbol selection... ");

        let mut constraints: Vec<SymbolConstraint> = Vec::new();

        for (out_i,node) in self.nodes.iter().enumerate() {
            match node {
                IRNode::Input(_) => (),
                IRNode::Constant(_) => (),
                IRNode::Removed => (),
                IRNode::Output(..) => (),
                IRNode::BinOpSame(..) => (),
                IRNode::BinOp(lhs,_,rhs) => {
                    if let IRArg::Link(lhs_in,_) = lhs {
                        if let IRArg::Link(rhs_in,_) = rhs {
                            constraints.push(SymbolConstraint::NotEqual(*lhs_in,*rhs_in));
                        }
                    }
                },
                IRNode::BinOpCmpGate(lhs,_,_,gated) => {
                    assert!(lhs.is_link());
                    assert!(gated.is_link());
                    if let IRArg::Link(lhs_in,_) = lhs {
                        if let IRArg::Link(gated_in,_) = gated {
                            constraints.push(SymbolConstraint::NotEqual(*lhs_in,*gated_in));
                            constraints.push(SymbolConstraint::Equal(*gated_in,out_i as u32));
                        }
                    }
                },
                IRNode::MultiDriver(list) => {
                    // all input symbols must match
                    // TODO it's probably going to work way better to add a MultiEqual constraint that can update everything in one step
                    for arg in list {
                        if let IRArg::Link(arg_in,_) = arg {
                            constraints.push(SymbolConstraint::Equal(*arg_in,out_i as u32));
                        }
                    }
                },
                _ => panic!("Node {:?} is not supported at this stage.",node)
            }
        }

        // Build equal sets.
        let mut equal_sets = DisjointSet::new(self.nodes.len());
        for cons in &constraints {
            match cons {
                SymbolConstraint::Equal(a,b) => {
                    equal_sets.merge(*a as usize, *b as usize);
                },
                _ => ()
            }
        }

        // Panic in the event of un-solvable constraints.
        for cons in &constraints {
            match cons {
                SymbolConstraint::NotEqual(a,b) => {
                    let set_a = equal_sets.get(*a as usize);
                    let set_b = equal_sets.get(*b as usize);
                    if set_a == set_b {
                        panic!("Conflicting equality and inequality constraints.");
                    }
                },
                _ => ()
            }
        }

        // Fix inequalities
        self.out_symbols.resize(self.nodes.len(),0);
        let mut pass_num = 1;
        let mut errors = 0;
        loop {
            for cons in &constraints {
                match cons {
                    SymbolConstraint::NotEqual(a,b) => {
                        let set_a = equal_sets.get(*a as usize);
                        let set_b = equal_sets.get(*b as usize);

                        if self.out_symbols[set_a] == self.out_symbols[set_b] {
                            errors += 1;
                            self.out_symbols[set_a] += 1;
                        }
                    },
                    _ => ()
                }
            }
            if errors == 0 {
                break;
            }
            pass_num += 1;
            errors = 0;
        }

        // Correct the signal id for all nodes.
        for i in 0..self.nodes.len() {
            let set_i = equal_sets.get(i);
            self.out_symbols[i] = self.out_symbols[set_i];
        }

        println!("Done in {} passes.",pass_num);
    }
}
