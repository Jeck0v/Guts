use crossterm::event::{self, Event, KeyCode};

pub fn handle_events() -> Option<KeyCode> {
    if event::poll(std::time::Duration::from_millis(100)).ok()? {
        if let Event::Key(key_event) = event::read().ok()? {
            return Some(key_event.code);
        }
    }
    None
}
