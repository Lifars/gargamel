use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;

pub fn file_is_empty(target_downloaded: &Path) -> bool {
    let mut file = match File::open(target_downloaded){
        Ok(file) => file,
        Err(_) => return true,
    };
    let mut buf: [u8; 100] = [0; 100];
    if file.read_exact(&mut buf).is_err() {
        return true
    }
    false
}

pub fn path_to_part(path: &Path, part: usize) -> PathBuf {
    let joined = match part {
        part if part < 10 => format!("{}.00{}", path.display(), part),
        part if part < 100 => format!("{}.0{}", path.display(), part),
        part => format!("{}.{}", path.display(), part)
    };
    PathBuf::from(joined)
}

