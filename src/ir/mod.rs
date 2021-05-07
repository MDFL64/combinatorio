use std::collections::HashMap;
use std::convert::TryInto;

use crate::common::BinOp;
use crate::parser::{Module, Statement, Expr};

use self::placement::{Grid, WireLink};

mod select_colors;
mod select_symbols;
mod placement;
mod to_blueprint;

#[derive(Debug)]
pub struct IRModule {
    name: String,
    port_count: i32,
    bindings: HashMap<String,IRArg>,
    nodes: Vec<IRNode>,
    outputs_set: bool,
    out_symbols: Vec<u32>,
    grid: Grid,
    links: Vec<WireLink>
}

#[derive(Debug)]
enum IRNode {
    Input(u32),
    Output(u32, IRArg),
    Constant(i32), // <- totally redundant??? there may be some niche situations it's needed
    BinOp(IRArg,BinOp,IRArg),
    Multi(Vec<IRArg>) // <- still not sure how to actually handle these
}

#[derive(Debug,Clone)]
enum IRArg {
    Link(u32,WireColor),
    Constant(i32)
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum WireColor {
    Red,
    Green,
    None
}

impl IRModule {
    fn new(name: String) -> Self {
        IRModule{
            name,
            port_count: 0,
            bindings: HashMap::new(),
            nodes: Vec::new(),
            outputs_set: false,
            out_symbols: Vec::new(),
            grid: Default::default(),
            links: Vec::new()
        }
    }

    pub fn print(&self) {
        println!("IR MODULE: {}",self.name);
        println!("NODES:");
        for (i,node) in self.nodes.iter().enumerate() {
            let pos = self.get_true_pos(i as u32 );
            let symbol = self.out_symbols.get(i).unwrap();
            println!("    {}: {:?}, pos = {:?}, symbol = {}",i,node,pos,symbol);
        }
        println!("LINKS:");
        for link in &self.links {
            println!("    {:?}",link);
        }
    }

    fn get_true_pos(&self, id: u32) -> (f32,f32) {
        let pos = self.grid.get_pos_for(id);
        let x = pos.0 as f32;
        let base_y = pos.1 as f32;
        let offset_y = match self.nodes[id as usize] {
            IRNode::BinOp(..) => 0.5,
            _ => 0.0
        };
        (x, base_y + offset_y)
    }

    fn add_args(&mut self, arg_names: &Vec<&str>) {
        self.port_count += arg_names.len() as i32;
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
                self.port_count += out_exprs.len() as i32;
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
            //_ => panic!("todo handle expr {:?}",expr)
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
        
        return ir;
    }

    // No modules provided.
    panic!("No modules to compile.");
}
