use std::path::Path;

pub trait Quoted{
    fn quoted(&self) -> String;
}

impl Quoted for str {
    fn quoted(&self) -> String {
        format!("\"{}\"", self)
    }
}


pub fn path_join_to_string_ntfs(path: &Path) -> String {
    path
        .to_str()
        .unwrap_or("y")
        .replace("*", "--S--")
        .replace("?", "--Q--")
        .replace(":", "")
        .replace("\\", "-")
        .replace("/", "-")
}
