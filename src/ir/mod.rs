use std::{collections::HashMap, rc::Rc};
use std::convert::TryInto;

use crate::{CompileSettings, common::{BinOp, UnaryOp}};
use crate::parser::{Module, Statement, Expr};

use self::layout::{Grid, WireLink};

mod select_colors;
mod select_symbols;
mod layout;
mod to_blueprint;
mod prune;
mod opt;

pub struct IRModule {
    name: String,
    settings: Rc<CompileSettings>,
    port_count: i32,
    bindings: HashMap<String,(IRArg,bool)>,
    nodes: Vec<IRNode>,
    outputs_set: bool,
    out_symbols: Vec<u32>,
    grid: Grid,
    links: Vec<WireLink>
}

#[derive(Debug,Clone)]
enum IRNode {
    Input(u32),
    Output(u32, IRArg),
    Constant(i32), // <- totally redundant??? there may be some niche situations it's needed
    BinOp(IRArg,BinOp,IRArg), // <- use this node for compares as well, there are just a few cases we need to treat compares differently.
    //BinOpCmp(IRArg,BinOp,IRArg), // <- LHS *MUST* be a signal
    Gate(IRArg,bool,IRArg),
    MultiDriver(Vec<IRArg>),

    // These are generated later in compilation...
    BinOpSame(IRArg,BinOp), // <- special case for when both inputs are the same result value
    BinOpCmpGate(IRArg,BinOp,i32,IRArg), // <- LHS *MUST* be a signal, RHS *MUST* be a constant, GATED *MUST* be a signal
}

#[derive(Debug,Clone,PartialEq)]
enum IRArg {
    Link(u32,WireColor),
    Constant(i32)
}

impl IRArg {
    fn is_link(&self) -> bool {
        return if let IRArg::Link(..) = self { true } else { false };
    }

    fn is_const(&self) -> bool {
        !self.is_link()
    }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum WireColor {
    Red,
    Green,
    None
}

impl IRModule {
    fn new(name: String, settings: Rc<CompileSettings>) -> Self {
        IRModule{
            name,
            settings,
            port_count: 0,
            bindings: HashMap::new(),
            nodes: Vec::new(),
            outputs_set: false,
            out_symbols: Vec::new(),
            grid: Default::default(),
            links: Vec::new()
        }
    }

    #[allow(unused)]
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
        let base_y = pos.1 as f32 * 2.0;
        let node = &self.nodes[id as usize];
        let offset_y = match node {
            IRNode::BinOp(..) |
            IRNode::BinOpCmpGate(..) |
            IRNode::BinOpSame(..) => 0.5,
            IRNode::Input(..) |
            IRNode::Output(..) |
            IRNode::Constant(..) => 0.0,
            _ => panic!("todo offset {:?}",node)
        };
        (x, base_y + offset_y)
    }

