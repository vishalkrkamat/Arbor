use crate::utils::{convert_to_listitems, popup_area};
use crate::InteractionMode;
use crate::PopupType;
use crate::PreviewContent;
use crate::{FileContent, FileManager};
use ratatui::prelude::*;
use ratatui::{
    layout::{Constraint, Flex},
    widgets::{Block, BorderType::Rounded, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
impl FileManager {
    pub fn render(&mut self, f: &mut Frame) {
        let selection_state = &mut self.selection;
        let parent_files = &self.parent_view.entries;
        let current_entries = &self.entries;
        let list_current_items: Vec<ListItem> = convert_to_listitems(current_entries).unwrap();

        let list_parent_items: Vec<ListItem> = convert_to_listitems(parent_files).unwrap();
        let current_directory = self.current_path.to_string_lossy();
        let block = Block::bordered().border_type(Rounded).borders(Borders::ALL);

        let main_layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.area());

        let entry_lists = List::new(list_current_items)
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black))
            .add_modifier(Modifier::BOLD)
            .highlight_symbol(">>")
            .block(Block::bordered().border_type(Rounded).borders(Borders::ALL));
        let list_parent_files = List::new(list_parent_items).block(block.clone());

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(20),
                Constraint::Percentage(50),
                Constraint::Percentage(30),
            ])
            .split(main_layout[1]);

        match &self.preview {
            PreviewContent::Directory(sub_files) => {
                let list_sub_items: Vec<ListItem> = convert_to_listitems(sub_files).unwrap();
                f.render_widget(Clear, layout[2]);
                f.render_widget(&block, layout[2]);

                let preview_directory_list = List::new(list_sub_items);
                let inner_area = block.inner(layout[2]);
                f.render_widget(preview_directory_list, inner_area);
            }
            PreviewContent::File(FileContent::Text(data)) => {
                f.render_widget(Clear, layout[2]);
                f.render_widget(block.clone(), layout[2]);

                let preview_file_content_txt = Paragraph::new(String::from(data))
                    .wrap(Wrap { trim: true })
                    .block(Block::default());

                let inner_area = block.inner(layout[2]);
                f.render_widget(preview_file_content_txt, inner_area);
            }
            PreviewContent::File(FileContent::Binary(data)) => {
                f.render_widget(Clear, layout[2]);
                f.render_widget(&block, layout[2]);
                let preview_file_content_binary = Paragraph::new(hex::encode(data))
                    .wrap(Wrap { trim: true })
                    .block(Block::default());
                let inner_area = block.inner(layout[2]);
                f.render_widget(preview_file_content_binary, inner_area);
            }
        }

        f.render_widget(current_directory.to_string(), main_layout[0]);
        f.render_widget(list_parent_files, layout[0]);

        if entry_lists.is_empty() {
            let par = Paragraph::new("No files")
                .alignment(Alignment::Center)
                .block(Block::bordered().border_type(Rounded).borders(Borders::ALL));
            f.render_widget(par, layout[1]);
        } else {
            f.render_stateful_widget(entry_lists, layout[1], selection_state);
        }

        if let Some(PopupType::Confirm) = self.popup.clone() {
            let mut confirm_file_list = Paragraph::new("").wrap(Wrap { trim: false }); // placeholder

            match self.mode {
                InteractionMode::Normal => {
                    if let Some(index) = self.selection.selected() {
                        if let Some(file) = self.entries.get(index) {
                            let name = file.name.clone();
                            let path = self.current_path.join(name).to_string_lossy().to_string();

                            confirm_file_list = Paragraph::new(path)
                                .alignment(Alignment::Left)
                                .wrap(Wrap { trim: false });
                        }
                    }
                }

                InteractionMode::MultiSelect => {
                    let selected_field = self.get_selected_paths();
                    let mut text = vec![Line::from("")];
                    for file in selected_field {
                        text.push(Line::from(file.to_string_lossy().to_string()));
                    }

                    confirm_file_list = Paragraph::new(text)
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

            let popup_layout = Layout::default()
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
                .split(popup_layout[0]);

            let separator = Paragraph::new(Span::styled(
                "─".repeat(popup_layout[1].width as usize),
                Style::default().fg(Color::LightBlue),
            ));

            let seperator_layout = Layout::horizontal([Constraint::Percentage(95)])
                .flex(Flex::Center)
                .split(popup_layout[1]);

            let options = Paragraph::new("Yes(Y)")
                .block(Block::default().borders(Borders::NONE))
                .alignment(ratatui::layout::Alignment::Center);
            let options1 = Paragraph::new("No(N)")
                .block(Block::default().borders(Borders::NONE))
                .alignment(ratatui::layout::Alignment::Center);

            let section2 = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(popup_layout[2]);

            f.render_widget(Clear, area);
            f.render_widget(block, area);
            f.render_widget(confirm_file_list, sub_section[0]);
            f.render_widget(separator, seperator_layout[0]);
            f.render_widget(options, section2[0]);
            f.render_widget(options1, section2[1]);
        }

        if let Some(PopupType::Rename) = self.popup.clone() {
            let input = self.input_buffer.clone();
            let input_paragraph = Paragraph::new(input.clone()).block(
                Block::bordered()
                    .border_type(Rounded)
                    .title("Rename")
                    .blue(),
            );

            let area = popup_area(f.area(), 30, 20);

            f.render_widget(Clear, area);
            f.render_widget(input_paragraph, area);
        }

        if let Some(PopupType::Create) = self.popup.clone() {
            let input = self.input_buffer.clone();
            let input_paragraph = Paragraph::new(input.clone()).block(
                Block::bordered()
                    .border_type(Rounded)
                    .title("Create:")
                    .blue(),
            );

            let area = popup_area(f.area(), 30, 10);

            f.render_widget(Clear, area);
            f.render_widget(input_paragraph, area);
        }
    }
}
