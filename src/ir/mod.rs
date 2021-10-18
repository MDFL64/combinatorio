use std::{collections::HashMap, rc::Rc};
use std::convert::TryInto;

use crate::{CompileSettings, common::{BinOp, UnaryOp}};
use crate::parser::{Module, Statement, Expr};

use self::layout::{Grid, WireLink};

mod select_colors;
mod select_symbols;
mod layout;
mod to_blueprint;
mod opt;

#[derive(Debug)]
pub struct IRModule {
    name: String,
    settings: Rc<CompileSettings>,
    port_count: i32,
    bindings: HashMap<String,IRArg>,
    nodes: Vec<IRNode>,
    outputs_set: bool,
    out_symbols: Vec<u32>,
    grid: Grid,
    links: Vec<WireLink>
}

#[derive(Debug,Clone,PartialEq)]
enum IRNode {
    Input(u32),
    Output(u32, IRArg),
    Constant(i32), // represents a constant combinator, rather than just a constant in another combinator
    BinOp(IRArg,BinOp,IRArg), // <- use this node for compares as well, there are just a few cases we need to treat compares differently.
    //BinOpCmp(IRArg,BinOp,IRArg), // <- LHS *MUST* be a signal
    Gate(IRArg,bool,IRArg),
    MultiDriver(Vec<IRArg>),

    // These are generated later in compilation...
    BinOpSame(IRArg,BinOp), // <- special case for when both inputs are the same result value
    BinOpCmpGate(IRArg,BinOp,i32,IRArg), // <- LHS *MUST* be a signal, RHS *MUST* be a constant, GATED *MUST* be a signal

    PlaceHolder,
    Removed
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

