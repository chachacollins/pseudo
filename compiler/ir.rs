use crate::parser::{AstNode, Expr, Op, Stmts, Type};
use std::fmt;

#[derive(Debug)]
pub enum CType {
    Int,
    Uint,
    String,
    Bool,
    Void,
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
            CType::String => write!(f, "string_t"),
            CType::Bool => write!(f, "bool"),
            CType::Void => write!(f, "void"),
        }
    }
}
#[derive(Debug)]
pub enum CValue {
    NumLiteral(i128),
    StringLiteral(String),
    Bool(bool),
    Variable(String),
    BinaryOp(Box<CValue>, Op, Box<CValue>),
    SubProgCall(String, Vec<CValue>),
}

impl fmt::Display for CValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CValue::NumLiteral(n) => write!(f, "{n}"),
            CValue::StringLiteral(s) => write!(f, "StrLit(\"{s}\")"),
            CValue::Variable(name) => write!(f, "{name}"),
            CValue::Bool(val) => write!(f, "{val}"),
            CValue::BinaryOp(lhs, op, rhs) => match &**lhs {
                CValue::StringLiteral(_) => match op {
                    Op::Add => write!(f, "string_concat(&gc, &{lhs}, &{rhs})"),
                    _ => unreachable!(),
                },
                _ => {
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
            },
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
    Else(Vec<Cir>),
    VariableDef(String, CType, CValue, bool),
    VarAssign(String, CValue),
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
            Type::Bool => CType::Bool,
            Type::Void => CType::Void,
            Type::Unknown => unreachable!(),
        }
    }

    fn to_c_value(self: &Self, expr: Expr) -> CValue {
        match expr {
            Expr::String(str) => CValue::StringLiteral(str),
            //TODO: Actually implement this
            Expr::Number(num) => CValue::NumLiteral(num),
            Expr::Bool(bool_val) => CValue::Bool(bool_val),
            Expr::Binary { lhs, op, rhs } => CValue::BinaryOp(
                Box::new(self.to_c_value(lhs.value)),
                op,
                Box::new(self.to_c_value(rhs.value)),
            ),
            Expr::Variable(name) => CValue::Variable(name),
            Expr::SubprogramCall { name, args } => {
                let mut cvalues = Vec::new();
                for arg in args {
                    cvalues.push(self.to_c_value(arg.value));
                }
                CValue::SubProgCall(name, cvalues)
            }
        }
    }
    fn generate_stmt_cir(self: &Self, node: AstNode<Stmts>) -> Cir {
        match node.value {
            Stmts::Write { type_, expr } => {
                let cvalue = self.to_c_value(expr.value);
                let ctype = self.to_c_type(type_);
                Cir::Write(ctype, cvalue)
            }
            Stmts::Return { expr, .. } => {
                let cvalue = self.to_c_value(expr.value);
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
                let cvalue = self.to_c_value(expr.value);
                let mut stmts_cir = Vec::new();
                for stmt in stmts {
                    stmts_cir.push(self.generate_stmt_cir(stmt));
                }
                Cir::If(cvalue, stmts_cir)
            }
            Stmts::Set {
                name,
                expr,
                var_type,
                mutable,
            } => {
                let cvalue = self.to_c_value(expr.value);
                let ctype = self.to_c_type(var_type);
                Cir::VariableDef(name, ctype, cvalue, mutable)
            }
            Stmts::Assign { name, expr } => {
                let cvalue = self.to_c_value(expr.value);
                Cir::VarAssign(name, cvalue)
            }
            Stmts::Else(stmts) => {
                let mut stmts_cir = Vec::new();
                for stmt in stmts {
                    stmts_cir.push(self.generate_stmt_cir(stmt));
                }
                Cir::Else(stmts_cir)
            }
            Stmts::SubProgramCall { .. } => todo!(),
        }
    }
}
