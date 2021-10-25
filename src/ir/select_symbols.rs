
use crate::disjoint_set::DisjointSet;

use super::{IRModule, IRNode, IRArg};

#[derive(Debug)]
enum SymbolConstraint {
    Equal(u32,u32),
    NotEqual(u32,u32),
    EqualSymbol(u32,u32)
}

impl IRModule {
    pub fn select_symbols(&mut self) {
        print!("Symbol selection... ");

        let mut constraints: Vec<SymbolConstraint> = Vec::new();
        //println!("!! {:?}",self.arg_types);
        //println!("!! {:?}",self.ret_types);

        for (out_i,node) in self.nodes.iter().enumerate() {
            match node {
                IRNode::Input(arg_n) => {
                    if let Some(sym) = self.arg_types[*arg_n as usize] {
                        constraints.push(SymbolConstraint::EqualSymbol(out_i as u32,sym));
                    }
                },
                IRNode::Output(n,arg) => {
                    if let Some(ret_types) = &self.ret_types {
                        if let Some(sym) = ret_types[*n as usize] {
                            if let IRArg::Link(arg_i,_) = arg {
                                constraints.push(SymbolConstraint::EqualSymbol(*arg_i,sym));
                            } else {
                                constraints.push(SymbolConstraint::EqualSymbol(out_i as u32,sym));
                            }
                        }
                    }
                },
                IRNode::Constant(_) => (),
                IRNode::Removed => (),
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

        // Set up symbol vector.
        self.out_symbols.resize(self.nodes.len(),0);
        // A secondary vector that indicates a symbol is pinned and unable to be changed without violating a constraint.
        let mut pinned_symbols = Vec::new();
        pinned_symbols.resize(self.nodes.len(),false);

        // Panic in the event of un-solvable constraints.
        for cons in &constraints {
            match cons {
                SymbolConstraint::EqualSymbol(index,sym) => {
                    let set = equal_sets.get(*index as usize);
                    if pinned_symbols[set] && self.out_symbols[set] != *sym {
                        panic!("Conflicting equality and type signature constraints.");
                    }
                    pinned_symbols[set] = true;
                    self.out_symbols[set] = *sym;
                },
                _ => ()
            }
        }
        for cons in &constraints {
            match cons {
                SymbolConstraint::NotEqual(a,b) => {
                    let set_a = equal_sets.get(*a as usize);
                    let set_b = equal_sets.get(*b as usize);
                    if set_a == set_b {
                        panic!("Conflicting equality and inequality constraints.");
                    }
                    if pinned_symbols[set_a] && pinned_symbols[set_b] {
                        if self.out_symbols[set_a] != self.out_symbols[set_b] {
                            panic!("Conflicting inequality and type signature constraints.");
                        }
                    }
                },
                _ => ()
            }
        }

        // Fix inequalities
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
                            if !pinned_symbols[set_a] {
                                self.out_symbols[set_a] += 1;
                            } else if !pinned_symbols[set_b] {
                                self.out_symbols[set_b] += 1;
                            } else {
                                panic!("two symbols pinned");
                            }
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
