use std::path::Path;
use std::{fs, io};

fn main() -> io::Result<()> {
    let dir_path = "./examples";
    println!("Starting cleaning");
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let file_path = &entry.path();
        let file_path = Path::new(file_path);
        let extension = file_path.extension();
        match extension {
            Some(ext) => {
                if ext.to_str().unwrap() == "c" {
                    println!("Removing file: {}", file_path.to_str().unwrap());
                    fs::remove_file(file_path.to_str().unwrap())?
                }
            }
            None => {
                println!("Removing file: {}", file_path.to_str().unwrap());
                fs::remove_file(file_path.to_str().unwrap())?
            }
        }
    }
    println!("Finished cleaning");
    Ok(())
}
