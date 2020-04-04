use std::path::{Path, PathBuf};
use crate::remote::{Computer, Connector, Command, PsExec, PsRemote, Local, Wmi, Ssh, Rdp, Copier, RemoteCopier, XCopy, PsCopyItem, WindowsRemoteCopier, RdpCopy};
use uuid::Uuid;
use crate::process_runner::create_report_path;
use std::io::Error;
use std::thread;
use std::time::Duration;

pub struct RegistryAcquirer<'a> {
    remote_computer: &'a Computer,
    store_directory: &'a Path,
    connector: Box<dyn Connector>,
    copier: Box<dyn RemoteCopier>,

    registry_hklm_command: Vec<String>,
    registry_hkcu_command: Vec<String>,
    registry_hkcr_command: Vec<String>,
    registry_hku_command: Vec<String>,
    registry_hkcc_command: Vec<String>,
}

impl<'a> RegistryAcquirer<'a> {
    fn new_standard_acquirer(
        remote_computer: &'a Computer,
        store_directory: &'a Path,
        connector: Box<dyn Connector>,
        copier: Box<dyn RemoteCopier>,
    ) -> RegistryAcquirer<'a> {
        RegistryAcquirer {
            remote_computer,
            store_directory,
            connector,
            copier,
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
        }
    }

    pub fn psexec(
        remote_computer: &'a Computer,
        store_directory: &'a Path,
    ) -> RegistryAcquirer<'a> {
        RegistryAcquirer::new_standard_acquirer(
            remote_computer,
            store_directory,
            Box::new(PsExec {}),
            Box::new(
                WindowsRemoteCopier::new(
                    remote_computer.clone(),
                    Box::new(XCopy {}),
                )
            ),
        )
    }

    pub fn psremote(
        remote_computer: &'a Computer,
        store_directory: &'a Path,
    ) -> RegistryAcquirer<'a> {
        RegistryAcquirer::new_standard_acquirer(
            remote_computer,
            store_directory,
            Box::new(PsRemote {}),
            Box::new(WindowsRemoteCopier::new(
                remote_computer.clone(),
                Box::new(PsCopyItem {}),
            )),
        )
    }

    pub fn local(
        remote_computer: &'a Computer,
        store_directory: &'a Path,
    ) -> RegistryAcquirer<'a> {
        RegistryAcquirer::new_standard_acquirer(
            remote_computer,
            store_directory,
            Box::new(Local::new()),
            Box::new(Local::new()),
        )
    }

    pub fn rdp(
        remote_computer: &'a Computer,
        store_directory: &'a Path,
    ) -> RegistryAcquirer<'a> {
        RegistryAcquirer::new_standard_acquirer(
            remote_computer,
            store_directory,
            Box::new(Rdp {}),
            Box::new(RdpCopy { computer: remote_computer.clone() }),
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
            self.remote_computer,
            self.store_directory,
            report_filename_prefix,
            self.connector.connect_method_name(),
        );

        let remote_report_path = format!("C:\\Users\\Public\\{}", report_path.file_name().unwrap().to_string_lossy());
        let mut command = command.to_vec();
        command.push(remote_report_path.clone());
        command.push("/y".to_string());
        let remote_connection = Command::new(
            &self.remote_computer,
            command,
            None,
            report_filename_prefix,
        );

        info!("{}: Checking {}",
              self.connector.connect_method_name(),
              report_filename_prefix.replace("_", " ")
        );

        match self.connector.connect_and_run_command(remote_connection) {
            Ok(_) => {}
            Err(err) => {
                error!(
                    "Error running command to acquire {}. Cause: {}",
                    report_filename_prefix,
                    err
                )
            }
        }
        thread::sleep(Duration::from_millis(1500));
        match self.copier.copy_from_remote(Path::new(&remote_report_path), report_path.parent().unwrap()) {
            Ok(_) => {}
            Err(err) => {
                error!("Cannot download {} report from {} using method {}",
                       report_filename_prefix,
                       self.remote_computer.address,
                       self.connector.connect_method_name()
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
        let command = &self.registry_hkcu_command;
        self.run(
            command,
            "registry-hkcu",
        );
        let command = &self.registry_hku_command;
        self.run(
            command,
            "registry-hku",
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