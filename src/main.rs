mod event_handler;
mod ui;
mod utils;
use ratatui::widgets::ListState;
use std::{fs, path::PathBuf};
use utils::{copy_dir_iterative, get_state_data, move_file};
mod modals;
use crate::modals::{
    Action, FileContent, FileManager, FsEntry, FsEntryType, InteractionMode, PopupType,
    PreviewContent,
};

impl FileManager {
    async fn refresh_current_directory(&mut self, new_path: PathBuf) {
        match get_state_data(&new_path).await {
            Ok((entries, parent_path, parent_entries)) => {
                self.set_current_path(new_path);
                self.set_entries(entries);
                self.parent_view_mut().set_path(parent_path);
                self.parent_view_mut().set_entries(parent_entries);
            }
            Err(e) => self.show_notification(e.to_string()),
        }
    }

    async fn refresh_preview_with_directory(&mut self, items: Vec<FsEntry>) {
        self.set_preview(PreviewContent::Directory(items));
        self.update_parent_selection();
    }

    async fn refresh_preview_with_text_file(&mut self, content: String) {
        self.set_preview(PreviewContent::File(FileContent::Text(content)));
        self.update_parent_selection();
    }

    fn refresh_preview_with_binary_file(&mut self, content: String) {
        self.set_preview(PreviewContent::File(FileContent::Binary(content)));
    }

