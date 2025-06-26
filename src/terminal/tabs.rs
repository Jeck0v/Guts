pub struct Tab {
    pub title: String,
}

impl Tab {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
        }
    }
}
