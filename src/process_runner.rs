use std::process::{Command, Stdio};
use std::ops::Not;
use std::path::{Path, PathBuf};
use std::io::Result;
use crate::remote::Computer;
use std::fs::File;
use std::time::Duration;
use wait_timeout::ChildExt;

extern crate dunce;

pub fn run_process_blocking(
    command_name: &str,
    command_args: &[String],
) -> Result<()> {
    trace!("Starting process {}, with args: {:?}", command_name, command_args);
    let mut command = Command::new(command_name);
    // command.stdout(Stdio::null());
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
    trace!("Starting process \"{}\" with params {:?}", command_name_first, command_args_first);
    let mut first = first.stdout(Stdio::piped())
        .spawn()?;

    if let Some(first_output) = first.stdout.take() {
        let mut second = Command::new(command_name_second);
        second.stdin(first_output);
        if !command_args_second.is_empty() {
            second.args(command_args_second);
        }
        trace!("Starting process {} with params {:?}", command_name_second, command_args_second);
        let output = second.output()?;
        trace!("Command {} output: {}", command_name_second, String::from_utf8_lossy(&output.stdout));
        trace!("Command {} error: {}", command_name_second, String::from_utf8_lossy(&output.stderr));
    } else {
        trace!("Child not invoked")
    }
    Ok(())
}
pub fn run_process_blocking_maybe_timed(
    command_name: &str,
    command_args: &[String],
    wait_for: Option<Duration>,
) -> Result<()>{
    match wait_for {
        None => run_process_blocking(command_name, command_args),
        Some(wait) => run_process_blocking_timed(command_name, command_args, wait),
    }
}

pub fn run_process_blocking_timed(
    command_name: &str,
    command_args: &[String],
    wait_for: Duration,
) -> Result<()> {
    trace!("Starting process {}, with args: {:?} and timeout of {} seconds", command_name, command_args, wait_for.as_secs());
    let mut command = Command::new(command_name);
    if command_args.is_empty().not() {
        command.args(command_args);
    }
    let mut child = command.spawn()?;
    match child.wait_timeout(wait_for)? {
        Some(_) => {}
        None => {
            // child hasn't exited yet
            match child.kill() {
                Ok(_) => {}
                Err(_) => {}
            }

            trace!("Process \"{} {}\" reached time out", command_name, command_args.join(" "));
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
    let address_formatted = remote_computer.address.replace(".", "-");
    let filename = format!("{}-{}-{}-{}.txt",
                           method_name,
                           filename_prefix,
                           address_formatted,
                           remote_computer.username.replace(" ", "")
    );
    let file_path = store_directory.join(filename);
    {
        File::create(&file_path).expect(&format!("Cannot create file {}", file_path.display()));
    }
    let result = dunce::canonicalize(file_path).expect("Cannot canonicalize");
    trace!("Report will be saved at {}", result.display());
    result
}