    async fn delete_selected(&mut self) {
        if let Some(entry) = self.get_selected_index_entry() {
            let path = entry.entry_path().clone();
            let result = match entry.entry_type() {
                FsEntryType::File => fs::remove_file(&path),
                FsEntryType::Directory => fs::remove_dir_all(&path),
            };

            if result.is_ok() {
                self.set_popup(PopupType::None);
                self.refresh_current_directory(self.current_path().clone())
                    .await;
                self.refresh_preview().await;
            } else if let Err(err) = result {
                self.set_notify(format!("Failed to delete {:?}: {}", path, err));
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

        self.refresh_current_directory(self.current_path().clone())
            .await;
        self.toggle_confirmation_popup();
        self.set_mode(InteractionMode::Normal);
    }

    async fn rename_selected(&mut self, input: &mut str) {
        if let Some(entry) = self.get_selected_index_entry() {
            let old_path = &entry.entry_path().clone();
            let new_path = self.current_path().join(input.trim_end_matches('/'));

            if fs::rename(&old_path, &new_path).is_ok() {
                self.refresh_current_directory(self.current_path().clone())
                    .await;
                self.mut_input_buffer().clear();
                self.set_popup(PopupType::None);
            }
        }
    }

    async fn create_entry(&mut self, input: String) {
        let is_directory = input.ends_with('/');
        let trimmed_input = input.trim_end_matches('/');
        let mut segments: Vec<&str> = trimmed_input.split('/').collect();

        if let Some(name) = segments.pop() {
            let mut path = self.current_path().clone();
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
        self.refresh_current_directory(self.current_path().clone())
            .await;
        self.refresh_preview().await;
        self.mut_input_buffer().clear();
        self.set_popup(PopupType::None);
    }

    fn set_clipboard_entries(&mut self) {
        if *self.mode() == InteractionMode::Normal
            && !self.entries().iter().any(|entry| entry.is_selected)
        {
            if let Some(current_selection) = self.selection_mut().selected() {
                if let Some(selected_item) = self.entries_mut().get_mut(current_selection) {
                    selected_item.is_selected = true;
                }
            }

            if let Some(entry) = self.get_selected_index_entry_unmut() {
                //self.clipboard.paths = vec![entry.entry_path().clone()];
                let entry = entry.entry_path().clone();
                self.clipboard_mut().set_clipboard_paths(vec![entry]);
            }
        } else {
            let paths = self.get_selected_paths();
            //self.clipboard().paths() = self.get_selected_paths();
            self.clipboard_mut().set_clipboard_paths(paths);
        }
    }

    async fn copy_selected_entries(&mut self) {
        self.set_clipboard_actions(Action::Copy);
        self.set_clipboard_entries();
    }

    async fn move_selected_entries(&mut self) {
        self.set_clipboard_actions(Action::Move);
        self.set_clipboard_entries();
    }

    async fn paste_clipboard(&mut self) {
        let clipboard = self.clipboard().clone();
        for src in clipboard.get_path() {
            let dst = self.current_path().join(src.file_name().unwrap());
            if src.is_file() {
                match self.clipboard_actions() {
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
                match self.clipboard_actions() {
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
        self.refresh_current_directory(self.current_path().clone())
            .await;
        self.set_clipboard_actions(Action::None)
    }

    fn get_selected_paths(&self) -> Vec<PathBuf> {
        self.entries()
            .iter()
            .filter_map(|entry| entry.is_selected.then(|| entry.entry_path().clone()))
            .collect()
    }

    async fn refresh_preview(&mut self) {
        if let Some(entry) = self.get_selected_index_entry() {
            let path = entry.entry_path().clone();
            match entry.entry_type() {
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
        if let InteractionMode::MultiSelect = self.mode() {
            if let Some(index) = self.selection_mut().selected() {
                if let Some(entry) = self.entries_mut().get_mut(index) {
                    entry.is_selected = true;
                }
            }
        }
    }

    async fn deselect_all(&mut self) {
        if let InteractionMode::Normal = self.mode() {
            for entry in self.entries_mut() {
                entry.is_selected = false;
            }
            self.refresh_current_directory(self.current_path().clone())
                .await;
        }
    }

    async fn navigate_down(&mut self) {
        self.selection_mut().select_next();
        if self.selection().selected().unwrap_or(0) >= self.entries().len() {
            self.selection_mut().select(Some(0));
        }
        self.select_current().await;
        self.refresh_preview().await;
    }

    async fn navigate_up(&mut self) {
        let len = self.entries().len();
        if self.selection().selected().unwrap_or(0) == 0 {
            self.selection_mut().select(Some(len));
        }
        self.selection_mut().select_previous();
        self.select_current().await;
        self.refresh_preview().await;
    }

    fn update_parent_selection(&mut self) {
        if let Some(current_name) = self.current_path().file_name().map(|n| n.to_string_lossy()) {
            if let Some(index) = self
                .parent_view()
                .entries()
                .iter()
                .position(|entry| entry.name() == current_name)
            {
                //self.parent_view().selection_mut = ListState::default().with_selected(Some(index));
                self.parent_view_mut()
                    .set_selection(ListState::default().with_selected(Some(index)));
            }
        }
    }

    async fn navigate_to_parent(&mut self) {
        if let Some(ref parent_path) = self.parent_view().path() {
            self.refresh_current_directory(parent_path.clone()).await;
            //self.selection = self.parent_view().selection().clone();
            self.set_selection(self.parent_view().selection().clone());
            self.refresh_preview().await;
        }
    }

    async fn navigate_to_child(&mut self) {
        let selection = self.selection().clone();
        if let Some(entry) = self.get_selected_index_entry_unmut() {
            if let FsEntryType::Directory = entry.entry_type() {
                let mut path = self.current_path().clone();
                path.push(&entry.name());
                self.refresh_current_directory(path).await;
                //self.parent_view().selection = self.selection().clone();
                self.parent_view_mut().set_selection(selection);
                //self.selection = ListState::default().with_selected(Some(0));
                self.set_selection(ListState::default().with_selected(Some(0)));
                self.refresh_preview().await;
            }
        }
    }

    fn toggle_confirmation_popup(&mut self) {
        if !self.entries().is_empty() {
            match self.popup() {
                PopupType::Confirm => self.set_popup(PopupType::None),
                _ => self.set_popup(PopupType::Confirm),
            };
        }
    }

    fn show_notification(&mut self, message: String) {
        self.set_notify(message);
    }

    fn clear_expired_notifications(&mut self) {
        if let Some(notify) = self.notify() {
            if notify.created_at().elapsed() >= notify.duration() {
                self.clear_notify()
            }
        }
    }

    fn get_selected_index_entry(&mut self) -> Option<&FsEntry> {
        self.selection()
            .selected()
            .and_then(|index| self.entries().get(index))
    }

    fn get_selected_index_entry_unmut(&self) -> Option<&FsEntry> {
        self.selection()
            .selected()
            .and_then(|index| self.entries().get(index).clone())
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
