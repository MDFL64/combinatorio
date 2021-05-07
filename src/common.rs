#[derive(Debug,Clone,Copy)]
pub enum BinOp {
    Add,
}

impl BinOp {
    pub fn prec(&self) -> u32 {
        return 1;
    }

    pub fn to_str(&self) -> &'static str {
        return "+";
    }
}

#[derive(Debug,Hash,PartialEq,Eq,Clone)]
pub enum ConnectType {
    In,
    Out
}
