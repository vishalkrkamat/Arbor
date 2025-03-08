use crate::ItemType;
use crate::ListsItem;
use std::fs;
use std::path::PathBuf;

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
        };
        items.push(item);
    }
    Ok(items)
}
