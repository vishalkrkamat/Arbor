use crate::ItemType;
use crate::ListsItem;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::ListItem;
use std::path::PathBuf;
use std::{fs, io};

pub fn list_dir(p: &PathBuf) -> std::io::Result<Vec<ListsItem>> {
    let mut items = Vec::new();
    for entry in fs::read_dir(p)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        let file_type = if meta.is_dir() {
            ItemType::Dir
        } else {
            ItemType::File
        };
        let item = ListsItem {
            name: entry.file_name().into_string().unwrap(),
            item_type: file_type,
            selected: false,
        };
        items.push(item);
    }
    Ok(items)
}
pub fn get_state_data(start: &PathBuf) -> (Vec<ListsItem>, Option<PathBuf>, Vec<ListsItem>) {
    let files = list_dir(start).unwrap();
    let parent_dir = start.parent().map(|p| p.to_path_buf());
    let parent_items = parent_dir
        .as_ref()
        .map_or_else(Vec::new, |p| list_dir(p).unwrap());
    (files, parent_dir, parent_items)
}

pub fn convert_to_listitems(f: &[ListsItem]) -> io::Result<Vec<ListItem>> {
    let list_items: Vec<ListItem> = f
        .iter()
        .map(|item| {
            let display = match item.item_type {
                ItemType::Dir => format!("📁 {}", item.name),
                ItemType::File => format!("📄 {}", item.name),
            };
            let mut style = Style::default();
            if item.selected {
                style = style.bg(Color::DarkGray);
            } else {
                style = Style::default();
            }
            ListItem::new(Span::styled(display, style))
        })
        .collect();
    Ok(list_items)
}

//POPUp UI constructor
pub fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
