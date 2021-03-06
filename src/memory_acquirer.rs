use crate::remote::{Connector, Computer, Command, PsExec, PsRemote, Rdp, Wmi, SevenZipCompressCopier, RemoteFileCopier, Compression, Local, RevShareConnector};
use std::path::{Path, PathBuf};
use std::{io, thread};
use std::time::Duration;
use crate::process_runner::create_report_path;

pub struct MemoryAcquirer<'a> {
    pub local_store_directory: &'a Path,
    pub connector: Box<dyn Connector>,
    pub image_timeout: Option<Duration>,
    pub compress_timeout: Option<Duration>,
    pub compression: Compression,
}

impl<'a> MemoryAcquirer<'a> {
    pub fn psexec32(
        remote_computer: Computer,
        local_store_directory: &'a Path,
        no_7zip: bool,
        remote_temp_storage: PathBuf,
        custom_share_folder: Option<String>,
        reverse: bool,
    ) -> MemoryAcquirer<'a> {
        let connector = Box::new(PsExec::psexec32(remote_computer, remote_temp_storage, custom_share_folder));
        MemoryAcquirer {
            local_store_directory,
            connector: if reverse { Box::new(RevShareConnector::new(connector)) } else { connector },
            image_timeout: None,
            compress_timeout: None,
            compression: if no_7zip { Compression::No } else { Compression::Yes },
        }
    }

    pub fn local(
        username: String,
        local_store_directory: &'a Path,
        temp_storage: PathBuf,
    ) -> MemoryAcquirer<'a> {
        MemoryAcquirer {
            local_store_directory,
            connector: Box::new(Local::new(username, temp_storage)),
            image_timeout: None,
            compress_timeout: None,
            compression: Compression::No,
        }
    }

    pub fn psexec64(
        remote_computer: Computer,
        local_store_directory: &'a Path,
        no_7zip: bool,
        remote_temp_storage: PathBuf,
        custom_share_folder: Option<String>,
        reverse: bool,
    ) -> MemoryAcquirer<'a> {
        let connector = Box::new(PsExec::psexec64(remote_computer, remote_temp_storage, custom_share_folder));
        MemoryAcquirer {
            local_store_directory,
            connector: if reverse { Box::new(RevShareConnector::new(connector)) } else { connector },
            image_timeout: None,
            compress_timeout: None,
            compression: if no_7zip { Compression::No } else { Compression::Yes },
        }
    }

    pub fn psremote(
        remote_computer: Computer,
        local_store_directory: &'a Path,
        _no_7zip: bool,
        remote_temp_storage: PathBuf,
        custom_share_folder: Option<String>,
        reverse: bool,
    ) -> MemoryAcquirer<'a> {
        let connector = Box::new(PsRemote::new(remote_computer, remote_temp_storage, custom_share_folder));
        MemoryAcquirer {
            local_store_directory,
            connector: if reverse { Box::new(RevShareConnector::new(connector)) } else { connector },
            image_timeout: None,
            compress_timeout: None,
            compression: Compression::No,
        }
    }

    pub fn wmi(
        remote_computer: Computer,
        local_store_directory: &'a Path,
        timeout: Duration,
        compress_timeout: Duration,
        no_7zip: bool,
        remote_temp_storage: PathBuf,
    ) -> MemoryAcquirer<'a> {
        MemoryAcquirer {
            local_store_directory,
            connector: Box::new(Wmi { computer: remote_computer.clone(), remote_temp_storage }),
            image_timeout: Some(timeout),
            compress_timeout: Some(compress_timeout),
            compression: if no_7zip { Compression::No } else { Compression::YesSplit },
        }
    }

    pub fn rdp(
        remote_computer: Computer,
        local_store_directory: &'a Path,
        nla: bool,
        image_timeout: Duration,
        compress_timeout: Duration,
        no_7zip: bool,
        remote_temp_storage: PathBuf,
    ) -> MemoryAcquirer<'a> {
        MemoryAcquirer {
            local_store_directory,
            connector: Box::new(Rdp {
                nla,
                computer: remote_computer.clone(),
                remote_temp_storage,
            }),
            image_timeout: Some(image_timeout),
            compress_timeout: Some(compress_timeout),
            compression: if no_7zip { Compression::No } else { Compression::YesSplit },
        }
    }

    pub fn image_memory(
        &self
    ) -> io::Result<()> {
        let local_store_directory = self.local_store_directory;
        let winpmem = "winpmem.exe";

        // let target_name = remote_storage_file(target_name.file_name().unwrap());
        let target_name = create_report_path(
            self.connector.computer(),
            self.connector.remote_temp_storage(),
            "mem-image",
            self.connector.connect_method_name(),
            "aff4",
        );
        let connection = Command {
            command: vec![
                winpmem.to_string(),
                "--format".to_string(),
                "map".to_string(),
                "-t".to_string(),
                "-o".to_string(),
                target_name.to_string_lossy().to_string(),
            ],
            report_store_directory: None,
            report_filename_prefix: "mem-ack-log",
            elevated: true,
        };
        self.connector.connect_and_run_local_program_in_current_directory(
            connection,
            self.image_timeout,
        )?;
        let _copier = self.connector.copier();
        let _compression_split_copier = SevenZipCompressCopier::new(self.connector.as_ref(), true, self.compress_timeout, false);
        let _compression_copier = SevenZipCompressCopier::new(self.connector.as_ref(), false, self.compress_timeout, false);
        let copier = match self.compression {
            Compression::No => _copier,
            Compression::Yes => &_compression_copier as &dyn RemoteFileCopier,
            Compression::YesSplit => &_compression_split_copier as &dyn RemoteFileCopier,
        };
        match copier.copy_from_remote(
            &target_name,
            &local_store_directory,
            // &self.local_store_directory.join(target_name.file_name().unwrap()),
        ) {
            Ok(_) => {}
            Err(err) => {
                error!("Cannot download {} report from {} using method {} due to {}",
                       target_name.display(),
                       self.connector.computer().address,
                       self.connector.connect_method_name(),
                       err
                )
            }
        }
        thread::sleep(Duration::from_millis(1000));
        let winpem_path = self.connector.remote_temp_storage().join(winpmem);
        match copier.delete_remote_file(&winpem_path) {
            Ok(_) => {}
            Err(err) => {
                error!("Cannot delete remote file {} using method {} due to {}",
                       winpem_path.display(),
                       self.connector.connect_method_name(),
                       err
                )
            }
        };
        thread::sleep(Duration::from_millis(1000));
        match copier.delete_remote_file(&target_name) {
            Ok(_) => {}
            Err(err) => {
                error!("Cannot delete remote file {} using method {} due to {}",
                       target_name.display(),
                       self.connector.connect_method_name(),
                       err
                )
            }
        };
        Ok(())
    }
}