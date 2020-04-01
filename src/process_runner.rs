use std::process::{Command, Stdio};
use std::ops::Not;
use std::path::{Path, PathBuf};
use std::io::Result;
use crate::remote::Computer;

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

pub fn run_piped_processes_blocking(
    command_name_first: &str,
    command_args_first: &[String],
    command_name_second: &str,
    command_args_second: &[String],
) -> Result<()> {
    let mut first = Command::new(command_name_first);
    if !command_args_first.is_empty() {
        first.args(command_args_first);
    }
    trace!("Running command \"{}\" with params {:?}", command_name_first, command_args_first);
    let mut first = first.stdout(Stdio::piped())
        .spawn()?;

    if let Some(first_output) = first.stdout.take() {
        let mut second = Command::new(command_name_second);
        second.stdin(first_output);
        if !command_args_second.is_empty() {
            second.args(command_args_second);
        }
        trace!("Running command {} with params {:?}", command_name_second, command_args_second);
        let output = second.output()?;
        trace!("Command {} output: {}", command_name_second, String::from_utf8_lossy(&output.stdout));
        trace!("Command {} error: {}", command_name_second, String::from_utf8_lossy(&output.stderr));
    } else {
        trace!("Child not invoked")
    }
    Ok(())
}

// pub fn run_process_blocking_timed(
//     command_name: &str,
//     command_args: &[String],
//     wait_for: Duration,
// ) -> Result<()> {
//     debug!("Starting process {}, with args: {:?}", command_name, command_args);
//     let mut command = Command::new(command_name);
//     if command_args.is_empty().not() {
//         command.args(command_args);
//     }
//     let mut child = command.spawn()?;
//     match child.wait_timeout(wait_for)? {
//         Some(_) => Ok(()),
//         None => {
//             // child hasn't exited yet
//             child.kill()?;
//             Err(Error::new(ErrorKind::Other, "Process reached the time limit"))
//         }
//     }?;
//     Ok(())
// }


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