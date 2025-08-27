mod codegen;
mod lexer;
mod parser;

use lexer::Lexer;
use std::process::{self, Command};
use std::{env, fs};

fn cli_error(msg: &str) -> ! {
    eprintln!("[ERROR]: {msg}");
    process::exit(1)
}

fn print_usage() {
    println!("[USAGE]: pseudo <input> [-o <output>]");
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        cli_error("not enough arguements passed. See usage using --help");
    }
    let filename = &args[1];
    let mut output_file = None;
    for (i, arg) in args.iter().enumerate() {
        if arg == "-o" {
            if i + 1 >= args.len() {
                cli_error("file output path should be specified after the -o flag");
            }
            output_file = Some(args[i + 1].as_str());
        } else if arg == "--help" {
            print_usage();
        }
    }
    //TODO: Check file extension
    let default_name;
    if output_file.is_none() {
        default_name = filename
            .split('.')
            .next()
            .unwrap_or_else(|| cli_error("use the .pseudo extension"));
        output_file = Some(default_name);
    }
    let source = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(err) => cli_error(&format!("could not open file: {filename} {err}")),
    };
    let lexer = Lexer::new(filename.to_string(), source);
    let mut parser = parser::Parser::new(lexer);
    let stmts = parser.parse_program();
    let mut code = String::new();
    codegen::generate_c_code(&mut code, stmts)
        .unwrap_or_else(|err| cli_error(&format!("could not generate c code {err}")));
    let c_filename = format!(
        "{}.c",
        output_file.expect("There should be a valid output file here")
    );
    fs::write(&c_filename, code).unwrap_or_else(|err| {
        cli_error(&format!("could not write generated c code to file {err}"))
    });
    let _ = Command::new("cc")
        .arg(&c_filename)
        .arg("-o")
        .arg(output_file.expect("There should be a valid file path here"))
        .output()
        .expect("Failed to compile the c program {c_filename}");
    fs::remove_file(c_filename).expect("Failed to remove {c_filename}");
}
