mod event_handler;
use std::time::{Duration, Instant};
mod ui;
mod utils;

use ratatui::widgets::ListState;
use std::fs;
use std::path::PathBuf;
use utils::get_state_data;

#[derive(Debug, Clone, PartialEq)]
pub enum FsEntryType {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FsEntry {
    name: String,
    entry_type: FsEntryType,
    is_selected: bool,
}

#[derive(Debug, Clone)]
pub enum FileContent {
    Text(String),
    Binary(Vec<u8>),
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

#[derive(Debug)]
pub struct FileManager {
    parent_view: ParentView,
    current_path: PathBuf,
    entries: Vec<FsEntry>,
    preview: PreviewContent,
    selection: ListState,
    mode: InteractionMode,
    notify: Option<Notification>,
    clipboard: Vec<PathBuf>,
    input_buffer: String,
    popup: Option<PopupType>,
}

#[derive(Debug)]
pub struct ParentView {
    entries: Vec<FsEntry>,
    path: Option<PathBuf>,
    selection: ListState,
}

impl FileManager {
    fn new(start_path: &PathBuf) -> Self {
        let (entries, parent_path, parent_entries) = get_state_data(start_path);

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
            clipboard: vec![],
            input_buffer: String::new(),
            popup: None,
        };

        state.refresh_preview();
        state.update_parent_selection();
        state
    }

    fn refresh_current_directory(&mut self, new_path: PathBuf) {
        let (entries, parent_path, parent_entries) = get_state_data(&new_path);
        self.current_path = new_path;
        self.entries = entries;
        self.parent_view.path = parent_path;
        self.parent_view.entries = parent_entries;
    }

    fn refresh_preview_with_directory(&mut self, items: Vec<FsEntry>) {
        self.preview = PreviewContent::Directory(items);
        self.update_parent_selection();
    }

    fn refresh_preview_with_text_file(&mut self, content: String) {
        self.preview = PreviewContent::File(FileContent::Text(content));
        self.update_parent_selection();
    }

    fn refresh_preview_with_binary_file(&mut self, content: Vec<u8>) {
        self.preview = PreviewContent::File(FileContent::Binary(content));
    }

    fn delete_selected(&mut self) {
        if let Some(index) = self.selection.selected() {
            if let Some(entry) = self.entries.get(index) {
                let path = self.current_path.join(&entry.name);
                let result = match entry.entry_type {
                    FsEntryType::File => fs::remove_file(&path),
                    FsEntryType::Directory => fs::remove_dir_all(&path),
                };

                if result.is_ok() {
                    self.popup = None;
                    self.refresh_current_directory(self.current_path.clone());
                } else if let Err(err) = result {
                    eprintln!("Failed to delete {:?}: {}", path, err);
                }
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
        if let Some(index) = self.selection.selected() {
            if let Some(entry) = self.entries.get(index) {
                let old_path = self.current_path.join(&entry.name);
                let new_path = self.current_path.join(input.trim_end_matches('/'));

                if fs::rename(&old_path, &new_path).is_ok() {
                    self.refresh_current_directory(self.current_path.clone());
                    self.input_buffer.clear();
                    self.popup = None;
                }
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
                self.show_notification("hell".to_string());
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
        self.input_buffer.clear();
        self.popup = None;
    }

    fn copy_selected(&mut self) {
        self.clipboard = self.get_selected_paths();
    }

    fn paste_clipboard(&mut self) {
        let clipboard = self.clipboard.clone();
        for src in clipboard {
            let dst = self.current_path.join(src.file_name().unwrap());
            if src.is_file() {
                let _ = fs::copy(src, &dst);
            } else if src.is_dir() {
                self.recursively_copy_dir(&src, &dst);
            }
        }
        self.refresh_current_directory(self.current_path.clone());
    }

    fn recursively_copy_dir(&mut self, src: &PathBuf, dst: &PathBuf) {
        if let Err(e) = fs::create_dir_all(dst) {
            self.show_notification(format!("Error creating dir {:?}: {}", dst, e));
            return;
        }

        if let Ok(entries) = fs::read_dir(src) {
            for entry in entries.flatten() {
                let src_path = entry.path();
                let dst_path = dst.join(entry.file_name());

                if src_path.is_file() {
                    if let Err(e) = fs::copy(&src_path, &dst_path) {
                        eprintln!("Failed to copy file {:?}: {}", src_path, e);
                    }
                } else if src_path.is_dir() {
                    self.recursively_copy_dir(&src_path, &dst_path);
                }
            }
        }
    }

    fn get_selected_paths(&self) -> Vec<PathBuf> {
        self.entries
            .iter()
            .filter(|entry| entry.is_selected)
            .filter_map(|entry| self.current_path.join(&entry.name).canonicalize().ok())
            .collect()
    }

    fn refresh_preview(&mut self) {
        if let Some(index) = self.selection.selected() {
            if let Some(entry) = self.entries.get(index) {
                let path = self.current_path.join(&entry.name);
                match entry.entry_type {
                    FsEntryType::Directory => match utils::list_dir(&path) {
                        Ok(items) => self.refresh_preview_with_directory(items),
                        Err(e) => self.show_notification(format!("{}", e)),
                    },
                    FsEntryType::File => match fs::read_to_string(&path) {
                        Ok(text) => self.refresh_preview_with_text_file(text),
                        Err(_) => match fs::read(&path) {
                            Ok(bytes) => self.refresh_preview_with_binary_file(bytes),
                            Err(e) => self.show_notification(format!("{}", e)),
                        },
                    },
                }
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
        self.select_current();
        self.selection.select_next();
        if self.selection.selected().unwrap_or(0) >= self.entries.len() {
            self.selection.select(Some(0));
        }
        self.refresh_preview();
    }

    fn navigate_up(&mut self) {
        self.select_current();
        let len = self.entries.len();
        if self.selection.selected().unwrap_or(0) == 0 {
            self.selection.select(Some(len));
        }
        self.selection.select_previous();
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
        if let Some(index) = self.selection.selected() {
            if let Some(entry) = self.entries.get(index) {
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
    }

    fn toggle_confirmation_popup(&mut self) {
        self.popup = match self.popup {
            Some(PopupType::Confirm) => None,
            _ => Some(PopupType::Confirm),
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
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();

    let start_dir = PathBuf::from(".");
    let absolute_path = start_dir.canonicalize().expect("Failed to resolve path");

    let exit_result = FileManager::new(&absolute_path).run(terminal);

    ratatui::restore();
    exit_result
}
