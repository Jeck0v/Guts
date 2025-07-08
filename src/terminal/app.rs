use tui_tree_widget::TreeItem;

pub enum Tab {
    Cli,
    Editor,
}

pub struct App {
    pub input: String,
    pub tab: Tab,
    pub project_tree: Vec<TreeItem<'static, std::string::String>>,
}

impl App {
    pub fn new() -> Self {
        let project_tree = crate::terminal::tree::build_tree_from_current_dir();
        Self {
            input: String::new(),
            tab: Tab::Editor,
            project_tree,
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
        self.next_tab()
    }
}
