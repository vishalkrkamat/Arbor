use anyhow::Result;
use ratatui::widgets::ListState;
use std::path::{Path, PathBuf};
use tokio::fs;
mod event_handler;
mod file_ops;
mod ui;
mod utils;
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
                        if fs::copy(src, dst).await.is_ok() {
                            if let Err(e) = fs::remove_file(src).await {
                                self.show_notification(e.to_string())
                            }
                        };
                    }
                    Action::Copy => {
                        if let Err(e) = fs::copy(src, &dst).await {
                            self.show_notification(e.to_string())
                        }
                    }
                    _ => {}
                }
            } else if src.is_dir() {
                match self.clipboard_actions() {
                    Action::Move => {
                        if let Err(e) = move_file(src, &dst).await {
                            self.show_notification(e.to_string())
                        }
                    }
                    Action::Copy => {
                        if let Err(e) = copy_dir_iterative(src, &dst).await {
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
            .filter(|&entry| entry.is_selected)
            .map(|entry| entry.entry_path().clone())
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

                FsEntryType::Symlink => {
                    if let Some(target_path) = self.symlink_resolver(&path).await {
                        if target_path.is_dir() {
                            match utils::list_dir(&target_path).await {
                                Ok(items) => self.refresh_preview_with_directory(items).await,
                                Err(e) => self.show_notification(e.to_string()),
                            }
                        } else {
                            match utils::read_valid_file(&target_path).await {
                                Ok(text) => self.refresh_preview_with_text_file(text).await,
                                Err(e) => self.refresh_preview_with_binary_file(e.to_string()),
                            }
                        }
                    } else {
                        self.show_notification("Broken symlink".to_string());
                    }
                }
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

    async fn symlink_resolver(&mut self, symlink_path: &Path) -> Option<PathBuf> {
        match tokio::fs::read_link(symlink_path).await {
            Ok(target) => {
                let symlink_parent = symlink_path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("/"));

                let full_target = if target.is_relative() {
                    symlink_parent.join(target)
                } else {
                    target
                };

                match tokio::fs::canonicalize(&full_target).await {
                    Ok(resolved) => Some(resolved),
                    Err(e) => {
                        self.show_notification(e.to_string());
                        None
                    }
                }
            }
            Err(e) => {
                self.show_notification(e.to_string());
                None
            }
        }
    }

    async fn navigate_to_child(&mut self) {
        let selection = self.selection().clone();
        if let Some(entry) = self.get_selected_index_entry_unmut() {
            if entry.entry_type() == &FsEntryType::Directory {
                let mut path = self.current_path().clone();
                path.push(entry.name());
                self.refresh_current_directory(path).await;
                self.parent_view_mut().set_selection(selection);
                self.set_selection(ListState::default().with_selected(Some(0)));
                self.refresh_preview().await;
            } else if entry.entry_type() == &FsEntryType::Symlink {
                let mut path = self.current_path().clone();
                path.push(entry.name());

                if let Some(target_path) = self.symlink_resolver(&path).await {
                    if target_path.is_dir() {
                        self.refresh_current_directory(target_path).await;
                        self.parent_view_mut().set_selection(selection);
                        self.set_selection(ListState::default().with_selected(Some(0)));
                        self.refresh_preview().await;
                    } else {
                        self.refresh_preview().await;
                    }
                }
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
            .and_then(|index| self.entries().get(index))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let terminal = ratatui::init();

    let start_dir = PathBuf::from(".");
    let absolute_path = start_dir.canonicalize().expect("Failed to resolve path");

    let fm = FileManager::new(&absolute_path).await?;
    let result = fm.run(terminal).await;

    ratatui::restore();
    result
}
