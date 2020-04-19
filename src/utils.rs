use std::path::{PathBuf, Path};
use std::str::FromStr;

pub trait Quoted{
    fn quoted(&self) -> String;
}

impl Quoted for str {
    fn quoted(&self) -> String {
        format!("\"{}\"", self)
    }
}

pub fn remote_storage() -> PathBuf {
    PathBuf::from_str("C:\\Users\\Public")
        .expect("Internal error: Remote store directory is ill formatted.")
}

pub fn remote_storage_file<T: AsRef<Path>>(file: T) -> PathBuf {
    remote_storage().join(file)
}