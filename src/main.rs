mod codegen;
mod ir;
mod lexer;
mod parser;
mod semantic;

use crate::ir::CirGenerator;
use lexer::Lexer;
use semantic::SemanticAnalyzer;
use std::process::{self, Command};
use std::{env, fs};

//TODO: ADD ALL THE OPTIONS
fn print_usage() {
    println!("[USAGE]: pseudo <input> [-o <output>] [OPTIONS]");
}

fn cli_error(msg: &str) -> ! {
    eprintln!("[ERROR]: {msg}");
    print_usage();
    process::exit(1)
}

#[derive(Default)]
struct CompilerCtx<'a> {
    c_file_path: &'a str,
    output_path: &'a str,
    optimize: bool,
    keep_ir_output: bool,
}

fn compile_c_code(ctx: CompilerCtx) {
    let mut args = Vec::new();
    if ctx.optimize {
        args.push("-O3");
    }
    args.push(ctx.c_file_path);
    args.push("-o");
    args.push(ctx.output_path);
    //TODO: check the return status of this
    let _ = Command::new("cc")
        .args(args)
        .output()
        .expect("Failed to compile the c program {c_filename}");
    if !ctx.keep_ir_output {
        fs::remove_file(ctx.c_file_path).expect("Failed to remove {c_filename}");
    }
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<String>>();

    if args.is_empty() {
        cli_error("not enough arguements passed. See usage using --help");
    }
    let mut compiler_ctx = CompilerCtx::default();

    let input_file_path = &args[0];
    let mut output_file_path = None;

    let mut args = args[1..].iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" => match args.next() {
                Some(path) => output_file_path = Some(path.clone()),
                None => cli_error("file output path should be specified after the -o flag"),
            },
            "--help" => {
                print_usage();
            }
            "--optimize" => {
                compiler_ctx.optimize = true;
            }
            "--keep" => {
                compiler_ctx.keep_ir_output = true;
            }
            arg => {
                cli_error(&format!("Unknown arguement {arg} provided. See --help"));
            }
        }
    }
    //TODO: Check file extension
    if output_file_path.is_none() {
        output_file_path = Some(
            input_file_path
                .split('.')
                .next()
                .unwrap_or_else(|| cli_error("use the .pseudo extension"))
                .to_string(),
        );
    }
    let source = match fs::read_to_string(input_file_path) {
        Ok(contents) => contents,
        Err(err) => cli_error(&format!("could not open file: {input_file_path} {err}")),
    };
    let lexer = Lexer::new(input_file_path.to_string(), source);
    let mut parser = parser::Parser::new(lexer);
    let mut ast = parser.parse_program();
    let mut semanalyzer = SemanticAnalyzer::new();
    semanalyzer.analyze_ast(&mut ast);
    let mut code = String::new();
    let ir_generator = CirGenerator::new();
    let ir = ir_generator.generate_cir(ast);
    codegen::generate_c_code(&mut code, ir)
        .unwrap_or_else(|err| cli_error(&format!("could not generate c code {err}")));
    let c_file_path = format!(
        "{}.c",
        output_file_path
            .as_ref()
            .expect("There should be a valid output file here")
    );
    fs::write(&c_file_path, code).unwrap_or_else(|err| {
        cli_error(&format!("could not write generated c code to file {err}"))
    });
    compiler_ctx.c_file_path = &c_file_path;
    compiler_ctx.output_path = output_file_path
        .as_ref()
        .expect("There should be a valid output file here");
    compile_c_code(compiler_ctx);
}
