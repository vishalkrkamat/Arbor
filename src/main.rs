use std::fs;
use std::io;
use walkdir::WalkDir;
fn main() -> io::Result<()> {
    let path = ".";
    for entry in WalkDir::new(path) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_dir() {
                    println!("Directory: {})", path.display());
                } else {
                    println!("File: {}", path.display())
                }
            }
            Err(_e) => eprintln!("Erro"),
        }
    }
    Ok(())
}
