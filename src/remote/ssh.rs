use crate::remote::{Connector, Computer, Command, FileCopier, RemoteFileCopier};
use std::io;
use crate::process_runner::{create_report_path, run_piped_processes_blocking};
use std::fs::File;
use std::path::{PathBuf, Path};
use std::time::Duration;

pub struct Ssh {
    pub computer: Computer,
    pub key_file: Option<PathBuf>
}

impl Connector for Ssh {
    fn connect_method_name(&self) -> &'static str {
        "SSH"
    }

    fn computer(&self) -> &Computer {
        &self.computer
    }

    fn copier(&self) -> &dyn RemoteFileCopier {
        self as &dyn RemoteFileCopier
    }

    fn remote_temp_storage(&self) -> &Path {
        Path::new("/tmp")
    }

    fn connect_and_run_command(&self,
                               remote_connection: Command<'_>,
                               _timeout: Option<Duration>
    ) -> io::Result<()> {
        debug!("Trying to run command {:?} on {}",
               remote_connection.command,
               &self.computer().address
        );
        let output_file_path = match remote_connection.report_store_directory {
            None => None,
            Some(store_directory) => {
                let file_path = create_report_path(
                    &self.computer(),
                    store_directory,
                    &remote_connection.report_filename_prefix,
                    self.connect_method_name(),
                    "txt"
                );
                {
                    File::create(&file_path)?;
                }
                Some(dunce::canonicalize(file_path)?.to_str().unwrap().to_string())
            }
        };

        let echo = vec!["echo".to_string(), "n".to_string()];
        let prepared_echo = self.prepare_remote_process(echo);

        let processed_command = self.prepare_command(
            remote_connection.command,
            output_file_path,
            false
        );
        let prepared_command = self.prepare_remote_process(processed_command);
        run_piped_processes_blocking(
            "cmd.exe",
            &prepared_echo,
            "cmd.exe",
            &prepared_command)
    }

    fn prepare_command(&self,
                       command: Vec<String>,
                       output_file_path: Option<String>,
                       elevated: bool,
    ) -> Vec<String> {
        let remote_computer = self.remote_computer();
        let program_name = "plink.exe".to_string();
        let mut prepared_command = vec![
            program_name,
            "-ssh".to_string(),
            remote_computer.address.clone(),
            "-l".to_string(),
            remote_computer.username.clone(),
            "-no-antispoof".to_string()
        ];
        if let Some(password) = &remote_computer.password {
            prepared_command.push("-pw".to_string());
            prepared_command.push(password.clone());
        }

        if let Some(key_file) = &self.key_file {
            prepared_command.push("-i".to_string());
            prepared_command.push(key_file.to_string_lossy().to_string())
        }
        if elevated {
            if let Some(password) = &remote_computer.password {
                prepared_command.push(format!("echo {} | sudo -S {}", password, command.join(" ")));
            } else {
                prepared_command.push(format!("sudo -S {}", command.join(" ")));
            }
        }else {
            prepared_command.push(command.join(" "));
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

impl FileCopier for Ssh {
    fn copy_file(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        let mut scp = vec![
            "-l".to_string(),
            self.computer.username.clone(),
        ];
        if let Some(password) = &self.computer.password {
            scp.push("-pw".to_string());
            scp.push(password.clone());
        }
        if let Some(key_file) = &self.key_file {
            scp.push("-i".to_string());
            scp.push(key_file.to_string_lossy().to_string())
        }
        scp.push(format!("{}", source.to_string_lossy()));
        scp.push(format!("{}", target.to_string_lossy()));
        run_piped_processes_blocking(
            "cmd",
            &[
                "/c".to_string(),
                "echo".to_string(),
                "n".to_string()
            ],
            "pscp.exe",
            &scp,
        )
    }

    fn delete_file(&self, target: &Path) -> io::Result<()> {
        let mut params = vec![
            "-ssh".to_string(),
            self.computer.address.clone(),
            "-l".to_string(),
            self.computer.username.clone(),
            "-no-antispoof".to_string()
        ];
        if let Some(password) = &self.computer.password {
            params.push("-pw".to_string());
            params.push(password.clone());
        }
        if let Some(key_file) = &self.key_file {
            params.push("-i".to_string());
            params.push(key_file.to_string_lossy().to_string())
        }
        params.push("rm".to_string());
        params.push("-f".to_string());
        params.push(target.to_string_lossy().to_string());
        run_piped_processes_blocking(
            "cmd",
            &[
                "/c".to_string(),
                "echo".to_string(),
                "n".to_string()
            ],
            "plink.exe",
            &params,
        )
    }

    fn method_name(&self) -> &'static str {
        "SCP"
    }
}

impl RemoteFileCopier for Ssh {
    fn remote_computer(&self) -> &Computer {
        &self.computer
    }

    fn copier_impl(&self) -> &dyn FileCopier {
        self as &dyn FileCopier
    }

    fn path_to_remote_form(&self, path: &Path) -> PathBuf {
        PathBuf::from(format!(
            "{}:{}",
            self.remote_computer().address,
            path.to_str().unwrap()
        ))
    }
}
