use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use super::{app::{App, Tab}, ui::draw};

pub fn run_app() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| draw(f, &app))?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char(c) => app.on_key(c),
                        KeyCode::Backspace => app.backspace(),
                        KeyCode::Right => app.next_tab(),
                        KeyCode::Left => app.prev_tab(),
                        KeyCode::Down => {
                            if matches!(app.tab, Tab::Cli) {
                                app.select_next();
                            }
                        }
                        KeyCode::Up => {
                            if matches!(app.tab, Tab::Cli) {
                                app.select_previous();
                            }
                        }
                        KeyCode::Enter => {
                            if matches!(app.tab, Tab::Cli) {
                                // Soon..
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
