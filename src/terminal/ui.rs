use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Line, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};


use crate::terminal::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)].as_ref())
        .split(size);

    let banner = Paragraph::new(Line::from(Span::styled(
        " GUTS - Git in Rust from scratch",
        Style::default().add_modifier(Modifier::BOLD),
    )))
        .block(Block::default().borders(Borders::ALL).title("Banner"));

    f.render_widget(banner, vertical[0]);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(vertical[1]);

    draw_editor(f, app, horizontal[0]);
    draw_project_tree(f, app, horizontal[1]);
}

fn draw_editor(f: &mut Frame, app: &App, area: Rect) {
    let lines: Vec<Line> = (0..20)
        .map(|i| {
            Line::from(Span::raw(format!(
                "{:>2} │ {}",
                i,
                if i == 10 { &app.input } else { "" }
            )))
        })
        .collect();

    let editor = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title("CLI - Editor"));

    f.render_widget(editor, area);
}

fn draw_project_tree(f: &mut Frame, app: &App, area: Rect) {
    let lines: Vec<Line> = app
        .project_tree
        .iter()
        .map(|line| Line::from(Span::raw(line)))
        .collect();

    let tree = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title("Project Tree"));

    f.render_widget(tree, area);
}
