use std::fs;
use std::io;
use std::path::PathBuf;
use walkdir::WalkDir;
fn main() -> io::Result<()> {
    let path = "/home/vishal/";
    //list_dir(path.to_string(), 1);
    let mut vec: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(path)? {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                vec.push(path);
            }
            Err(_e) => eprintln!("Erro"),
        }
    }
    for path in &vec {
        if path.is_dir() {
            println!("Directory: {:?}", path);
        } else {
            println!("File: {:?}", path);
        }
    }

    Ok(())
}

fn list_dir(path: String, dept: u8) {
    let path = path;
    for entry in WalkDir::new(path).max_depth(dept.into()) {
        match entry {
            Ok(entry) => {
                println!("List{:?} ", entry)
            }
            Err(_e) => println!("The erro"),
        }
    }
}
