use std::process::Command;
use std::ops::Not;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{Result, Error, ErrorKind};
use crate::remote::{Computer, Connector};
use std::time::Duration;
use wait_timeout::ChildExt;

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
    let output = command.output()?;
    trace!("Command {} output: {}", command_name, String::from_utf8_lossy(&output.stdout));
    trace!("Command {} error: {}", command_name, String::from_utf8_lossy(&output.stderr));
    Ok(())
}

pub fn run_process_blocking_timed(
    command_name: &str,
    command_args: &[String],
    wait_for: Duration
) -> Result<()> {
    debug!("Starting process {}, with args: {:?}", command_name, command_args);
    let mut command = Command::new(command_name);
    if command_args.is_empty().not() {
        command.args(command_args);
    }
    let mut child = command.spawn()?;
    match child.wait_timeout(wait_for).unwrap() {
        Some(_) => Ok(()),
        None => {
            // child hasn't exited yet
            child.kill()?;
            Err(Error::new(ErrorKind::Other, "Process reached the time limit"))
        }
    };
    Ok(())
}


pub fn create_report_path(
    remote_computer: &Computer,
    store_directory: &Path,
    filename_prefix: &str,
    method_name: &str,
) -> PathBuf {
    let address_formatted = remote_computer.address.replace(".", "_");
    let filename = format!("{}_{}_{}_{}.txt",
                           method_name,
                           filename_prefix,
                           address_formatted,
                           remote_computer.username
    );
    store_directory.join(filename)
}