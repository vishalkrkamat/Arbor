use crate::utils::{convert_to_listitems, popup_area};
use crate::FileManagerState;
use crate::FileType;
use crate::Mode;
use crate::PopUI;
use crate::Preview;
use ratatui::prelude::*;
use ratatui::{
    layout::{Constraint, Flex},
    widgets::{Block, BorderType::Rounded, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
impl FileManagerState {
    pub fn render(&mut self, f: &mut Frame) {
        let ustate = &mut self.selected_index;
        let parent_files = &self.parent.parent_items;
        let current_files = &self.current_items;
        let list_current_items: Vec<ListItem> = convert_to_listitems(current_files).unwrap();

        let list_parent_items: Vec<ListItem> = convert_to_listitems(parent_files).unwrap();
        let current_directory = self.current_dir.to_string_lossy();

        let mainlay = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.area());

        let list = List::new(list_current_items)
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black))
            .add_modifier(Modifier::BOLD)
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
                let list_sub_items: Vec<ListItem> = convert_to_listitems(sub_files).unwrap();

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
            let mut list_of_file = Paragraph::new("").wrap(Wrap { trim: false }); // placeholder

            match self.mode {
                Mode::Normal => {
                    if let Some(loc) = self.selected_index.selected() {
                        if let Some(file) = self.current_items.get(loc) {
                            let name = file.name.clone();
                            let path = self.current_dir.join(name).to_string_lossy().to_string();

                            list_of_file = Paragraph::new(path)
                                .alignment(Alignment::Left)
                                .wrap(Wrap { trim: false });
                        }
                    }
                }

                Mode::Selection => {
                    let selected_field = self.get_selected_items();
                    let mut text = vec![Line::from("")];
                    for file in selected_field {
                        text.push(Line::from(file.to_string_lossy().to_string()));
                    }

                    list_of_file = Paragraph::new(text)
                        .alignment(Alignment::Left)
                        .wrap(Wrap { trim: false });
                }
            };

            let block = Block::bordered()
                .border_type(Rounded)
                .title("Confirm your action")
                .blue();
            let area = popup_area(f.area(), 37, 40);

            let inner_area = block.inner(area);

            let section = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Percentage(90),
                    Constraint::Length(1),
                    Constraint::Percentage(10),
                ])
                .split(inner_area);

            let sub_section = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(100)])
                .split(section[0]);

            let separator = Paragraph::new(Span::styled(
                "─".repeat(section[1].width as usize),
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
            f.render_widget(list_of_file, sub_section[0]);
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

        if let Some(PopUI::Creation) = self.pop.clone() {
            let input = self.temp.clone();
            let inputp = Paragraph::new(input.clone()).block(
                Block::bordered()
                    .border_type(Rounded)
                    .title("Create:")
                    .blue(),
            );

            let area = popup_area(f.area(), 30, 10);

            f.render_widget(Clear, area);
            f.render_widget(inputp, area);
        }
    }
}
