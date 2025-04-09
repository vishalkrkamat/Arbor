use crate::FileManagerState;
use crate::Mode;
use crate::PopUI;
use crossterm::event::{self, Event, KeyCode};
use ratatui::DefaultTerminal;
use std::io;

impl FileManagerState {
    pub fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;

            if let Event::Key(key) = event::read()? {
                if let Some(PopUI::Confirmation) = self.pop.clone() {
                    match key.code {
                        KeyCode::Char('n') => self.toggle(),
                        KeyCode::Char('y') => match self.mode {
                            Mode::Normal => self.delete(),
                            Mode::Selection => self.mass_deletion(),
                        },
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
                            self.rename(&mut self.temp.clone());
                        }
                        KeyCode::Esc => self.pop = None,
                        _ => {}
                    }
                    continue;
                }

                if let Some(PopUI::Creation) = self.pop.clone() {
                    match key.code {
                        KeyCode::Char(c) => {
                            self.temp.push(c);
                        }
                        // Append character to input
                        KeyCode::Backspace => {
                            self.temp.pop();
                        } // Remove last character
                        KeyCode::Enter => {
                            self.create(self.temp.clone());
                        }
                        KeyCode::Esc => self.pop = None,
                        _ => {}
                    }
                    continue;
                }
                if let Mode::Normal = self.mode {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('j') => self.down(),
                        KeyCode::Char('k') => self.up(),
                        KeyCode::Char('h') => self.previous_dir(),
                        KeyCode::Char('l') => self.next_dir(),
                        KeyCode::Char('d') => self.toggle(),
                        KeyCode::Char('r') => self.pop = Some(PopUI::RenameUI),
                        KeyCode::Char('a') => self.pop = Some(PopUI::Creation),
                        KeyCode::Char('y') => self.copy(),
                        KeyCode::Char('p') => self.paste(),
                        KeyCode::Esc => self.unselect(),
                        KeyCode::Char('v') => {
                            self.mode = Mode::Selection;
                            if let Mode::Selection = self.mode {
                                if let Some(loc) = self.selected_index.selected() {
                                    if let Some(selected_item) = self.current_items.get_mut(loc) {
                                        selected_item.selected = true;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                if let Mode::Selection = self.mode {
                    match key.code {
                        KeyCode::Char('j') => self.down(),
                        KeyCode::Char('k') => self.up(),
                        KeyCode::Char('d') => self.toggle(),
                        KeyCode::Char('q') => break,
                        KeyCode::Esc => self.mode = Mode::Normal,
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}
