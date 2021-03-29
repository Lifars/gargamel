use crate::remote::{Connector, Computer, Command, PsExec, PsRemote, Rdp, Wmi, SevenZipCompressCopier, RemoteFileCopier, Compression, Local, FileCopier};
use std::path::{Path, PathBuf, StripPrefixError};
use std::{io, thread, fs};
use std::time::Duration;
use crate::process_runner::{create_report_path, run_process_blocking};
use std::io::{ErrorKind, BufRead};
use uuid::Uuid;
use rev_lines::RevLines;
use crate::remote::Compression::No;
use std::cell::RefCell;
use std::ops::Deref;
use std::fmt::format;


pub struct ShadowCopier<'a> {
    connector_impl: &'a dyn Connector,
    pub copier_impl: &'a dyn RemoteFileCopier,
    shadow_drive: PathBuf,
}

impl<'a> ShadowCopier<'a> {
    pub fn new(
        connector_impl: &'a dyn Connector,
        local_store_directory: &Path,
        copier_impl: Option<&'a dyn RemoteFileCopier>,
    ) -> ShadowCopier<'a> {
        let shadow_drive = ShadowCopier::make_shadow_copy(connector_impl, local_store_directory);
        let result = ShadowCopier {
            connector_impl,
            shadow_drive,
            copier_impl: match copier_impl {
                None => connector_impl.copier(),
                Some(copier_impl) => copier_impl
            },
        };
        result
    }

    pub fn make_shadow_copy(
        connector: &dyn Connector,
        local_store_directory: &Path,
    ) -> PathBuf {
        let create_vss_command = Command {
            command: vec![
                "cmd.exe".to_string(),
                "/c".to_string(),
                "powershell.exe".to_string(),
                "-Command".to_string(),
                "(gwmi -list win32_shadowcopy).Create('C:\\','ClientAccessible')".to_string()
            ],
            report_store_directory: None,
            report_filename_prefix: "VSS_RESULT",
            elevated: true,
        };

        let timeout = Some(Duration::from_secs(20));
        if let Err(err) = connector.connect_and_run_command(
            create_vss_command,
            timeout.clone(),
        ) {
            error!("{}", io::Error::new(ErrorKind::InvalidData, "No output from VSS shadow create"));
            return PathBuf::from("C:\\");
        }
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

        let vss_list_output_path = connector.connect_and_run_command(
            list_vss_command,
            timeout.clone(),
        );
        if let Err(err) = vss_list_output_path {
            error!("{}", err);
            return PathBuf::from("C:\\");
        }
        if let None = vss_list_output_path.as_ref().unwrap() {
            error!("{}", io::Error::new(ErrorKind::InvalidData, "No output from VSS shadow create"));
            return PathBuf::from("C:\\");
        }

        let vss_list_output_path = vss_list_output_path.unwrap().unwrap();

        let vss_shadow_volume_path = {
            let vss_list_output_file = fs::File::open(&vss_list_output_path);
            if let Err(err) = vss_list_output_file {
                error!("{}", err);
                return PathBuf::from("C:\\");
            }
            RevLines::new(io::BufReader::new(&vss_list_output_file.unwrap()))
                .unwrap()
                .find(|it| it.contains("Shadow Copy Volume"))
                .map(|shadow_volume_line| {
                    let id_start_idx = shadow_volume_line.find(':').unwrap_or_default();
                    (shadow_volume_line[id_start_idx + 2..]).to_string()
                })
        };
        let _ = fs::remove_file(&vss_list_output_path);
        if let None = vss_shadow_volume_path {
            error!("{}", io::Error::new(ErrorKind::InvalidData, "No output from VSS shadow create"));
            return PathBuf::from("C:\\");
        }
        let vss_shadow_volume_path = vss_shadow_volume_path.unwrap();
        let vss_link_path = connector
            .remote_temp_storage()
            .join(Uuid::new_v4().to_string().replace("-", ""));
        // self.acquire_perms(&vss_link_path);
        let link_vss_command = Command {
            command: vec![
                "cmd.exe".to_string(),
                "/c".to_string(),
                "mklink".to_string(),
                "/d".to_string(),
                vss_link_path.to_string_lossy().to_string(),
                format!("{}\\", vss_shadow_volume_path.to_string()),
            ],
            report_store_directory: Some(local_store_directory),
            report_filename_prefix: "",
            elevated: true,
        };

        connector.acquire_perms(&vss_link_path);
        if let Err(err) = connector.connect_and_run_command(
            link_vss_command,
            timeout.clone(),
        ) {
            error!("{}", io::Error::new(ErrorKind::InvalidData, "No output from VSS shadow create"));
            return PathBuf::from("C:\\");
        }
        vss_link_path
    }
}

impl Drop for ShadowCopier<'_> {
    fn drop(&mut self) {
        delete_shadow_copy(self.connector_impl, self.shadow_drive.as_path())
    }
}

impl<'a> RemoteFileCopier for ShadowCopier<'a> {
    fn remote_computer(&self) -> &Computer {
        self.copier_impl.remote_computer()
    }

    fn copier_impl(&self) -> &dyn FileCopier {
        self.copier_impl.copier_impl()
    }

    fn path_to_remote_form(
        &self,
        path: &Path,
    ) -> PathBuf {
        self.copier_impl.path_to_remote_form(path)
    }

    fn copy_to_remote(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        self.copier_impl.copy_to_remote(source, target)
    }

    fn delete_remote_file(&self, target: &Path) -> io::Result<()> {
        self.copier_impl.delete_remote_file(target)
    }

    fn copy_from_remote(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        let without_c_drive = source.strip_prefix("C:\\");
        let new_source = match without_c_drive {
            Ok(without_c_drive) => self.shadow_drive.join(without_c_drive),
            Err(err) => {
                error!("{}", err);
                source.to_path_buf()
            }
        };
        self.copier_impl.copy_from_remote(&new_source, target)
    }
}

fn delete_shadow_copy(
    connector: &dyn Connector,
    shadow_path: &Path,
) {
    connector.release_perms(shadow_path);
    let unlink_vss_command = Command {
        command: vec![
            "cmd.exe".to_string(),
            "/c".to_string(),
            "rmdir".to_string(),
            shadow_path.to_string_lossy().to_string(),
        ],
        report_store_directory: None,
        report_filename_prefix: "",
        elevated: true,
    };
    if let Err(err) = connector.connect_and_run_command(
        unlink_vss_command,
        Some(Duration::from_secs(1)),
    ) {
        error!("{}", err)
    }
}