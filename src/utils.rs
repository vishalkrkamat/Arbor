use crate::{FsEntry, FsEntryType};
use mime_guess::Mime;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::ListItem,
};
use std::collections::VecDeque;

use std::{
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};
use tokio::{fs, io};

pub async fn list_dir(p: &PathBuf) -> tokio::io::Result<Vec<FsEntry>> {
    let mut rd = fs::read_dir(p).await?;
    let mut items = Vec::new();

    while let Some(entry) = rd.next_entry().await? {
        let file_path = entry.path();
        let meta = fs::symlink_metadata(&file_path).await?;
        let file_size = meta.size();
        let permission: u32 = meta.mode();
        let file_type = if meta.is_dir() {
            FsEntryType::Directory
        } else if meta.file_type().is_symlink() {
            FsEntryType::Symlink
        } else {
            FsEntryType::File
        };
        let mimetype = get_mime(&file_path).await;
        let item = FsEntry::new(
            entry.file_name().into_string().unwrap(),
            file_path,
            file_type,
            file_size,
            permission,
            false,
            mimetype,
        );

        items.push(item);
    }
    Ok(items)
}

pub async fn copy_dir_iterative(src: &Path, dst: &Path) -> io::Result<()> {
    let mut todo = VecDeque::new();
    todo.push_back((src.to_path_buf(), dst.to_path_buf()));

    while let Some((cur_src, cur_dst)) = todo.pop_back() {
        fs::create_dir_all(&cur_dst).await?; // create directory

        let mut dir = fs::read_dir(&cur_src).await?;
        while let Some(entry) = dir.next_entry().await? {
            let src_path = entry.path();
            let dst_path = cur_dst.join(entry.file_name());
            let ty = entry.file_type().await?;

            if ty.is_file() {
                fs::copy(&src_path, &dst_path).await?;
            } else if ty.is_dir() {
                todo.push_back((src_path, dst_path));
            }
        }
    }
    Ok(())
}

pub async fn move_file(src: &Path, dst: &Path) -> io::Result<()> {
    let result = copy_dir_iterative(src, dst).await;
    if result.is_ok() {
        if src.is_file() {
            fs::remove_file(src).await?
        } else {
            fs::remove_dir_all(src).await?
        }
    }
    result
}

pub async fn get_mime(src: &Path) -> Option<Mime> {
    mime_guess::from_path(src).first()
}

pub async fn read_valid_file(path: &PathBuf) -> io::Result<String> {
    if fs::metadata(path).await?.len() == 0 {
        Ok("Empty File".to_string())
    } else {
        fs::read_to_string(path).await
    }
}

pub fn mode_to_string(mode: u32) -> String {
    let mut result = String::new();

    // Each tuple is (bitmask, char to use if bit is set)
    let flags = [
        (0o400, 'r'),
        (0o200, 'w'),
        (0o100, 'x'), // user
        (0o040, 'r'),
        (0o020, 'w'),
        (0o010, 'x'), // group
        (0o004, 'r'),
        (0o002, 'w'),
        (0o001, 'x'), // others
    ];

    for (bit, ch) in flags {
        if mode & bit != 0 {
            result.push(ch);
        } else {
            result.push('-');
        }
    }

    result
}

pub async fn get_state_data(
    start: &PathBuf,
) -> tokio::io::Result<(Vec<FsEntry>, Option<PathBuf>, Vec<FsEntry>)> {
    let entries = list_dir(start).await?;
    let parent_path = start.parent().map(|p| p.to_path_buf());
    let parent_entries = if let Some(ref p) = parent_path {
        list_dir(p).await?
    } else {
        Vec::new()
    };
    Ok((entries, parent_path, parent_entries))
}

pub fn convert_to_listitems(f: &[FsEntry]) -> Vec<ListItem> {
    let list_items: Vec<ListItem> = f
        .iter()
        .map(|item| {
            let display = match item.entry_type() {
                FsEntryType::Directory => format!("📁 {}", item.name()),
                FsEntryType::File => format!("📄 {}", item.name()),
                FsEntryType::Symlink => format!("🔗{}", item.name()),
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

pub fn format_size(size: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let size_f = size as f64;

    if size_f >= GB {
        format!("{:.2} GB", size_f / GB)
    } else if size_f >= MB {
        format!("{:.2} MB", size_f / MB)
    } else if size_f >= KB {
        format!("{:.2} KB", size_f / KB)
    } else {
        format!("{size} B")
    }
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
