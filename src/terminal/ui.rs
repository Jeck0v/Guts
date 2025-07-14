use crate::terminal::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(f.size());

    // left panel - ASCII Art
    render_ascii_art(f, chunks[0]);

    // right panel - CLI Interface
    render_cli_interface(f, chunks[1], app);
}

fn render_ascii_art(f: &mut Frame, area: Rect) {
    let ascii_art = r#"
         ██████╗  ██╗   ██╗████████╗ ███████╗
        ██╔════╝ ██║   ██║╚══██╔══╝██╔════╝
        ██║  ███╗██║   ██║   ██║   ███████╗
        ██║   ██║██║   ██║   ██║   ╚════██║
        ╚██████╔╝╚██████╔╝   ██║   ███████║
         ╚═════╝  ╚═════╝    ╚═╝   ╚══════╝


    ╔══════════════════════════╗
    ║     Git-like VCS         ║
    ║     Version Control      ║
    ║     System in Rust       ║
    ╚══════════════════════════╝

    Available Commands:
    • guts init
    • ls, pwd, cd
    • clear, exit

    Navigation:
    • ↑/↓ - Command history
    • Ctrl+↑/↓ - Scroll output
    • Ctrl+C - Quit
    • Enter - Execute command

    Soon:
    • guts add .
    • guts status
    • guts commit -m "message"
"#;

    let paragraph = Paragraph::new(ascii_art)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_cli_interface(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Banner
            Constraint::Min(0),         // Command history
            Constraint::Length(3),      // Input area
        ])
        .split(area);

    app.update_visible_lines(chunks[1].height as usize);

    // banner
    render_banner(f, chunks[0]);

    // command history avec scrollbar
    render_command_history_with_scroll(f, chunks[1], app);

    // input area
    render_input_area(f, chunks[2], app);
}

fn render_banner(f: &mut Frame, area: Rect) {
    let banner = Paragraph::new("Team UNFAIR")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);

    f.render_widget(banner, area);
}

fn render_command_history_with_scroll(f: &mut Frame, area: Rect, app: &App) {
    let mut items = Vec::new();

    // add welcome message if history is empty
    if app.command_history.is_empty() {
        items.push(ListItem::new(vec![
            Line::from(vec![
                Span::styled("Welcome to GUTS Terminal!", Style::default().fg(Color::LightGreen)),
            ]),
            Line::from(vec![
                Span::styled("Team UNFAIR: Jecko, Max, Kae, Algont", Style::default().fg(Color::LightGreen)),
            ]),
            Line::from(vec![
                Span::styled("Type 'guts' commands or regular shell commands.", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("Press Ctrl+C to quit. Use Ctrl+↑/↓ to scroll.", Style::default().fg(Color::Gray)),
            ]),
        ]));
    }

    // add command history
    for result in &app.command_history {
        items.push(ListItem::new(vec![
            Line::from(vec![
                Span::styled("$ ", Style::default().fg(Color::Green)),
                Span::styled(&result.command, Style::default().fg(Color::White)),
            ]),
        ]));

        // output
        if !result.output.is_empty() {
            for line in result.output.lines() {
                items.push(ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(line, Style::default().fg(Color::LightBlue)),
                    ]),
                ]));
            }
        }

        // error
        if let Some(error) = &result.error {
            for line in error.lines() {
                items.push(ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(line, Style::default().fg(Color::LightRed)),
                    ]),
                ]));
            }
        }

        // add empty line between commands
        items.push(ListItem::new(vec![Line::from("")]));
    }

    let total_lines = app.total_history_lines();
    let title = if total_lines > app.max_visible_lines {
        format!("History ({}↑↓{})", app.scroll_offset + 1, total_lines)
    } else {
        "History".to_string()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(Color::White));

    //  ListState for scroll
    let mut list_state = ListState::default();
    list_state.select(Some(app.scroll_offset));

    // Render list with scroll
    f.render_stateful_widget(list, area, &mut list_state);

    if total_lines > app.max_visible_lines {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("║"))
            .thumb_symbol("█");

        let mut scrollbar_state = ScrollbarState::new(total_lines)
            .position(app.scroll_offset);

        f.render_stateful_widget(
            scrollbar,
            area.inner(&ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

fn render_input_area(f: &mut Frame, area: Rect, app: &App) {
    let current_dir = std::path::Path::new(&app.current_dir)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    let prompt = format!("{}$ ", current_dir);
    let input_text = format!("{}{}", prompt, app.input);

    let input = Paragraph::new(input_text)
        .block(Block::default().borders(Borders::ALL).title("Input"))
        .style(Style::default().fg(Color::White));

    f.render_widget(input, area);

    // Input cursor position
    let cursor_x = area.x + 1 + prompt.len() as u16 + app.cursor_position as u16;
    let cursor_y = area.y + 1;
    f.set_cursor(cursor_x, cursor_y);
}