use crate::{FileManager, InteractionMode, PopupType};
use crossterm::event::{self, Event, KeyCode};
use ratatui::DefaultTerminal;
use std::io;
use std::time::Duration;
impl FileManager {
    pub fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        let poll_interval = Duration::from_millis(200);
        loop {
            terminal.draw(|f| self.render(f))?;
            if event::poll(poll_interval)? {
                if let Event::Key(key) = event::read()? {
                    if let PopupType::Confirm = self.popup.clone() {
                        match key.code {
                            KeyCode::Char('n') => self.toggle_confirmation_popup(),
                            KeyCode::Char('y') => match self.mode {
                                InteractionMode::Normal => self.delete_selected(),
                                InteractionMode::MultiSelect => self.delete_multiple(),
                            },
                            _ => {}
                        }
                        continue;
                    }

                    if let PopupType::Rename = self.popup.clone() {
                        match key.code {
                            KeyCode::Char(input) => {
                                self.input_buffer.push(input);
                            }
                            // Append character to input
                            KeyCode::Backspace => {
                                self.input_buffer.pop();
                            } // Remove last character
                            KeyCode::Enter => {
                                self.rename_selected(&mut self.input_buffer.clone());
                            }
                            KeyCode::Esc => self.popup = PopupType::None,
                            _ => {}
                        }
                        continue;
                    }

                    if let PopupType::Create = self.popup.clone() {
                        match key.code {
                            KeyCode::Char(c) => {
                                self.input_buffer.push(c);
                            }
                            // Append character to input
                            KeyCode::Backspace => {
                                self.input_buffer.pop();
                            } // Remove last character
                            KeyCode::Enter => {
                                self.create_entry(self.input_buffer.clone());
                            }
                            KeyCode::Esc => self.popup = PopupType::None,
                            _ => {}
                        }
                        continue;
                    }
                    if let InteractionMode::Normal = self.mode {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('j') => self.navigate_down(),
                            KeyCode::Char('k') => self.navigate_up(),
                            KeyCode::Char('h') => self.navigate_to_parent(),
                            KeyCode::Char('l') => self.navigate_to_child(),
                            KeyCode::Char('d') => self.toggle_confirmation_popup(),
                            KeyCode::Char('r') => self.popup = PopupType::Rename,
                            KeyCode::Char('a') => self.popup = PopupType::Create,
                            KeyCode::Char('y') => self.copy_selected_entries(),
                            KeyCode::Char('x') => self.move_selected_entries(),
                            KeyCode::Char('p') => self.paste_clipboard(),
                            KeyCode::Esc => self.deselect_all(),
                            KeyCode::Char('v') => {
                                self.mode = InteractionMode::MultiSelect;
                                if let InteractionMode::MultiSelect = self.mode {
                                    if let Some(current_selection) = self.selection.selected() {
                                        if let Some(selected_item) =
                                            self.entries.get_mut(current_selection)
                                        {
                                            selected_item.is_selected = true;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if let InteractionMode::MultiSelect = self.mode {
                        match key.code {
                            KeyCode::Char('j') => self.navigate_down(),
                            KeyCode::Char('k') => self.navigate_up(),
                            KeyCode::Char('d') => self.toggle_confirmation_popup(),
                            KeyCode::Char('q') => break,
                            KeyCode::Esc => self.mode = InteractionMode::Normal,
                            _ => {}
                        }
                    }
                }
            } else {
                self.clear_expired_notifications();
            }
        }
        Ok(())
    }
}
