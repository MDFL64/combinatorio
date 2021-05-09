#[derive(Debug,Clone,Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Power,

    BitAnd,
    BitOr,
    BitXor,

    ShiftLeft,
    ShiftRight,

    CmpEq,
    CmpNeq,
    CmpLt,
    CmpGt,
    CmpLeq,
    CmpGeq
}

impl BinOp {
    pub fn prec(&self) -> u32 {
        // For better or worse, based on c operator precedence.
        // https://en.cppreference.com/w/c/language/operator_precedence
        match self {
            BinOp::Power => 1,
            BinOp::Mul | BinOp::Div | BinOp::Mod => 2,
            BinOp::Add | BinOp::Sub => 3,
            BinOp::ShiftLeft | BinOp::ShiftRight => 4,
            BinOp::CmpLt | BinOp::CmpGt | BinOp::CmpLeq | BinOp::CmpGeq => 5,
            BinOp::CmpEq | BinOp::CmpNeq => 6,
            BinOp::BitAnd => 7,
            BinOp::BitXor => 8,
            BinOp::BitOr => 9,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Power => "^",

            BinOp::BitAnd => "AND",
            BinOp::BitOr => "OR",
            BinOp::BitXor => "XOR",

            BinOp::ShiftLeft => "<<",
            BinOp::ShiftRight => ">>",

            BinOp::CmpEq => "=",
            BinOp::CmpNeq => "!=",

            BinOp::CmpGt => ">",
            BinOp::CmpLt => "<",
            BinOp::CmpGeq => ">=",
            BinOp::CmpLeq => "<=",
        }
    }

    pub fn fold(&self, lhs: i32, rhs: i32) -> i32 {
        match self {
            BinOp::Add => lhs.wrapping_add(rhs),
            BinOp::Sub => lhs.wrapping_sub(rhs),
            BinOp::Mul => lhs.wrapping_mul(rhs),
            BinOp::Div => lhs.wrapping_div(rhs),
            BinOp::Mod => lhs % rhs,

            BinOp::CmpEq => if lhs == rhs { 1 } else { 0 },
            BinOp::CmpNeq => if lhs != rhs { 1 } else { 0 },
            BinOp::CmpLt => if lhs < rhs { 1 } else { 0 },
            BinOp::CmpGt => if lhs > rhs { 1 } else { 0 },
            BinOp::CmpLeq => if lhs <= rhs { 1 } else { 0 },
            BinOp::CmpGeq => if lhs >= rhs { 1 } else { 0 },

            BinOp::Power => if rhs < 0 {
                panic!("negative pow")
            } else {
                lhs.wrapping_pow(rhs as u32)
            },
            _ => panic!("todo fold {:?}",self)
        }
    }

    pub fn is_compare(&self) -> bool {
        match self {
            BinOp::CmpLt | BinOp::CmpGt | BinOp::CmpLeq | BinOp::CmpGeq |
            BinOp::CmpEq | BinOp::CmpNeq => true,
            _ => false
        }
    }

    pub fn flip(&self) -> BinOp {
        match self {
            BinOp::CmpEq => BinOp::CmpEq,
            BinOp::CmpNeq => BinOp::CmpNeq,
            BinOp::CmpLt => BinOp::CmpGt,
            BinOp::CmpGt => BinOp::CmpLt,
            BinOp::CmpLeq => BinOp::CmpGeq,
            BinOp::CmpGeq => BinOp::CmpLeq,
            _ => panic!("attempt to flip unflippable operator {:?}",self)
        }
    }

    pub fn invert(&self) -> BinOp {
        match self {
            BinOp::CmpEq => BinOp::CmpNeq,
            BinOp::CmpNeq => BinOp::CmpEq,
            BinOp::CmpLt => BinOp::CmpGeq,
            BinOp::CmpGt => BinOp::CmpLeq,
            BinOp::CmpLeq => BinOp::CmpGt,
            BinOp::CmpGeq => BinOp::CmpLt,
            _ => panic!("attempt to invert uninvertable operator {:?}",self)
        }
    }

    pub fn fold_same(&self) -> i32 {
        match self {
            BinOp::CmpEq => 1,
            BinOp::CmpNeq => 0,
            BinOp::CmpLt => 0,
            BinOp::CmpGt => 0,
            BinOp::CmpLeq => 1,
            BinOp::CmpGeq => 1,
            _ => panic!("attempt to fold-same bad operator {:?}",self)
        }
    }
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum UnaryOp {
    Negate,
    // Do we even want other unary ops? logical not would probably be the most useful, and it could be optimized
}

#[derive(Debug,Hash,PartialEq,Eq,Clone)]
pub enum ConnectType {
    In,
    Out
}
