use crate::modals::{FileManager, FsEntryType, InteractionMode, PopupType};
use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;
use zip::ZipArchive;

impl FileManager {
    pub async fn create_entry(&mut self, input: String) {
        let is_directory = input.ends_with('/');
        let trimmed_input = input.trim_end_matches('/');
        let mut segments: Vec<&str> = trimmed_input.split('/').collect();

        if let Some(name) = segments.pop() {
            let mut path = self.current_path().clone();
            for segment in segments {
                path.push(segment);
            }

            if let Err(e) = fs::create_dir_all(&path).await {
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
        match fs::create_dir_all(&path).await {
            Ok(_) => self.on_create_success().await,
            Err(e) => self.show_notification(e.to_string()),
        }
    }

    async fn create_file(&mut self, path: PathBuf) {
        match fs::File::create(&path).await {
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

    pub async fn delete_selected(&mut self) {
        if let Some(entry) = self.get_selected_index_entry() {
            let path = entry.entry_path().clone();
            let result = match entry.entry_type() {
                FsEntryType::File => fs::remove_file(&path).await,
                FsEntryType::Directory => fs::remove_dir_all(&path).await,
                FsEntryType::Symlink => fs::remove_file(&path).await,
            };

            if result.is_ok() {
                self.set_popup(PopupType::None);
                self.refresh_current_directory(self.current_path().clone())
                    .await;
                self.refresh_preview().await;
            } else if let Err(err) = result {
                self.set_notify(format!("Failed to delete {path:?}: {err}"));
            }
        }
    }

    pub async fn delete_multiple(&mut self) {
        for path in self.get_selected_paths() {
            let _ = if path.is_file() {
                fs::remove_file(path).await
            } else {
                fs::remove_dir_all(path).await
            };
        }

        self.refresh_current_directory(self.current_path().clone())
            .await;
        self.toggle_confirmation_popup();
        self.set_mode(InteractionMode::Normal);
    }

    pub async fn rename_selected(&mut self, input: &mut str) {
        if let Some(entry) = self.get_selected_index_entry() {
            let old_path = &entry.entry_path().clone();
            let new_path = self.current_path().join(input.trim_end_matches('/'));

            if fs::rename(old_path, &new_path).await.is_ok() {
                self.refresh_current_directory(self.current_path().clone())
                    .await;
                self.mut_input_buffer().clear();
                self.set_popup(PopupType::None);
            }
        }
    }

    pub async fn operation(&mut self) -> Result<()> {
        let current_dir = self.current_path().clone();
        if let Some(entry) = self.get_selected_index_entry() {
            let filepath = entry.entry_path().to_owned();

            if entry
                .mime_type()
                .clone()
                .is_some_and(|mime| mime.subtype() == "zip")
            {
                let result = tokio::task::spawn_blocking(move || {
                    let file = std::fs::File::open(&filepath)?;
                    let mut archive = ZipArchive::new(file)?;
                    archive.extract(current_dir)?;
                    Ok::<_, zip::result::ZipError>(())
                })
                .await;

                match result {
                    Ok(inner) => match inner {
                        Ok(_) => {
                            self.refresh_current_directory(self.current_path().clone())
                                .await;
                            self.show_notification("✅ Zip extracted".to_string());
                        }
                        Err(e) => {
                            self.show_notification(format!("❌ Zip error: {e}"));
                        }
                    },
                    // task itself failed (panicked or cancelled)
                    Err(e) => {
                        self.show_notification(format!("❌ Task failed: {e}"));
                    }
                }
            }
        }

        Ok(())
    }
}
