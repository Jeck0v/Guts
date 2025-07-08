pub struct App {
    pub input: String,
    pub project_tree: Vec<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            project_tree: vec![
                "src/".into(),
                "├── main.rs".into(),
                "├── lib.rs".into(),
                "└── core/".into(),
            ],
        }
    }

    pub fn on_key(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn backspace(&mut self) {
        self.input.pop();
    }
}
