use crate::parser::{Expr, Param, Stmts, Type};
use std::fmt::{Result, Write};

fn generate_prelude(sink: &mut impl Write) -> Result {
    writeln!(sink, "#include <stdio.h>")?;
    writeln!(sink, "#include <stdint.h>")?;
    Ok(())
}

fn generate_write_stmt(sink: &mut impl Write, expr: Expr) -> Result {
    let write_value = match expr {
        Expr::I32Number(num) => {
            format!("\"%d\", {num}")
        }
        Expr::U32Number(_) => todo!(),
        Expr::String(str) => {
            format!("\"{str}\"")
        }
    };
    writeln!(sink, "printf({write_value});")?;
    Ok(())
}

fn generate_return_stmt(sink: &mut impl Write, expr: Expr) -> Result {
    let ret_value = match expr {
        Expr::I32Number(num) => {
            format!("{num}")
        }
        Expr::U32Number(_) => todo!(),
        Expr::String(str) => {
            format!("\"{str}\"")
        }
    };
    writeln!(sink, "return {ret_value};")?;
    Ok(())
}

fn generate_subprogram_stmt(
    sink: &mut impl Write,
    name: String,
    return_type: Type,
    params: Vec<Param>,
    stmts: Vec<Stmts>,
) -> Result {
    match return_type {
        Type::Int => write!(sink, "int32_t ")?,
        Type::String => write!(sink, "char* ")?,
        Type::Nat => todo!(),
        Type::Unknown => unreachable!(),
    }

    write!(sink, "{name}")?;

    write!(sink, "(")?;
    let param_strings: Vec<String> = params
        .iter()
        .map(|param| {
            let type_str = match param.param_type {
                Type::Int => "int32_t",
                Type::String => "char*",
                Type::Nat => todo!(),
                Type::Unknown => unreachable!(),
            };
            format!("{} {}", type_str, param.name)
        })
        .collect();
    write!(sink, "{}", param_strings.join(", "))?;
    write!(sink, ")")?;

    writeln!(sink, "{{")?;
    generate_stmts(sink, stmts)?;
    writeln!(sink, "}}")?;
    Ok(())
}

fn generate_stmts(sink: &mut impl Write, stmts: Vec<Stmts>) -> Result {
    for stmt in stmts {
        match stmt {
            Stmts::Write(expr) => generate_write_stmt(sink, expr)?,
            Stmts::Return(expr) => generate_return_stmt(sink, expr)?,
            Stmts::SubProgramDef {
                name,
                return_type,
                params,
                stmts,
            } => generate_subprogram_stmt(sink, name, return_type, params, stmts)?,
        }
    }
    Ok(())
}

pub fn generate_c_code(sink: &mut impl Write, program: Vec<Stmts>) -> Result {
    generate_prelude(sink)?;
    generate_stmts(sink, program)?;
    Ok(())
}
