use crossterm::event::{self, Event, KeyCode};
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    widgets::{Block, List, ListDirection, ListItem},
    DefaultTerminal, Frame,
};
use std::io::Error;
use std::{fs, io, io::Write, path::PathBuf};
use walkdir::WalkDir;

enum Fsys {
    File { name: String },
    Direc { name: String, subdir: Vec<Fsys> },
}
impl Fsys {
    fn read() -> io::Result<()> {
        const ROOT: &str = ".";
        for entry in fs::read_dir(ROOT)? {
            let entry = entry?;
            println!("ENry is {entry:?}");
        }
        Ok(())
    }
}
fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<(), Error> {
    loop {
        terminal.draw(render)?;
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                break;
            }
        }
    }
    Ok(())
}
fn render(frame: &mut Frame) {
    //let fed = list_dir("/".to_string(), 3);
    //   println!("{fed:?}");
    let mainlay = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(frame.area());
    let conte = Layout::horizontal([
        Constraint::Percentage(30),
        Constraint::Percentage(40),
        Constraint::Percentage(30),
    ])
    .split(mainlay[1]);

    let chared = vec!["hell", "heleddd", "ok df", "hekfd", "jfkej"];
    let chare2 = ["file", "file3", "file4", "file5", "file6"];

    let chare3 = ["file", "file3", "hello", "file4", "file5", "file6"];
    let list = List::new(chared)
        .block(Block::bordered().title("List"))
        .style(Style::new().white())
        .highlight_style(Style::new().italic())
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);

    let list2 = List::new(chare2)
        .block(Block::bordered().title("List"))
        .style(Style::new().white())
        .highlight_style(Style::new().italic())
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);

    //let item = ListItem::new(chared[2]);
    let list3 = List::new(chare3)
        .block(Block::bordered().title("List"))
        .style(Style::new().white())
        .highlight_style(Style::new().italic())
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);

    //frame.render_widget(greetin, frame.area());
    frame.render_widget(Block::new().title("Header"), mainlay[0]);
    frame.render_widget(list, conte[0]);
    frame.render_widget(list2, conte[1]);
    frame.render_widget(list3, conte[2]);
}

fn ls(path: String) -> io::Result<String> {
    let mut vec: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(path.clone())? {
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
            //println!("Directory: {:?}", path);
            let _ = highlight(path);
        } else {
            println!("File: {:?}", path);
        }
    }
    Ok(path)
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
