use std::fs;
use std::path::Path;
use std::string::String;
use tui_tree_widget::TreeItem;

pub fn build_tree_from_current_dir() -> Vec<TreeItem<'static, String>> {
    let cwd = std::env::current_dir().unwrap();
    vec![build_tree_from_path(&cwd)]
}

fn build_tree_from_path(path: &Path) -> TreeItem<'static, String> {
    let name = path.file_name()
        .unwrap_or_else(|| path.as_os_str())
        .to_string_lossy()
        .to_string();

    let id = path.to_string_lossy().to_string();

    if path.is_dir() {
        let children: Vec<TreeItem<'static, String>> = fs::read_dir(path)
            .unwrap_or_else(|_| panic!("Cannot read dir {:?}", path))
            .filter_map(|entry| entry.ok())
            .map(|entry| build_tree_from_path(&entry.path()))
            .collect();

        TreeItem::new(id, name, children).expect("Failed to create TreeItem with children")
    } else {
        TreeItem::new(id, name, vec![]).expect("Failed to create TreeItem for file")
    }
}
