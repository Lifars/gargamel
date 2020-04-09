use std::io::Result;
use crate::process_runner::{run_process_blocking, create_report_path, run_process_blocking_timed};
use std::iter;
use std::path::Path;
use crate::arg_parser::Opts;
use std::time::Duration;

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

pub struct PreparedProgramToRun {
    pub program_path: String,
    pub all_program_args: Vec<String>,
}

pub struct Command<'a> {
    pub command: Vec<String>,
    pub store_directory: Option<&'a Path>,
    pub report_filename_prefix: &'a str,
    pub elevated: bool,
    pub timeout: Option<Duration>
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
        timeout: Option<Duration>
    ) -> Command<'a> {
        Command {
            command,
            store_directory,
            report_filename_prefix,
            elevated,
            timeout
        }
    }
}

pub trait Connector {
    fn connect_method_name(&self) -> &'static str;

    fn computer(&self) -> &Computer;

    fn connect_and_run_command(
        &self,
        remote_connection: Command<'_>,
    ) -> Result<()> {
        debug!("Trying to run command {:?} on {}",
               remote_connection.command,
               &self.computer().address
        );
        let output_file_path = match remote_connection.store_directory {
            None => None,
            Some(store_directory) => {
                let file_path = create_report_path(
                    self.computer(),
                    store_directory,
                    &remote_connection.report_filename_prefix,
                    self.connect_method_name(),
                );
                Some(file_path.to_str().unwrap().to_string())
            }
        };

        let processed_command = self.prepare_command(
            remote_connection.command,
            output_file_path,
            remote_connection.elevated
        );

        let prepared_command = self.prepare_remote_process(processed_command);
        match remote_connection.timeout {
            None => run_process_blocking(&prepared_command.program_path, &prepared_command.all_program_args),
            Some(timeout) => run_process_blocking_timed(&prepared_command.program_path, &prepared_command.all_program_args, timeout),
        }
    }

    fn prepare_remote_process(&self,
                              // pre_command: Vec<String>,
                              processed_command: Vec<String>,
                              // post_command: Vec<String>,
    ) -> PreparedProgramToRun {
        let all_args = iter::once("/c".to_string())
            // .chain(pre_command.into_iter())
            .chain(processed_command.into_iter())
            // .chain(post_command.into_iter())
            .collect();
        PreparedProgramToRun {
            program_path: "cmd.exe".to_string(),
            all_program_args: all_args,
        }
    }

    fn prepare_command(&self,
                       command: Vec<String>,
                       output_file_path: Option<String>,
                       elevated: bool,
    ) -> Vec<String>;
}