    /*fn is_const(&self) -> bool {
        !self.is_link()
    }*/
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
            if self.bindings.insert((*arg_name).to_owned(), IRArg::Link(i as u32,WireColor::None) ).is_some() {
                panic!("Module '{}': Duplicate argument '{}'.",self.name,arg_name);
            }
        }
    }

    /// Run in its own pass before add_stmt
    fn add_stmt_bindings(&mut self, stmt: &Statement) {
        match stmt {
            Statement::VarBinding(idents,_expr) => {
                for var_name in idents {
                    self.nodes.push(IRNode::PlaceHolder);
                    let arg = IRArg::Link(self.nodes.len() as u32 - 1, WireColor::None);

                    if self.bindings.insert((*var_name).to_owned(), arg ).is_some() {
                        panic!("Module '{}': Duplicate variable binding '{}'.",self.name,var_name);
                    }
                }
            },
            _ => ()
        }
    }

    fn add_stmt(&mut self, stmt: &Statement, module_table: &HashMap<String, IRModule>) {
        if self.outputs_set {
            panic!("Module '{}': No statements may appear after output(...).",self.name);
        }
        match stmt {
            Statement::Output(out_exprs) => {
                for (out_i, expr) in out_exprs.iter().enumerate() {
                    let out_arg = self.add_expr(expr, module_table, None);
                    self.nodes.push(IRNode::Output(out_i as u32, out_arg));
                }
                self.port_count += out_exprs.len() as i32;
                self.outputs_set = true;
            },
            Statement::VarBinding(idents,expr) => {
                // TODO check sub-module calls (???)
                assert!(idents.len() == 1);
                let ident = idents[0];

                if let IRArg::Link(out_slot,_) = self.bindings.get(ident).unwrap() {
                    self.add_expr(expr, module_table, Some(*out_slot));
                } else {
                    panic!("var bindings should always be links");
                }
            },
            _ => panic!("todo handle stmt {:?}",stmt)
        }
    }

    fn add_node(&mut self, node: IRNode, slot: Option<u32>) -> IRArg {
        if let Some(slot) = slot {
            assert_eq!(self.nodes[slot as usize],IRNode::PlaceHolder);
            self.nodes[slot as usize] = node;
            IRArg::Link(slot, WireColor::None)
        } else {
            self.nodes.push(node);
            IRArg::Link(self.nodes.len() as u32 - 1, WireColor::None)
        }
    }

    fn add_expr(&mut self, expr: &Expr, module_table: &HashMap<String, IRModule>, desired_slot: Option<u32>) -> IRArg {
        match expr {
            Expr::Ident(name) => {
                if let Some(arg) = self.bindings.get(*name) {
                    if desired_slot.is_some() {
                        // hack to make assignments work properly
                        let arg = arg.clone();
                        return self.add_node(IRNode::MultiDriver(vec!(arg)),desired_slot);
                    }
                    arg.clone()
                } else {
                    panic!("Module '{}': '{}' is not defined.",self.name,name);
                }
            },
            Expr::Constant(num) => {
                let num_32 = narrow_constant(*num);
                self.add_node(IRNode::Constant(num_32), desired_slot)
            },
            Expr::BinOp(lhs,op,rhs) => {
                let lex = self.add_expr(lhs, module_table, None);
                let rex = self.add_expr(rhs, module_table, None);
                
                self.add_node(IRNode::BinOp(lex,*op,rex), desired_slot)
            },
            Expr::UnOp(op,arg) => {
                if *op != UnaryOp::Negate {
                    panic!("unary op nyi");
                }

                // TODO re-evaluate this:
                // SPECIAL CASE: Negate constants immediately to deal with possible i32::MIN
                // Do this REGARDLESS of whether constant folding is enabled.
                /*
                if let Expr::Constant(const_val) = arg.as_ref() {
                    return narrow_constant(*const_val);
                }*/

                // Try normal constant-folding
                let ir_arg = self.add_expr(arg, module_table, None);

                // Convert to a subtraction bin-op
                self.add_node(IRNode::BinOp(IRArg::Constant(0),BinOp::Sub,ir_arg), desired_slot)
            },
            Expr::If(cond,val_true,val_false) => {

                let arg_cond = self.add_expr(cond, module_table, None);
                let arg_true = self.add_expr(val_true, module_table, None);

                if let Some(val_false) = val_false {
                    let arg_false = self.add_expr(val_false, module_table, None);
                    
                    let true_result = self.add_node(IRNode::Gate(arg_cond.clone(),true,arg_true),None);
                    let false_result = self.add_node(IRNode::Gate(arg_cond,false,arg_false),None);
                    
                    self.add_node(IRNode::MultiDriver(vec!(true_result,false_result)),desired_slot)
                } else {
                    self.add_node(IRNode::Gate(arg_cond,true,arg_true),desired_slot)
                }
            },
            Expr::Match(expr_in,match_list) => {
                let arg_in = self.add_expr(expr_in, module_table,None);

                let mut results = Vec::new();
                for (expr_test,expr_res) in match_list {
                    let arg_test = self.add_expr(expr_test, module_table,None);
                    let arg_res = self.add_expr(expr_res, module_table,None);

                    let compare = self.add_node(IRNode::BinOp(arg_in.clone(),BinOp::CmpEq,arg_test),None);
                    results.push(self.add_node(IRNode::Gate(compare,true,arg_res),None));
                }

                self.add_node(IRNode::MultiDriver(results),desired_slot)
            },
            Expr::SubModule(name,args) => {
                let args: Vec<_> = args.iter().map(|arg| self.add_expr(arg, module_table, None)).collect();
                if let Some(submod) = module_table.get(name) {
                    let offset = self.nodes.len() as u32;
                    let mut result: Option<IRArg> = None;
                    for node in &submod.nodes {
                        if let Some((out_i,out_arg)) = self.add_node_from_submodule(node, offset, &args) {
                            assert!(out_i == 0);
                            result = Some(out_arg);
                        }
                    }
                    let result = result.expect("submodule result missing");
                    // HACK! copy result to output
                    if desired_slot.is_some() {
                        self.add_node(IRNode::MultiDriver(vec!(result)), desired_slot)
                    } else {
                        result
                    }
                } else {
                    panic!("Module '{}': Submodule '{}' is not defined.",self.name,name);
                }
            }
        }
    }

    fn add_node_from_submodule(&mut self, node: &IRNode, offset: u32, inputs: &Vec<IRArg>) -> Option<(u32,IRArg)> {
        
        let offset_arg = |arg: &IRArg| {
            if let IRArg::Link(n,c) = arg {
                IRArg::Link(*n + offset, *c)
            } else {
                arg.clone()
            }
        };

        let adjusted = match node {
            // HACK: Input abuses multi-drivers to proxy signals.
            IRNode::Input(n) => {
                let arg = &inputs[*n as usize];
                IRNode::MultiDriver(vec!(arg.clone()))
            },
            IRNode::Output(n,arg) => {
                return Some((*n,offset_arg(arg)));
            },
            IRNode::BinOp(lhs,op,rhs) => {
                IRNode::BinOp(offset_arg(lhs),*op,offset_arg(rhs))
            },
            IRNode::MultiDriver(args) => {
                let fixed_args = args.iter().map(offset_arg).collect();
                IRNode::MultiDriver(fixed_args)
            },
            IRNode::BinOpCmpGate(lhs,op,rhs,gated) => {
                IRNode::BinOpCmpGate(offset_arg(lhs),*op,*rhs,offset_arg(gated))
            },
            IRNode::Removed => IRNode::Removed,
            _ => panic!("submodule node {:?}",node)
        };
        self.nodes.push(adjusted);
        None
    }

    /*fn check_multi_driver(&self) {
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
    }*/
}

/// We narrow from 64 bit constants so we can use both signed and unsigned 32 bit constants.
fn narrow_constant(x: i64) -> i32 {
    if let Ok(n) = x.try_into() {
        n
    } else if let Ok(n) = x.try_into() {
        let n: u32 = n;
        n as i32
    } else {
        panic!("constant too wide: {}",x)
    }
}

// Consumes a list of AST modules and returns the IR for the final module.
// Runs checks on the modules. May panic if an error is encountered.
pub fn build_ir(parse_mods: Vec<Module>, settings: Rc<CompileSettings>) -> HashMap<String,IRModule> {
    let mut defined: HashMap<String,IRModule> = HashMap::new();

    for p_mod in parse_mods {
        let mut ir = IRModule::new(p_mod.name.to_owned(), settings.clone());

        ir.add_args(&p_mod.arg_names);

        for stmt in p_mod.stmts.iter() {
            ir.add_stmt_bindings(&stmt);
        }

        for stmt in p_mod.stmts {
            ir.add_stmt(&stmt, &defined);
        }

        //ir.check_multi_driver();

        if ir.settings.fold_constants {
            ir.fold_constants();
        }

        if ir.settings.prune {
            ir.prune();
        }

        ir.fix_nodes();

        // Prune again, gate expansion can leave behind orphan comparators.
        if ir.settings.prune {
            ir.prune();
        }
        
        defined.insert(ir.name.clone(), ir);
    }

    defined
}
