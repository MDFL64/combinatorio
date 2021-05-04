#[derive(Debug,Clone,Copy)]
pub enum BinOp {
    Add,
}

impl BinOp {
    pub fn prec(&self) -> u32 {
        return 1;
    }
}
