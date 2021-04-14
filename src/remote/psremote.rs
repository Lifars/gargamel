use crate::remote::{Connector, Computer, FileCopier, RemoteFileCopier, WindowsRemoteFileHandler, copy_from_remote_wildcards};
use std::path::{Path, PathBuf};
use std::io;
use crate::process_runner::run_process_blocking;

pub struct PsRemote {
    computer: Computer,
    copier_impl: WindowsRemoteFileHandler,
    remote_temp_storage: PathBuf,
}

impl PsRemote {
    pub fn new(computer: Computer, remote_temp_storage: PathBuf, custom_share_folder: Option<String>) -> PsRemote {
        PsRemote {
            computer: computer.clone(),
            copier_impl: WindowsRemoteFileHandler::new(computer, Box::new(Powershell {}), custom_share_folder),
            remote_temp_storage,
        }
    }
}

impl Connector for PsRemote {
    fn connect_method_name(&self) -> &'static str {
        return "PSREM";
    }

    fn computer(&self) -> &Computer {
        &self.computer
    }

    fn copier(&self) -> &dyn RemoteFileCopier {
        &self.copier_impl
    }

    fn remote_temp_storage(&self) -> &Path {
        self.remote_temp_storage.as_path()
    }

    fn prepare_command(&self,
                       command: Vec<String>,
                       output_file_path: Option<&str>,
                       _elevated: bool,
    ) -> Vec<String> {
        let remote_computer = self.computer();
        let program_name = "powershell.exe".to_string();
        let mut prepared_command = vec![
            program_name,
            "-command".to_string(),
            "Invoke-Command".to_string(),
            "-ComputerName".to_string(),
            remote_computer.address.clone(),
            "-ScriptBlock".to_string(),
            "{".to_string(),
        ];
        prepared_command.extend(command);
        let username = remote_computer.username.clone();
        let credential = match &remote_computer.password {
            None => username,
            Some(password) =>
                format!(
                    "(New-Object Management.Automation.PSCredential ('{}', (ConvertTo-SecureString '{}' -AsPlainText -Force)))",
                    username,
                    password
                ),
        };
        prepared_command.push("}".to_string());
        prepared_command.push("-credential".to_string());
        prepared_command.push(credential);
        match output_file_path {
            None => prepared_command,
            Some(output_file_path) => {
                prepared_command.push(">".to_string());
                prepared_command.push(output_file_path.to_string());
                prepared_command
            }
        }
    }
}

impl RemoteFileCopier for PsRemote {
    fn remote_computer(&self) -> &Computer {
        self.computer()
    }

    fn copier_impl(&self) -> &dyn FileCopier {
        self.copier_impl.copier_impl()
    }

    fn path_to_remote_form(&self, path: &Path) -> PathBuf {
        self.copier_impl.path_to_remote_form(path)
    }

    fn copy_to_remote(&self, source: &Path, target: &Path) -> io::Result<()> {
        self.copier_impl.copy_from_remote(source, target)
    }

    fn delete_remote_file(&self, target: &Path) -> io::Result<()> {
        self.copier_impl.delete_remote_file(target)
    }

    fn copy_from_remote(&self, source: &Path, target: &Path) -> io::Result<()> {
        copy_from_remote_wildcards(
            source,
            target,
            self,
            |s, t| self.copier_impl.copy_from_remote(s, t),
        )
    }
}

pub struct Powershell {}

impl FileCopier for Powershell {
    fn copy_file(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        let args = vec![
            "Copy-Item".to_string(),
            format!("'{}'", source.to_string_lossy()),
            format!("'{}'", target.to_string_lossy()),
        ];
        run_process_blocking(
            "powershell.exe",
            &args,
        )
    }

    fn delete_file(&self, target: &Path) -> io::Result<()> {
        let args = vec![
            "Remove-Item".to_string(),
            "-Force".to_string(),
            format!("'{}'", target.to_string_lossy()),
        ];
        run_process_blocking(
            "powershell.exe",
            &args,
        )
    }

    fn method_name(&self) -> &'static str {
        "PSCOPY"
    }
}
