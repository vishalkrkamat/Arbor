mod utils;
use crossterm::event::{self, Event, KeyCode};
use hex;
use ratatui::prelude::*;
use ratatui::{
    layout::{Constraint, Flex, Rect},
    widgets::{
        Block, BorderType::Rounded, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
    },
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
enum FileType {
    Text(String),
    Byes(Vec<u8>),
}
#[derive(Debug, Clone)]
enum Preview {
    Files(FileType),
    Directory(Vec<ListsItem>),
}

#[derive(Debug, Clone, PartialEq)]
enum PopUI {
    Confirmation,
    RenameUI,
}

#[derive(Debug)]
struct FileManagerState {
    parent_items: Vec<ListsItem>,
    parent_dir: Option<PathBuf>,   // The parent's dir
    current_dir: PathBuf,          // The path currently in
    current_items: Vec<ListsItem>, // Items in the current directory
    child_items: Preview,          // Items in the selected subdirectory
    selected_index: ListState,     // Which item in current_items is selected
    temp: String,
    pop: Option<PopUI>,
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
            pop: None,
            temp: "".to_string(),
        }
    }

    fn get_state_data(start: &PathBuf) -> (Vec<ListsItem>, Option<PathBuf>, Vec<ListsItem>) {
        let files = utils::list_dir(start).unwrap();
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
        self.child_items = Preview::Files(FileType::Text(con));
    }

    fn update_file_state_binary(&mut self, con: Vec<u8>) {
        self.child_items = Preview::Files(FileType::Byes(con));
    }

    fn delete(&mut self) {
        if let Some(loc) = self.selected_index.selected() {
            if let Some(file) = self.current_items.get(loc) {
                let name = file.name.clone();
                let path = self.current_dir.join(name);

                match file.item_type {
                    ItemType::File => {
                        if fs::remove_file(path).is_ok() {
                            self.pop = None;
                        };
                        self.update_state(&self.current_dir.clone());
                    }
                    ItemType::Dir => {
                        if fs::remove_dir_all(path).is_ok() {
                            self.pop = None;
                        };
                        self.update_state(&self.current_dir.clone());
                    }
                }
            }
        }
    }

    fn rename(&mut self, input: String) {
        if let Some(ind) = self.selected_index.selected() {
            if let Some(sel) = self.current_items.get(ind) {
                let filename = &sel.name;
                if fs::rename(filename, input).is_ok() {
                    self.update_state(&self.current_dir.clone());
                    self.pop = None;
                };
            }
        };
    }
    fn create(&mut self) {
        todo!()
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
                                match fs::read_to_string(&current_file) {
                                    Ok(con) => self.update_file_state_file(con),
                                    Err(_) => match fs::read(current_file) {
                                        Ok(con) => self.update_file_state_binary(con),
                                        Err(_e) => eprint!("error"),
                                    },
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
        if self.current_items.len() == self.selected_index.selected().unwrap() {
            self.selected_index.select(Some(0));
        }
        self.get_sub_files();
    }

    fn up(&mut self) {
        let lastl = self.current_items.len();
        if self.selected_index.selected().unwrap() == 0 {
            self.selected_index.select(Some(lastl));
        }
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
                    ItemType::File => println!(),
                }
            }
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;

            if let Event::Key(key) = event::read()? {
                if let Some(PopUI::Confirmation) = self.pop.clone() {
                    match key.code {
                        KeyCode::Char('n') => self.toggle(),
                        KeyCode::Char('y') => self.delete(),
                        _ => {}
                    }
                    continue;
                }

                if let Some(PopUI::RenameUI) = self.pop.clone() {
                    match key.code {
                        KeyCode::Char(c) => {
                            self.temp.push(c);
                        }
                        // Append character to input
                        KeyCode::Backspace => {
                            self.temp.pop();
                        } // Remove last character
                        KeyCode::Enter => {
                            self.rename(self.temp.clone()); // Call function with input
                        }
                        KeyCode::Esc => self.pop = None, // Exit without processing
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') => self.down(),
                    KeyCode::Char('k') => self.up(),
                    KeyCode::Char('h') => self.previous_dir(),
                    KeyCode::Char('l') => self.next_dir(),
                    KeyCode::Char('d') => self.toggle(),
                    KeyCode::Char('r') => self.pop = Some(PopUI::RenameUI),
                    KeyCode::Char('a') => self.create(),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn toggle(&mut self) {
        if let Some(PopUI::Confirmation) = self.pop.clone() {
            self.pop = None
        } else {
            self.pop = Some(PopUI::Confirmation)
        }
    }

    fn render(&mut self, f: &mut Frame) {
        let ustate = &mut self.selected_index;
        let parent_files = &self.parent_items;
        let current_files = &self.current_items;
        let list_current_items: Vec<ListItem> =
            FileManagerState::convert_to_listitems(current_files).unwrap();

        let list_parent_items: Vec<ListItem> =
            FileManagerState::convert_to_listitems(parent_files).unwrap();
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

        match &self.child_items.clone() {
            Preview::Directory(sub_files) => {
                let list_sub_items: Vec<ListItem> =
                    FileManagerState::convert_to_listitems(sub_files).unwrap();

                let list_child_fiels = List::new(list_sub_items)
                    .block(Block::bordered().border_type(Rounded).borders(Borders::ALL));
                f.render_widget(Clear, layout[2]);
                f.render_widget(list_child_fiels, layout[2]);
            }
            Preview::Files(FileType::Text(con)) => {
                let cont = Paragraph::new(String::from(con))
                    .wrap(Wrap { trim: true })
                    .block(Block::bordered().border_type(Rounded).borders(Borders::ALL));
                f.render_widget(Clear, layout[2]);
                f.render_widget(cont, layout[2]);
            }
            Preview::Files(FileType::Byes(con)) => {
                let cont = Paragraph::new(hex::encode(con))
                    .wrap(Wrap { trim: true })
                    .block(Block::bordered().border_type(Rounded).borders(Borders::ALL));
                f.render_widget(Clear, layout[2]);
                f.render_widget(cont, layout[2]);
            }
        }

        f.render_widget(current_directory.to_string(), mainlay[0]);
        f.render_widget(list_parent_files, layout[0]);
        f.render_stateful_widget(list, layout[1], ustate);

        if let Some(PopUI::Confirmation) = self.pop.clone() {
            let block = Block::bordered()
                .border_type(Rounded)
                .title("Confirm your action")
                .blue();
            let area = popup_area(f.area(), 37, 40);

            let section = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Percentage(90),
                    Constraint::Length(1),
                    Constraint::Percentage(10),
                ])
                .split(area);

            let separator = Paragraph::new(Span::styled(
                "─".repeat(section[1].width as usize), // Line spans full width
                Style::default().fg(Color::LightBlue),
            ));

            let vertical = Layout::horizontal([Constraint::Percentage(95)])
                .flex(Flex::Center)
                .split(section[1]);

            let options = Paragraph::new("Yes(Y)")
                .block(Block::default().borders(Borders::NONE))
                .alignment(ratatui::layout::Alignment::Center);
            let options1 = Paragraph::new("No(N)")
                .block(Block::default().borders(Borders::NONE))
                .alignment(ratatui::layout::Alignment::Center);

            let section2 = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(section[2]);

            f.render_widget(Clear, area);
            f.render_widget(block, area);

            f.render_widget(separator, vertical[0]);
            f.render_widget(options, section2[0]);
            f.render_widget(options1, section2[1]);
        }

        if let Some(PopUI::RenameUI) = self.pop.clone() {
            let input = self.temp.clone();
            let inputp = Paragraph::new(input.clone()).block(
                Block::bordered()
                    .border_type(Rounded)
                    .title("Rename")
                    .blue(),
            );

            let area = popup_area(f.area(), 30, 20);

            f.render_widget(Clear, area);
            f.render_widget(inputp, area);
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
