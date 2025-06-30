use crate::{FileManager, InteractionMode, PopupType};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::DefaultTerminal;
use tokio::time::{interval, Duration};

impl FileManager {
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
        let mut tick = interval(Duration::from_millis(16));

        loop {
            tick.tick().await;

            terminal.draw(|f| self.render(f))?;

            while event::poll(Duration::from_millis(0))? {
                let evt = event::read()?;
                if let Event::Key(key) = evt {
                    if self.process_key(key).await? {
                        return Ok(());
                    }
                }
            }

            self.clear_expired_notifications();
        }
    }

    async fn process_key(&mut self, key: KeyEvent) -> std::io::Result<bool> {
        match self.popup {
            PopupType::Confirm => {
                match key.code {
                    KeyCode::Char('n') => self.toggle_confirmation_popup(),
                    KeyCode::Char('y') => match self.mode {
                        InteractionMode::Normal => self.delete_selected().await,
                        InteractionMode::MultiSelect => self.delete_multiple().await,
                    },
                    _ => {}
                }
                return Ok(false);
            }
            PopupType::Rename => {
                match key.code {
                    KeyCode::Char(c) => self.input_buffer.push(c),
                    KeyCode::Backspace => {
                        self.input_buffer.pop();
                    }
                    KeyCode::Enter => self.rename_selected(&mut self.input_buffer.clone()).await,
                    KeyCode::Esc => self.popup = PopupType::None,
                    _ => {}
                }
                return Ok(false);
            }
            PopupType::Create => {
                match key.code {
                    KeyCode::Char(c) => self.input_buffer.push(c),
                    KeyCode::Backspace => {
                        self.input_buffer.pop();
                    }
                    KeyCode::Enter => self.create_entry(self.input_buffer.clone()).await,
                    KeyCode::Esc => self.popup = PopupType::None,
                    _ => {}
                }
                return Ok(false);
            }
            PopupType::None => {}
        }

        match self.mode {
            InteractionMode::Normal => {
                use KeyCode::*;
                match key.code {
                    Char('q') => return Ok(true), // ← quit
                    Char('j') => self.navigate_down().await,
                    Char('k') => self.navigate_up().await,
                    Char('h') => self.navigate_to_parent().await,
                    Char('l') => self.navigate_to_child().await,
                    Char('d') => self.toggle_confirmation_popup(),
                    Char('r') if !self.entries.is_empty() => self.popup = PopupType::Rename,
                    Char('a') => self.popup = PopupType::Create,
                    Char('y') => self.copy_selected_entries().await,
                    Char('x') => self.move_selected_entries().await,
                    Char('p') => self.paste_clipboard().await,
                    Esc => self.deselect_all().await,
                    Char('v') => {
                        self.mode = InteractionMode::MultiSelect;
                        if let Some(idx) = self.selection.selected() {
                            if let Some(item) = self.entries.get_mut(idx) {
                                item.is_selected = true;
                            }
                        }
                    }
                    _ => {}
                }
            }

            InteractionMode::MultiSelect => {
                use KeyCode::*;
                match key.code {
                    Char('q') => return Ok(true),
                    Char('j') => self.navigate_down().await,
                    Char('k') => self.navigate_up().await,
                    Char('d') => self.toggle_confirmation_popup(),
                    Esc => self.mode = InteractionMode::Normal,
                    _ => {}
                }
            }
        }

        Ok(false)
    }
}

