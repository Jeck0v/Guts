use crate::terminal::{app::App, ui};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

pub fn run_app() -> Result<()> {
    // setup TUI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    let res = run_app_loop(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

// Editor gestion
fn run_app_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key_event(key)?;

                if let Some(cmd) = app.last_executed_command.take() {
                    if cmd.starts_with("nano") || cmd.starts_with("vim") || cmd.starts_with("vi") {
                        app.handle_editor_command(terminal, &cmd)?;

                        // restores TUI
                        enable_raw_mode()?;
                        execute!(
                                io::stdout(),
                                EnterAlternateScreen,
                                EnableMouseCapture
                        )?;
                        let backend = CrosstermBackend::new(io::stdout());
                        *terminal = Terminal::new(backend)?;
                        terminal.clear()?;

                        //  Reset input state
                        app.input.clear();
                        app.cursor_position = 0;
                        app.force_redraw = true;

                        continue;
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
