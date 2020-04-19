use std::path::Path;
use crate::remote::{Computer, Connector, Command, PsExec, PsRemote, Rdp, Wmi, CompressCopier, RemoteFileCopier, Compression};
use crate::process_runner::create_report_path;
use std::thread;
use std::time::Duration;
use crate::utils::remote_storage;

pub struct RegistryAcquirer<'a> {
    store_directory: &'a Path,
    connector: Box<dyn Connector>,

    registry_hklm_command: Vec<String>,
    registry_hkcu_command: Vec<String>,
    registry_hkcr_command: Vec<String>,
    registry_hku_command: Vec<String>,
    registry_hkcc_command: Vec<String>,

    compress_timeout: Option<Duration>,
    compression: Compression,
}

impl<'a> RegistryAcquirer<'a> {
    pub fn new(
        store_directory: &'a Path,
        connector: Box<dyn Connector>,
        compress_timeout: Option<Duration>,
        compression: Compression,
    ) -> RegistryAcquirer<'a> {
        RegistryAcquirer {
            store_directory,
            connector,
            registry_hklm_command: vec![
                "reg".to_string(),
                "export".to_string(),
                "HKLM".to_string(),
            ],
            registry_hkcu_command: vec![
                "reg".to_string(),
                "export".to_string(),
                "HKCU".to_string(),
            ],
            registry_hkcr_command: vec![
                "reg".to_string(),
                "export".to_string(),
                "HKCR".to_string(),
            ],
            registry_hku_command: vec![
                "reg".to_string(),
                "export".to_string(),
                "HKU".to_string(),
            ],
            registry_hkcc_command: vec![
                "reg".to_string(),
                "export".to_string(),
                "HKCC".to_string(),
            ],
            compress_timeout,
            compression,
        }
    }

    pub fn psexec(
        store_directory: &'a Path,
        computer: Computer,
        no_7zip: bool,
    ) -> RegistryAcquirer {
        RegistryAcquirer::new(
            store_directory,
            Box::new(PsExec::psexec(computer)),
            None,
            if no_7zip { Compression::No } else { Compression::Yes },
        )
    }

    pub fn psremote(
        store_directory: &'a Path,
        computer: Computer,
        _no_7zip: bool,
    ) -> RegistryAcquirer {
        RegistryAcquirer::new(
            store_directory,
            Box::new(PsRemote::new(computer)),
            None,
            Compression::No,
        )
    }

    pub fn wmi(
        store_directory: &'a Path,
        computer: Computer,
        compress_timeout: Duration,
        no_7zip: bool,
    ) -> RegistryAcquirer {
        RegistryAcquirer::new(
            store_directory,
            Box::new(Wmi { computer }),
            Some(compress_timeout),
            if no_7zip { Compression::No } else { Compression::YesSplit }
        )
    }

    pub fn rdp(
        store_directory: &'a Path,
        computer: Computer,
        compress_timeout: Duration,
        nla: bool,
        no_7zip: bool,
    ) -> RegistryAcquirer {
        RegistryAcquirer::new(
            store_directory,
            Box::new(Rdp { computer, nla }),
            Some(compress_timeout),
            if no_7zip { Compression::No } else { Compression::YesSplit }
        )
    }

    fn run(
        &self,
        command: &[String],
        report_filename_prefix: &str,
    ) {
        if command.is_empty() {
            return;
        }
        let report_path = create_report_path(
            self.connector.computer(),
            self.store_directory,
            report_filename_prefix,
            self.connector.connect_method_name(),
        );

        let remote_report_path = remote_storage()
            .join(report_path.file_name().unwrap())
            .to_string_lossy()
            .to_string();
        let mut command = command.to_vec();
        command.push(remote_report_path.clone());
        command.push("/y".to_string());
        let remote_connection = Command::new(
            command,
            None,
            report_filename_prefix,
            false,
        );

        info!("{}: Checking {}",
              self.connector.connect_method_name(),
              report_filename_prefix.replace("-", " ")
        );

        match self.connector.connect_and_run_command(remote_connection, None) {
            Ok(_) => {}
            Err(err) => {
                error!(
                    "Error running command to acquire {}. Cause: {}",
                    report_filename_prefix,
                    err
                )
            }
        }
        thread::sleep(Duration::from_millis(10_000));

        let _compression_split_copier = CompressCopier::new(self.connector.as_ref(), true, self.compress_timeout.clone());
        let _compression_copier = CompressCopier::new(self.connector.as_ref(), false, self.compress_timeout.clone());
        let copier = match self.compression {
            Compression::No => self.connector.copier(),
            Compression::Yes => &_compression_copier as &dyn RemoteFileCopier,
            Compression::YesSplit => &_compression_split_copier as &dyn RemoteFileCopier,
        };

        match copier.copy_from_remote(Path::new(&remote_report_path), report_path.parent().unwrap()) {
            Ok(_) => {}
            Err(err) => {
                error!("Cannot download {} report from {} using method {} due to {}",
                       report_filename_prefix,
                       self.connector.computer().address,
                       self.connector.connect_method_name(),
                       err
                )
            }
        }
        thread::sleep(Duration::from_secs(2));
        match copier.delete_remote_file(Path::new(&remote_report_path)) {
            Ok(_) => {}
            Err(err) => {
                error!("Cannot delete remote file {} using method {} due to: {}",
                       report_filename_prefix,
                       self.connector.connect_method_name(),
                       err
                )
            }
        }
    }

    pub fn acquire(&self) {
        let command = &self.registry_hklm_command;
        self.run(
            command,
            "registry-hklm",
        );
        let command = &self.registry_hku_command;
        self.run(
            command,
            "registry-hku",
        );
        let command = &self.registry_hkcu_command;
        self.run(
            command,
            "registry-hkcu",
        );
        let command = &self.registry_hkcr_command;
        self.run(
            command,
            "registry-hkcr",
        );
        let command = &self.registry_hkcc_command;
        self.run(
            command,
            "registry-hkcc",
        );
    }
}