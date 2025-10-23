use std::path::Path;
use std::process::Command;
use std::{env, fs, io};

const HIDE_CURSOR: &str = "\x1b[?25l";
const UNHIDE_CURSOR: &str = "\x1b[?25h";
enum LogLevel {
    Success,
    Error,
}

fn pretty_print(msg: &str, level: LogLevel) {
    const GREEN: &str = "\x1b[32m";
    const RED: &str = "\x1b[31m";
    const RESET: &str = "\x1b[0m";
    match level {
        LogLevel::Success => eprintln!("{GREEN}{msg}{RESET}"),
        LogLevel::Error => eprintln!("{RED}{msg}{RESET}"),
    }
}

fn get_output_path(input_file_path: &str) -> String {
    let mut acc = Vec::new();
    input_file_path.split("/").for_each(|s| acc.push(s));
    let last = acc.pop().unwrap().split(".").next().unwrap();
    acc.push(last);
    return acc.join("/");
}

fn run_test(file_path: &str) {
    eprint!("\rCompiling file {file_path}                              ");
    let output = Command::new("cargo")
        .args(["pseudo", file_path, "--keep"])
        .output()
        .expect("Failed to run cargo pseudo command");
    if !output.status.success() {
        pretty_print(
            &format!("Example {} failed test because of : ", file_path),
            LogLevel::Error,
        );
        pretty_print(
            &format!("{}", String::from_utf8_lossy(&output.stderr)),
            LogLevel::Error,
        );
        eprint!("{UNHIDE_CURSOR}");
        std::process::exit(1);
    }
    let executable_path = get_output_path(file_path);

    eprint!("\rRunning file {executable_path}                              ");
    let output = Command::new(executable_path)
        .output()
        .expect(&format!("Failed to execute {file_path} command"));
    if !output.status.success() {
        pretty_print(
            &format!("Example {} failed test because of : ", file_path),
            LogLevel::Error,
        );
        pretty_print(
            &format!("{}", String::from_utf8_lossy(&output.stderr)),
            LogLevel::Error,
        );
        eprint!("{UNHIDE_CURSOR}");
        std::process::exit(1);
    }
}

fn main() -> io::Result<()> {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        eprintln!("Pass the folder to run the tests from");
        eprint!("{UNHIDE_CURSOR}");
        std::process::exit(1);
    }

    eprint!("{HIDE_CURSOR}");
    let mut i = 1;
    let dir_path = &args[1];
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let file_path = &entry.path();
        let file_path = Path::new(file_path);
        let extension = file_path.extension();
        match extension {
            Some(ext) => {
                let ext = ext.to_str().unwrap();
                if ext == "pseudo" {
                    let file_path = file_path.to_str().unwrap();
                    eprint!(
                        "\rRunning test: {i} on file: {file_path}                           \n"
                    );
                    i += 1;
                    run_test(file_path);
                }
            }
            None => {}
        }
    }
    pretty_print("\rDone! All tests passed succesfully!", LogLevel::Success);
    eprint!("{UNHIDE_CURSOR}");
    Ok(())
}
