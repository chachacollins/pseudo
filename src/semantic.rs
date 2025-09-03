use crate::parser::{AstNode, Position, Stmts, Type};
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
    pub fn analyze_ast(self: &mut Self, ast: Vec<AstNode<Stmts>>) {
        for node in ast {
            match node.value {
                Stmts::SubProgramDef {
                    name,
                    return_type,
                    params,
                    ..
                } => {
                    if self.subprogram_table.contains_key(&name) {
                        self.errors.push(SemError {
                            msg: format!("redefinition of function {name}"),
                            position: node.position,
                        });
                    } else {
                        let is_main = name == "main";
                        if is_main {
                            //TODO: CHECK FOR PROC
                            if return_type != Type::Int {
                                self.errors.push(SemError {
                                    msg: "main function must have return type int".to_string(),
                                    position: node.position.clone(),
                                });
                            }
                            if params.len() != 0 {
                                self.errors.push(SemError {
                                    msg: "main function does not take any arguement".to_string(),
                                    position: node.position,
                                });
                            }
                        }
                        let param_types = params
                            .into_iter()
                            .map(|param| param.param_type)
                            .collect::<Vec<Type>>();
                        self.subprogram_table.insert(
                            name,
                            SubProgCtx {
                                param_types,
                                return_type,
                            },
                        );
                    }
                }
                _ => continue,
            }
        }
        dbg!(&self.subprogram_table);
        dbg!(&self.errors);
    }
}
