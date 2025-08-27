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
        Expr::U32Number(num) => {
            format!("\"%u\", {num}")
        }
        Expr::String(str) => {
            format!("\"{str}\"")
        }
        Expr::SubprogramCall { .. } => {
            //TODO: Handle return type
            format!("\"%d\", {expr}")
        }
        Expr::Variable { var_type, name } => match var_type {
            Type::Nat => format!("\"%u\", {name}"),
            Type::Int => format!("\"%d\", {name}"),
            Type::String => format!("\"%s\", {name}"),
            Type::Void | Type::Unknown => unreachable!(),
        },
    };
    writeln!(sink, "printf({write_value});")?;
    Ok(())
}

fn generate_return_stmt(sink: &mut impl Write, expr: Expr) -> Result {
    writeln!(sink, "return {expr};")?;
    Ok(())
}

fn generate_subprogdef_stmt(
    sink: &mut impl Write,
    name: String,
    return_type: Type,
    params: Vec<Param>,
    stmts: Vec<Stmts>,
) -> Result {
    match return_type {
        Type::Int => write!(sink, "int32_t ")?,
        Type::String => write!(sink, "char* ")?,
        Type::Nat => write!(sink, "uint32_t ")?,
        Type::Void => write!(sink, "void ")?,
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
                Type::Nat => "uint32_t",
                Type::Void => unreachable!(),
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

fn generate_subprogcall_stmt(sink: &mut impl Write, name: String, args: Vec<Expr>) -> Result {
    let args_str = args
        .iter()
        .map(|arg| arg.to_string())
        .collect::<Vec<String>>()
        .join(", ");
    writeln!(sink, "{name}({args_str});")?;
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
            } => generate_subprogdef_stmt(sink, name, return_type, params, stmts)?,
            Stmts::SubProgramCall { name, args, .. } => {
                generate_subprogcall_stmt(sink, name, args)?
            }
        }
    }
    Ok(())
}

pub fn generate_c_code(sink: &mut impl Write, program: Vec<Stmts>) -> Result {
    generate_prelude(sink)?;
    generate_stmts(sink, program)?;
    Ok(())
}
