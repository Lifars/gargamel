use std::process::Command;
use std::ops::Not;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::Result;
use crate::remote::{RemoteComputer, Connector};

extern crate dunce;

pub fn run_process_blocking(
    command_name: &str,
    command_args: &[String],
) -> Result<()> {
    debug!("Starting process {}, with args: {:?}", command_name, command_args);
    let mut command = Command::new(command_name);
    if command_args.is_empty().not() {
        command.args(command_args);
    }
    command.output()?;
    Ok(())
}

pub fn create_report_path(
    remote_computer: &RemoteComputer,
    store_directory: &Path,
    filename_prefix: &str,
    method_name: &str,
) -> Result<PathBuf> {
    let address_formatted = remote_computer.address.replace(".", "_");
    let filename = format!("{}_{}_{}_{}.txt",
                           method_name,
                           filename_prefix,
                           address_formatted,
                           remote_computer.username
    );
    Ok(store_directory.join(filename))
}

pub struct RemoteConnection<'a> {
    pub remote_computer: &'a RemoteComputer,
    pub remote_computer_connector: &'a dyn Connector,
    pub command: Vec<String>,
    pub store_directory: &'a Path,
    pub report_filename_prefix: &'a str,
}

impl<'a> RemoteConnection<'a> {
    pub fn new(
        remote_computer: &'a RemoteComputer,
        remote_computer_connector: &'a dyn Connector,
        command: &[&'a str],
        store_directory: &'a Path,
        report_filename_prefix: &'a str,
    ) -> RemoteConnection<'a> {
        let command_owned = command
            .iter()
            .map(|it| it.to_string())
            .collect::<Vec<String>>();
        RemoteConnection {
            remote_computer,
            remote_computer_connector,
            command: command_owned,
            store_directory,
            report_filename_prefix,
        }
    }
}

pub fn run_remote_blocking_and_save(
    remote_connection: RemoteConnection<'_>
) -> std::io::Result<()> {
    let file_path = create_report_path(
        &remote_connection.remote_computer,
        &remote_connection.store_directory,
        &remote_connection.report_filename_prefix,
        &remote_connection.remote_computer_connector.connect_method_name()
    )?;
    {
        File::create(&file_path)?;
    }
    let file_path = dunce::canonicalize(file_path)?.to_str().unwrap().to_string();

    remote_connection.remote_computer_connector
        .connect_and_run_command(
            &remote_connection.remote_computer,
            file_path,
            remote_connection.command.clone(),
        )?;
    Ok(())
}
