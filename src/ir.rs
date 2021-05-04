use std::collections::HashMap;
use std::convert::TryInto;

use crate::common::BinOp;
use crate::parser::{Module, Statement, Expr};

#[derive(Debug)]
pub struct IRModule {
    name: String,
    bindings: HashMap<String,IRArg>,
    exprs: Vec<IRExpr>,
    outputs: Vec<IRArg>
}

#[derive(Debug)]
enum IRExpr{
    Input(u32),
    Constant(i32),
    BinOp(IRArg,BinOp,IRArg),
    Multi(Vec<IRExpr>)
}

#[derive(Debug,Clone)]
enum IRArg {
    Link(u32,WireColor),
    Constant(i32)
}

#[derive(Debug,Clone)]
enum WireColor {
    Red,
    Green,
    Unknown
}

impl IRModule {
    fn new(name: String) -> Self {
        IRModule{
            name,
            bindings: HashMap::new(),
            exprs: Vec::new(),
            outputs: Vec::new()
        }
    }

    fn add_args(&mut self, arg_names: &Vec<&str>) {
        for (i,arg_name) in arg_names.iter().enumerate() {
            self.exprs.push(IRExpr::Input(i as u32));
            if self.bindings.insert((*arg_name).to_owned(), IRArg::Link(i as u32,WireColor::Unknown) ).is_some() {
                panic!("Module '{}': Duplicate argument '{}'.",self.name,arg_name);
            }
        }
    }

    fn add_stmt(&mut self, stmt: &Statement) {
        if self.outputs.len() > 0 {
            panic!("Module '{}': No statements may appear after output(...).",self.name);
        }
        match stmt {
            Statement::Output(out_exprs) => {
                for expr in out_exprs {
                    let expr_id = self.add_expr(expr);
                    self.outputs.push(expr_id);
                }
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
                //self.exprs.push(IRExpr::Constant(num_32));
                //self.exprs.len() as u32 - 1
                IRArg::Constant(num_32)
            },
            Expr::BinOp(lhs,op,rhs) => {
                let lex = self.add_expr(lhs);
                let rex = self.add_expr(rhs);
                // TODO constant folding
                self.exprs.push(IRExpr::BinOp(lex,*op,rex));
                IRArg::Link(self.exprs.len() as u32 - 1, WireColor::Unknown)
            },
            _ => panic!("todo handle expr {:?}",expr)
        }
    }
}

// Consumes a list of AST modules and returns the IR for the final module.
// Runs checks on the modules. May panic if an error is encountered.
pub fn build_ir(parse_mods: Vec<Module>) -> Option<IRModule> {
    //let defined: HashMap<String,IRModule> = HashMap::new();

    for p_mod in parse_mods {
        let mut ir = IRModule::new(p_mod.name.to_owned());

        ir.add_args(&p_mod.arg_names);

        for stmt in p_mod.stmts {
            ir.add_stmt(&stmt);
        }

        println!("Make IR for: {} {:#?}",p_mod.name,ir);

        return Some(ir);
    }

    // No modules provided.
    None
}
