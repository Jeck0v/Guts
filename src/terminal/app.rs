use crate::terminal::tab::Tab;

pub struct App {
    pub current_tab: Tab,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_tab: Tab::Cli,
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = self.current_tab.next();
    }

    pub fn prev_tab(&mut self) {
        self.current_tab = self.current_tab.previous();
    }
}
