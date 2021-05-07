#[derive(Debug,Clone,Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Power
}

impl BinOp {
    pub fn prec(&self) -> u32 {
        match self {
            BinOp::Add | BinOp::Sub => 3,
            BinOp::Mul | BinOp::Div | BinOp::Mod => 2,
            BinOp::Power => 1
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "/",
            BinOp::Power => "^"
        }
    }
}

#[derive(Debug,Hash,PartialEq,Eq,Clone)]
pub enum ConnectType {
    In,
    Out
}