    fn add_args(&mut self, arg_names: &Vec<&str>) {
        self.port_count += arg_names.len() as i32;
        for (i,arg_name) in arg_names.iter().enumerate() {
            self.nodes.push(IRNode::Input(i as u32));
            if self.bindings.insert((*arg_name).to_owned(), (IRArg::Link(i as u32,WireColor::None),false) ).is_some() {
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
            Statement::VarBinding(idents,expr) => {
                // TODO check sub-module calls
                assert!(idents.len() == 1);
                let (ident,is_multi_driver) = idents[0];
                let result_arg = self.add_expr(expr);

                if is_multi_driver {
                    if !self.bindings.contains_key(&ident.to_owned()) {
                        self.nodes.push(IRNode::MultiDriver(Vec::new()));
                        self.bindings.insert(ident.to_owned(), (IRArg::Link(self.nodes.len() as u32 - 1, WireColor::None), true) );
                    }

                    let (md_arg, is_arg_md) = self.bindings.get(&ident.to_owned()).unwrap().clone();
                    if !is_arg_md {
                        panic!("Module '{}': Attempt to multi-drive existing simple variable '{}'.",self.name,ident);
                    }
                    if let IRArg::Link(md_id,_) = md_arg {
                        let md_node = &mut self.nodes[md_id as usize];
                        if let IRNode::MultiDriver(md_list) = md_node {
                            md_list.push(result_arg);
                        } else {
                            panic!("MD node was not MD?");
                        }
                    } else {
                        panic!("MD arg was not a link?");
                    }
                } else {
                    if self.bindings.insert(ident.to_owned(), (result_arg,false)).is_some() {
                        panic!("Module '{}': Duplicate variable binding '{}'.",self.name,ident);
                    }
                }
            },
            _ => panic!("todo handle stmt {:?}",stmt)
        }
    }

    fn add_expr(&mut self, expr: &Expr) -> IRArg {
        match expr {
            Expr::Ident(name) => {
                if let Some((arg,_is_md)) = self.bindings.get(*name) {
                    arg.clone()
                } else {
                    panic!("Module '{}': '{}' is not defined.",self.name,name);
                }
            },
            Expr::Constant(num) => {
                let num_32: i32 = (*num).try_into().expect("bad constant");
                IRArg::Constant(num_32)
            },
            Expr::BinOp(lhs,op,rhs) => {
                let lex = self.add_expr(lhs);
                let rex = self.add_expr(rhs);
                
                self.nodes.push(IRNode::BinOp(lex,*op,rex));

                IRArg::Link(self.nodes.len() as u32 - 1, WireColor::None)
            },
            Expr::UnOp(op,arg) => {
                if *op != UnaryOp::Negate {
                    panic!("unary op nyi");
                }

                // SPECIAL CASE: Negate constants immediately to deal with possible i32::MIN
                // Do this REGARDLESS of whether constant folding is enabled.
                if let Expr::Constant(const_val) = arg.as_ref() {
                    let negated = -const_val;
                    let num_32: i32 = negated.try_into().expect("bad constant");
                    return IRArg::Constant(num_32);
                }

                // Try normal constant-folding
                let ir_arg = self.add_expr(arg);

                // Convert to a subtraction bin-op
                self.nodes.push(IRNode::BinOp(IRArg::Constant(0),BinOp::Sub,ir_arg));
                IRArg::Link(self.nodes.len() as u32 - 1, WireColor::None)
            },
            Expr::If(cond,val_true,val_false) => {

                let arg_cond = self.add_expr(cond);
                let arg_true = self.add_expr(val_true);

                let true_result = self.add_node(IRNode::Gate(arg_cond.clone(),true,arg_true));

                if let Some(val_false) = val_false {
                    let arg_false = self.add_expr(val_false);

                    let false_result = self.add_node(IRNode::Gate(arg_cond,false,arg_false));

                    self.add_node(IRNode::MultiDriver(vec!(true_result,false_result)))
                } else {
                    true_result
                }
            },
            Expr::Match(expr_in,match_list) => {
                panic!("match currently unsupported");
                /*let arg_in = self.add_expr(expr_in);

                let mut results = Vec::new();
                for (expr_test,expr_res) in match_list {
                    let arg_test = self.add_expr(expr_test);
                    let arg_res = self.add_expr(expr_res);
                    if arg_test.is_const() {
                        results.push(self.add_compare_gate(arg_in.clone(), BinOp::CmpEq, arg_test, arg_res));
                    } else {
                        // must add an extra compare
                        self.nodes.push(IRNode::BinOpCmp(arg_in.clone(),BinOp::CmpEq,arg_test));
                        let cmp_res = IRArg::Link(self.nodes.len() as u32 - 1, WireColor::None);
                        results.push(self.add_compare_gate(cmp_res, BinOp::CmpNeq, IRArg::Constant(0), arg_res));
                    }
                }

                self.add_multi_driver(results)*/
            }
        }
    }

    fn add_node(&mut self, node: IRNode) -> IRArg {
        self.nodes.push(node);
        IRArg::Link(self.nodes.len() as u32 - 1, WireColor::None)
    }

    /*fn add_compare_gate(&mut self, mut lhs: IRArg, op: BinOp, rhs: IRArg, mut gated: IRArg) -> IRArg {
        if self.settings.fold_constants {
            // If the gated value is zero, this gate is a no-op.
            if gated == IRArg::Constant(0) {
                return IRArg::Constant(0);
            }

            if let IRArg::Constant(lhs_n) = lhs {
                if let IRArg::Constant(rhs_n) = rhs {
                    if op.fold(lhs_n, rhs_n) != 0 {
                        return gated;
                    } else {
                        return IRArg::Constant(0);
                    }
                }
            }
        }

        if let IRArg::Constant(n) = lhs {
            lhs = self.add_const_node(n);
        }

        if let IRArg::Constant(rhs_n) = rhs {
            // Gated value *MUST* be a link.
            if let IRArg::Constant(n) = gated {
                gated = self.add_const_node(n);
            }
    
            self.nodes.push(IRNode::BinOpCmpGate(lhs,op,rhs_n,gated));
            IRArg::Link(self.nodes.len() as u32 - 1, WireColor::None)
        } else {
            panic!("Bad compare gate, rhs must be a constant.");
        }
    }*/

    /*fn add_const_node(&mut self, n: i32) -> IRArg {
        // TODO merge same constants into same nodes (save constants in a LUT)
        //      WARNING: This is a bad idea for multi-drivers? (feedback)
        // TODO make two constants fill a single cell somehow
        self.nodes.push(IRNode::Constant(n));
        IRArg::Link(self.nodes.len() as u32 - 1, WireColor::None)
    }*/

    /*fn add_multi_driver(&mut self, args: Vec<IRArg>) -> IRArg {
        self.nodes.push(IRNode::MultiDriver(args));
        IRArg::Link(self.nodes.len() as u32 - 1, WireColor::None)
    }*/

    fn check_multi_driver(&self) {
        for (name,(arg,is_md)) in &self.bindings {
            if *is_md {
                if let IRArg::Link(node_id,_) = arg {
                    if let IRNode::MultiDriver(arg_list) = &self.nodes[*node_id as usize] {
                        if arg_list.len() < 2 {
                            panic!("Module '{}': Multi-driver binding '{}' has less than two drivers.",self.name,name);
                        }
                    } else {
                        panic!("multi-driver binding must be multi-driver!");
                    }
                } else {
                    panic!("multi-driver binding cannot be constant!");
                }
            }
        }
    }
}

// Consumes a list of AST modules and returns the IR for the final module.
// Runs checks on the modules. May panic if an error is encountered.
pub fn build_ir(parse_mods: Vec<Module>, settings: Rc<CompileSettings>) -> IRModule {
    //let defined: HashMap<String,IRModule> = HashMap::new();

    for p_mod in parse_mods {
        let mut ir = IRModule::new(p_mod.name.to_owned(), settings);

        ir.add_args(&p_mod.arg_names);

        for stmt in p_mod.stmts {
            ir.add_stmt(&stmt);
        }

        ir.check_multi_driver();
        ir.fold_constants();

        if ir.settings.prune {
            ir.prune();
        }
        
        return ir;
    }

    // No modules provided.
    panic!("No modules to compile.");
}
