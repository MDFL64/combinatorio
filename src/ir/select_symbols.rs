
use super::{IRModule, IRNode, IRArg};


#[derive(Debug)]
enum SymbolConstraint {
    Equal(u32,u32),
    NotEqual(u32,u32)
}

impl IRModule {
    pub fn select_symbols(&mut self) {
        let mut constraints: Vec<SymbolConstraint> = Vec::new();

        for (out_i,node) in self.nodes.iter().enumerate() {
            match node {
                IRNode::Input(_) => (),
                IRNode::Constant(_) => (),
                IRNode::Output(..) => (),
                IRNode::BinOpSame(..) => (),
                IRNode::BinOp(lhs,_,rhs) |
                IRNode::BinOpCmp(lhs,_,rhs) => {
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
                //_ => panic!("todo symbol constraint {:?}",node)
            }
        }

        self.out_symbols.resize(self.nodes.len(),0);
        let mut pass_num = 1;
        let mut errors = 0;
        print!("Symbol selection... ");
        loop {
            for cons in &constraints {
                match cons {
                    SymbolConstraint::NotEqual(a,b) => {
                        if self.out_symbols[*a as usize] == self.out_symbols[*b as usize] {
                            errors += 1;
                            // TODO should we pick a symbol to increment at random?
                            self.out_symbols[*b as usize] += 1;
                        }
                    },
                    SymbolConstraint::Equal(a,b) => {
                        if self.out_symbols[*a as usize] != self.out_symbols[*b as usize] {
                            errors += 1;
                            // TODO should we pick a symbol to copy at random?
                            self.out_symbols[*b as usize] += self.out_symbols[*a as usize];
                        }
                    },
                    //_ => panic!("todo handle constraint {:?}",cons)
                }
            }
            if errors == 0 {
                break;
            }
            pass_num += 1;
            errors = 0;
            if pass_num > 1000 {
                panic!("too many passes");
            }
        }
        println!("Done in {} passes.",pass_num);
    }
}
