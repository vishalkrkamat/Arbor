use crate::get_state_data;
use ratatui::widgets::ListState;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum FsEntryType {
    File,
    Directory,
    Symlink,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FsEntry {
    name: String,
    entry_path: PathBuf,
    entry_type: FsEntryType,
    size: u64,
    file_permission: u32,
    pub is_selected: bool,
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

#[derive(Clone, Debug)]
pub struct Clipboard {
    paths: Vec<PathBuf>,
    action: Action,
}
#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct ParentView {
    entries: Vec<FsEntry>,
    path: Option<PathBuf>,
    selection: ListState,
}

impl FileManager {
    pub async fn new(start_path: &PathBuf) -> Result<Self, std::io::Error> {
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

    pub fn entries(&self) -> &[FsEntry] {
        &self.entries
    }
    pub fn set_entries(&mut self, entry: Vec<FsEntry>) {
        self.entries = entry
    }

    pub fn entries_mut(&mut self) -> &mut Vec<FsEntry> {
        &mut self.entries
    }

    pub fn selection(&self) -> &ListState {
        &self.selection
    }

    pub fn selection_mut(&mut self) -> &mut ListState {
        &mut self.selection
    }

    pub fn set_selection(&mut self, state: ListState) {
        self.selection = state
    }

    pub fn set_preview(&mut self, preview: PreviewContent) {
        self.preview = preview;
    }

    pub fn current_path(&self) -> &PathBuf {
        &self.current_path
    }
    pub fn set_current_path(&mut self, path: PathBuf) {
        self.current_path = path;
    }

    pub fn parent_view(&self) -> &ParentView {
        &self.parent_view
    }

    pub fn parent_view_mut(&mut self) -> &mut ParentView {
        &mut self.parent_view
    }

    pub fn mode(&self) -> &InteractionMode {
        &self.mode
    }
    pub fn set_mode(&mut self, mode: InteractionMode) {
        self.mode = mode
    }

    pub fn popup(&mut self) -> &PopupType {
        &self.popup
    }

    pub fn set_popup(&mut self, action: PopupType) {
        self.popup = action;
    }

    pub fn parent_view_entries(&self) -> &[FsEntry] {
        &self.parent_view.entries
    }

    pub fn clipboard(&self) -> &Clipboard {
        &self.clipboard
    }
    pub fn clipboard_mut(&mut self) -> &mut Clipboard {
        &mut self.clipboard
    }

    pub fn clipboard_actions(&self) -> &Action {
        &self.clipboard.action
    }
    pub fn set_clipboard_actions(&mut self, action: Action) {
        self.clipboard.action = action
    }

    pub fn mut_input_buffer(&mut self) -> &mut String {
        &mut self.input_buffer
    }

    pub fn input_buffer(&mut self) -> &String {
        &self.input_buffer
    }

    pub fn notify(&self) -> Option<&Notification> {
        self.notify.as_ref()
    }

    pub fn set_notify(&mut self, message: String) {
        self.notify = Some(Notification {
            message,
            created_at: Instant::now(),
            duration: Duration::from_secs(3),
        });
    }

    pub fn clear_notify(&mut self) {
        self.notify = None;
    }

    pub fn preview_mut(&self) -> &PreviewContent {
        &self.preview
    }
}

impl FsEntry {
    // pub fn is_selected(&self) -> bool {
    //     self.is_selected
    // }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn entry_type(&self) -> &FsEntryType {
        &self.entry_type
    }
    pub fn entry_path(&self) -> &PathBuf {
        &self.entry_path
    }
    pub fn file_permission(&self) -> u32 {
        self.file_permission
    }
    pub fn size(&self) -> u64 {
        self.size
    }
}

impl Notification {
    pub fn message(&self) -> &str {
        &self.message
    }
}
impl ParentView {
    pub fn path(&self) -> &Option<PathBuf> {
        &self.path
    }
    pub fn set_path(&mut self, path: Option<PathBuf>) {
        self.path = path
    }
    pub fn entries(&self) -> &Vec<FsEntry> {
        &self.entries
    }
    pub fn set_entries(&mut self, entries: Vec<FsEntry>) {
        self.entries = entries
    }

    pub fn selection(&self) -> &ListState {
        &self.selection
    }

    pub fn set_selection(&mut self, state: ListState) {
        self.selection = state
    }

    // pub fn selection_mut(&mut self) -> &mut ListState {
    //     &mut self.selection
    // }
}
impl FsEntry {
    pub fn new(
        name: String,
        entry_path: PathBuf,
        entry_type: FsEntryType,
        size: u64,
        file_permission: u32,
        is_selected: bool,
    ) -> Self {
        Self {
            name,
            entry_path,
            entry_type,
            size,
            file_permission,
            is_selected,
        }
    }
}
impl Notification {
    pub fn created_at(&self) -> Instant {
        self.created_at
    }
    pub fn duration(&self) -> Duration {
        self.duration
    }
}
impl Clipboard {
    pub fn get_path(&self) -> &Vec<PathBuf> {
        &self.paths
    }
    pub fn set_clipboard_paths(&mut self, path: Vec<PathBuf>) {
        self.paths = path
    }
}
