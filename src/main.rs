mod event_handler;
use std::time::{Duration, Instant};
mod ui;
mod utils;
use ratatui::widgets::ListState;
use std::{fs, path::PathBuf, sync::mpsc, thread};
use utils::{get_state_data, move_file, recursively_copy_dir};

#[derive(Debug, Clone, PartialEq)]
pub enum FsEntryType {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FsEntry {
    name: String,
    entry_type: FsEntryType,
    size: u64,
    file_permission: u32,
    is_selected: bool,
}

#[derive(Debug, Clone)]
pub enum FileContent {
    Text(String),
    Binary(String),
}

#[derive(Debug, Clone)]
pub enum PreviewContent {
    File(FileContent),
    Directory(Vec<FsEntry>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PopupType {
    Confirm,
    Rename,
    Create,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InteractionMode {
    Normal,
    MultiSelect,
}

#[derive(Debug, Clone)]
pub struct Notification {
    message: String,
    created_at: Instant,
    duration: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Move,
    Copy,
    None,
}

#[derive(Debug, Clone)]
pub struct Clipboard {
    paths: Vec<PathBuf>,
    action: Action,
}

#[derive(Debug)]
pub struct FileManager {
    parent_view: ParentView,
    current_path: PathBuf,
    entries: Vec<FsEntry>,
    preview: PreviewContent,
    selection: ListState,
    mode: InteractionMode,
    notify: Option<Notification>,
    clipboard: Clipboard,
    input_buffer: String,
    popup: PopupType,
}

#[derive(Debug)]
pub struct ParentView {
    entries: Vec<FsEntry>,
    path: Option<PathBuf>,
    selection: ListState,
}

impl FileManager {
    fn new(start_path: &PathBuf) -> Result<Self, std::io::Error> {
        let (entries, parent_path, parent_entries) = get_state_data(start_path).unwrap();

        let mut state = Self {
            parent_view: ParentView {
                path: parent_path,
                entries: parent_entries,
                selection: ListState::default(),
            },
            current_path: start_path.clone(),
            entries,
            preview: PreviewContent::Directory(vec![]),
            selection: ListState::default().with_selected(Some(0)),
            mode: InteractionMode::Normal,
            notify: None,
            clipboard: Clipboard {
                paths: vec![],
                action: Action::None,
            },
            input_buffer: String::new(),
            popup: PopupType::None,
        };

        state.refresh_preview();
        state.update_parent_selection();
        Ok(state)
    }

    fn refresh_current_directory(&mut self, new_path: PathBuf) {
        match get_state_data(&new_path) {
            Ok((entries, parent_path, parent_entries)) => {
                self.current_path = new_path;
                self.entries = entries;
                self.parent_view.path = parent_path;
                self.parent_view.entries = parent_entries;
            }
            Err(e) => self.show_notification(e.to_string()),
        }
    }

    fn refresh_preview_with_directory(&mut self, items: Vec<FsEntry>) {
        self.preview = PreviewContent::Directory(items);
        self.update_parent_selection();
    }

    fn refresh_preview_with_text_file(&mut self, content: String) {
        self.preview = PreviewContent::File(FileContent::Text(content));
        self.update_parent_selection();
    }

    fn refresh_preview_with_binary_file(&mut self, content: String) {
        self.preview = PreviewContent::File(FileContent::Binary(content));
    }

    fn delete_selected(&mut self) {
        if let Some(entry) = self.get_selected_index_entry() {
            let path = self.current_path.join(&entry.name);
            let result = match entry.entry_type {
                FsEntryType::File => fs::remove_file(&path),
                FsEntryType::Directory => fs::remove_dir_all(&path),
            };

            if result.is_ok() {
                self.popup = PopupType::None;
                self.refresh_current_directory(self.current_path.clone());
                self.refresh_preview();
            } else if let Err(err) = result {
                self.show_notification(format!("Failed to delete {:?}: {}", path, err));
            }
        }
    }

    fn delete_multiple(&mut self) {
        for path in self.get_selected_paths() {
            let _ = if path.is_file() {
                fs::remove_file(path)
            } else {
                fs::remove_dir_all(path)
            };
        }

        self.refresh_current_directory(self.current_path.clone());
        self.toggle_confirmation_popup();
        self.mode = InteractionMode::Normal;
    }

    fn rename_selected(&mut self, input: &mut String) {
        if let Some(entry) = self.get_selected_index_entry() {
            let old_path = self.current_path.join(&entry.name);
            let new_path = self.current_path.join(input.trim_end_matches('/'));

            if fs::rename(&old_path, &new_path).is_ok() {
                self.refresh_current_directory(self.current_path.clone());
                self.input_buffer.clear();
                self.popup = PopupType::None;
            }
        }
    }

    fn create_entry(&mut self, input: String) {
        let is_directory = input.ends_with('/');
        let trimmed_input = input.trim_end_matches('/');
        let mut segments: Vec<&str> = trimmed_input.split('/').collect();

        if let Some(name) = segments.pop() {
            let mut path = self.current_path.clone();
            for segment in segments {
                path.push(segment);
            }

            if let Err(e) = fs::create_dir_all(&path) {
                self.show_notification(format!("Error creating directories: {e}"));
                return;
            }

            path.push(name);

            if is_directory {
                self.create_directory(path);
            } else {
                self.create_file(path);
            }
        }
    }

    fn create_directory(&mut self, path: PathBuf) {
        match fs::create_dir_all(&path) {
            Ok(_) => self.on_create_success(),
            Err(e) => self.show_notification(e.to_string()),
        }
    }

    fn create_file(&mut self, path: PathBuf) {
        match fs::File::create(&path) {
            Ok(_) => self.on_create_success(),
            Err(e) => self.show_notification(e.to_string()),
        }
    }

    fn on_create_success(&mut self) {
        self.refresh_current_directory(self.current_path.clone());
        self.refresh_preview();
        self.input_buffer.clear();
        self.popup = PopupType::None;
    }

    fn copy_selected_entries(&mut self) {
        self.clipboard.action = Action::Copy;
        self.clipboard.paths = self.get_selected_paths();
    }
    fn move_selected_entries(&mut self) {
        self.clipboard.action = Action::Move;
        self.clipboard.paths = self.get_selected_paths();
    }

    fn paste_clipboard(&mut self) {
        let clipboard = self.clipboard.clone();
        for src in clipboard.paths {
            let dst = self.current_path.join(src.file_name().unwrap());
            if src.is_file() {
                match self.clipboard.action {
                    Action::Move => {
                        if fs::copy(&src, &dst).is_ok() {
                            if let Err(e) = fs::remove_file(&src) {
                                self.show_notification(e.to_string())
                            }
                        };
                    }
                    Action::Copy => {
                        if let Err(e) = fs::copy(src, &dst) {
                            self.show_notification(e.to_string())
                        }
                    }
                    _ => {}
                }
            } else if src.is_dir() {
                match self.clipboard.action {
                    Action::Move => {
                        if let Err(e) = move_file(&src, &dst) {
                            self.show_notification(e.to_string())
                        }
                    }
                    Action::Copy => {
                        if let Err(e) = recursively_copy_dir(&src, &dst) {
                            self.show_notification(e.to_string())
                        }
                    }
                    _ => {}
                }
            }
        }
        self.refresh_current_directory(self.current_path.clone());
        self.clipboard.action = Action::None
    }

    fn get_selected_paths(&self) -> Vec<PathBuf> {
        self.entries
            .iter()
            .filter(|entry| entry.is_selected)
            .filter_map(|entry| self.current_path.join(&entry.name).canonicalize().ok())
            .collect()
    }

    fn refresh_preview(&mut self) {
        if let Some(entry) = self.get_selected_index_entry() {
            let path = self.current_path.join(&entry.name);
            match entry.entry_type {
                FsEntryType::Directory => {
                    let path_clone = path.clone();
                    let (tx, rx) = mpsc::channel();

                    thread::spawn(move || {
                        let result = utils::list_dir(&path_clone);
                        let _ = tx.send(result);
                    });

                    if let Ok(result) = rx.recv() {
                        match result {
                            Ok(items) => self.refresh_preview_with_directory(items),
                            Err(e) => self.show_notification(e.to_string()),
                        }
                    }
                }

                FsEntryType::File => match utils::read_valid_file(&path) {
                    Ok(text) => self.refresh_preview_with_text_file(text),
                    Err(e) => self.refresh_preview_with_binary_file(e.to_string()),
                },
            }
        }
    }

    fn select_current(&mut self) {
        if let InteractionMode::MultiSelect = self.mode {
            if let Some(index) = self.selection.selected() {
                if let Some(entry) = self.entries.get_mut(index) {
                    entry.is_selected = true;
                }
            }
        }
    }

    fn deselect_all(&mut self) {
        if let InteractionMode::Normal = self.mode {
            for entry in &mut self.entries {
                entry.is_selected = false;
            }
            self.refresh_current_directory(self.current_path.clone());
        }
    }

    fn navigate_down(&mut self) {
        self.selection.select_next();
        if self.selection.selected().unwrap_or(0) >= self.entries.len() {
            self.selection.select(Some(0));
        }
        self.select_current();
        self.refresh_preview();
    }

    fn navigate_up(&mut self) {
        let len = self.entries.len();
        if self.selection.selected().unwrap_or(0) == 0 {
            self.selection.select(Some(len));
        }
        self.selection.select_previous();
        self.select_current();
        self.refresh_preview();
    }

    fn update_parent_selection(&mut self) {
        if let Some(current_name) = self.current_path.file_name().map(|n| n.to_string_lossy()) {
            if let Some(index) = self
                .parent_view
                .entries
                .iter()
                .position(|entry| entry.name == current_name)
            {
                self.parent_view.selection = ListState::default().with_selected(Some(index));
            }
        }
    }

    fn navigate_to_parent(&mut self) {
        if let Some(ref parent_path) = self.parent_view.path {
            self.refresh_current_directory(parent_path.clone());
            self.selection = self.parent_view.selection.clone();
            self.refresh_preview();
        }
    }

    fn navigate_to_child(&mut self) {
        if let Some(entry) = self.get_selected_index_entry() {
            if let FsEntryType::Directory = entry.entry_type {
                let mut path = self.current_path.clone();
                path.push(&entry.name);
                self.refresh_current_directory(path);
                self.parent_view.selection = self.selection.clone();
                self.selection = ListState::default().with_selected(Some(0));
                self.refresh_preview();
            }
        }
    }

    fn toggle_confirmation_popup(&mut self) {
        self.popup = match self.popup {
            PopupType::Confirm => PopupType::None,
            _ => PopupType::Confirm,
        };
    }

    fn show_notification(&mut self, message: String) {
        self.notify = Some(Notification {
            message,
            created_at: Instant::now(),
            duration: Duration::from_secs(3),
        });
    }

    fn clear_expired_notifications(&mut self) {
        if let Some(noti) = &self.notify {
            if noti.created_at.elapsed() >= noti.duration {
                self.notify = None;
            }
        }
    }

    fn get_selected_index_entry(&self) -> Option<&FsEntry> {
        self.selection
            .selected()
            .and_then(|index| self.entries.get(index))
    }
}

fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();

    let start_dir = PathBuf::from(".");
    let absolute_path = start_dir.canonicalize().expect("Failed to resolve path");

    let exit_result = FileManager::new(&absolute_path).unwrap().run(terminal);

    ratatui::restore();
    exit_result
}
