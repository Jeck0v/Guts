#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Cli,
    Editor,
}

pub struct App {
    pub input: String,
    pub tab: Tab,
    pub project_tree: Vec<String>,
    pub selected_index: usize,
}

impl App {
    pub fn new() -> Self {
        // Need change this for the real arbo
        let project_tree = vec![
            "src/".to_string(),
            "  └── main.rs".to_string(),
            "  └── lib.rs".to_string(),
            "Cargo.toml".to_string(),
            "README.md".to_string(),
        ];

        Self {
            input: String::new(),
            tab: Tab::Cli,
            project_tree,
            selected_index: 0,
        }
    }

    pub fn on_key(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn backspace(&mut self) {
        self.input.pop();
    }

    pub fn next_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Cli => Tab::Editor,
            Tab::Editor => Tab::Cli,
        }
    }

    pub fn prev_tab(&mut self) {
        self.next_tab(); // toggle
    }

    pub fn select_next(&mut self) {
        if self.selected_index + 1 < self.project_tree.len() {
            self.selected_index += 1;
        }
    }

    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }
}
