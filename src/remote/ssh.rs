use crate::remote::{Connector, Computer, Command, Copier, RemoteCopier};
use std::io;
use crate::process_runner::{create_report_path, run_piped_processes_blocking};
use std::fs::File;
use std::path::{PathBuf, Path};

pub struct Ssh{
    pub key_file: Option<PathBuf>
}

impl Connector for Ssh {
    fn connect_method_name(&self) -> &'static str {
        "SSH"
    }

    fn connect_and_run_command(&self, remote_connection: Command<'_>) -> io::Result<()> {
        debug!("Trying to run command {:?} on {}",
               remote_connection.command,
               remote_connection.remote_computer.address
        );
        let output_file_path = match remote_connection.store_directory {
            None => None,
            Some(store_directory) => {
                let file_path = create_report_path(
                    &remote_connection.remote_computer,
                    store_directory,
                    &remote_connection.report_filename_prefix,
                    self.connect_method_name(),
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
            remote_connection.remote_computer,
            remote_connection.command,
            output_file_path,
        );
        let prepared_command = self.prepare_remote_process(processed_command);
        run_piped_processes_blocking(
            &prepared_echo.program_path,
            &prepared_echo.all_program_args,
            &prepared_command.program_path,
            &prepared_command.all_program_args)
    }

    fn prepare_command(&self,
                       remote_computer: &Computer,
                       command: Vec<String>,
                       output_file_path: Option<String>
    ) -> Vec<String> {
        let program_name = "plink.exe".to_string();
        let mut prefix = vec![
            program_name,
            "-ssh".to_string(),
            remote_computer.address.clone(),
            "-l".to_string(),
            remote_computer.username.clone(),
            "-pw".to_string(),
            remote_computer.password.clone(),
            "-no-antispoof".to_string()
        ];
        if self.key_file.is_some() {
            prefix.push("-i".to_string());
            prefix.push(self.key_file.as_ref().unwrap().to_string_lossy().to_string())
        }
        let almost_result = prefix.into_iter()
            .chain(command.into_iter());
        match output_file_path {
            None => almost_result.collect(),
            Some(output_file_path) =>
                almost_result
                    .chain(vec![
                        ">".to_string(),
                        output_file_path
                    ]).collect()
        }
    }
}

pub struct Scp {
    pub computer: Computer,
    pub key_file: Option<PathBuf>,
}

impl Copier for Scp {
    fn copy_file(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        let mut scp = vec![
            "-l".to_string(),
            self.computer.username.clone(),
            "-pw".to_string(),
            self.computer.password.clone(),
        ];
        if self.key_file.is_some() {
            scp.push("-i".to_string());
            scp.push(self.key_file.as_ref().unwrap().to_string_lossy().to_string())
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
            "-pw".to_string(),
            self.computer.password.clone(),
            "-no-antispoof".to_string()
        ];
        if self.key_file.is_some() {
            params.push("-i".to_string());
            params.push(self.key_file.as_ref().unwrap().to_string_lossy().to_string())
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

impl RemoteCopier for Scp {
    fn computer(&self) -> &Computer {
        &self.computer
    }

    fn copier_impl(&self) -> &dyn Copier {
        self as &dyn Copier
    }

    fn path_to_remote_form(&self, path: &Path) -> PathBuf {
        PathBuf::from(format!(
            "{}:{}",
            self.computer().address,
            path.to_str().unwrap()
        ))
    }
}
