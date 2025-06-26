use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Tabs},
};

use crate::terminal::app::App;

pub fn draw_ui(f: &mut Frame, app: &App) {
    let size = f.size();

    let titles = app
        .tabs
        .iter()
        .map(|t| t.title.as_str())
        .collect::<Vec<&str>>();

    let tabs = Tabs::new(titles)
        .select(app.selected_tab)
        .block(Block::default().borders(Borders::ALL).title("Onglets"));

    f.render_widget(tabs, size);
}
