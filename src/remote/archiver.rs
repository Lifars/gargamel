use crate::remote::{Connector, Command, Local, FileCopier, RemoteFileCopier, Computer, file_is_empty, path_to_part};
use std::path::{Path, PathBuf};
use std::{io, thread};
use std::time::Duration;
use std::io::{Error, Read};
use std::fs::File;

#[derive(Clone, Copy)]
pub enum Compression {
    No,
    Yes,
    YesSplit,
}

pub struct Archiver<'a> {
    connector: &'a dyn Connector,
    timeout: Option<Duration>,
}

impl<'a> Archiver<'a> {
    pub fn remote(
        connector: &'a dyn Connector,
        timeout: Option<Duration>,
    ) -> Archiver<'a> {
        Archiver {
            connector,
            timeout,
        }
    }

    pub fn local(local: &'a Local) -> Archiver {
        Archiver::remote(local, None)
    }

    pub fn compress(&self, path: &Path, split: bool) -> io::Result<()> {
        let path_string = path.to_string_lossy().to_string();
        let mut run_params = vec![
            "7za.exe".to_string(),
        ];

        if split {
            run_params.push("-bd".to_string());
//            run_params.push("-mnt4".to_string());
            run_params.push("-mx5".to_string());
            run_params.push("-sdel".to_string());
            run_params.push("-t7z".to_string());
            run_params.push("-v2m".to_string());
        }

        run_params.push("a".to_string());
        run_params.push(format!("{}.7z", path_string));
        run_params.push(path_string);

        let command = Command {
            command: run_params,
            report_store_directory: None,
            report_filename_prefix: "",
            elevated: false,
        };
        if let Err(err) = self.connector.connect_and_run_local_program_in_current_directory(
            command,
            self.timeout.clone(),
        ) {
            debug!("{}", err)
        }
        if split {
            // already deleted by 7zip itself
        } else {
            if let Err(err) = self.connector.copier().delete_remote_file(path) {
                debug!("{}", err)
            }
        }
        Ok(())
    }

    pub fn uncompress(&self, path: &Path) -> io::Result<()> {
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
            elevated: false,
        };
        self.connector.connect_and_run_local_program_in_current_directory(
            command,
            self.timeout.clone(),
        )
    }
}

pub struct CompressCopier<'a> {
    archiver: Archiver<'a>,
    split: bool,
}

impl<'a> CompressCopier<'a> {
    pub fn new(
        connector: &'a dyn Connector,
        split: bool,
        timeout: Option<Duration>,
    ) -> CompressCopier {
        CompressCopier {
            archiver: Archiver::remote(connector, timeout),
            split,
        }
    }
}

impl<'a> RemoteFileCopier for CompressCopier<'a> {
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
        let local = Local::new();
        let local_archiver = Archiver::local(&local);
        if let Err(err) = local_archiver.compress(source, self.split) {
            debug!("{}", err);
        }

        let archive_name = format!("{}.7z", source.display());
        let archived_source = Path::new(&archive_name);
        let wait_time_s = Duration::from_secs(1);
        let wait_time_l = Duration::from_secs(10);
        if self.split {
            let mut i = 1;
            let mut part = path_to_part(archived_source, i);
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
                part = path_to_part(archived_source, i);
            }
            std::thread::sleep(wait_time_l.clone());
            if let Err(err) = self.archiver.uncompress(
                &target.join(
                    path_to_part(archived_source, 1).file_name().unwrap()
                )
            ) {
                debug!("{}", err);
            } else {
                i -= 1;
                let mut remote_part = target.join(
                    path_to_part(archived_source, 1).file_name().unwrap());

                while i > 0 {
                    if let Err(err) = remote_copier_impl.delete_remote_file(&remote_part) {
                        debug!("{}", err);
                    }
                    std::thread::sleep(wait_time_l.clone());
                    i -= 1;
                    remote_part = target.join(path_to_part(archived_source, 1).file_name().unwrap());
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
        trace!("Copying {} from {} using compression", source.display(), &self.archiver.connector.computer().address);
        if let Err(err) = self.archiver.compress(source, self.split) {
            debug!("{}", err);
        }

        let wait_time_s = Duration::from_secs(10);
        let wait_time_l = Duration::from_secs(30);
        thread::sleep(wait_time_s.clone());

        let archive_name = format!("{}.7z", source.display());
        let archived_source = Path::new(&archive_name);
        let remote_copier_impl = self.archiver.connector.copier();

        let local = Local::new();
        let local_archiver = Archiver::local(&local);

        if self.split {
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
        } else {
            if let Err(err) = remote_copier_impl.copy_from_remote(archived_source, target) {
                debug!("{}", err);
            } else {
                thread::sleep(wait_time_s.clone());
                if let Err(err) = remote_copier_impl.delete_remote_file(archived_source) {
                    debug!("{}", err);
                }
            }

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
        Ok(())
    }
}



