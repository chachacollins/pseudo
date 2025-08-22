use crate::parser::{Expr, Param, Program, Stmts, Type};
use std::fmt::{Error, Write};

fn generate_prelude(sink: &mut impl Write) -> Result<(), Error> {
    writeln!(sink, "#include <stdio.h>")?;
    writeln!(sink, "#include <stdint.h>")?;
    Ok(())
}

fn generate_write_stmt(sink: &mut impl Write, expr: Option<Expr>) -> Result<(), Error> {
    if let Some(expr) = expr {
        let write_value = match expr {
            Expr::I32Number(num) => {
                format!("\"%d\", {num}")
            }
            Expr::String(str) => {
                format!("\"{str}\"")
            }
        };
        writeln!(sink, "printf({write_value});")?;
    } else {
        writeln!(sink, "printf(\"\");")?;
    }
    Ok(())
}

fn generate_return_stmt(sink: &mut impl Write, expr: Option<Expr>) -> Result<(), Error> {
    if let Some(expr) = expr {
        let ret_value = match expr {
            Expr::I32Number(num) => {
                format!("{num}")
            }
            Expr::String(str) => {
                format!("\"{str}\"")
            }
        };
        writeln!(sink, "return {ret_value};")?;
    } else {
        writeln!(sink, "return;")?;
    }
    Ok(())
}

fn generate_subprogram_stmt(
    sink: &mut impl Write,
    name: String,
    return_type: Type,
    params: Vec<Param>,
    stmts: Vec<Stmts>,
) -> Result<(), Error> {
    match return_type {
        Type::I32 => write!(sink, "int32_t ")?,
        Type::String => write!(sink, "char* ")?,
    }

    write!(sink, "{name}")?;

    write!(sink, "(")?;
    let param_strings: Vec<String> = params
        .iter()
        .map(|param| {
            let type_str = match param.param_type {
                Type::I32 => "int32_t",
                Type::String => "char*",
            };
            format!("{} {}", type_str, param.name)
        })
        .collect();
    write!(sink, "{}", param_strings.join(", "))?;
    write!(sink, ")")?;

    write!(sink, "{{\n")?;
    let funct_stmt = generate_stmts(stmts, false)?;
    write!(sink, "{funct_stmt}")?;
    write!(sink, "}}\n")?;
    Ok(())
}

pub fn generate_stmts(program: Program, prelude: bool) -> Result<String, Error> {
    let mut code = String::new();
    if prelude {
        generate_prelude(&mut code)?;
    }
    for stmt in program {
        match stmt {
            Stmts::Write(expr) => generate_write_stmt(&mut code, expr)?,
            Stmts::Return(expr) => generate_return_stmt(&mut code, expr)?,
            Stmts::SubProgramDef {
                name,
                return_type,
                params,
                stmts,
            } => generate_subprogram_stmt(&mut code, name, return_type, params, stmts)?,
        }
    }
    Ok(code)
}
