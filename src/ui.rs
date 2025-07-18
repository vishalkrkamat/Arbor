use crate::utils::{
    bottom_right_area, convert_to_listitems, format_size, mode_to_string, popup_area,
};
use crate::{
    Action, FileContent, FileManager, FsEntryType, InteractionMode, PopupType, PreviewContent,
};
use ratatui::prelude::*;
use ratatui::{
    layout::{Constraint, Flex},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType::Rounded, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

impl FileManager {
    pub fn render(&mut self, f: &mut Frame) {
        let parent_files = self.parent_view_entries();
        let current_entries = self.entries();
        let clipboard_action = self.clipboard_actions().clone(); // cloned to avoid re-borrowing
        let cursor_index = self.selection().selected(); // read-only first
        let current_path = self.current_path().to_string_lossy(); // Cow<str> clone
                                                                  //

        let list_current_items: Vec<ListItem> = current_entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let (bar, bar_style) = if entry.is_selected {
                    match clipboard_action {
                        Action::Move => ("▌", Style::default().fg(Color::Red)),
                        Action::Copy => ("▌", Style::default().fg(Color::Green)),
                        Action::None => ("▌", Style::default().fg(Color::Yellow)),
                    }
                } else {
                    (" ", Style::default())
                };

                let icon = match entry.entry_type() {
                    FsEntryType::Directory => "📁",
                    FsEntryType::File => "📄",
                };

                let is_cursor_row = cursor_index == Some(index);

                let text = Line::from(vec![
                    Span::styled(bar, bar_style),
                    Span::raw(" "),
                    Span::styled(
                        format!("{} {}", icon, entry.name()),
                        if is_cursor_row {
                            Style::default()
                                .bg(Color::Blue)
                                .fg(Color::Black)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        },
                    ),
                ]);

                ListItem::new(text)
            })
            .collect();

        let list_parent_items: Vec<ListItem> = convert_to_listitems(parent_files);

        let current_directory = Paragraph::new(current_path);
        let block = Block::bordered().border_type(Rounded).borders(Borders::ALL);
        let empty_lists = Paragraph::new("No Files")
            .alignment(Alignment::Center)
            .block(block.clone());

        let main_layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.area());

        let entry_lists = List::new(list_current_items)
            .highlight_style(
                Style::default().bg(Color::Blue), //     .fg(Color::Black)
            )
            .add_modifier(Modifier::BOLD)
            .block(block.clone());
        let list_parent_files = List::new(list_parent_items).block(block.clone());

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(20),
                Constraint::Percentage(50),
                Constraint::Percentage(30),
            ])
            .split(main_layout[1]);

        //let selection_state = self.selection_mut();
        match &self.preview_mut() {
            PreviewContent::Directory(sub_files) => {
                let list_sub_items: Vec<ListItem> = convert_to_listitems(sub_files);

                let preview_directory_list = List::new(list_sub_items);
                let inner_area = block.inner(layout[2]);

                f.render_widget(Clear, layout[2]);
                f.render_widget(&block, layout[2]);

                if preview_directory_list.is_empty() {
                    f.render_widget(&empty_lists, layout[2]);
                } else {
                    f.render_widget(preview_directory_list, inner_area);
                }
            }
            PreviewContent::File(FileContent::Text(data)) => {
                let preview_file_content_txt =
                    Paragraph::new(String::from(data)).wrap(Wrap { trim: true });

                let inner_area = block.inner(layout[2]);

                f.render_widget(Clear, layout[2]);
                f.render_widget(&block, layout[2]);
                f.render_widget(preview_file_content_txt, inner_area);
            }
            PreviewContent::File(FileContent::Binary(data)) => {
                let preview_file_content_binary =
                    Paragraph::new(data.to_string()).wrap(Wrap { trim: true });
                let inner_area = block.inner(layout[2]);

                f.render_widget(Clear, layout[2]);
                f.render_widget(&block, layout[2]);
                f.render_widget(preview_file_content_binary, inner_area);
            }
        }

        f.render_widget(current_directory, main_layout[0]);
        f.render_widget(list_parent_files, layout[0]);

        if entry_lists.is_empty() {
            f.render_widget(&empty_lists, layout[1]);
        } else {
            //let selection_state = self.selection_mut();
            f.render_stateful_widget(entry_lists, layout[1], self.selection_mut());
        }

        if let PopupType::Confirm = &self.popup() {
            let mut confirm_file_list = Paragraph::new("").wrap(Wrap { trim: false });

            match self.mode() {
                InteractionMode::Normal => {
                    if let Some(index) = self.selection().selected() {
                        if let Some(file) = self.entries().get(index) {
                            let name = file.name();
                            let path = self.current_path().join(name).to_string_lossy().to_string();

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

        if let PopupType::Rename = &self.popup() {
            let input = self.input_buffer();
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

        if let PopupType::Create = &self.popup() {
            let input = self.input_buffer();
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

        // Render notification if available
        if let Some(noti) = &self.notify() {
            let area = bottom_right_area(main_layout[1], 35, 5);

            let block = Block::bordered()
                .border_type(Rounded)
                .title("Notification")
                .style(Style::default().fg(Color::Yellow));

            let text = Paragraph::new(noti.message())
                .style(Style::default().fg(Color::Yellow))
                .bg(Color::Black)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
                .block(block);

            f.render_widget(Clear, area);
            f.render_widget(text, area);
        }

        let bottom_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_layout[2]);

        let mut size_display = Span::raw("");

        if let Some(entry) = self.get_selected_index_entry() {
            if *entry.entry_type() == FsEntryType::File {
                let size = format_size(entry.size());
                size_display = Span::styled(
                    format!(" | Size: {}", size),
                    Style::default().fg(Color::LightMagenta),
                );
            }
        }

        let mode_display = match self.mode() {
            InteractionMode::Normal => Span::styled(
                "🔵 Mode: Normal",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            InteractionMode::MultiSelect => Span::styled(
                "🟢 Mode: Multi-Select",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        };

        // Combine mode + size
        let combined_info = Line::from(vec![mode_display, size_display]);

        let mode_paragraph = Paragraph::new(combined_info)
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Left);

        f.render_widget(mode_paragraph, bottom_layout[0]);

        let mut per_display = Span::raw("");
        if let Some(entry) = self.get_selected_index_entry() {
            let permission = entry.file_permission();

            let permisson_str = mode_to_string(permission);
            per_display = Span::styled(
                format!("Permisson: {} ", permisson_str),
                Style::default().fg(Color::LightCyan),
            );
        }

        let per_paragraph = Paragraph::new(per_display)
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Right);

        f.render_widget(per_paragraph, bottom_layout[1]);
    }
}
