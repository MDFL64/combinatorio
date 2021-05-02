use crate::lexer::{Lexer, LexToken};

use std::iter::Peekable;

#[derive(Debug)]
pub struct Module<'a> {
    name: &'a str,
    arg_names: Vec<&'a str>,
    stmts: Vec<Statement<'a>>
}

#[derive(Debug)]
enum Statement<'a> {
    Terminator,
    Empty,
    Output(Vec<Expr<'a>>)
}

#[derive(Debug)]
enum Expr<'a> {
    Ident(&'a str),
    Number(u32),
    BinOp(Box<Expr<'a>>,BinOp,Box<Expr<'a>>)
}

#[derive(Debug)]
enum BinOp {
    Add,
}

impl BinOp {
    fn prec(&self) -> u32 {
        return 1;
    }
}

struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>
}

impl<'a> Parser<'a> {
    fn new(lexer: Lexer<'a>) -> Self {
        Self{lexer: lexer.peekable()}
    }

    fn take(&mut self, tok: LexToken) {
        let present = self.next();
        if present != tok {
            panic!("Expected {:?}, found {:?}.",tok,present);
        }
    }

    fn take_ident(&mut self) -> &'a str {
        let present = self.next();
        if let LexToken::Ident(ident_str) = present {
            ident_str
        } else {
            panic!("Expected ident, found {:?}.",present);
        }
    }

    fn take_comma_or_close_paren(&mut self) -> bool {
        let present = self.next();
        match present {
            LexToken::OpComma => false,
            LexToken::OpParenClose => true,
            _ => panic!("Expected comma or close paren, found {:?}.",present)
        }
    }

    fn next(&mut self) -> LexToken<'a> {
        self.lexer.next().expect("Expected token, found EOF.")
    }

    fn peek(&mut self) -> LexToken<'a> {
        *self.lexer.peek().expect("Expected token, found EOF.")
    }

    fn is_eof(&mut self) -> bool {
        self.lexer.peek().is_none()   
    }
}

pub fn parse<'a>(source: &'a str) -> Vec<Module<'a>> {

    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);

    // Module declaration
    let mut modules = Vec::new();
    while !parser.is_eof() {
        parser.take(LexToken::KeyMod);
        let mod_name = parser.take_ident();
        let mut mod_args = Vec::new();
        let mut mod_stmts = Vec::new();
        
        // Arguments
        parser.take(LexToken::OpParenOpen);
        if parser.peek() != LexToken::OpParenClose {
            loop {
                mod_args.push(parser.take_ident());
                if parser.take_comma_or_close_paren() {
                    break;
                }
            }
        } else {
            parser.take(LexToken::OpParenClose);
        }
        
        parser.take(LexToken::OpBraceOpen);
        loop {
            let stmt = parse_stmt(&mut parser);
            match stmt {
                Statement::Empty => (),
                Statement::Terminator => break,
                _ => mod_stmts.push(stmt)
            }
        }
        modules.push(Module{
            name: mod_name,
            arg_names: mod_args,
            stmts: mod_stmts
        });
    }

    modules
}

fn parse_stmt<'a>(parser: &mut Parser<'a>) -> Statement<'a> {
    let tok = parser.next();
    match tok {
        LexToken::KeyOutput => {
            let mut out_args = Vec::new();
            parser.take(LexToken::OpParenOpen);
            // Don't worry about the empty case, why output nothing?
            loop {
                out_args.push(parse_expr(parser));
                if parser.take_comma_or_close_paren() {
                    break;
                }
            }
            Statement::Output(out_args)
        },
        LexToken::OpSemicolon => Statement::Empty,
        LexToken::OpBraceClose => Statement::Terminator,
        _ => panic!("Expected statment, found {:?}.",tok)
    }
}

fn parse_expr<'a>(parser: &mut Parser<'a>) -> Expr<'a> {

    let mut expr_stack: Vec<Expr> = Vec::new();
    let mut op_stack: Vec<BinOp> = Vec::new();

    expr_stack.push(parse_leaf(parser));

    loop {
        // try parsing an operator, or end the expression
        let next_tok = parser.peek();

        let new_op = match next_tok {
            LexToken::OpAdd => BinOp::Add,

            // sane expression terminators
            LexToken::OpParenClose => break,
            _ => panic!("Expected operator, found {:?}",next_tok)
        };

        while let Some(top_op) = op_stack.last() {
            if top_op.prec() <= new_op.prec() {
                let op = op_stack.pop().unwrap();
                let rhs = expr_stack.pop().unwrap();
                let lhs = expr_stack.pop().unwrap();

                let bin_expr = Expr::BinOp(Box::new(lhs),op,Box::new(rhs));
                expr_stack.push(bin_expr);
            }
        }
        op_stack.push(new_op);

        // advance
        parser.next();

        // rhs of the parsed operator
        expr_stack.push(parse_leaf(parser));
    }

    while let Some(op) = op_stack.pop() {
        let rhs = expr_stack.pop().unwrap();
        let lhs = expr_stack.pop().unwrap();
        let bin_expr = Expr::BinOp(Box::new(lhs),op,Box::new(rhs));
        expr_stack.push(bin_expr);
    }

    assert_eq!(expr_stack.len(),1);
    expr_stack.pop().unwrap()
}

fn parse_leaf<'a>(parser: &mut Parser<'a>) -> Expr<'a> {
    let tok = parser.next();

    match tok {
        // TODO could be a module call!
        LexToken::Ident(id) => Expr::Ident(id),
        LexToken::Number(num) => Expr::Number(num),
        _ => panic!("Expected expression, found {:?}.",tok)
    }
}
