use std::collections::HashMap;
use std::convert::TryInto;

use crate::common::BinOp;
use crate::parser::{Module, Statement, Expr};

mod select_colors;

#[derive(Debug)]
pub struct IRModule {
    name: String,
    bindings: HashMap<String,IRArg>,
    nodes: Vec<IRNode>,
    outputs_set: bool
}

#[derive(Debug)]
enum IRNode {
    Input(u32),
    Output(u32, IRArg),
    Constant(i32), // <- totally redundant??? there may be some niche situations it's needed
    BinOp(IRArg,BinOp,IRArg),
    Multi(Vec<IRNode>)
}

#[derive(Debug,Clone)]
enum IRArg {
    Link(u32,WireColor),
    Constant(i32)
}

#[derive(Debug,Clone,Copy,PartialEq)]
enum WireColor {
    Red,
    Green,
    None
}

impl IRModule {
    fn new(name: String) -> Self {
        IRModule{
            name,
            bindings: HashMap::new(),
            nodes: Vec::new(),
            outputs_set: false
        }
    }

    pub fn print(&self) {
        println!("IR MODULE: {}",self.name);
        for (i,node) in self.nodes.iter().enumerate() {
            println!("    {}: {:?}",i,node);
        }
    }

    fn add_args(&mut self, arg_names: &Vec<&str>) {
        for (i,arg_name) in arg_names.iter().enumerate() {
            self.nodes.push(IRNode::Input(i as u32));
            if self.bindings.insert((*arg_name).to_owned(), IRArg::Link(i as u32,WireColor::None) ).is_some() {
                panic!("Module '{}': Duplicate argument '{}'.",self.name,arg_name);
            }
        }
    }

    fn add_stmt(&mut self, stmt: &Statement) {
        if self.outputs_set {
            panic!("Module '{}': No statements may appear after output(...).",self.name);
        }
        match stmt {
            Statement::Output(out_exprs) => {
                for (out_i, expr) in out_exprs.iter().enumerate() {
                    let out_arg = self.add_expr(expr);
                    self.nodes.push(IRNode::Output(out_i as u32, out_arg));
                }
                self.outputs_set = true;
            },
            _ => panic!("todo handle stmt {:?}",stmt)
        }
    }

    fn add_expr(&mut self, expr: &Expr) -> IRArg {
        match expr {
            Expr::Ident(name) => {
                if let Some(arg) = self.bindings.get(*name) {
                    arg.clone()
                } else {
                    panic!("Module '{}': '{}' is not defined.",self.name,name);
                }
            },
            Expr::Constant(num) => {
                let num_32: i32 = (*num).try_into().expect("bad constant todo msg");
                //self.exprs.push(IRNode::Constant(num_32));
                //self.exprs.len() as u32 - 1
                IRArg::Constant(num_32)
            },
            Expr::BinOp(lhs,op,rhs) => {
                let lex = self.add_expr(lhs);
                let rex = self.add_expr(rhs);
                // TODO constant folding
                self.nodes.push(IRNode::BinOp(lex,*op,rex));
                IRArg::Link(self.nodes.len() as u32 - 1, WireColor::None)
            },
            _ => panic!("todo handle expr {:?}",expr)
        }
    }
}

// Consumes a list of AST modules and returns the IR for the final module.
// Runs checks on the modules. May panic if an error is encountered.
pub fn build_ir(parse_mods: Vec<Module>) -> IRModule {
    //let defined: HashMap<String,IRModule> = HashMap::new();

    for p_mod in parse_mods {
        let mut ir = IRModule::new(p_mod.name.to_owned());

        ir.add_args(&p_mod.arg_names);

        for stmt in p_mod.stmts {
            ir.add_stmt(&stmt);
        }

        //println!("Make IR for: {} {:#?}",p_mod.name,ir);

        return ir;
    }

    // No modules provided.
    panic!("No modules to compile.");
}
