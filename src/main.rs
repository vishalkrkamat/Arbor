mod utils;
use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use ratatui::{
    layout::{Constraint, Flex, Rect},
    widgets::{Block, BorderType::Rounded, Borders, Clear, List, ListItem, ListState},
    DefaultTerminal, Frame,
};
use std::path::PathBuf;
use std::{fs, io};

#[derive(Debug, Clone)]
enum ItemType {
    File,
    Dir,
}

#[derive(Debug, Clone)]
struct ListsItem {
    name: String,
    item_type: ItemType,
}

#[derive(Debug, Clone)]
enum Preview {
    FileContent(String),
    Directory(Vec<ListsItem>),
}

#[derive(Debug)]
struct FileManagerState {
    parent_items: Vec<ListsItem>,
    parent_dir: Option<PathBuf>,   // The parent's dir
    current_dir: PathBuf,          // The path currently in
    current_items: Vec<ListsItem>, // Items in the current directory
    child_items: Preview,          // Items in the selected subdirectory
    selected_index: ListState,     // Which item in current_items is selected
    pop: bool,
}

impl FileManagerState {
    fn new(star_dir: &PathBuf) -> Self {
        let (files, parent_dir, parent_items) = Self::get_state_data(star_dir);
        Self {
            parent_items,
            current_items: files,
            current_dir: star_dir.to_path_buf(),
            parent_dir,
            child_items: Preview::Directory(vec![]),
            selected_index: ListState::default(),
            pop: false,
        }
    }

    fn get_state_data(start: &PathBuf) -> (Vec<ListsItem>, Option<PathBuf>, Vec<ListsItem>) {
        let files = utils::list_dir(&start).unwrap();
        let parent_dir = start.parent().map(|p| p.to_path_buf());
        let parent_items = parent_dir
            .as_ref()
            .map_or_else(Vec::new, |p| utils::list_dir(p).unwrap());
        (files, parent_dir, parent_items)
    }

    fn update_state(&mut self, new_dir: &PathBuf) {
        let (files, parent_dir, parent_items) = Self::get_state_data(new_dir);

        self.current_dir = new_dir.to_path_buf();
        self.current_items = files;
        self.parent_dir = parent_dir;
        self.parent_items = parent_items;
    }

    fn get_file_update_state(&mut self, items: Vec<ListsItem>) {
        self.child_items = Preview::Directory(items);
    }

    fn update_file_state_file(&mut self, con: String) {
        self.child_items = Preview::FileContent(con);
    }

    fn delete(&mut self) {
        if let Some(loc) = self.selected_index.selected() {
            if let Some(file) = self.current_items.get(loc) {
                let name = file.name.clone();
                let path = self.current_dir.join(name);

                match file.item_type {
                    ItemType::File => {
                        fs::remove_file(path).unwrap();
                        //self.pop = true;
                        self.update_state(&self.current_dir.clone());
                    }
                    ItemType::Dir => {
                        fs::remove_dir_all(path).unwrap();
                        //self.pop = true;
                        self.update_state(&self.current_dir.clone());
                    }
                }
            }
        }
    }

    fn convert_to_listitems(f: &Vec<ListsItem>) -> io::Result<Vec<ListItem>> {
        let list_items: Vec<ListItem> = f
            .iter()
            .map(|item| {
                let display = match item.item_type {
                    ItemType::Dir => format!("📁 {}", item.name),
                    ItemType::File => format!("📄 {}", item.name),
                };
                ListItem::new(display)
            })
            .collect();
        Ok(list_items)
    }

    fn get_sub_files(&mut self) {
        if let Some(loc) = self.selected_index.selected() {
            if let Some(selected_dir) = self.current_items.get(loc) {
                let current_dir = &self.current_dir;
                match selected_dir.item_type {
                    ItemType::Dir => {
                        let chilpath = current_dir.join(&selected_dir.name);
                        let sub_files = utils::list_dir(&chilpath).unwrap();
                        self.get_file_update_state(sub_files);
                    }
                    ItemType::File => {
                        if let Some(loc) = self.selected_index.selected() {
                            if let Some(selected_file) = self.current_items.get(loc) {
                                let current_file =
                                    self.current_dir.clone().join(selected_file.name.clone());
                                match fs::read_to_string(current_file) {
                                    Ok(con) => self.update_file_state_file(con),
                                    Err(_e) => eprint!("error"),
                                };
                            }
                        }
                    }
                };
            }
        }
    }

