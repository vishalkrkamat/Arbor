mod event_handler;
use std::time::{Duration, Instant};
mod ui;
mod utils;
use ratatui::widgets::ListState;
use std::{fs, path::PathBuf};
use utils::{copy_dir_iterative, get_state_data, move_file};

#[derive(Debug, Clone, PartialEq)]
pub enum FsEntryType {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FsEntry {
    name: String,
    entry_path: PathBuf,
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
    async fn new(start_path: &PathBuf) -> Result<Self, std::io::Error> {
        let (entries, parent_path, parent_entries) = get_state_data(start_path).await.unwrap();

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

        state.refresh_preview().await;
        state.update_parent_selection();
        Ok(state)
    }

    async fn refresh_current_directory(&mut self, new_path: PathBuf) {
        match get_state_data(&new_path).await {
            Ok((entries, parent_path, parent_entries)) => {
                self.current_path = new_path;
                self.entries = entries;
                self.parent_view.path = parent_path;
                self.parent_view.entries = parent_entries;
            }
            Err(e) => self.show_notification(e.to_string()),
        }
    }

    async fn refresh_preview_with_directory(&mut self, items: Vec<FsEntry>) {
        self.preview = PreviewContent::Directory(items);
        self.update_parent_selection();
    }

    async fn refresh_preview_with_text_file(&mut self, content: String) {
        self.preview = PreviewContent::File(FileContent::Text(content));
        self.update_parent_selection();
    }

    fn refresh_preview_with_binary_file(&mut self, content: String) {
        self.preview = PreviewContent::File(FileContent::Binary(content));
    }

    async fn delete_selected(&mut self) {
        if let Some(entry) = self.get_selected_index_entry() {
            let path = &entry.entry_path;
            let result = match entry.entry_type {
                FsEntryType::File => fs::remove_file(&path),
                FsEntryType::Directory => fs::remove_dir_all(&path),
            };

            if result.is_ok() {
                self.popup = PopupType::None;
                self.refresh_current_directory(self.current_path.clone())
                    .await;
                self.refresh_preview().await;
            } else if let Err(err) = result {
                self.show_notification(format!("Failed to delete {:?}: {}", path, err));
            }
        }
    }

    async fn delete_multiple(&mut self) {
        for path in self.get_selected_paths() {
            let _ = if path.is_file() {
                fs::remove_file(path)
            } else {
                fs::remove_dir_all(path)
            };
        }

        self.refresh_current_directory(self.current_path.clone())
            .await;
        self.toggle_confirmation_popup();
        self.mode = InteractionMode::Normal;
    }

    async fn rename_selected(&mut self, input: &mut str) {
        if let Some(entry) = self.get_selected_index_entry() {
            let old_path = &entry.entry_path;
            let new_path = self.current_path.join(input.trim_end_matches('/'));

            if fs::rename(&old_path, &new_path).is_ok() {
                self.refresh_current_directory(self.current_path.clone())
                    .await;
                self.input_buffer.clear();
                self.popup = PopupType::None;
            }
        }
    }

