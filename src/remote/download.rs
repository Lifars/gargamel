use std::path::Path;
use crate::remote::{RemoteFileCopier, file_is_empty, path_to_part};
use std::thread;
use std::time::Duration;

pub struct ReDownloader<'a> {
    pub copier: &'a dyn RemoteFileCopier,
    pub target_dir: &'a Path,
}

impl<'a> ReDownloader<'a> {
    pub fn retry_download(&self, remote_path: &Path) -> bool {
        let extension = remote_path.extension().unwrap().to_string_lossy();
        match extension.parse::<u32>() {
            Ok(_) => self.download_as_splitted(remote_path),
            Err(_) => {
                let success = self.download_as_non_splitted(remote_path);
                if !success {
                    let remote_path_7z_part = if extension.ends_with("7z") {
                        remote_path
                            .parent().unwrap()
                            .join(format!("{}.001", remote_path.file_name().unwrap().to_string_lossy()))
                    } else {
                        remote_path
                            .parent().unwrap()
                            .join(format!("{}.7z.001", remote_path.file_name().unwrap().to_string_lossy()))
                    };

                    self.download_as_splitted(&remote_path_7z_part)
                } else {
                    true
                }
            }
        }
    }

    fn download_as_splitted(&self, remote_path: &Path) -> bool {
        let extension = remote_path.extension().unwrap().to_string_lossy();
        let wait_time_s = Duration::from_secs(10);

        let mut i = extension.parse::<usize>().unwrap() - 1;
        let archived_remote_file = remote_path.parent().unwrap().join(remote_path.file_stem().unwrap());
        let mut unsuccessful_trials = 0;
        loop {
            i += 1;
            let part = path_to_part(&archived_remote_file, i);
            trace!("Copying {} from {}", part.display(), &self.copier.remote_computer().address);
            if let Err(err) = self.copier.copy_from_remote(&part, self.target_dir) {
                debug!("{}", err);
            }
            let target_downloaded = self.target_dir.join(part.file_name().unwrap());

            if file_is_empty(&target_downloaded) {
                unsuccessful_trials += 1;
                i -= 1;
                if unsuccessful_trials == 2 {
                    break;
                }
                debug!("File download may ended with errors. Waiting 30 seconds before retry.");
                thread::sleep(Duration::from_secs(30));
            } else {
                unsuccessful_trials = 0;
            }

            thread::sleep(wait_time_s.clone());
            if unsuccessful_trials == 0 {
                if let Err(err) = self.copier.delete_remote_file(&part) {
                    debug!("{}", err);
                }
            }
        }
        true
    }

    fn download_as_non_splitted(&self, remote_path: &Path) -> bool {
        if self.download_as_is(remote_path) {
            return true;
        }

        let remote_path_7z = remote_path
            .parent().unwrap()
            .join(format!("{}.7z", remote_path.file_name().unwrap().to_string_lossy()));

        self.download_as_is(&remote_path_7z)
    }

    fn download_as_is(&self, remote_path: &Path) -> bool {
        if !self.target_dir.exists() {
           if let Err(err) =  std::fs::create_dir_all(self.target_dir){
               error!("{}", err)
           }
        }
        if let Err(err) = self.copier.copy_from_remote(remote_path, self.target_dir) {
            debug!("{}", err)
        }
        let target = self.target_dir.join(remote_path.file_name().unwrap());
        let result = file_is_empty(&target);
        if result {
            info!("Downloaded remote file {} to {}", remote_path.display(), self.target_dir.display());
        }
        !result
    }
}
