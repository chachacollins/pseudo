use crate::ir::{CParam, CType, CValue, Cir};
use std::fmt::{self, Write};

pub struct CodeGen {
    sink: String,
    is_main: bool,
}

impl CodeGen {
    pub fn new() -> Self {
        Self {
            sink: String::new(),
            is_main: false,
        }
    }

    fn generate_prelude(self: &mut Self) -> fmt::Result {
        writeln!(self.sink, "#include <pseudo.h>")?;
        writeln!(self.sink, "static tgc_t gc;")?;
        Ok(())
    }

    fn generate_write_stmt(self: &mut Self, ctype: &CType, cvalue: &CValue) -> fmt::Result {
        let print_func = match ctype {
            CType::Int => {
                format!("print_int({cvalue})")
            }
            CType::Uint => {
                format!("print_uint({cvalue})")
            }
            CType::String => {
                format!("print_str({cvalue})")
            }
            CType::Bool => {
                format!("print_bool({cvalue})")
            }
            CType::Void => {
                //TODO: Warn that you can't print void values
                format!("void")
            }
        };
        writeln!(self.sink, "{print_func};")?;
        Ok(())
    }

    fn generate_return_stmt(self: &mut Self, cvalue: &CValue) -> fmt::Result {
        if self.is_main {
            writeln!(self.sink, "tgc_stop(&gc);")?;
        }
        writeln!(self.sink, "return {cvalue};")?;
        Ok(())
    }

    fn generate_subprogdef_stmt(
        self: &mut Self,
        name: String,
        cparams: Vec<CParam>,
        return_type: &CType,
        stmts: Vec<Cir>,
    ) -> fmt::Result {
        self.is_main = &name == "main";
        write!(self.sink, "{return_type} {name}")?;

        if self.is_main {
            write!(self.sink, "(int argc, char** argv)")?;
        } else {
            let param_strings: Vec<String> = cparams
                .iter()
                .map(|param| format!("{} {}", param.param_type, param.name))
                .collect();
            write!(self.sink, "({})", param_strings.join(", "))?;
        }

        writeln!(self.sink, "{{")?;
        if self.is_main {
            writeln!(self.sink, "tgc_start(&gc, &argc);")?;
        }
        self.generate_stmts(stmts)?;
        writeln!(self.sink, "}}")?;
        Ok(())
    }

    fn generate_if_stmt(self: &mut Self, expr: CValue, stmts: Vec<Cir>) -> fmt::Result {
        writeln!(self.sink, "if ({expr}) {{")?;
        self.generate_stmts(stmts)?;
        writeln!(self.sink, "}}")?;
        Ok(())
    }

    fn generate_set_stmt(
        self: &mut Self,
        name: String,
        var_type: CType,
        expr: CValue,
        mutable: bool,
    ) -> fmt::Result {
        if !mutable {
            write!(self.sink, "const ")?;
        }
        writeln!(self.sink, "{var_type} {name} = {expr};")?;
        Ok(())
    }

    fn generate_varassign_stmt(self: &mut Self, name: String, expr: CValue) -> fmt::Result {
        writeln!(self.sink, "{name} = {expr};")?;
        Ok(())
    }

    fn generate_else_stmt(self: &mut Self, stmts: Vec<Cir>) -> fmt::Result {
        writeln!(self.sink, "}}")?;
        writeln!(self.sink, "else {{")?;
        self.generate_stmts(stmts)?;
        Ok(())
    }

    fn generate_stmts(self: &mut Self, stmts: Vec<Cir>) -> fmt::Result {
        for stmt in stmts {
            match stmt {
                Cir::Write(ctype, cvalue) => self.generate_write_stmt(&ctype, &cvalue)?,
                Cir::Return(cvalue) => self.generate_return_stmt(&cvalue)?,
                Cir::If(cvalue, stmts_cir) => self.generate_if_stmt(cvalue, stmts_cir)?,
                Cir::Else(stmts_cir) => self.generate_else_stmt(stmts_cir)?,
                Cir::SubProgDef {
                    name,
                    return_type,
                    stmts_cir,
                    cparams,
                } => {
                    self.generate_subprogdef_stmt(
                        name.to_string(),
                        cparams,
                        &return_type,
                        stmts_cir,
                    )?;
                }
                Cir::VariableDef(name, var_type, cvalue, mutable) => {
                    self.generate_set_stmt(name, var_type, cvalue, mutable)?
                }
                Cir::VarAssign(name, cvalue) => self.generate_varassign_stmt(name, cvalue)?,
            }
        }
        Ok(())
    }

    pub fn generate_c_code(self: &mut Self, ir: Vec<Cir>) -> Result<String, std::fmt::Error> {
        self.generate_prelude()?;
        self.generate_stmts(ir)?;
        Ok(self.sink.clone())
    }
}

// fn generate_subprogcall_stmt(sink: &mut impl Write, name: String, args: Vec<Expr>) -> fmt::Result {
//     let args_str = args
//         .iter()
//         .map(|arg| arg.to_string())
//         .collect::<Vec<String>>()
//         .join(", ");
//     writeln!(sink, "{name}({args_str});")?;
//     Ok(())
// }
//
//
//
