use std::path::Path;
use std::process::Command;
use std::{env, fs, io};

fn main() -> io::Result<()> {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        eprintln!("Pass the folder to run the tests from");
        std::process::exit(1);
    }

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
                    eprint!("\rRunning test: {i} file: {file_path}                           ");
                    i += 1;
                    let output = Command::new("cargo")
                        .args(["pseudo", file_path])
                        .output()
                        .expect("Failed to run cargo pseudo command");
                    if !output.status.success() {
                        eprintln!("Example {} failed test because of : ", file_path);
                        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                        std::process::exit(1);
                    }
                }
            }
            None => {}
        }
    }
    eprintln!("\r\nDone! All tests passed succesfully!");
    Ok(())
}