    async fn create_entry(&mut self, input: String) {
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
                self.create_directory(path).await;
            } else {
                self.create_file(path).await;
            }
        }
    }

    async fn create_directory(&mut self, path: PathBuf) {
        match fs::create_dir_all(&path) {
            Ok(_) => self.on_create_success().await,
            Err(e) => self.show_notification(e.to_string()),
        }
    }

    async fn create_file(&mut self, path: PathBuf) {
        match fs::File::create(&path) {
            Ok(_) => self.on_create_success().await,
            Err(e) => self.show_notification(e.to_string()),
        }
    }

    async fn on_create_success(&mut self) {
        self.refresh_current_directory(self.current_path.clone())
            .await;
        self.refresh_preview().await;
        self.input_buffer.clear();
        self.popup = PopupType::None;
    }

    fn set_clipboard_entries(&mut self) {
        if self.mode == InteractionMode::Normal
            && !self.entries.iter().any(|entry| entry.is_selected)
        {
            if let Some(current_selection) = self.selection.selected() {
                if let Some(selected_item) = self.entries.get_mut(current_selection) {
                    selected_item.is_selected = true;
                }
            }

            if let Some(entry) = self.get_selected_index_entry() {
                self.clipboard.paths = vec![entry.entry_path.clone()];
            }
        } else {
            self.clipboard.paths = self.get_selected_paths();
        }
    }

    async fn copy_selected_entries(&mut self) {
        self.clipboard.action = Action::Copy;
        self.set_clipboard_entries();
    }

    async fn move_selected_entries(&mut self) {
        self.clipboard.action = Action::Move;
        self.set_clipboard_entries();
    }

    async fn paste_clipboard(&mut self) {
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
                        if let Err(e) = move_file(&src, &dst).await {
                            self.show_notification(e.to_string())
                        }
                    }
                    Action::Copy => {
                        if let Err(e) = copy_dir_iterative(&src, &dst).await {
                            self.show_notification(e.to_string())
                        }
                    }
                    _ => {}
                }
            }
        }
        self.refresh_current_directory(self.current_path.clone())
            .await;
        self.clipboard.action = Action::None
    }

    fn get_selected_paths(&self) -> Vec<PathBuf> {
        self.entries
            .iter()
            .filter_map(|entry| entry.is_selected.then(|| entry.entry_path.clone()))
            .collect()
    }

    async fn refresh_preview(&mut self) {
        if let Some(entry) = self.get_selected_index_entry() {
            let path = entry.entry_path.clone();
            match entry.entry_type {
                FsEntryType::Directory => match utils::list_dir(&path).await {
                    Ok(items) => self.refresh_preview_with_directory(items).await,
                    Err(e) => self.show_notification(e.to_string()),
                },

                FsEntryType::File => match utils::read_valid_file(&path).await {
                    Ok(text) => self.refresh_preview_with_text_file(text).await,
                    Err(e) => self.refresh_preview_with_binary_file(e.to_string()),
                },
            }
        }
    }

    async fn select_current(&mut self) {
        if let InteractionMode::MultiSelect = self.mode {
            if let Some(index) = self.selection.selected() {
                if let Some(entry) = self.entries.get_mut(index) {
                    entry.is_selected = true;
                }
            }
        }
    }

    async fn deselect_all(&mut self) {
        if let InteractionMode::Normal = self.mode {
            for entry in &mut self.entries {
                entry.is_selected = false;
            }
            self.refresh_current_directory(self.current_path.clone())
                .await;
        }
    }

    async fn navigate_down(&mut self) {
        self.selection.select_next();
        if self.selection.selected().unwrap_or(0) >= self.entries.len() {
            self.selection.select(Some(0));
        }
        self.select_current().await;
        self.refresh_preview().await;
    }

    async fn navigate_up(&mut self) {
        let len = self.entries.len();
        if self.selection.selected().unwrap_or(0) == 0 {
            self.selection.select(Some(len));
        }
        self.selection.select_previous();
        self.select_current().await;
        self.refresh_preview().await;
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

    async fn navigate_to_parent(&mut self) {
        if let Some(ref parent_path) = self.parent_view.path {
            self.refresh_current_directory(parent_path.clone()).await;
            self.selection = self.parent_view.selection.clone();
            self.refresh_preview().await;
        }
    }

    async fn navigate_to_child(&mut self) {
        if let Some(entry) = self.get_selected_index_entry() {
            if let FsEntryType::Directory = entry.entry_type {
                let mut path = self.current_path.clone();
                path.push(&entry.name);
                self.refresh_current_directory(path).await;
                self.parent_view.selection = self.selection.clone();
                self.selection = ListState::default().with_selected(Some(0));
                self.refresh_preview().await;
            }
        }
    }

    fn toggle_confirmation_popup(&mut self) {
        if !self.entries.is_empty() {
            self.popup = match self.popup {
                PopupType::Confirm => PopupType::None,
                _ => PopupType::Confirm,
            };
        }
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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();

    let start_dir = PathBuf::from(".");
    let absolute_path = start_dir.canonicalize().expect("Failed to resolve path");

    let fm = FileManager::new(&absolute_path).await?;
    let result = fm.run(terminal).await;

    ratatui::restore();
    result
}
