mod lexer;

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
    let source = fs::read_to_string(file_name)
        .map_err(|err| eprintln!("Could not open file: {file_name} because of {}", err))
        .unwrap();
    let lexer = Lexer::new(source);
    for token in lexer {
        println!("{:?}", token);
    }
}
