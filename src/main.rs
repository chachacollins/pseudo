mod codegen;
mod lexer;
mod parser;

use lexer::Lexer;
use std::process::{self, Command};
use std::{env, fs};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        eprintln!("[ERROR]: not enough arguements passed");
        println!("[USAGE]: pseudo <filename> [-o <output_name>]");
        process::exit(1);
    }
    let filename = &args[1];
    let mut output_file = None;
    for (i, arg) in args.iter().enumerate() {
        if arg == "-o" {
            if i + 1 >= args.len() {
                eprintln!("[ERROR]: file output path should be specified after the -o flag");
                process::exit(1);
            }
            output_file = Some(args[i + 1].as_str())
        }
    }
    //TODO: Check file extension
    let default_name;
    if output_file.is_none() {
        default_name = filename.splitn(2, '.').next().unwrap_or(&filename);
        output_file = Some(default_name);
    }
    //TODO: REMOVE THE UNWRAPS AND ACTUALLY HANDLE THE ERROR
    let source = fs::read_to_string(filename)
        .map_err(|err| eprintln!("Could not open file: {filename} because of {err}"))
        .unwrap();
    let lexer = Lexer::new(filename.to_string(), source);
    let mut parser = parser::Parser::new(lexer);
    let stmts = parser.parse_program();
    let mut code = String::new();
    codegen::generate_c_code(&mut code, stmts).unwrap();
    let c_filename = format!(
        "{}.c",
        output_file.expect("There should be a valid output file here")
    );
    fs::write(&c_filename, code).unwrap();
    let _ = Command::new("cc")
        .arg(&c_filename)
        .arg("-o")
        .arg(output_file.expect("There should be a valid file path here"))
        .output()
        .expect("Failed to compile the c program {c_filename}");
    fs::remove_file(c_filename).expect("Failed to remove {c_filename}");
}
