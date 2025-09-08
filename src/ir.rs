use std::fmt;
#[derive(Debug)]
pub enum CType {
    Int,
    Uint,
    String,
}

impl fmt::Display for CType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CType::Int => write!(f, "int32_t"),
            CType::Uint => write!(f, "unt32_t"),
            _ => todo!(),
        }
    }
}
#[derive(Debug)]
pub enum CValue {
    IntLiteral(i32),
    UintLiteral(u32),
    StringLiteral(String),

    Variable(String),
    Temporary(usize),
    BinaryOp(Box<CValue>, CBinaryOp, Box<CValue>),
}
impl fmt::Display for CValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CValue::IntLiteral(n) => write!(f, "{}", n),
            CValue::UintLiteral(n) => write!(f, "{}", n),
            CValue::StringLiteral(s) => write!(f, "\"{}\"", s),
            CValue::Variable(name) => write!(f, "{}", name),
            CValue::Temporary(id) => write!(f, "__tmp_{}", id),
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}
#[derive(Debug)]
pub enum Ir {
    Write(CType, CValue),
    Return(CValue),
    SubProgDef(String, CType),
}
