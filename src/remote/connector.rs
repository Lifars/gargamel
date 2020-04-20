use std::io::Result;
use crate::process_runner::{run_process_blocking, create_report_path, run_process_blocking_timed};
use std::{iter, thread};
use std::path::Path;
use crate::arg_parser::Opts;
use std::time::Duration;
use crate::remote::RemoteFileCopier;

#[derive(Clone)]
pub struct Computer {
    pub address: String,
    pub username: String,
    pub domain: Option<String>,
    pub password: Option<String>,
}

impl Computer {
    pub fn domain_username(&self) -> String {
        match &self.domain {
            None =>
                self.username.clone(),
            Some(domain) =>
                format!("{}\\{}", domain, self.username),
        }
    }
}

pub struct Command<'a> {
    pub command: Vec<String>,
    pub report_store_directory: Option<&'a Path>,
    pub report_filename_prefix: &'a str,
    pub elevated: bool,
}

impl From<Opts> for Computer {
    fn from(opts: Opts) -> Self {
        Computer {
            address: opts.computer,
            username: opts.user,
            domain: opts.domain,
            password: opts.password,
        }
    }
}

impl<'a> Command<'a> {
    pub fn new(
        command: Vec<String>,
        store_directory: Option<&'a Path>,
        report_filename_prefix: &'a str,
        elevated: bool,
    ) -> Command<'a> {
        Command {
            command,
            report_store_directory: store_directory,
            report_filename_prefix,
            elevated,
        }
    }
}

pub trait Connector {
    fn connect_method_name(&self) -> &'static str;

    fn computer(&self) -> &Computer;

    fn copier(&self) -> &dyn RemoteFileCopier;

    fn remote_temp_storage(&self) -> &Path;

    fn connect_and_run_local_program_in_current_directory(
        &self,
        command_to_run: Command<'_>,
        timeout: Option<Duration>
    ) -> Result<()> {
        let mut command = command_to_run.command;
        command[0] = std::env::current_dir().unwrap()
            .join(Path::new(&command[0]).file_name().unwrap())
            .to_string_lossy().to_string();
        let command_to_run = Command {
            command,
            ..command_to_run
        };
        self.connect_and_run_local_program(
            command_to_run,
            timeout
        )
    }

    fn connect_and_run_local_program(
        &self,
        command_to_run: Command<'_>,
        timeout: Option<Duration>
    ) -> Result<()> {
        let local_program_path = Path::new(command_to_run.command.first().unwrap());
        let remote_storage = self.remote_temp_storage();
        let copier = self.copier();
        copier.copy_to_remote(&local_program_path, &remote_storage)?;
        thread::sleep(Duration::from_millis(20_000));
        let remote_program_path = remote_storage.join(local_program_path
            .file_name()
            .expect(&format!("Must specify file instead of {}", local_program_path.display())
            )
        );
        let mut command = command_to_run.command;
        command[0] = remote_program_path.to_string_lossy().to_string();
        let command_to_run = Command {
            command,
            ..command_to_run
        };
        self.connect_and_run_command(command_to_run, timeout)?;
        thread::sleep(Duration::from_millis(10_000));
        copier.delete_remote_file(&remote_program_path)
    }

    fn connect_and_run_command(
        &self,
        command_to_run: Command<'_>,
        timeout: Option<Duration>
    ) -> Result<()> {
        debug!("Trying to run command {:?} on {}",
               command_to_run.command,
               &self.computer().address
        );
        let output_file_path = match command_to_run.report_store_directory {
            None => None,
            Some(store_directory) => {
                let file_path = create_report_path(
                    self.computer(),
                    store_directory,
                    &command_to_run.report_filename_prefix,
                    self.connect_method_name(),
                    "txt"
                );
                Some(file_path.to_str().unwrap().to_string())
            }
        };

        let processed_command = self.prepare_command(
            command_to_run.command,
            output_file_path,
            command_to_run.elevated,
        );

        let prepared_command = self.prepare_remote_process(processed_command);
        match timeout {
            None =>
                run_process_blocking(
                    "cmd.exe",
                    &prepared_command,
                ),
            Some(timeout) =>
                run_process_blocking_timed(
                    "cmd.exe",
                    &prepared_command,
                    timeout.clone(),
                ),
        }
    }

    fn prepare_remote_process(&self,
                              // pre_command: Vec<String>,
                              processed_command: Vec<String>,
                              // post_command: Vec<String>,
    ) -> Vec<String> {
        let all_args = iter::once("/c".to_string())
            // .chain(pre_command.into_iter())
            .chain(processed_command.into_iter())
            // .chain(post_command.into_iter())
            .collect();
        all_args
    }

    fn prepare_command(&self,
                       command: Vec<String>,
                       output_file_path: Option<String>,
                       elevated: bool,
    ) -> Vec<String>;
}