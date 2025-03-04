use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::*;
use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState},
    DefaultTerminal, Frame,
};
use std::path::PathBuf;
use std::{fs, io};

#[derive(Debug, Clone)]
enum ItemType {
    File,
    Dir,
}

#[derive(Debug, Clone)]
struct ListsItem {
    name: String,
    item_type: ItemType,
}

#[derive(Debug)]
struct FileManagerState {
    parent_items: Vec<ListsItem>,
    parent_dir: Option<PathBuf>,         // The parent's dir
    current_dir: PathBuf,                // The path currently in
    current_items: Vec<ListsItem>,       // Items in the current directory
    child_items: Option<Vec<ListsItem>>, // Items in the selected subdirectory
    selected_index: ListState,           // Which item in current_items is selected
}

impl FileManagerState {
    fn new(star_dir: &PathBuf) -> Self {
        let (files, parent_dir, parent_items) = Self::get_state_data(star_dir);
        Self {
            parent_items,
            current_items: files,
            current_dir: star_dir.to_path_buf(),
            parent_dir,
            child_items: Some(vec![]),
            selected_index: ListState::default(),
        }
    }

    fn get_state_data(start: &PathBuf) -> (Vec<ListsItem>, Option<PathBuf>, Vec<ListsItem>) {
        let files = list_dir(&start).unwrap();
        let parent_dir = start.parent().map(|p| p.to_path_buf());
        let parent_items = parent_dir
            .as_ref()
            .map_or_else(Vec::new, |p| list_dir(p).unwrap());
        (files, parent_dir, parent_items)
    }

    fn update_state(&mut self, new_dir: &PathBuf) {
        let (files, parent_dir, parent_items) = Self::get_state_data(new_dir);

        self.current_dir = new_dir.to_path_buf();
        self.current_items = files;
        self.parent_dir = parent_dir;
        self.parent_items = parent_items;
    }

    fn get_file_update_state(&mut self, items: Vec<ListsItem>) {
        self.child_items = Some(items);
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

    fn get_sub_files(&mut self) {
        if let Some(loc) = self.selected_index.selected() {
            if let Some(selected_dir) = self.current_items.get(loc) {
                let current_dir = &self.current_dir;
                match selected_dir.item_type {
                    ItemType::Dir => {
                        let chilpath = current_dir.join(&selected_dir.name);
                        let sub_files = list_dir(&chilpath).unwrap();
                        self.get_file_update_state(sub_files);
                    }
                    //ItemType::File => Self::preview(),
                    ItemType::File => println!(""),
                };
            }
        }
    }

    fn down(&mut self) {
        self.selected_index.select_next();
        self.get_sub_files();
    }

    fn up(&mut self) {
        self.selected_index.select_previous();
        self.get_sub_files();
    }

    fn previous_dir(&mut self) {
        if let Some(parent) = self.parent_dir.clone() {
            self.update_state(&parent);
        }
    }

    fn preview() {
        todo!()
    }

    fn next_dir(&mut self) {
        if let Some(loc) = self.selected_index.selected() {
            if let Some(selected_file) = self.current_items.get(loc) {
                match &selected_file.item_type {
                    ItemType::Dir => {
                        let mut new_dir = self.current_dir.clone();
                        new_dir.push(&selected_file.name);
                        self.update_state(&new_dir);
                    }
                    ItemType::File => Self::preview(),
                }
            }
        }
    }

    fn run(mut terminal: DefaultTerminal, state: &mut FileManagerState) -> io::Result<()> {
        loop {
            terminal.draw(|f| render(f, state))?;
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') => state.down(),
                    KeyCode::Char('k') => state.up(),
                    KeyCode::Char('h') => state.previous_dir(),
                    KeyCode::Char('l') => state.next_dir(),
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
    //println!("{:?}", state);
    let mut ustate = &mut state.selected_index;
    let parent_files = &state.parent_items;
    let current_files = &state.current_items;
    let sub_files = &state.child_items.clone().unwrap();
    let list_current_items: Vec<ListItem> =
        FileManagerState::convert_to_listitems(&current_files).unwrap();
    let list_sub_items: Vec<ListItem> = FileManagerState::convert_to_listitems(&sub_files).unwrap();

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
    let list_child_fiels = List::new(list_sub_items).block(Block::default().borders(Borders::ALL));
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(mainlay[1]);

    f.render_widget(current_directory.to_string(), mainlay[0]);
    f.render_widget(list_parent_files, layout[0]);
    f.render_stateful_widget(list, layout[1], &mut ustate);
    f.render_widget(list_child_fiels, layout[2]);
}
