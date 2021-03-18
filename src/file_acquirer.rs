use crate::remote::{RemoteFileCopier};
use std::path::Path;
use std::io;
use std::fs::File;
use std::io::{BufReader, BufRead};

pub fn download_files(file_list: &Path,
                      local_store_directory: &Path,
                      downloader: &dyn RemoteFileCopier,
) -> io::Result<()> {
    let input_file = File::open(file_list)?;
    let local_store_directory = dunce::canonicalize(local_store_directory)
        .expect(&format!("Cannot canonicalize {}", local_store_directory.display()));
    for path_to_find in BufReader::new(input_file).lines() {
        if path_to_find.is_err() {
            warn!("Cannot read line in {}", file_list.display());
        }
        let path_to_find = path_to_find.unwrap();
        if path_to_find.starts_with("#"){
            continue;
        }
        let path_to_download = Path::new(&path_to_find);
        trace!("Establishing download of {} using {}", path_to_download.display(), downloader.method_name());
        let download_result = downloader.copy_from_remote(
            path_to_download,
            &local_store_directory,
        );
        match download_result {
            Ok(_) => { debug!("Remote file {} found and downloaded", path_to_find) }
            Err(err) => { warn!("Cannot find remote file {} due to: {}", path_to_find, err) }
        }
    }
    Ok(())
}