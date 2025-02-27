use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::*;
use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState},
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

#[derive(Debug)]
struct FileManagerState {
    parent_items: Vec<ListsItem>,
    parent_dir: Option<PathBuf>,   // For example, the parent's path
    current_dir: PathBuf,          // The path you are currently in
    current_items: Vec<ListsItem>, // Items in the current directory
    //child_items: Option<Vec<ListsItem>>, // Items in the selected subdirectory (if any)
    selected_index: ListState, // Which item in current_items is selected
}

impl FileManagerState {
    fn new(star_dir: &PathBuf) -> Self {
        let files = list_dir(&star_dir).unwrap();
        let parent_dir = star_dir.parent().map(|p| p.to_path_buf());

        let pathdw = parent_dir.clone().unwrap();
        let parent_items = list_dir(&pathdw).unwrap();
        let state = ListState::default();
        Self {
            parent_items,
            current_items: files,
            current_dir: star_dir.to_path_buf(),
            parent_dir,
            selected_index: state,
        }
    }

    fn convert_to_listitems(f: &Vec<ListsItem>) -> io::Result<Vec<ListItem>> {
        let list_items: Vec<ListItem> = f
            .iter()
            .map(|item| {
                let display = match item.item_type {
                    ItemType::Dir => format!("📁 {}", item.name),
                    ItemType::File => format!("📄 {}", item.name),
                };
                ListItem::new(display)
            })
            .collect();
        Ok(list_items)
    }
    fn down(&mut self) {
        self.selected_index.select_next();
    }
    fn up(&mut self) {
        self.selected_index.select_previous();
    }

    fn run(mut terminal: DefaultTerminal, state: &mut FileManagerState) -> io::Result<()> {
        loop {
            terminal.draw(|f| render(f, state))?;
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') => state.down(),
                    KeyCode::Char('k') => state.up(),
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();
    let start_dir = PathBuf::from(".");
    let absolute_path = start_dir.canonicalize().unwrap();
    let mut state = FileManagerState::new(&absolute_path);
    let result = FileManagerState::run(terminal, &mut state);
    ratatui::restore();
    result
}

fn list_dir(p: &PathBuf) -> std::io::Result<Vec<ListsItem>> {
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

fn render(f: &mut Frame, state: &mut FileManagerState) {
    let mut ustate = &mut state.selected_index;
    let parent_files = &state.parent_items;
    let current_files = &state.current_items;
    let list_current_items: Vec<ListItem> =
        FileManagerState::convert_to_listitems(&current_files).unwrap();

    let list_parent_items: Vec<ListItem> =
        FileManagerState::convert_to_listitems(&parent_files).unwrap();
    let current_directory = state.current_dir.to_string_lossy();

    let mainlay = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(f.area());

    let list = List::new(list_current_items)
        .highlight_symbol(">>")
        .block(Block::default().borders(Borders::ALL));
    let list_parent_files =
        List::new(list_parent_items).block(Block::default().borders(Borders::ALL));
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(mainlay[1]);

    f.render_widget(current_directory.to_string(), mainlay[0]);
    f.render_widget(list_parent_files, layout[0]);
    f.render_stateful_widget(list, layout[1], &mut ustate);
}
