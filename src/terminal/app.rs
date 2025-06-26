use crate::terminal::tabs::Tab;

pub struct App {
    pub tabs: Vec<Tab>,
    pub selected_tab: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            tabs: vec![
                Tab::new("Terminal"),
                Tab::new("Éditeur"),
            ],
            selected_tab: 0,
        }
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % self.tabs.len();
    }

    pub fn previous_tab(&mut self) {
        if self.selected_tab == 0 {
            self.selected_tab = self.tabs.len() - 1;
        } else {
            self.selected_tab -= 1;
        }
    }
}
