#[derive(Debug, Clone, Copy)]
pub enum Tab {
    Cli,
    Editor,
}

impl Tab {
    pub fn next(self) -> Self {
        match self {
            Tab::Cli => Tab::Editor,
            Tab::Editor => Tab::Cli,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Tab::Cli => Tab::Editor,
            Tab::Editor => Tab::Cli,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Tab::Cli => "CLI",
            Tab::Editor => "Editor",
        }
    }
}
