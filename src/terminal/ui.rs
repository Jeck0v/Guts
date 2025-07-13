use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Line, Text},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::terminal::app::{App, Tab};

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(size);

    draw_tabs(f, app, layout[0]);

    match app.tab {
        Tab::Cli => draw_cli(f, app, layout[1]),
        Tab::Editor => draw_editor_layout(f, app, layout[1]),
    }
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<_> = ["CLI", "Editor"]
        .iter()
        .map(|t| Line::from(Span::styled(*t, Style::default())))
        .collect();

    let tabs = Tabs::new(titles)
        .select(match app.tab {
            Tab::Cli => 0,
            Tab::Editor => 1,
        })
        .block(Block::default().title("GUTS - TUI").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_widget(tabs, area);
}

fn draw_cli(f: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    let editor = Paragraph::new(app.input.as_str())
        .block(Block::default().title("CLI Input").borders(Borders::ALL));
    f.render_widget(editor, layout[0]);

    draw_project_tree(f, app, layout[1]);
}

fn draw_editor_layout(f: &mut Frame, app: &App, size: Rect) {
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
    let lines: Vec<Line> = (0..100)
        .map(|i| {
            Line::from(Span::raw(format!(
                "{:>2} │ {}",
                i,
                if i == 0 { &app.input } else { "" }
            )))
        })
        .collect();

    let editor = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title("CLI - Editor"));

    f.render_widget(editor, area);
}

fn draw_project_tree(f: &mut Frame, app: &App, area: Rect) {
    // Soon....
    let tree_lines = app
        .project_tree
        .iter()
        .map(|line| Line::from(Span::raw(line.clone())))
        .collect::<Vec<Line>>();

    let tree_paragraph = Paragraph::new(Text::from(tree_lines))
        .block(Block::default().title("Project Tree").borders(Borders::ALL));

    f.render_widget(tree_paragraph, area);
}