    fn down(&mut self) {
        self.selected_index.select_next();
        self.get_sub_files();
    }

    fn up(&mut self) {
        self.selected_index.select_previous();
        self.get_sub_files();
    }

    fn previous_dir(&mut self) {
        if let Some(parent) = self.parent_dir.clone() {
            self.update_state(&parent);
        }
    }

    fn next_dir(&mut self) {
        if let Some(loc) = self.selected_index.selected() {
            if let Some(selected_file) = self.current_items.get(loc) {
                match &selected_file.item_type {
                    ItemType::Dir => {
                        let mut new_dir = self.current_dir.clone();
                        new_dir.push(&selected_file.name);
                        self.update_state(&new_dir);
                    }
                    ItemType::File => println!(""),
                }
            }
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') => self.down(),
                    KeyCode::Char('k') => self.up(),
                    KeyCode::Char('h') => self.previous_dir(),
                    KeyCode::Char('l') => self.next_dir(),
                    KeyCode::Char('d') => self.delete(),
                    KeyCode::Char('o') => self.toggle(),

                    _ => {}
                }
            }
        }
        Ok(())
    }
    fn toggle(&mut self) {
        if self.pop == false {
            self.pop = true
        } else {
            self.pop = false
        }
    }
    fn render(&mut self, f: &mut Frame) {
        let mut ustate = &mut self.selected_index;
        let parent_files = &self.parent_items;
        let current_files = &self.current_items;
        let list_current_items: Vec<ListItem> =
            FileManagerState::convert_to_listitems(&current_files).unwrap();

        let list_parent_items: Vec<ListItem> =
            FileManagerState::convert_to_listitems(&parent_files).unwrap();
        let current_directory = self.current_dir.to_string_lossy();

        let mainlay = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.area());

        let list = List::new(list_current_items)
            .highlight_symbol(">>")
            .block(Block::bordered().border_type(Rounded).borders(Borders::ALL));
        let list_parent_files = List::new(list_parent_items)
            .block(Block::bordered().border_type(Rounded).borders(Borders::ALL));

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(20),
                Constraint::Percentage(50),
                Constraint::Percentage(30),
            ])
            .split(mainlay[1]);

        if let Preview::Directory(sub_files) = &self.child_items.clone() {
            let list_sub_items: Vec<ListItem> =
                FileManagerState::convert_to_listitems(&sub_files).unwrap();

            let list_child_fiels = List::new(list_sub_items)
                .block(Block::bordered().border_type(Rounded).borders(Borders::ALL));

            f.render_widget(list_child_fiels, layout[2]);
        }

        if let Preview::FileContent(con) = &self.child_items.clone() {
            let cont = Paragraph::new(String::from(con))
                .block(Block::bordered().border_type(Rounded).borders(Borders::ALL));
            f.render_widget(cont, layout[2]);
        }

        f.render_widget(current_directory.to_string(), mainlay[0]);
        f.render_widget(list_parent_files, layout[0]);
        f.render_stateful_widget(list, layout[1], &mut ustate);

        if self.pop {
            let block = Block::bordered()
                .border_type(Rounded)
                .title("Confirm your action")
                .blue();
            let area = popup_area(f.area(), 37, 40);

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(area);

            let conts =
                Paragraph::new(String::from("hello")).block(Block::default().borders(Borders::TOP));
            f.render_widget(Clear, area);
            f.render_widget(block, area);
            f.render_widget(conts, layout[1]);
        }

        //POP up ui
        fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
            let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
            let horizontal =
                Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
            let [area] = vertical.areas(area);
            let [area] = horizontal.areas(area);
            area
        }
    }
}

fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();
    let start_dir = PathBuf::from(".");
    let absolute_path = start_dir.canonicalize().unwrap();
    let appstate = FileManagerState::new(&absolute_path).run(terminal);
    ratatui::restore();
    appstate
}
