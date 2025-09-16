use crate::parser::{AstNode, Expr, Stmts, Type};
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
pub enum Cir {
    Write(CType, CValue),
    Return(CValue),
    SubProgDef {
        name: String,
        return_type: CType,
        stmts_cir: Vec<Cir>,
    },
}

pub struct CirGenerator {}

impl CirGenerator {
    pub fn new() -> CirGenerator {
        CirGenerator {}
    }
    pub fn generate_cir(self: &Self, ast: Vec<AstNode<Stmts>>) -> Vec<Cir> {
        ast.into_iter()
            .map(|node| self.generate_stmt_cir(node))
            .collect()
    }

    fn to_c_type(self: &Self, type_: Type) -> CType {
        match type_ {
            Type::Nat => CType::Uint,
            Type::String => CType::String,
            Type::Int => CType::Int,
            Type::Unknown => unreachable!(),
            _ => todo!(),
        }
    }

    fn to_c_value(self: &Self, expr: Expr, _type_: Type) -> CValue {
        match expr {
            Expr::String(str) => CValue::StringLiteral(str),
            //TODO: Actually implement this
            Expr::Number(num) => CValue::IntLiteral(num as i32),
            _ => todo!(),
        }
    }
    fn generate_stmt_cir(self: &Self, node: AstNode<Stmts>) -> Cir {
        match node.value {
            Stmts::Write { type_, expr } => {
                let cvalue = self.to_c_value(expr.value, type_);
                let ctype = self.to_c_type(type_);
                Cir::Write(ctype, cvalue)
            }
            Stmts::Return { return_type, expr } => {
                let cvalue = self.to_c_value(expr.value, return_type);
                Cir::Return(cvalue)
            }
            Stmts::SubProgramDef {
                name,
                return_type,
                stmts,
                ..
            } => {
                let return_type = self.to_c_type(return_type);
                let mut stmts_cir = Vec::new();
                for stmt in stmts {
                    stmts_cir.push(self.generate_stmt_cir(stmt));
                }
                Cir::SubProgDef {
                    name,
                    return_type,
                    stmts_cir,
                }
            }
            _ => todo!(),
        }
    }
}
