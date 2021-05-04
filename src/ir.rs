use std::collections::HashMap;

use crate::common::BinOp;
use crate::parser::{Module, Statement, Expr};

#[derive(Debug)]
pub struct IRModule {
    name: String,
    bindings: HashMap<String,u32>,
    exprs: Vec<IRExpr>,
    outputs: Vec<u32>
}
#[derive(Debug)]
enum IRExpr{
    Input(u32),
    Constant(i64),
    BinOp(u32,BinOp,u32),
    Multi(Vec<IRExpr>)
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
            if self.bindings.insert((*arg_name).to_owned(), i as u32).is_some() {
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

    fn add_expr(&mut self, expr: &Expr) -> u32 {
        match expr {
            Expr::Ident(name) => {
                if let Some(id) = self.bindings.get(*name) {
                    *id
                } else {
                    panic!("Module '{}': '{}' is not defined.",self.name,name);
                }
            },
            Expr::Constant(num) => {
                self.exprs.push(IRExpr::Constant(*num));
                self.exprs.len() as u32 - 1
            },
            Expr::BinOp(lhs,op,rhs) => {
                let lex = self.add_expr(lhs);
                let rex = self.add_expr(rhs);
                self.exprs.push(IRExpr::BinOp(lex,*op,rex));
                self.exprs.len() as u32 - 1
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
