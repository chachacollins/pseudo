use crate::parser::{AstNode, Expr, Op, Stmts, Type};
use std::fmt;

#[derive(Debug)]
pub enum CType {
    Int,
    Uint,
    String,
}

#[derive(Debug)]
pub struct CParam {
    pub name: String,
    pub param_type: CType,
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
    BinaryOp(Box<CValue>, Op, Box<CValue>),
    SubProgCall(String, Vec<CValue>),
}
impl fmt::Display for CValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CValue::IntLiteral(n) => write!(f, "{}", n),
            CValue::UintLiteral(n) => write!(f, "{}", n),
            CValue::StringLiteral(s) => write!(f, "\"{}\"", s),
            CValue::Variable(name) => write!(f, "{}", name),
            CValue::Temporary(id) => write!(f, "__tmp_{}", id),
            CValue::BinaryOp(lhs, op, rhs) => {
                write!(f, "{lhs} ")?;
                match op {
                    Op::Add => write!(f, "+")?,
                    Op::Minus => write!(f, "-")?,
                    Op::Mult => write!(f, "*")?,
                    Op::Div => write!(f, "/")?,
                    Op::Mod => write!(f, "%")?,
                    Op::Equal => write!(f, "==")?,
                    Op::NotEqual => write!(f, "!=")?,
                    Op::And => write!(f, "&&")?,
                    Op::Or => write!(f, "||")?,
                }
                write!(f, " {rhs}")
            }
            CValue::SubProgCall(name, args) => {
                write!(
                    f,
                    "{name}({})",
                    args.iter()
                        .map(|arg| arg.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
        }
    }
}

#[derive(Debug)]
pub enum Cir {
    Write(CType, CValue),
    Return(CValue),
    SubProgDef {
        name: String,
        cparams: Vec<CParam>,
        return_type: CType,
        stmts_cir: Vec<Cir>,
    },
    If(CValue, Vec<Cir>),
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

    fn to_c_value(self: &Self, expr: Expr, _type_: Option<Type>) -> CValue {
        match expr {
            Expr::String(str) => CValue::StringLiteral(str),
            //TODO: Actually implement this
            Expr::Number(num) => CValue::IntLiteral(num as i32),
            Expr::Binary { lhs, op, rhs } => CValue::BinaryOp(
                Box::new(self.to_c_value(lhs.unwrap().value, None)),
                op,
                Box::new(self.to_c_value(rhs.unwrap().value, None)),
            ),
            Expr::Variable(name) => CValue::Variable(name),
            Expr::SubprogramCall { name, args } => {
                let mut cvalues = Vec::new();
                for arg in args {
                    cvalues.push(self.to_c_value(arg.value, None));
                }
                CValue::SubProgCall(name, cvalues)
            }
        }
    }
    fn generate_stmt_cir(self: &Self, node: AstNode<Stmts>) -> Cir {
        match node.value {
            Stmts::Write { type_, expr } => {
                let cvalue = self.to_c_value(expr.value, Some(type_));
                let ctype = self.to_c_type(type_);
                Cir::Write(ctype, cvalue)
            }
            Stmts::Return { return_type, expr } => {
                let cvalue = self.to_c_value(expr.value, Some(return_type));
                Cir::Return(cvalue)
            }
            Stmts::SubProgramDef {
                name,
                return_type,
                stmts,
                params,
            } => {
                let return_type = self.to_c_type(return_type);
                let mut stmts_cir = Vec::new();
                for stmt in stmts {
                    stmts_cir.push(self.generate_stmt_cir(stmt));
                }
                let mut cparams = Vec::new();
                for param in params {
                    cparams.push(CParam {
                        name: param.name,
                        param_type: self.to_c_type(param.param_type),
                    })
                }
                Cir::SubProgDef {
                    name,
                    cparams,
                    return_type,
                    stmts_cir,
                }
            }
            Stmts::If { expr, stmts } => {
                let cvalue = self.to_c_value(expr.value, None);
                let mut stmts_cir = Vec::new();
                for stmt in stmts {
                    stmts_cir.push(self.generate_stmt_cir(stmt));
                }
                Cir::If(cvalue, stmts_cir)
            }
            Stmts::Set { .. } => todo!(),
            Stmts::Else(_) => todo!(),
            Stmts::SubProgramCall { .. } => todo!(),
        }
    }
}
