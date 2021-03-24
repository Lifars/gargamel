use crate::remote::{Connector, Computer, Command, PsExec, PsRemote, Rdp, Wmi, CompressCopier, RemoteFileCopier, Compression, Local};
use std::path::{Path, PathBuf};
use std::{io, thread, fs};
use std::time::Duration;
use crate::process_runner::create_report_path;
use std::io::{ErrorKind, BufRead};
use uuid::Uuid;
use rev_lines::RevLines;


pub struct EdbAcquirer<'a> {
    pub local_store_directory: &'a Path,
    pub connector: Box<dyn Connector>,
    pub image_timeout: Option<Duration>,
    pub compress_timeout: Option<Duration>,
    pub compression: Compression,
}

impl<'a> EdbAcquirer<'a> {
    pub fn psexec32(
        remote_computer: Computer,
        local_store_directory: &'a Path,
        no_7zip: bool,
        remote_temp_storage: PathBuf,
        custom_share_folder: Option<String>
    ) -> EdbAcquirer<'a> {
        EdbAcquirer {
            local_store_directory,
            connector: Box::new(PsExec::psexec32(remote_computer, remote_temp_storage, custom_share_folder)),
            image_timeout: Some(Duration::from_secs(20)),
            compress_timeout: None,
            compression: if no_7zip { Compression::No } else { Compression::Yes },
        }
    }

    pub fn local(
        local_store_directory: &'a Path,
    ) -> EdbAcquirer<'a> {
        EdbAcquirer {
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
        custom_share_folder: Option<String>
    ) -> EdbAcquirer<'a> {
        EdbAcquirer {
            local_store_directory,
            connector: Box::new(PsExec::psexec64(remote_computer, remote_temp_storage, custom_share_folder)),
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
        custom_share_folder: Option<String>
    ) -> EdbAcquirer<'a> {
        EdbAcquirer {
            local_store_directory,
            connector: Box::new(PsRemote::new(remote_computer, remote_temp_storage, custom_share_folder)),
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
    ) -> EdbAcquirer<'a> {
        EdbAcquirer {
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
    ) -> EdbAcquirer<'a> {
        EdbAcquirer {
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


    pub fn download_edb(
        &self
    ) -> io::Result<()> {
        let local_store_directory = self.local_store_directory;

        let create_vss_command = Command {
            command: vec![
                "cmd.exe".to_string(),
                "/c".to_string(),
                "powershell.exe".to_string(),
                "-Command".to_string(),
                "\"(gwmi -list win32_shadowcopy).Create('C:\','ClientAccessible')\"".to_string()
            ],
            report_store_directory: None,
            report_filename_prefix: "VSS_RESULT",
            elevated: true,
        };

        self.connector.connect_and_run_local_program_in_current_directory(
            create_vss_command,
            self.image_timeout,
        )?;

        let list_vss_command = Command {
            command: vec![
                "cmd.exe".to_string(),
                "/c".to_string(),
                "vssadmin".to_string(),
                "list".to_string(),
                "shadows".to_string()
            ],
            report_store_directory: Some(local_store_directory),
            report_filename_prefix: "VSS_RESULT",
            elevated: true,
        };

        let vss_list_output_path = self.connector.connect_and_run_command(
            list_vss_command,
            self.image_timeout,
        )?;
        if let None = vss_list_output_path {
            return Err(io::Error::new(ErrorKind::InvalidData, "No output from VSS shadow create"));
        }

        let vss_list_output_path = vss_list_output_path.unwrap();

        let vss_shadow_volume_path = {
            let vss_list_output_file = fs::File::open(&vss_list_output_path)?;
            RevLines::new(io::BufReader::new(&vss_list_output_file))
                .unwrap()
                .find(|it| it.contains("Shadow Copy Volume"))
                .map(|shadow_volume_line| {
                    let id_start_idx = shadow_volume_line.find(':').unwrap_or_default();
                    (shadow_volume_line[id_start_idx + 2..]).to_string()
                })
        };
        let _ = fs::remove_file(&vss_list_output_path);
        if let None = vss_shadow_volume_path {
            return Err(io::Error::new(ErrorKind::InvalidData, "No output from VSS shadow list"));
        }
        let vss_shadow_volume_path = vss_shadow_volume_path.unwrap();
        let vss_link_path = format!("C:\\{}", Uuid::new_v4().to_string().replace("-", ""));
        let link_vss_command = Command {
            command: vec![
                "cmd.exe".to_string(),
                "/c".to_string(),
                "mklink".to_string(),
                "/d".to_string(),
                vss_link_path.clone(),
                vss_shadow_volume_path
            ],
            report_store_directory: Some(local_store_directory),
            report_filename_prefix: "",
            elevated: true,
        };
        self.connector.connect_and_run_command(
            link_vss_command,
            self.image_timeout,
        )?;


        let _copier = self.connector.copier();
        let _compression_split_copier = CompressCopier::new(self.connector.as_ref(), true, self.compress_timeout, false);
        let _compression_copier = CompressCopier::new(self.connector.as_ref(), false, self.compress_timeout, false);
        let copier = match self.compression {
            Compression::No => _copier,
            Compression::Yes => &_compression_copier as &dyn RemoteFileCopier,
            Compression::YesSplit => &_compression_split_copier as &dyn RemoteFileCopier,
        };
        let edb_path = Path::new(&vss_link_path)
            .join(Path::new("ProgramData\\Microsoft\\Search\\Data\\Applications\\Windows\\Windows.edb"));

        match copier.copy_from_remote(
            &edb_path,
            &local_store_directory,
            // &self.local_store_directory.join(target_name.file_name().unwrap()),
        ) {
            Ok(_) => {}
            Err(err) => {
                error!("Cannot download {} report from {} using method {} due to {}",
                       &edb_path.display(),
                       self.connector.computer().address,
                       self.connector.connect_method_name(),
                       err
                )
            }
        }
        thread::sleep(Duration::from_millis(20000));
        let unlink_vss_command = Command {
            command: vec![
                "cmd.exe".to_string(),
                "/c".to_string(),
                "rmdir".to_string(),
                vss_link_path.clone(),
            ],
            report_store_directory: Some(local_store_directory),
            report_filename_prefix: "",
            elevated: true,
        };
        self.connector.connect_and_run_command(
            unlink_vss_command,
            self.image_timeout,
        )?;
        Ok(())
    }
}