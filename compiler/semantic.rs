use crate::parser::{AstNode, Expr, Position, Stmts, Type};
use std::collections::HashMap;

struct SubProgCtx {
    param_types: Vec<Type>,
    return_type: Type,
}

struct VarCtx {
    var_type: Type,
    mutable: bool,
}

struct SemError {
    msg: String,
    position: Position,
}

impl std::fmt::Display for SemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}:{}:{}: \x1b[31merror:\x1b[0m {}",
            self.position.filename, self.position.row, self.position.column, self.msg
        )
    }
}

pub struct SemanticAnalyzer {
    is_subprogram: bool,
    expected_return_type: Type,
    subprogram_table: HashMap<String, SubProgCtx>,
    local_var_table: HashMap<String, VarCtx>,
    errors: Vec<SemError>,
}

impl SemanticAnalyzer {
    pub fn new() -> SemanticAnalyzer {
        SemanticAnalyzer {
            is_subprogram: false,
            expected_return_type: Type::Unknown,
            subprogram_table: HashMap::new(),
            local_var_table: HashMap::new(),
            errors: Vec::new(),
        }
    }
    pub fn analyze_ast(self: &mut Self, ast: &mut [AstNode<Stmts>]) {
        for node in ast.iter_mut() {
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
                            if *return_type == Type::Void {
                                self.errors.push(SemError {
                                    msg: "main should be a function not a procedure".to_string(),
                                    position: node.position.clone(),
                                });
                            } else if *return_type != Type::Int {
                                self.errors.push(SemError {
                                    msg: "main function must have return type Int".to_string(),
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
        if !self.subprogram_table.contains_key("main") {
            eprintln!("\x1b[31merror:\x1b[0m main function not found");
            std::process::exit(1);
        }

        for node in ast {
            self.analyze_stmt(node);
        }

        if !self.errors.is_empty() {
            self.errors.iter_mut().for_each(|err| {
                eprintln!("{err}");
            });
            std::process::exit(1);
        }
    }

    //TODO: investigate whether we should return early when we detect errors
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
            Expr::Bool(_) => {
                if expected_type == Type::Bool || expected_type == Type::Unknown {
                    Type::Bool
                } else {
                    self.errors.push(SemError {
                        msg: format!("Expected type {:?}, found boolean", expected_type),
                        position: expr.position.clone(),
                    });
                    expected_type
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
            //TODO: get name of variable and check if the type matches
            Expr::Variable(name) => {
                if !self.local_var_table.contains_key(name) {
                    self.errors.push(SemError {
                        msg: format!("use of unknown variable {name}"),
                        position: expr.position.clone(),
                    });
                    return expected_type;
                }
                self.local_var_table.get(name).unwrap().var_type
            }
            Expr::SubprogramCall { name, args } => {
                if !self.subprogram_table.contains_key(name) {
                    self.errors.push(SemError {
                        msg: format!("subprogram {name} is not defined"),
                        position: expr.position.clone(),
                    });
                    return expected_type;
                }
                //TODO: check arity
                let arg_types = self.subprogram_table.get(name).unwrap().param_types.clone();
                for (i, arg) in args.iter().enumerate() {
                    let _ = self.analyze_expr(arg, arg_types[i]);
                }
                self.subprogram_table.get(name).unwrap().return_type
            }
            //TODO: analyze based on the operation kind
            Expr::Binary { lhs, rhs, .. } => {
                let mut lhs_type = expected_type;
                let mut rhs_type = expected_type;
                lhs_type = self.analyze_expr(lhs, lhs_type);

                rhs_type = self.analyze_expr(rhs, rhs_type);
                //TODO: make this error better
                if rhs_type != lhs_type {
                    self.errors.push(SemError {
                        msg: format!(
                            "type mismatch in binary expression lhs:{lhs_type:?} while rhs:{rhs_type:?}"
                        ),
                        position: expr.position.clone(),
                    });
                }
                lhs_type
            }
        }
    }

    fn analyze_stmt(self: &mut Self, node: &mut AstNode<Stmts>) {
        match &mut node.value {
            Stmts::Write { type_, expr } => {
                let gotten_type = self.analyze_expr(&expr, Type::Unknown);
                *type_ = gotten_type;
            }
            Stmts::Return { return_type, expr } => {
                //TODO: Check if it matches function return type
                let gotten_type = self.analyze_expr(&expr, Type::Unknown);
                if gotten_type != self.expected_return_type {
                    self.errors.push(SemError {
                        msg: format!(
                            "Expected return type {:?}, found {:?}",
                            self.expected_return_type, gotten_type
                        ),
                        position: expr.position.clone(),
                    });
                }
                *return_type = gotten_type;
            }
            Stmts::Set {
                name,
                var_type,
                expr,
                mutable,
            } => {
                if self.local_var_table.contains_key(name) {
                    self.errors.push(SemError {
                        msg: format!("redefinition of variable {name}",),
                        position: node.position.clone(),
                    });
                }
                let gotten_type = self.analyze_expr(expr, var_type.clone());
                *var_type = gotten_type;
                self.local_var_table.insert(
                    name.clone(),
                    VarCtx {
                        var_type: var_type.clone(),
                        mutable: mutable.clone(),
                    },
                );
            }
            Stmts::Assign { name, expr } => {
                if !self.local_var_table.contains_key(name) {
                    self.errors.push(SemError {
                        msg: format!("trying to assing value to unexisting variable: {name}",),
                        position: node.position.clone(),
                    });
                } else {
                    //TODO: check if we can reassign the variable type for example after type
                    //inference
                    let var_ctx = self.local_var_table.get(name).unwrap();
                    if var_ctx.mutable {
                        let _ = self.analyze_expr(expr, var_ctx.var_type);
                    } else {
                        self.errors.push(SemError {
                            msg: format!("trying to assing value to immutable variable: {name}",),
                            position: node.position.clone(),
                        });
                    }
                }
            }
            //TODO: Check the return statement
            Stmts::SubProgramDef {
                return_type,
                stmts,
                params,
                name,
            } => {
                if self.is_subprogram {
                    self.errors.push(SemError {
                        msg: format!("Cannot define subprogram {name} inside another subprogram"),
                        position: node.position.clone(),
                    });
                    return;
                }
                self.is_subprogram = true;
                self.expected_return_type = *return_type;
                for param in params {
                    self.local_var_table.insert(
                        param.name.clone(),
                        VarCtx {
                            var_type: param.param_type,
                            mutable: false,
                        },
                    );
                }
                let mut return_stmt_exists = false;
                //TODO: stronger, better checks for if stmts and what not
                for stmt in stmts.iter_mut() {
                    if matches!(
                        stmt.value,
                        Stmts::Return {
                            return_type: _,
                            expr: _
                        }
                    ) {
                        return_stmt_exists = true;
                    }
                    self.analyze_stmt(stmt)
                }
                if !return_stmt_exists && self.expected_return_type != Type::Void {
                    self.errors.push(SemError {
                        msg: format!("subprogram {name} does not have a return statement"),
                        position: node.position.clone(),
                    });
                    return;
                }
                self.local_var_table.clear();
                self.is_subprogram = false;
            }
            Stmts::SubProgramCall { name, args } => {
                if !self.subprogram_table.contains_key(name) {
                    self.errors.push(SemError {
                        msg: format!("subprogram {name} is not defined"),
                        position: node.position.clone(),
                    });
                    return;
                }
                let arg_types = self.subprogram_table.get(name).unwrap().param_types.clone();
                for (i, arg) in args.iter().enumerate() {
                    let _ = self.analyze_expr(arg, arg_types[i]);
                }
                if self.subprogram_table.get(name).unwrap().return_type != Type::Void {
                    self.errors.push(SemError {
                        msg: format!("subprogram {name} returns a value which is not used"),
                        position: node.position.clone(),
                    });
                }
            }
            Stmts::If { expr, stmts } => {
                //TODO: check if this is type bool
                let _gotten_type = self.analyze_expr(expr, Type::Unknown);
                for stmt in stmts.iter_mut() {
                    self.analyze_stmt(stmt)
                }
            }
            Stmts::While { expr, stmts } => {
                //TODO: check if this is type bool
                let _gotten_type = self.analyze_expr(expr, Type::Unknown);
                for stmt in stmts.iter_mut() {
                    self.analyze_stmt(stmt)
                }
            }
            //TODO: ensure it is within an if
            Stmts::Else(stmts) => {
                for stmt in stmts.iter_mut() {
                    self.analyze_stmt(stmt)
                }
            }
        }
    }
}
