use crate::remote::{RemoteFileCopier};
use std::path::Path;
use std::io;
use std::fs::File;
use std::io::{BufReader, BufRead};
use crate::embedded_search_list::embedded_search_list;
use crate::kape_handler;

pub fn download_files(file_list: &str,
                      local_store_directory: &Path,
                      downloader: &dyn RemoteFileCopier,
                      separate_stores: bool
) -> io::Result<()> {
    if file_list == "EMBEDDED" {
        download_files_from_embedded(local_store_directory, downloader, separate_stores)
    } else {
        download_files_from_path(Path::new(file_list), local_store_directory, downloader, separate_stores)
    }
}

pub fn download_files_from_embedded(local_store_directory: &Path,
                                    downloader: &dyn RemoteFileCopier,
                                    separate_stores: bool
) -> io::Result<()> {
    let local_store_directory = dunce::canonicalize(local_store_directory)
        .expect(&format!("Cannot canonicalize {}", local_store_directory.display()));
    for path_to_find in embedded_search_list() {
        if path_to_find.starts_with("#") {
            continue;
        }
        let _ = download_file(&path_to_find, &local_store_directory, downloader, separate_stores);
    }
    Ok(())
}

pub fn download_files_from_path(file_list: &Path,
                                local_store_directory: &Path,
                                downloader: &dyn RemoteFileCopier,
                                separate_stores: bool
) -> io::Result<()> {
    let input_file = File::open(file_list)?;

    let local_store_directory = dunce::canonicalize(local_store_directory)
        .expect(&format!("Cannot canonicalize {}", local_store_directory.display()));

    for path_to_find in BufReader::new(input_file).lines() {
        if path_to_find.is_err() {
            warn!("Cannot read line in {}", file_list.display());
            continue
        }

        let path_to_find = path_to_find.unwrap();
        if path_to_find.ends_with("tkape") {
            match kape_handler::parse_tkape(Path::new(&path_to_find)) {
                Ok(tkape) => {
                    for take_target in tkape.targets {
                        if let Err(e) = download_file(&take_target.path, &local_store_directory, downloader, separate_stores) {
                            warn!("{}", e)
                        }
                    }
                },
                _ => println!("Error parsing {}", path_to_find)
            };


        }else {
            if let Err(e) = download_file(&path_to_find, &local_store_directory, downloader, separate_stores) {
                warn!("{}", e)
            }
        }

    }
    Ok(())
}

pub fn download_file(
    path: &str,
    local_store_directory: &Path,
    downloader: &dyn RemoteFileCopier,
    separate_stores: bool,
) -> io::Result<()> {
    if path.starts_with("#") {
        return Ok(());
    }
    let path_to_download = Path::new(path);
    trace!("Establishing download of {} using {}", path_to_download.display(), downloader.method_name());

    let local_store_directory = if separate_stores {
        let dir_name = path
            .replace("\\", "-")
            .replace("/", "-")
            .replace(" ", "")
            .replace("*", "--S--")
            .replace("?", "--Q--")
            .replace(":", "");
        let local_store_directory = local_store_directory.join(dir_name);
        let _ =std::fs::create_dir(&local_store_directory);
        local_store_directory
    }else{
        local_store_directory.to_path_buf()
    };

    let download_result = downloader.copy_from_remote(
        path_to_download,
        &local_store_directory,
    );
    match &download_result {
        Ok(_) => { debug!("Remote file {} found and downloaded", path) }
        Err(err) => { warn!("Cannot find remote file {} due to: {}", path, err) }
    }
    download_result
}
