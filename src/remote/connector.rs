use std::io::Result;
use crate::process_runner::{run_process_blocking, create_report_path};
use std::iter;
use std::path::Path;
use crate::arg_parser::Opts;

#[derive(Clone)]
pub struct Computer {
    pub address: String,
    pub username: String,
    pub domain: Option<String>,
    pub password: Option<String>,
}

impl Computer{
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
    pub remote_computer: &'a Computer,
    pub command: Vec<String>,
    pub store_directory: Option<&'a Path>,
    pub report_filename_prefix: &'a str,
}

impl From<Opts> for Computer {
    fn from(opts: Opts) -> Self {
        Computer{
            address: opts.computer,
            username: opts.user,
            domain: opts.domain,
            password: opts.password
        }
    }
}

impl<'a> Command<'a> {
    pub fn new(
        remote_computer: &'a Computer,
        command: Vec<String>,
        store_directory: Option<&'a Path>,
        report_filename_prefix: &'a str,
    ) -> Command<'a> {
        Command {
            remote_computer,
            command,
            store_directory,
            report_filename_prefix,
        }
    }
}

pub trait Connector {
    fn connect_method_name(&self) -> &'static str;

    fn connect_and_run_command(
        &self,
        remote_connection: Command<'_>,
    ) -> Result<()> {
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
                Some(file_path.to_str().unwrap().to_string())
            }
        };

        let processed_command = self.prepare_command(
            remote_connection.remote_computer,
            remote_connection.command,
            output_file_path,
        );

        let prepared_command = self.prepare_remote_process(processed_command);
        run_process_blocking(&prepared_command.program_path, &prepared_command.all_program_args)
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
                       remote_computer: &Computer,
                       command: Vec<String>,
                       output_file_path: Option<String>,
    ) -> Vec<String>;
}