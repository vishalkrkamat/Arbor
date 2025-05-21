use crate::FsEntry;
use crate::FsEntryType;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::ListItem;
use std::path::PathBuf;
use std::{fs, io};

pub fn list_dir(p: &PathBuf) -> std::io::Result<Vec<FsEntry>> {
    let mut items = Vec::new();
    for entry in fs::read_dir(p)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        let file_type = if meta.is_dir() {
            FsEntryType::Directory
        } else {
            FsEntryType::File
        };
        let item = FsEntry {
            name: entry.file_name().into_string().unwrap(),
            entry_type: file_type,
            is_selected: false,
        };
        items.push(item);
    }
    Ok(items)
}

pub fn recursively_copy_dir(src: &PathBuf, dst: &PathBuf) -> io::Result<()> {
    fs::create_dir_all(dst)?;

    if let Ok(entries) = fs::read_dir(src) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_file() {
                fs::copy(&src_path, &dst_path)?;
            } else if src_path.is_dir() {
                recursively_copy_dir(&src_path, &dst_path)?;
            }
        }
    }
    Ok(())
}

pub fn move_file(src: &PathBuf, dst: &PathBuf) -> io::Result<()> {
    let result = recursively_copy_dir(src, dst);
    if result.is_ok() {
        if src.is_file() {
            fs::remove_file(src)?
        } else {
            fs::remove_dir_all(src)?
        }
    }
    result
}

pub fn read_valid_file(path: &PathBuf) -> io::Result<String> {
    if fs::metadata(path)?.len() == 0 {
        Ok("Empty File".to_string())
    } else {
        fs::read_to_string(path)
    }
}

pub fn get_state_data(
    start: &PathBuf,
) -> std::io::Result<(Vec<FsEntry>, Option<PathBuf>, Vec<FsEntry>)> {
    let entries = list_dir(start)?;
    let parent_path = start.parent().map(|p| p.to_path_buf());
    let parent_entries = parent_path
        .as_ref()
        .map_or_else(Vec::new, |p| list_dir(p).unwrap());
    Ok((entries, parent_path, parent_entries))
}

pub fn convert_to_listitems(f: &[FsEntry]) -> Vec<ListItem> {
    let list_items: Vec<ListItem> = f
        .iter()
        .map(|item| {
            let display = match item.entry_type {
                FsEntryType::Directory => format!("📁 {}", item.name),
                FsEntryType::File => format!("📄 {}", item.name),
            };
            let mut style = Style::default();
            if item.is_selected {
                style = style.bg(Color::DarkGray);
            } else {
                style = Style::default();
            }
            ListItem::new(Span::styled(display, style))
        })
        .collect();
    list_items
}

//PopUp UI constructor
pub fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

pub fn bottom_right_area(area: Rect, width: u16, height: u16) -> Rect {
    let x = area.x + area.width.saturating_sub(width);
    let y = area.y + area.height.saturating_sub(height);
    Rect::new(x, y, width, height)
}
