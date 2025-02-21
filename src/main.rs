use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::*;
use ratatui::{
    widgets::{Block, List, ListItem},
    DefaultTerminal, Frame,
};
use std::path::{Path, PathBuf};
use std::{fs, io};

#[derive(Debug)]
enum ItemType {
    File,
    Dir,
}
#[derive(Debug)]
struct ListsItem {
    name: String,
    item_type: ItemType,
}

struct FileManagerState {
    parent_dir: Option<String>,          // For example, the parent's path
    current_dir: String,                 // The path you are currently in
    current_items: Vec<ListsItem>,       // Items in the current directory
    child_items: Option<Vec<ListsItem>>, // Items in the selected subdirectory (if any)
    selected_index: usize,               // Which item in current_items is selected
}

fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    loop {
        terminal.draw(render)?;
        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => break,
                _ => {}
            },

            _ => {}
        }
    }
    Ok(())
}

fn list_dir(p: &str) -> std::io::Result<Vec<ListsItem>> {
    let mut items = Vec::new();
    for entry in fs::read_dir(p)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        let file_type = if meta.is_dir() {
            ItemType::Dir
        } else {
            ItemType::File
        };
        let item = ListsItem {
            name: entry.file_name().into_string().unwrap(),
            item_type: file_type,
        };
        items.push(item);
    }
    Ok(items)
}
fn render(f: &mut Frame) {
    let tes = list_dir(".").unwrap();

    let list_items: Vec<ListItem> = tes
        .iter()
        .map(|item| {
            let display = match item.item_type {
                ItemType::Dir => format!("📁 {}", item.name),
                ItemType::File => format!("📄 {}", item.name),
            };
            ratatui::widgets::ListItem::new(display)
        })
        .collect();

    let list = List::new(list_items);
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.area());

    f.render_widget(list, layout[0]);
    f.render_widget("ok see", layout[1]);
}
