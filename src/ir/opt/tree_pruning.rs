
use super::super::{IRModule, IRNode, IRArg};

fn add_arg(arg: &IRArg, saved: &mut Vec<bool>, stack: &mut Vec<usize>) {
    if let IRArg::Link(id,_) = arg {
        if !saved[*id as usize] {
            saved[*id as usize] = true;
            stack.push(*id as usize);
        }
    }
}

impl IRModule {
    pub fn prune(&mut self) {
        let mut saved = Vec::new();
        saved.resize(self.nodes.len(), false);

        let mut stack: Vec<usize> = Vec::new();

        // save inputs, add outputs to stack
        for (i,node) in self.nodes.iter().enumerate() {
            match node {
                IRNode::Input(..) => {
                    saved[i] = true;
                },
                IRNode::Output(..) => {
                    saved[i] = true;
                    stack.push(i);
                },
                _ => ()
            }
        }

        while stack.len() > 0 {
            let i = stack.pop().unwrap();
            let node = &self.nodes[i];

            match node {
                IRNode::Output(_,arg) => {
                    add_arg(arg, &mut saved, &mut stack);
                },
                IRNode::BinOp(lhs,_,rhs) => {
                    add_arg(lhs, &mut saved, &mut stack);
                    add_arg(rhs, &mut saved, &mut stack);
                },
                IRNode::MultiDriver(args) => {
                    for arg in args {
                        add_arg(arg, &mut saved, &mut stack);
                    }
                },
                _ => panic!("todo prune {:?}",node)
            }
        }

        let mut remove_count = 0;
        for i in 0..self.nodes.len() {
            if !saved[i] {
                self.nodes[i] = IRNode::Removed;
                remove_count += 1;
            }
        }
        println!("Pruned {} nodes.",remove_count);
    }
}
