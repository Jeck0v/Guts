use ratatui::{prelude::*, widgets::*};
use crate::terminal::app::App;

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(f.size());

    let tabs = Tabs::new(vec!["CLI", "Editor"].into_iter().map(Line::from).collect())
        .select(app.current_tab as usize)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    f.render_widget(tabs, chunks[0]);

    let content = Paragraph::new(format!("Current tab: {}", app.current_tab.title()));
    f.render_widget(content, chunks[1]);
}
