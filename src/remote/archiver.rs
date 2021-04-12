use crate::remote::{Connector, Command, Local, FileCopier, RemoteFileCopier, Computer, file_is_empty, path_to_part, copy_from_remote_wildcards};
use std::path::{Path, PathBuf};
use std::{io, thread};
use std::time::Duration;
use std::io::{Error};
use uuid::Uuid;
use crate::utils::path_join_to_string_ntfs;


#[derive(Clone, Copy)]
pub enum Compression {
    No,
    Yes,
    YesSplit,
}

pub trait Archiver {
    fn compress(&self, path: &Path, split: bool) -> PathBuf;
    fn uncompress(&self, path: &Path) -> io::Result<()>;
}

pub struct SevenZipArchiver<'a> {
    connector: &'a dyn Connector,
    timeout: Option<Duration>,
}

impl<'a> SevenZipArchiver<'a> {
    pub fn remote(
        connector: &'a dyn Connector,
        timeout: Option<Duration>,
    ) -> SevenZipArchiver<'a> {
        SevenZipArchiver {
            connector,
            timeout,
        }
    }

    pub fn local(local: &'a Local) -> SevenZipArchiver {
        SevenZipArchiver::remote(local, None)
    }
}


impl<'a> Archiver for SevenZipArchiver<'a> {
    fn compress(&self, path: &Path, split: bool) -> PathBuf {
        let archive_file_name = format!("{}_{}__{}.7z",
                                        self.connector.computer().address.replace(".", "-"),
                                        path_join_to_string_ntfs(path),
                                        Uuid::new_v4().to_string().replace("-", "")
        ).replace(" ", "");
        let path_string_7z = self.connector.remote_temp_storage().join(archive_file_name);
        let mut run_params = vec![
            "7za.exe".to_string(),
        ];

        if split {
            run_params.push("-bd".to_string());
            run_params.push("-mx5".to_string());
            // run_params.push("-sdel".to_string());
            run_params.push("-t7z".to_string());
            run_params.push("-v2m".to_string());
        }

        run_params.push("a".to_string());
        run_params.push(path_string_7z.to_string_lossy().to_string());
        run_params.push(path.to_string_lossy().to_string());

        if path.file_name().unwrap_or_default().to_string_lossy().contains("*") {
            run_params.push("-r".to_string())
        }

        let command = Command {
            command: run_params,
            report_store_directory: None,
            report_filename_prefix: "",
            elevated: true,
        };
        if let Err(err) = self.connector.connect_and_run_local_program_in_current_directory(
            command,
            self.timeout.clone(),
        ) {
            debug!("{}", err)
        }
        // if split {
        // //  already deleted by 7zip itself
        // } else {
        //     if let Err(err) = self.connector.copier().delete_remote_file(&path_string_7z) {
        //         debug!("{}", err)
        //     }
        // }
        path_string_7z
    }

    fn uncompress(&self, path: &Path) -> io::Result<()> {
        let path_string = path.to_string_lossy().to_string();
        let command = Command {
            command: vec![
                "7za.exe".to_string(),
                "-aoa".to_string(),
                "-bd".to_string(),
                "e".to_string(),
                path_string,
                format!("-o{}", path.parent().unwrap().display())
            ],
            report_store_directory: None,
            report_filename_prefix: "",
            elevated: true,
        };
        self.connector.connect_and_run_local_program_in_current_directory(
            command,
            self.timeout.clone(),
        ).map(|_| ())
    }
}

pub struct SevenZipCompressCopier<'a> {
    archiver: SevenZipArchiver<'a>,
    split: bool,
    uncompress_downloaded: bool,
}

impl<'a> SevenZipCompressCopier<'a> {
    pub fn new(
        connector: &'a dyn Connector,
        split: bool,
        timeout: Option<Duration>,
        uncompress_downloaded: bool,
    ) -> SevenZipCompressCopier {
        SevenZipCompressCopier {
            archiver: SevenZipArchiver::remote(connector, timeout),
            split,
            uncompress_downloaded,
        }
    }
}

impl<'a> RemoteFileCopier for SevenZipCompressCopier<'a> {
    fn remote_computer(&self) -> &Computer {
        self.archiver.connector.computer()
    }

    fn copier_impl(&self) -> &dyn FileCopier {
        self.archiver.connector.copier().copier_impl()
    }

    fn path_to_remote_form(&self, path: &Path) -> PathBuf {
        self.archiver.connector.copier().path_to_remote_form(path)
    }

