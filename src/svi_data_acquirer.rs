use crate::remote::{Connector, Computer, Command, PsExec, PsRemote, Rdp, Wmi, CompressCopier, RemoteFileCopier, Compression, Local, RevShareConnector};
use std::path::{Path, PathBuf};
use std::{io, thread, fs};
use std::time::Duration;
use crate::process_runner::create_report_path;
use std::io::{ErrorKind, BufRead};
use uuid::Uuid;
use rev_lines::RevLines;


pub struct SystemVolumeInformationAcquirer<'a> {
    pub local_store_directory: &'a Path,
    pub connector: Box<dyn Connector>,
    pub image_timeout: Option<Duration>,
    pub compress_timeout: Option<Duration>,
    pub compression: Compression,
}

impl<'a> SystemVolumeInformationAcquirer<'a> {
    pub fn psexec32(
        remote_computer: Computer,
        local_store_directory: &'a Path,
        no_7zip: bool,
        remote_temp_storage: PathBuf,
        custom_share_folder: Option<String>,
        reverse: bool,
    ) -> SystemVolumeInformationAcquirer<'a> {
        let connector = Box::new(PsExec::psexec32(remote_computer, remote_temp_storage, custom_share_folder));
        SystemVolumeInformationAcquirer {
            local_store_directory,
            connector: if reverse { Box::new(RevShareConnector::new(connector)) } else { connector },
            image_timeout: Some(Duration::from_secs(20)),
            compress_timeout: None,
            compression: if no_7zip { Compression::No } else { Compression::Yes },
        }
    }

    pub fn local(
        local_store_directory: &'a Path,
    ) -> SystemVolumeInformationAcquirer<'a> {
        SystemVolumeInformationAcquirer {
            local_store_directory,
            connector: Box::new(Local::new()),
            image_timeout: Some(Duration::from_secs(20)),
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
    ) -> SystemVolumeInformationAcquirer<'a> {
        let connector = Box::new(PsExec::psexec64(remote_computer, remote_temp_storage, custom_share_folder));
        SystemVolumeInformationAcquirer {
            local_store_directory,
            connector: if reverse { Box::new(RevShareConnector::new(connector)) } else { connector },
            image_timeout: Some(Duration::from_secs(20)),
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
    ) -> SystemVolumeInformationAcquirer<'a> {
        let connector = Box::new(PsRemote::new(remote_computer, remote_temp_storage, custom_share_folder));
        SystemVolumeInformationAcquirer {
            local_store_directory,
            connector: if reverse { Box::new(RevShareConnector::new(connector)) } else { connector },
            image_timeout: Some(Duration::from_secs(20)),
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
    ) -> SystemVolumeInformationAcquirer<'a> {
        SystemVolumeInformationAcquirer {
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
    ) -> SystemVolumeInformationAcquirer<'a> {
        SystemVolumeInformationAcquirer {
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

    fn acquire_perms(&self) {
        debug!("Acquiring SVI ownership");
        let grant_svi = Command {
            command: vec![
                "cmd.exe".to_string(),
                "/c".to_string(),
                "icacls.exe".to_string(),
                "C:\\System Volume Information".to_string(),
                "/grant".to_string(),
                format!("{}:F", self.connector.computer().username)
            ],
            report_store_directory: None,
            report_filename_prefix: "GRANT_VSI",
            elevated: true,
        };

        if let Err(err) = self.connector.connect_and_run_command(
            grant_svi,
            None,
        ) {
            warn!("Cannot acquire SVI ownership: {}", err)
        }
        thread::sleep(Duration::from_secs(5));
    }

    fn release_perms(&self) {
        thread::sleep(Duration::from_secs(5));
        debug!("Releasing SVI ownership");
        let grant_svi = Command {
            command: vec![
                "cmd.exe".to_string(),
                "/c".to_string(),
                "icacls.exe".to_string(),
                "C:\\System Volume Information".to_string(),
                "/deny".to_string(),
                format!("{}:F", self.connector.computer().username)
            ],
            report_store_directory: None,
            report_filename_prefix: "DENY_VSI",
            elevated: true,
        };

        if let Err(err) = self.connector.connect_and_run_command(
            grant_svi,
            None,
        ) {
            warn!("Cannot release SVI ownership: {}", err)
        }
    }

    pub fn download_data(
        &self
    ) -> io::Result<()> {
        let local_store_directory = self.local_store_directory;
        self.acquire_perms();

        let _copier = self.connector.copier();
        let _compression_split_copier = CompressCopier::new(self.connector.as_ref(), true, self.compress_timeout, false);
        let _compression_copier = CompressCopier::new(self.connector.as_ref(), false, self.compress_timeout, false);
        let copier = match self.compression {
            Compression::No => _copier,
            Compression::Yes => &_compression_copier as &dyn RemoteFileCopier,
            Compression::YesSplit => &_compression_split_copier as &dyn RemoteFileCopier,
        };
        let svi_path = Path::new("C:\\System Volume Information\\*.lnk");

        if let Err(err) = copier.copy_from_remote(
            &svi_path,
            &local_store_directory,
            // &self.local_store_directory.join(target_name.file_name().unwrap()),
        ) {
            error!("Cannot download {} from {} using method {} due to {}",
                   &svi_path.display(),
                   self.connector.computer().address,
                   self.connector.connect_method_name(),
                   err
            );
        }
        thread::sleep(Duration::from_millis(20000));

        self.release_perms();
        Ok(())
    }
}