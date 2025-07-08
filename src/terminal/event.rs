use crossterm::event::{self, Event, KeyCode};
use crate::terminal::app::App;
use std::time::Duration;

pub fn handle_event(app: &mut App) -> anyhow::Result<bool> {
    if event::poll(Duration::from_millis(200))? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Right => app.next_tab(),
                KeyCode::Left => app.prev_tab(),
                KeyCode::Char('q') => return Ok(true),
                _ => {}
            }
        }
    }
    Ok(false)
}
