mod event_handler;
mod ui;
mod utils;
use ratatui::widgets::ListState;
use std::fs;
use std::path::PathBuf;
use utils::get_state_data;

#[derive(Debug, Clone, PartialEq)]
enum ItemType {
    File,
    Dir,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListsItem {
    name: String,
    item_type: ItemType,
    selected: bool,
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

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Selection,
}

#[derive(Debug)]
pub struct FileManagerState {
    parent: Parent,
    current_dir: PathBuf,          // The path currently in
    current_items: Vec<ListsItem>, // Items in the current directory
    child_items: Preview,          // Items in the selected subdirectory
    selected_index: ListState,     // Which item in current_items is selected
    mode: Mode,
    selected_items: Vec<PathBuf>,
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
        let mut state = Self {
            parent: Parent {
                parent_dir,
                parent_items,
                selected: ListState::default(),
            },
            current_items: files,
            current_dir: star_dir.into(),
            child_items: Preview::Directory(vec![]),
            mode: Mode::Normal,
            selected_items: vec![],
            selected_index: ListState::default().with_selected(Some(0)),
            pop: None,
            temp: "".to_string(),
        };
        state.get_sub_files();
        state.get_parent_index();
        state
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
        self.get_parent_index();
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
                        eprintln!("Failed to delete {:?}: {}", path, err);
                    }
                }
            }
        }
    }

    fn mass_deletion(&mut self) {
        let selected_field = self.get_selected_items();

        for i in selected_field.iter() {
            if i.is_file() {
                fs::remove_file(i).unwrap();
            } else {
                fs::remove_dir_all(i).unwrap();
            }
        }

        self.update_state(self.current_dir.clone());
        self.toggle();
        self.mode = Mode::Normal;
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
        let input = input.trim_end_matches('/');
        let mut parts: Vec<&str> = input.split('/').collect();

        if let Some(last) = parts.pop() {
            let mut base_path = self.current_dir.clone();
            for part in parts {
                base_path.push(part);
            }

            if let Err(e) = fs::create_dir_all(&base_path) {
                eprintln!("Error creating parent directories: {e}");
                return;
            }

            base_path.push(last);

            if is_dir {
                self.create_dir(base_path);
            } else {
                self.create_file(base_path);
            }
        }
    }

    fn create_dir(&mut self, path: PathBuf) {
        if let Err(e) = fs::create_dir_all(&path) {
            eprintln!("Error creating directory {:?}: {}", path, e);
        } else {
            self.on_creation_success();
        }
    }

    fn create_file(&mut self, path: PathBuf) {
        if let Err(e) = fs::File::create(&path) {
            eprintln!("Error creating file {:?}: {}", path, e);
        } else {
            self.on_creation_success();
        }
    }

    fn on_creation_success(&mut self) {
        self.update_state(self.current_dir.clone());
        self.temp = "".into();
        self.pop = None;
    }

    fn copy(&mut self) {
        let selected_field = self.get_selected_items();
        self.selected_items = selected_field;
    }

    fn paste(&mut self) {
        let selected_items = self.selected_items.clone();

        for src in selected_items.iter() {
            let dst = self.current_dir.join(src.file_name().unwrap());

            if src.is_file() {
                let _ = fs::copy(src, &dst);
            } else if src.is_dir() {
                self.dir_paste(src, &dst);
            }
        }

        self.update_state(self.current_dir.clone());
    }

    fn dir_paste(&self, src: &PathBuf, dst: &PathBuf) {
        if let Err(e) = fs::create_dir_all(dst) {
            eprintln!("Failed to create directory {:?}: {}", dst, e);
            return;
        }

        if let Ok(entries) = fs::read_dir(src) {
            for entry in entries.flatten() {
                let child_src = entry.path();
                let child_dst = dst.join(entry.file_name());

                if child_src.is_file() {
                    if let Err(e) = fs::copy(&child_src, &child_dst) {
                        eprintln!("Failed to copy file {:?}: {}", child_src, e);
                    }
                } else if child_src.is_dir() {
                    self.dir_paste(&child_src, &child_dst);
                }
            }
        }
    }

    fn get_selected_items(&mut self) -> Vec<PathBuf> {
        let selected_field: Vec<PathBuf> = self
            .current_items
            .iter()
            .filter(|x| x.selected)
            .filter_map(|x| {
                let abs_path = self.current_dir.join(&x.name);
                abs_path.canonicalize().ok()
            })
            .collect();
        selected_field
    }

    fn get_sub_files(&mut self) {
        if let Some(loc) = self.selected_index.selected() {
            if let Some(selected_item) = self.current_items.get(loc) {
                let item_path = self.current_dir.join(&selected_item.name);

                match selected_item.item_type {
                    ItemType::Dir => match utils::list_dir(&item_path) {
                        Ok(sub_files) => self.get_file_update_state(sub_files),
                        Err(e) => eprintln!("Error listing directory {:?}: {}", item_path, e),
                    },
                    ItemType::File => match fs::read_to_string(&item_path) {
                        Ok(content) => self.update_file_state_file(content),
                        Err(_) => match fs::read(&item_path) {
                            Ok(content) => self.update_file_state_binary(content),
                            Err(e) => eprintln!("Error reading file {:?}: {}", item_path, e),
                        },
                    },
                }
            }
        }
    }

    fn select(&mut self) {
        if let Mode::Selection = self.mode {
            if let Some(loc) = self.selected_index.selected() {
                if let Some(selected_item) = self.current_items.get_mut(loc) {
                    selected_item.selected = true;
                }
            }
        }
    }

    fn unselect(&mut self) {
        if let Mode::Normal = self.mode {
            for i in 0..self.current_items.len() {
                if let Some(selected_item) = self.current_items.get_mut(i) {
                    selected_item.selected = false;
                    self.update_state(self.current_dir.clone());
                }
            }
        }
    }

    fn down(&mut self) {
        self.select();
        self.selected_index.select_next();
        if self.current_items.len() == self.selected_index.selected().unwrap() {
            self.selected_index.select(Some(0));
        }
        self.get_sub_files();
    }

    fn up(&mut self) {
        self.select();
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
