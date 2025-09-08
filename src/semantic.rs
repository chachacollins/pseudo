use crate::ir::{CType, CValue, Ir};
use crate::parser::{AstNode, Expr, Position, Stmts, Type};
use std::collections::HashMap;

#[derive(Debug)]
struct SubProgCtx {
    param_types: Vec<Type>,
    return_type: Type,
}

struct VarCtx {
    var_type: Type,
    mutable: bool,
}

#[derive(Debug)]
struct SemError {
    msg: String,
    position: Position,
}

pub struct SemanticAnalyzer {
    is_subprogram: bool,
    expected_return_type: Type,
    subprogram_table: HashMap<String, SubProgCtx>,
    local_var_table: HashMap<String, VarCtx>,
    errors: Vec<SemError>,
    pub ir: Vec<Ir>,
}

impl SemanticAnalyzer {
    pub fn new() -> SemanticAnalyzer {
        SemanticAnalyzer {
            is_subprogram: false,
            expected_return_type: Type::Unknown,
            subprogram_table: HashMap::new(),
            local_var_table: HashMap::new(),
            errors: Vec::new(),
            ir: Vec::new(),
        }
    }
    pub fn analyze_ast(self: &mut Self, ast: Vec<AstNode<Stmts>>) {
        for node in &ast {
            match &node.value {
                Stmts::SubProgramDef {
                    name,
                    return_type,
                    params,
                    ..
                } => {
                    if self.subprogram_table.contains_key(name) {
                        self.errors.push(SemError {
                            msg: format!("redefinition of function {name}"),
                            position: node.position.clone(),
                        });
                    } else {
                        let is_main = name == "main";
                        if is_main {
                            //TODO: CHECK FOR PROC
                            if *return_type != Type::Int {
                                self.errors.push(SemError {
                                    msg: "main function must have return type int".to_string(),
                                    position: node.position.clone(),
                                });
                            }
                            if params.len() != 0 {
                                self.errors.push(SemError {
                                    msg: "main function does not take any arguement".to_string(),
                                    position: node.position.clone(),
                                });
                            }
                        }
                        let param_types = params
                            .into_iter()
                            .map(|param| param.param_type)
                            .collect::<Vec<Type>>();
                        self.subprogram_table.insert(
                            name.to_string(),
                            SubProgCtx {
                                param_types,
                                return_type: *return_type,
                            },
                        );
                    }
                }
                _ => continue,
            }
        }
        //TODO: Ensure main exists
        for node in ast {
            self.analyze_stmt(node);
        }
    }

    fn analyze_expr(self: &mut Self, expr: &AstNode<Expr>, expected_type: Type) -> Type {
        match &expr.value {
            //TODO: Abstract this into it's own function and change it to use literals
            Expr::Number(num) => {
                if expected_type == Type::Unknown {
                    if *num >= i32::MIN as i128 && *num <= i32::MAX as i128 {
                        Type::Int
                    } else if *num >= u32::MIN as i128 && *num <= u32::MAX as i128 {
                        Type::Nat
                    } else {
                        todo!()
                    }
                } else if expected_type == Type::Int {
                    if *num < i32::MIN as i128 {
                        self.errors.push(SemError {
                            msg: "The number passed is too small to be represented by type int"
                                .to_string(),
                            position: expr.position.clone(),
                        });
                        expected_type
                    } else if *num > i32::MAX as i128 {
                        self.errors.push(SemError {
                            msg: "The number passed is too large to be represented by type int"
                                .to_string(),
                            position: expr.position.clone(),
                        });
                        expected_type
                    } else {
                        expected_type
                    }
                } else if expected_type == Type::Nat {
                    if *num < u32::MIN as i128 {
                        self.errors.push(SemError {
                            msg: "The number passed is too small to be represented by type nat"
                                .to_string(),
                            position: expr.position.clone(),
                        });
                        expected_type
                    } else if *num > u32::MAX as i128 {
                        self.errors.push(SemError {
                            msg: "The number passed is too large to be represented by type nat"
                                .to_string(),
                            position: expr.position.clone(),
                        });
                        expected_type
                    } else {
                        expected_type
                    }
                } else {
                    todo!()
                }
            }
            Expr::String(_) => {
                if expected_type == Type::String || expected_type == Type::Unknown {
                    Type::String
                } else {
                    self.errors.push(SemError {
                        msg: format!("Expected type {:?}, found string literal", expected_type),
                        position: expr.position.clone(),
                    });
                    expected_type
                }
            }
            Expr::Variable { .. } => todo!(),
            Expr::SubprogramCall { .. } => todo!(),
            Expr::Binary { .. } => todo!(),
        }
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

    fn analyze_stmt(self: &mut Self, node: AstNode<Stmts>) {
        match node.value {
            Stmts::Write(expr_node) => {
                let gotten_type = self.analyze_expr(&expr_node, Type::Unknown);
                let ctype = self.to_c_type(gotten_type);
                let cvalue = self.to_c_value(expr_node.value, gotten_type);
                self.ir.push(Ir::Write(ctype, cvalue))
            }
            Stmts::Return(expr_node) => {
                //TODO: Check if it matches function return type
                let gotten_type = self.analyze_expr(&expr_node, Type::Unknown);
                let ctype = self.to_c_type(gotten_type);
                let cvalue = self.to_c_value(expr_node.value, gotten_type);
                self.ir.push(Ir::Return(cvalue))
            }
            Stmts::Set { .. } => todo!(),
            Stmts::SubProgramDef {
                name,
                return_type,
                stmts,
                ..
            } => {
                self.expected_return_type = return_type;
                //TODO: Check if name exists
                let ctype = self.to_c_type(return_type);
                self.ir.push(Ir::SubProgDef(name, ctype));
                for stmt in stmts {
                    self.analyze_stmt(stmt)
                }
            }
            Stmts::SubProgramCall { .. } => todo!(),
            Stmts::If { .. } => todo!(),
            Stmts::Else { .. } => todo!(),
        }
    }
}
