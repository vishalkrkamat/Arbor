use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::*;
use ratatui::{DefaultTerminal, Frame};
use std::io;
fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    loop {
        terminal.draw(render)?;
        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Char('d') => break,
                _ => {}
            },

            _ => {}
        }
    }
    Ok(())
}

fn render(f: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.area());

    f.render_widget("hello", layout[0]);
    f.render_widget("ok see", layout[1]);
}
