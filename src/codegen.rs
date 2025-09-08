use crate::ir::{CType, CValue, Ir};
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
        _ => todo!(),
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
    return_type: &CType,
    rest: &[Ir],
) -> Result {
    write!(sink, "{return_type} {name}")?;

    write!(sink, "(")?;
    // let param_strings: Vec<String> = params
    //     .iter()
    //     .map(|param| format!("{} {}", param.param_type, param.name))
    //     .collect();
    // write!(sink, "{}", param_strings.join(", "))?;
    write!(sink, ")")?;

    writeln!(sink, "{{")?;
    generate_stmts(sink, rest, true)?;
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
// fn generate_if_stmt(sink: &mut impl Write, expr: Expr, stmts: Vec<Stmts>) -> Result {
//     writeln!(sink, "if ({expr}) {{")?;
//     generate_stmts(sink, stmts)?;
//     writeln!(sink, "}}")?;
//     Ok(())
// }
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

fn generate_stmts(sink: &mut impl Write, stmts: &[Ir], is_function: bool) -> Result {
    for (i, stmt) in stmts.into_iter().enumerate() {
        match stmt {
            Ir::Write(ctype, cvalue) => {
                if is_function {
                    generate_write_stmt(sink, ctype, cvalue)?
                }
            }
            Ir::Return(cvalue) => {
                if is_function {
                    generate_return_stmt(sink, cvalue)?
                }
            }
            Ir::SubProgDef(name, ctype) => {
                generate_subprogdef_stmt(sink, name.to_string(), ctype, &stmts[i + 1..])?;
            }
            _ => todo!(),
        }
    }
    Ok(())
}

pub fn generate_c_code(sink: &mut impl Write, ir: Vec<Ir>) -> Result {
    generate_prelude(sink)?;
    generate_stmts(sink, &ir, false)?;
    Ok(())
}
