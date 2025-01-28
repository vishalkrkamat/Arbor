use crossterm::terminal;
use crossterm::{
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, ClearType},
};
use std::{fs, io, io::Write, path::PathBuf};
use walkdir::WalkDir;
fn clear() {
    execute!(std::io::stdout(), terminal::Clear(ClearType::All)).unwrap();
}

fn main() -> io::Result<()> {
    //  clear();
    // start of terminal mode
    //    enable_raw_mode().expect("Failed to enter raw mode");
    //
    //   loop {
    //      match read().unwrap() {
    //         Event::Key(event) => match event.code {
    //            KeyCode::Char('k') => println!("Up "),
    //           KeyCode::Char('j') => println!("Down"),
    //          KeyCode::Char('q') => break,
    //         _ => {}
    //    },
    //   _ => {}
    //}
    //}

    //disable_raw_mode().expect("Failed to enter raw mode");
    // End of terminal mode

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
    for mut path in &vec {
        if path.is_dir() {
            //println!("Directory: {:?}", path);
            let _ = highlight(path);
        } else {
            println!("File: {:?}", path);
        }
    }

    Ok(())
}

//Highlight the directory
fn highlight(path: &PathBuf) -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(
        stdout,
        SetForegroundColor(Color::Red),
        SetBackgroundColor(Color::Black),
        Print(path.to_string_lossy()),
        ResetColor
    );
    writeln!(stdout)?;
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