    fn copy_to_remote(&self, source: &Path, target: &Path) -> Result<(), Error> {
        trace!("Copying {} to {} using compression", source.display(), &self.archiver.connector.computer().address);
        let remote_copier_impl = self.archiver.connector.copier();
        let local = Local::new_default(self.archiver.connector.computer().username.clone());
        let local_archiver = SevenZipArchiver::local(&local);

        let archived_source = local_archiver.compress(source, self.split);
        let wait_time_s = Duration::from_secs(1);
        let wait_time_l = Duration::from_secs(10);
        if self.split {
            let mut i = 1;
            let mut part = path_to_part(&archived_source, i);
            while part.exists() {
                if let Err(err) = remote_copier_impl.copy_to_remote(&part, target) {
                    debug!("{}", err)
                } else {
                    std::thread::sleep(wait_time_s.clone());
                    if let Err(err) = local.delete_file(&part) {
                        debug!("{}", err);
                    }
                }
                i += 1;
                part = path_to_part(&archived_source, i);
            }
            std::thread::sleep(wait_time_l.clone());
            if let Err(err) = self.archiver.uncompress(
                &target.join(
                    path_to_part(&archived_source, 1).file_name().unwrap()
                )
            ) {
                debug!("{}", err);
            } else {
                i -= 1;
                let mut remote_part = target.join(
                    path_to_part(&archived_source, 1).file_name().unwrap());

                while i > 0 {
                    if let Err(err) = remote_copier_impl.delete_remote_file(&remote_part) {
                        debug!("{}", err);
                    }
                    std::thread::sleep(wait_time_l.clone());
                    i -= 1;
                    remote_part = target.join(path_to_part(&archived_source, 1).file_name().unwrap());
                }
            }
        } else {
            if let Err(err) = remote_copier_impl.copy_to_remote(&archived_source, target) {
                debug!("{}", err);
            } else {
                if let Err(err) = local.delete_file(&archived_source) {
                    debug!("{}", err)
                }
            }
            std::thread::sleep(wait_time_l.clone());
            let target_archived = &target.join(
                archived_source.file_name().unwrap()
            );
            if let Err(err) = self.archiver.uncompress(&target_archived) {
                debug!("{}", err)
            } else {
                std::thread::sleep(wait_time_l.clone());
                if let Err(err) = remote_copier_impl.delete_remote_file(&target_archived) {
                    debug!("{}", err);
                }
            }
        }

        Ok(())
    }

    fn delete_remote_file(&self, target: &Path) -> Result<(), Error> {
        self.archiver.connector.copier().delete_remote_file(target)
    }

    fn copy_from_remote(&self, source: &Path, target: &Path) -> Result<(), Error> {
        copy_from_remote_wildcards(
            source,
            target,
            self.archiver.connector,
            |s, t| self.copy_from_remote_impl(s, t),
        )
    }
}


impl SevenZipCompressCopier<'_> {
    fn copy_from_remote_impl(&self, source: &Path, target: &Path) -> Result<(), Error> {
        trace!("Copying {} from {} using compression", source.display(), &self.archiver.connector.computer().address);
        let archived_source =
            self.archiver.compress(
                source,
                self.split
            );

        let wait_time_s = Duration::from_secs(10);
        let wait_time_l = Duration::from_secs(30);
        thread::sleep(wait_time_s.clone());

        let remote_copier_impl = self.archiver.connector.copier();

        let local = Local::new_default(self.archiver.connector.computer().username.clone());
        let local_archiver = SevenZipArchiver::local(&local);

        if self.split {
            self.copy_from_remote_splitted(target, wait_time_s, wait_time_l, &archived_source, remote_copier_impl, &local, &local_archiver)
        } else {
            self.copy_from_remote_whole(target, wait_time_s, &archived_source, remote_copier_impl, &local, &local_archiver)
        }
        Ok(())
    }
    fn copy_from_remote_splitted(&self,
                                 target: &Path,
                                 wait_time_s: Duration,
                                 wait_time_l: Duration,
                                 archived_source: &Path,
                                 remote_copier_impl: &dyn RemoteFileCopier,
                                 local: &Local,
                                 local_archiver: &SevenZipArchiver,
    ) {
        let mut unsuccessful_trials = 0;
        let mut i = 0;
        loop {
            i += 1;
            let part = path_to_part(archived_source, i);
            trace!("Copying {} from {} using compression", part.display(), &self.archiver.connector.computer().address);
            if let Err(err) = remote_copier_impl.copy_from_remote(&part, target) {
                debug!("{}", err);
            }
            let target_downloaded = target.join(part.file_name().unwrap());

            if file_is_empty(&target_downloaded) {
                unsuccessful_trials += 1;
                i -= 1;
                if unsuccessful_trials == 2 {
                    break;
                }
                debug!("File download may ended with errors. Waiting {} seconds before retry.", wait_time_l.as_secs());
                thread::sleep(wait_time_l);
            } else {
                unsuccessful_trials = 0;
            }

            thread::sleep(wait_time_s.clone());

            if unsuccessful_trials == 0 {
                if let Err(err) = remote_copier_impl.delete_remote_file(&part) {
                    debug!("{}", err);
                }
            }
        }
        if self.uncompress_downloaded {
            let target_downloaded_without_part_suffix = target.join(archived_source.file_name().unwrap());
            if let Err(err) = local_archiver.uncompress(&path_to_part(&target_downloaded_without_part_suffix, 1)) {
                debug!("{}", err);
            } else {
                let mut part = path_to_part(&target_downloaded_without_part_suffix, i);
                while i > 0 {
                    if let Err(err) = local.delete_file(&part) {
                        debug!("{}", err);
                    }
                    i -= 1;
                    part = path_to_part(&target_downloaded_without_part_suffix, i);
                }
            }
        }
    }

    fn copy_from_remote_whole(&self,
                              target: &Path,
                              wait_time_s: Duration,
                              archived_source: &Path,
                              remote_copier_impl: &dyn RemoteFileCopier,
                              local: &Local,
                              local_archiver: &SevenZipArchiver,
    ) {
        if let Err(err) = remote_copier_impl.copy_from_remote(archived_source, target) {
            debug!("{}", err);
        } else {
            thread::sleep(wait_time_s.clone());
            if let Err(err) = remote_copier_impl.delete_remote_file(archived_source) {
                debug!("{}", err);
            }
        }
        if self.uncompress_downloaded {
            let target_downloaded = target.join(archived_source.file_name().unwrap());
            if let Err(err) = local_archiver.uncompress(&target_downloaded) {
                debug!("{}", err);
            } else {
                thread::sleep(wait_time_s.clone());
                if let Err(err) = local.delete_file(&target_downloaded) {
                    debug!("{}", err);
                }
            }
        }
    }
}