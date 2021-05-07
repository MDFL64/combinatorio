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
            BinOp::Mod => "%",
            BinOp::Power => "^"
        }
    }

    pub fn fold(&self, lhs: i32, rhs: i32) -> i32 {
        match self {
            BinOp::Add => lhs.wrapping_add(rhs),
            BinOp::Sub => lhs.wrapping_sub(rhs),
            BinOp::Mul => lhs.wrapping_mul(rhs),
            BinOp::Div => lhs.wrapping_div(rhs),
            BinOp::Mod => lhs % rhs,
            BinOp::Power => if rhs < 0 {
                panic!("negative pow")
            } else {
                lhs.wrapping_pow(rhs as u32)
            }
        }
    }
}

#[derive(Debug,Hash,PartialEq,Eq,Clone)]
pub enum ConnectType {
    In,
    Out
}
