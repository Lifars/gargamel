use crate::remote::{Connector, Computer, FileCopier, RemoteFileCopier};
use std::path::{Path, PathBuf};
use std::io;
use crate::process_runner::{run_process_blocking_maybe_timed, run_process_blocking_timed};
use std::time::Duration;

pub struct Wmi {
    pub computer: Computer,
    pub remote_temp_storage: PathBuf
}

impl Connector for Wmi {
    fn connect_method_name(&self) -> &'static str {
        "WMI"
    }

    fn computer(&self) -> &Computer {
        &self.computer
    }

    fn copier(&self) -> &dyn RemoteFileCopier {
        self as &dyn RemoteFileCopier
    }

    fn remote_temp_storage(&self) -> &Path {
        self.remote_temp_storage.as_path()
    }

    fn prepare_command(&self,
                       command: Vec<String>,
                       output_file_path: Option<String>,
                       _elevated: bool,
    ) -> Vec<String> {
        let remote_computer = self.remote_computer();
        let program_name = "powershell.exe".to_string();
        // let wmi_implant = std::env::current_dir()
        //     .expect("Cannot get current working directory")
        //     .join("WMImplant.ps1")
        //     .to_string_lossy().to_string();
        let command_joined: String = command.join(" ");
        let mut prepared_command = vec![
            program_name,
            "-File".to_string(),
            "WMImplant.ps1".to_string(),
            "-ComputerName".to_string(),
            remote_computer.address.clone(),
            "-RemoteUser".to_string(),
            remote_computer.domain_username(),
            "-CommandExec".to_string(),
            "-RemoteCommand".to_string(),
            format!("{}", command_joined),
        ];

        if let Some(password) = &remote_computer.password {
            prepared_command.push("-RemotePass".to_string());
            prepared_command.push(password.clone());
        }
        match output_file_path {
            None => prepared_command,
            Some(output_file_path) => {
                prepared_command.push(">".to_string());
                prepared_command.push(output_file_path);
                prepared_command
            }
        }
    }
}

impl Wmi {
    fn copy_impl(&self,
                 source: &Path,
                 target: &Path,
                 method_name: &str,
                 target_is_remote: bool,
    ) -> io::Result<()>{
        let remote_computer = self.remote_computer();

        let target = match source.file_name() {
            None => target.to_path_buf(),
            Some(file_name) => target.join(file_name),
        };

        let mut prepared_command = vec![
            "-File".to_string(),
            "WMImplant.ps1".to_string(),
            method_name.to_string(),
            "-ComputerName".to_string(),
            remote_computer.address.clone(),
            "-RemoteUser".to_string(),
            remote_computer.domain_username()
        ];

        if target_is_remote {
            prepared_command.push("-RemoteFile".to_string());
            prepared_command.push(target.to_string_lossy().to_string());
            prepared_command.push("-LocalFile".to_string());
            prepared_command.push(source.to_string_lossy().to_string());
        } else {
            prepared_command.push("-RemoteFile".to_string());
            prepared_command.push(source.to_string_lossy().to_string());
            prepared_command.push("-LocalFile".to_string());
            prepared_command.push(target.to_string_lossy().to_string());
        }

        if let Some(password) = &remote_computer.password {
            prepared_command.push("-RemotePass".to_string());
            prepared_command.push(password.clone());
        }

        run_process_blocking_maybe_timed(
            "powershell.exe",
            &prepared_command,
            None
        )
    }
}

impl FileCopier for Wmi {
    fn copy_file(&self, source: &Path, target: &Path) -> io::Result<()> {
        self.copy_impl(source, target, "-Copy", true)
    }

    fn delete_file(&self, target: &Path) -> io::Result<()> {
        let remote_computer = self.remote_computer();

        let mut prepared_command = vec![
            "-File".to_string(),
            "WMImplant.ps1".to_string(),
            "-Delete".to_string(),
            "-LocalFile".to_string(),
            target.to_string_lossy().to_string(),
            "-ComputerName".to_string(),
            remote_computer.address.clone(),
            "-RemoteUser".to_string(),
            remote_computer.domain_username()
        ];

        if let Some(password) = &remote_computer.password {
            prepared_command.push("-RemotePass".to_string());
            prepared_command.push(password.clone());
        }

        run_process_blocking_timed(
            "powershell.exe",
            &prepared_command,
            Duration::from_secs(10)
        )
    }

    fn method_name(&self) -> &'static str {
        self.connect_method_name()
    }
}

impl RemoteFileCopier for Wmi {
    fn remote_computer(&self) -> &Computer {
        self.computer()
    }

    fn copier_impl(&self) -> &dyn FileCopier {
        self as &dyn FileCopier
    }

    fn path_to_remote_form(&self, path: &Path) -> PathBuf {
        path.to_path_buf()
    }

    fn copy_to_remote(&self, source: &Path, target: &Path) -> io::Result<()> {
        self.copy_impl(source, target, "-Upload", true)
    }

    fn delete_remote_file(&self, target: &Path) -> io::Result<()> {
        self.delete_file(target)
    }

    fn copy_from_remote(&self, source: &Path, target: &Path) -> io::Result<()> {
        self.copy_impl(source, target, "-Download", false)
    }
}