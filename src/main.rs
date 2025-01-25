use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let path = ".";
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let mut n = String::new();
            println!("the directory {}", path.display());
            io::stdin().read_line(&mut n);
            if n.trim() == "n" {
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        println!("The directory: {path:?}");
                    }
                }
            }
        } else {
            println!("The file {}", path.display())
        }
    }
    Ok(())
}
