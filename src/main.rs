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
}
impl FileManagerState {
    fn new(star_dir: &PathBuf) -> Self {
        let (files, parent_dir, parent_items) = get_state_data(star_dir);
        Self {
            parent: Parent {
                parent_dir,
                parent_items,
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

                match file.item_type {
                    ItemType::File => {
                        if fs::remove_file(path).is_ok() {
                            self.pop = None;
                        };
                        self.update_state(self.current_dir.clone());
                    }
                    ItemType::Dir => {
                        if fs::remove_dir_all(path).is_ok() {
                            self.pop = None;
                        };
                        self.update_state(self.current_dir.clone());
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
                if let Err(e) = fs::create_dir_all(PathBuf::from(&parent_path)) {
                    eprintln!("Error creating directory: {e}");
                }
            }

            if is_dir {
                let dir_path = PathBuf::from(format!("{}/{}", parent_path, last));

                match fs::create_dir_all(dir_path) {
                    Ok(_) => {
                        self.update_state(self.current_dir.clone());
                        self.temp = "".into();
                        self.pop = None;
                    }
                    Err(e) => eprint!("{e}"),
                }
            } else {
                let file_path = PathBuf::from(format!("{}/{}", parent_path, last));
                match File::create(&file_path) {
                    Ok(_) => {
                        self.update_state(self.current_dir.clone());
                        self.temp = "".into();
                        self.pop = None;
                    }
                    Err(e) => eprint!("{e}"),
                }
            }
        }
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

    fn previous_dir(&mut self) {
        if let Some(ref parent) = self.parent.parent_dir {
            self.update_state(parent.to_path_buf());
        }
    }

    fn next_dir(&mut self) {
        if let Some(loc) = self.selected_index.selected() {
            if let Some(selected_file) = self.current_items.get(loc) {
                if let ItemType::Dir = selected_file.item_type {
                    let mut new_dir = self.current_dir.clone();
                    new_dir.push(&selected_file.name);
                    self.update_state(new_dir);
                    self.selected_index = ListState::default().with_selected(Some(0));
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
