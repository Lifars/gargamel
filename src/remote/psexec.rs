use crate::remote::{Connector, Computer, Command, RemoteFileCopier, Cmd, WindowsRemoteFileHandler};
use std::time::Duration;
use std::io::Error;
use std::path::{PathBuf, Path};

pub struct PsExec {
    computer: Computer,
    copier: WindowsRemoteFileHandler,
    psexec_name: String,
    remote_temp_storage: PathBuf
}

impl PsExec {
    pub fn paexec(computer: Computer, remote_temp_storage: PathBuf) -> PsExec {
        PsExec {
            computer: computer.clone(),
            copier: WindowsRemoteFileHandler::new(computer, Box::new(Cmd {})),
            psexec_name: "paexec.exe".to_string(),
            remote_temp_storage
        }
    }

    pub fn psexec(computer: Computer, remote_temp_storage: PathBuf) -> PsExec {
        PsExec {
            computer: computer.clone(),
            copier: WindowsRemoteFileHandler::new(computer, Box::new(Cmd {})),
            psexec_name: "PsExec64.exe".to_string(),
            remote_temp_storage
        }
    }
}

impl Connector for PsExec {
    fn connect_method_name(&self) -> &'static str {
        return "PSEXEC";
    }

    fn computer(&self) -> &Computer {
        &self.computer
    }

    fn copier(&self) -> &dyn RemoteFileCopier {
        &self.copier
    }

    fn remote_temp_storage(&self) -> &Path {
        self.remote_temp_storage.as_path()
    }

    fn connect_and_run_local_program(&self,
                                     command_to_run: Command<'_>,
                                     timeout: Option<Duration>
    ) -> Result<(), Error> {
        let mut command = command_to_run.command;
        command.insert(0, "-c".to_string());
        command.insert(0, "-f".to_string());
        let command_to_run = Command {
            command,
            ..command_to_run
        };
        self.connect_and_run_command(command_to_run, timeout)
    }

    fn prepare_command(&self,
                       command: Vec<String>,
                       output_file_path: Option<String>,
                       elevated: bool,
    ) -> Vec<String> {
        let remote_computer = self.computer();
        let address = format!("\\\\{}", remote_computer.address);
        let program_name = self.psexec_name.clone();
        let mut prepared_command = vec![
            program_name,
            address,
            "-u".to_string(),
            remote_computer.domain_username(),
        ];
        if let Some(password) = &remote_computer.password {
            prepared_command.push("-p".to_string());
            prepared_command.push(password.clone());
        }
        if elevated {
            prepared_command.push("-h".to_string());
        }
        prepared_command.extend(command.into_iter());
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
