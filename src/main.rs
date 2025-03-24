use std::fs;
mod ui;
mod utils;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{widgets::ListState, DefaultTerminal};
use std::{fs::File, io, path::PathBuf};
use utils::get_state_data;

#[derive(Debug, Clone)]
enum ItemType {
    File,
    Dir,
}

#[derive(Debug, Clone)]
pub struct ListsItem {
    name: String,
    item_type: ItemType,
}

#[derive(Debug, Clone)]
pub enum FileType {
    Text(String),
    Byes(Vec<u8>),
}
#[derive(Debug, Clone)]
pub enum Preview {
    Files(FileType),
    Directory(Vec<ListsItem>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PopUI {
    Confirmation,
    RenameUI,
    Creation,
}

#[derive(Debug)]
pub struct FileManagerState {
    parent: Parent,
    current_dir: PathBuf,          // The path currently in
    current_items: Vec<ListsItem>, // Items in the current directory
    child_items: Preview,          // Items in the selected subdirectory
    selected_index: ListState,     // Which item in current_items is selected
    temp: String,
    pop: Option<PopUI>,
}

#[derive(Debug)]
pub struct Parent {
    parent_items: Vec<ListsItem>,
    parent_dir: Option<PathBuf>, // The parent's dir
    selected: ListState,
}

impl FileManagerState {
    fn new(star_dir: &PathBuf) -> Self {
        let (files, parent_dir, parent_items) = get_state_data(star_dir);
        Self {
            parent: Parent {
                parent_dir,
                parent_items,
                selected: ListState::default(),
            },
            current_items: files,
            current_dir: star_dir.into(),
            child_items: Preview::Directory(vec![]),
            selected_index: ListState::default().with_selected(Some(0)),
            pop: None,
            temp: "".to_string(),
        }
    }

    fn update_state(&mut self, new_dir: PathBuf) {
        let (files, parent_dir, parent_items) = get_state_data(&new_dir);

        self.current_dir = new_dir;
        self.current_items = files;
        self.parent.parent_dir = parent_dir;
        self.parent.parent_items = parent_items;
    }
    fn get_file_update_state(&mut self, items: Vec<ListsItem>) {
        self.child_items = Preview::Directory(items);
        self.get_parent_index();
    }

    fn update_file_state_file(&mut self, con: String) {
        self.child_items = Preview::Files(FileType::Text(con));
    }

    fn update_file_state_binary(&mut self, con: Vec<u8>) {
        self.child_items = Preview::Files(FileType::Byes(con));
    }

    fn delete(&mut self) {
        if let Some(loc) = self.selected_index.selected() {
            if let Some(file) = self.current_items.get(loc) {
                let name = file.name.clone();
                let path = self.current_dir.join(name);

                let deletion_result = match file.item_type {
                    ItemType::File => fs::remove_file(&path),
                    ItemType::Dir => fs::remove_dir_all(&path),
                };
                match deletion_result {
                    Ok(_) => {
                        self.pop = None;
                        self.update_state(self.current_dir.clone());
                    }
                    Err(err) => {
                        // Log or handle the error as needed
                        eprintln!("Failed to delete {:?}: {}", path, err);
                    }
                }
            }
        }
    }

    fn rename(&mut self, input: &mut String) {
        if let Some(ind) = self.selected_index.selected() {
            if let Some(sel) = self.current_items.get(ind) {
                let filename = &sel.name;
                if input.ends_with("/") {
                    input.pop();
                }
                if fs::rename(filename, input).is_ok() {
                    self.update_state(self.current_dir.clone());
                    self.temp = "".to_string();
                    self.pop = None;
                };
            }
        };
    }

    fn create(&mut self, input: String) {
        let is_dir = input.ends_with('/');
        let mut parts: Vec<&str> = input.trim_end_matches('/').split('/').collect();

        if let Some(last) = parts.pop() {
            let parent_path = parts.join("/");

            if !parent_path.is_empty() {
                let mut current_path = self.current_dir.clone();
                current_path.push(&parent_path);
                if let Err(e) = fs::create_dir_all(current_path) {
                    eprintln!("Error creating directory: {e}");
                }
            }

            if is_dir {
                if parent_path.is_empty() {
                    let dir_path = PathBuf::from(last.to_string());
                    self.create_dir(dir_path);
                } else {
                    let dir_path = PathBuf::from(format!("{}/{}", parent_path, last));
                    self.create_dir(dir_path);
                }
            } else if parent_path.is_empty() {
                let file_path = PathBuf::from(last.to_string());
                self.create_file(file_path);
            } else {
                let file_path = PathBuf::from(format!("{}/{}", parent_path, last));

                self.create_file(file_path);
            }
        }
    }

    fn create_dir(&mut self, dir_path: PathBuf) {
        let mut path = self.current_dir.clone();
        path.push(&dir_path);
        match fs::create_dir_all(path) {
            Ok(_) => {
                self.creation_file_toggle();
            }
            Err(e) => eprint!("{e}"),
        }
    }

    fn create_file(&mut self, file_name: PathBuf) {
        let mut path = self.current_dir.clone();
        path.push(&file_name);

        match File::create(path) {
            Ok(_) => {
                self.creation_file_toggle();
            }
            Err(e) => eprint!("{e}"),
        }
    }

    fn creation_file_toggle(&mut self) {
        self.update_state(self.current_dir.clone());
        self.temp = "".into();
        self.pop = None;
    }

    fn get_sub_files(&mut self) {
        if let Some(loc) = self.selected_index.selected() {
            if let Some(selected_dir) = self.current_items.get(loc) {
                let current_dir = &self.current_dir;
                match selected_dir.item_type {
                    ItemType::Dir => {
                        let chilpath = current_dir.join(&selected_dir.name);
                        let sub_files = utils::list_dir(&chilpath).unwrap();
                        self.get_file_update_state(sub_files);
                    }
                    ItemType::File => {
                        if let Some(loc) = self.selected_index.selected() {
                            if let Some(selected_file) = self.current_items.get(loc) {
                                let current_file =
                                    self.current_dir.clone().join(selected_file.name.clone());
                                match fs::read_to_string(&current_file) {
                                    Ok(con) => self.update_file_state_file(con),
                                    Err(_) => match fs::read(current_file) {
                                        Ok(con) => self.update_file_state_binary(con),
                                        Err(_e) => eprint!("error"),
                                    },
                                };
                            }
                        }
                    }
                };
            }
        }
    }

    fn down(&mut self) {
        self.selected_index.select_next();
        if self.current_items.len() == self.selected_index.selected().unwrap() {
            self.selected_index.select(Some(0));
        }
        self.get_sub_files();
    }

    fn up(&mut self) {
        let lastl = self.current_items.len();
        if self.selected_index.selected().unwrap() == 0 {
            self.selected_index.select(Some(lastl));
        }
        self.selected_index.select_previous();
        self.get_sub_files();
    }

    fn get_parent_index(&mut self) {
        if let Some(name) = self
            .current_dir
            .file_name()
            .map(|name| name.to_string_lossy())
        {
            if let Some(index) = self
                .parent
                .parent_items
                .iter()
                .position(|item| item.name == name)
            {
                self.parent.selected = ListState::default().with_selected(Some(index));
            };
        }
    }

    fn previous_dir(&mut self) {
        if let Some(ref parent) = self.parent.parent_dir {
            self.update_state(parent.to_path_buf());
            self.selected_index = self.parent.selected.clone();
            self.get_sub_files();
        }
    }

    fn next_dir(&mut self) {
        if let Some(loc) = self.selected_index.selected() {
            if let Some(selected_file) = self.current_items.get(loc) {
                if let ItemType::Dir = selected_file.item_type {
                    let mut new_dir = self.current_dir.clone();
                    new_dir.push(&selected_file.name);
                    self.update_state(new_dir);
                    self.parent.selected = self.selected_index.clone();
                    self.selected_index = ListState::default().with_selected(Some(0));
                    self.get_sub_files();
                }
            }
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;

            if let Event::Key(key) = event::read()? {
                if let Some(PopUI::Confirmation) = self.pop.clone() {
                    match key.code {
                        KeyCode::Char('n') => self.toggle(),
                        KeyCode::Char('y') => self.delete(),
                        _ => {}
                    }
                    continue;
                }

                if let Some(PopUI::RenameUI) = self.pop.clone() {
                    match key.code {
                        KeyCode::Char(c) => {
                            self.temp.push(c);
                        }
                        // Append character to input
                        KeyCode::Backspace => {
                            self.temp.pop();
                        } // Remove last character
                        KeyCode::Enter => {
                            self.rename(&mut self.temp.clone());
                        }
                        KeyCode::Esc => self.pop = None,
                        _ => {}
                    }
                    continue;
                }

                if let Some(PopUI::Creation) = self.pop.clone() {
                    match key.code {
                        KeyCode::Char(c) => {
                            self.temp.push(c);
                        }
                        // Append character to input
                        KeyCode::Backspace => {
                            self.temp.pop();
                        } // Remove last character
                        KeyCode::Enter => {
                            self.create(self.temp.clone());
                        }
                        KeyCode::Esc => self.pop = None,
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') => self.down(),
                    KeyCode::Char('k') => self.up(),
                    KeyCode::Char('h') => self.previous_dir(),
                    KeyCode::Char('l') => self.next_dir(),
                    KeyCode::Char('d') => self.toggle(),
                    KeyCode::Char('r') => self.pop = Some(PopUI::RenameUI),
                    KeyCode::Char('a') => self.pop = Some(PopUI::Creation),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn toggle(&mut self) {
        if let Some(PopUI::Confirmation) = self.pop.clone() {
            self.pop = None
        } else {
            self.pop = Some(PopUI::Confirmation)
        }
    }
}

fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();
    let start_dir = PathBuf::from(".");
    let absolute_path = start_dir.canonicalize().unwrap();
    let appstate = FileManagerState::new(&absolute_path).run(terminal);
    ratatui::restore();
    appstate
}
