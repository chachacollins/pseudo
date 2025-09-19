use crate::ir::{CParam, CType, CValue, Cir};
use std::fmt::{Result, Write};

fn generate_prelude(sink: &mut impl Write) -> Result {
    writeln!(sink, "#include <stdio.h>")?;
    writeln!(sink, "#include <stdint.h>")?;
    Ok(())
}

fn generate_write_stmt(sink: &mut impl Write, ctype: &CType, cvalue: &CValue) -> Result {
    let write_value = match ctype {
        CType::Int => {
            format!("\"%d\", {cvalue}")
        }
        CType::Uint => {
            format!("\"%u\", {cvalue}")
        }
        CType::String => {
            format!("\"%s\", {cvalue}")
        }
    };
    writeln!(sink, "printf({write_value});")?;
    Ok(())
}

fn generate_return_stmt(sink: &mut impl Write, cvalue: &CValue) -> Result {
    writeln!(sink, "return {cvalue};")?;
    Ok(())
}

fn generate_subprogdef_stmt(
    sink: &mut impl Write,
    name: String,
    cparams: Vec<CParam>,
    return_type: &CType,
    stmts: Vec<Cir>,
) -> Result {
    write!(sink, "{return_type} {name}")?;

    write!(sink, "(")?;
    let param_strings: Vec<String> = cparams
        .iter()
        .map(|param| format!("{} {}", param.param_type, param.name))
        .collect();
    write!(sink, "{}", param_strings.join(", "))?;
    write!(sink, ")")?;

    writeln!(sink, "{{")?;
    generate_stmts(sink, stmts)?;
    writeln!(sink, "}}")?;
    Ok(())
}

// fn generate_subprogcall_stmt(sink: &mut impl Write, name: String, args: Vec<Expr>) -> Result {
//     let args_str = args
//         .iter()
//         .map(|arg| arg.to_string())
//         .collect::<Vec<String>>()
//         .join(", ");
//     writeln!(sink, "{name}({args_str});")?;
//     Ok(())
// }
//
fn generate_if_stmt(sink: &mut impl Write, expr: CValue, stmts: Vec<Cir>) -> Result {
    writeln!(sink, "if ({expr}) {{")?;
    generate_stmts(sink, stmts)?;
    writeln!(sink, "}}")?;
    Ok(())
}
//
// fn generate_else_stmt(sink: &mut impl Write, stmts: Vec<Stmts>) -> Result {
//     writeln!(sink, "}}")?;
//     writeln!(sink, "else {{")?;
//     generate_stmts(sink, stmts)?;
//     Ok(())
// }
//
// fn generate_set_stmt(sink: &mut impl Write, name: String, var_type: Type, expr: Expr) -> Result {
//     writeln!(sink, "{var_type} {name} = {expr};")?;
//     Ok(())
// }

fn generate_stmts(sink: &mut impl Write, stmts: Vec<Cir>) -> Result {
    for stmt in stmts {
        match stmt {
            Cir::Write(ctype, cvalue) => generate_write_stmt(sink, &ctype, &cvalue)?,
            Cir::Return(cvalue) => generate_return_stmt(sink, &cvalue)?,
            Cir::If(cvalue, stmts_cir) => generate_if_stmt(sink, cvalue, stmts_cir)?,
            Cir::SubProgDef {
                name,
                return_type,
                stmts_cir,
                cparams,
            } => {
                generate_subprogdef_stmt(sink, name.to_string(), cparams, &return_type, stmts_cir)?;
            }
        }
    }
    Ok(())
}

pub fn generate_c_code(sink: &mut impl Write, ir: Vec<Cir>) -> Result {
    generate_prelude(sink)?;
    generate_stmts(sink, ir)?;
    Ok(())
}
