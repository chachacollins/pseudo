mod codegen;
mod lexer;
mod parser;

use lexer::Lexer;
use std::{env, fs, process};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        eprintln!("[ERROR]: not enough arguements passed");
        println!("[USAGE]: pseudo <filename> [-o <output_name>]");
        process::exit(1);
    }
    let file_name = &args[1];
    //TODO: REMOVE THE UNWRAPS AND ACTUALLY HANDLE THE ERROR
    let source = fs::read_to_string(file_name)
        .map_err(|err| eprintln!("Could not open file: {file_name} because of {err}"))
        .unwrap();
    let lexer = Lexer::new(file_name.to_string(), source);
    let mut parser = parser::Parser::new(lexer);
    let stmts = parser.parse_program();
    let mut code = String::new();
    codegen::generate_c_code(&mut code, stmts).unwrap();
    let output_filename = format!("{}.c", file_name.splitn(2, '.').collect::<Vec<&str>>()[0]);
    fs::write(output_filename, code).unwrap();
}
