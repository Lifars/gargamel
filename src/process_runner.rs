use std::process::{Stdio, ChildStdout, Command};
use std::ops::Not;
use bytes::BytesMut;
use crate::remote_computer::{RemoteComputerConnector, RemoteComputer};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::marker::PhantomData;
use std::io::{Write, BufReader, Error};
use std::ffi::OsStr;
use std::fmt::{Display, Debug};
use std::{io, iter};

extern crate dunce;
// use crate::path_utils::adjust_canonicalization;

// pub fn run_process_async_with_output(
//     command_name: &str,
//     command_args: &[String],
// ) -> io::Result<ProcessOutput> {
//     debug!("Starting procgram {}, with args: {:?}", command_name, command_args);
//     let mut command = Command::new(command_name);
//     if command_args.is_empty().not() {
//         command.args(command_args);
//     }
//     let mut cmd = command
//         .stdout(Stdio::piped()) // Can do the same for stderr
//         .spawn()?;
//
//     let stdout = cmd.stdout.take();
//     if stdout.is_none() {
//         return Err(io::Error::new(ErrorKind::Other, "Cannot use child process stdout"));
//     }
//     let stdout = stdout.unwrap();
//
// // To print out each line
// // BufReader::new(stdout)
// //     .lines()
// //     .for_each(|s| async move { println!("> {:?}", s) })
// //     .await;
//
//     Ok(ProcessOutput {
//         stdout_reader: BufReader::new(stdout)
//     })
// }

pub fn run_process_blocking(
    command_name: &str,
    command_args: &[String],
) -> io::Result<()> {
    debug!("Starting process {}, with args: {:?}", command_name, command_args);
    let mut command = Command::new(command_name);
    if command_args.is_empty().not() {
        command.args(command_args);
    }
    // command
    //     .stdout(Stdio::inherit())
    //     .spawn()
    //     ?.wait()?;
    let output = command.output()?;
    // trace!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}

pub fn create_report_path(
    remote_computer: &RemoteComputer,
    store_directory: &Path,
    filename_prefix: &str,
    method_name: &str,
) -> io::Result<PathBuf> {
    let address_formatted = remote_computer.address.replace(".", "-");
    let filename = format!("{}-{}_{}_{}.txt",
                           method_name,
                           filename_prefix,
                           address_formatted,
                           remote_computer.username
    );
    Ok(store_directory.join(filename))
}

pub struct RemoteConnection<
    'a,
    C: RemoteComputerConnector
> {
    pub remote_computer: &'a RemoteComputer,
    pub remote_computer_connector: &'a C,
    pub command: Vec<String>,
    pub store_directory: &'a Path,
    pub report_filename_prefix: &'a str,
}

impl<
    'a,
    C: RemoteComputerConnector
> RemoteConnection<'a, C> {
    pub fn new(
        remote_computer: &'a RemoteComputer,
        remote_computer_connector: &'a C,
        command: &[&'a str],
        store_directory: &'a Path,
        report_filename_prefix: &'a str,
    ) -> RemoteConnection<'a, C> {
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

pub fn run_remote_blocking_and_save<
    C: RemoteComputerConnector
>(
    remote_connection: RemoteConnection<'_, C>
) -> std::io::Result<()> {
    let file_path = create_report_path(
        &remote_connection.remote_computer,
        &remote_connection.store_directory,
        &remote_connection.report_filename_prefix,
        &remote_connection.remote_computer_connector.connect_method_name()
    )?;
    {
        File::create(&file_path);
    }

    let file_path = dunce::canonicalize(file_path)?.to_str().unwrap().to_string();
    // let file_path = match file_path.canonicalize() {
    //     Ok(file_path) => Ok(file_path.as_os_str().to_str().unwrap().to_string()),
    //     Err(err) => {
    //         error!("{}", err);
    //         Err(err)
    //     }
    // }?;

    // let args_with_save: Vec<String> = vec![
    //     remote_connection.command.join(" "),
    //     ">".to_string(),
    //     file_path,
    // ];

    let args_with_save: Vec<String> = remote_connection.command.into_iter().chain(vec![
        ">".to_string(),
        file_path,
    ]).collect();
    // let args_with_save: Vec<String> = remote_connection.command.into_iter().chain(iter::once(
    //     format!("> {}", file_path)
    // )).collect();
    remote_connection.remote_computer_connector
        .connect_and_run_command(
            &remote_connection.remote_computer,
            args_with_save,
        )?;
    Ok(())
}

// pub async fn run_remote_async_and_save<
//     C: RemoteComputerConnector
// >(
//     remote_connection: RemoteConnection<'_, C>
// ) -> std::io::Result<()> {
//     let mut remote_reader = remote_connection
//         .remote_computer_connector
//         .connect_and_run_command_with_output(
//             &remote_connection.remote_computer,
//             remote_connection.command,
//         )?;
//     let file = create_report_file(
//         remote_connection.remote_computer,
//         remote_connection.store_directory,
//         remote_connection.report_filename_prefix,
//     )?;
//     let mut writer = std::io::LineWriter::new(file);
//
//     let mut buffer = BytesMut::with_capacity(256);
//     loop {
//         remote_reader.read_buf(&mut buffer).await?;
//         if buffer.is_empty() {
//             debug!("Receiving from remote process \"{}\" finished",
//                    remote_connection.report_filename_prefix
//             );
//             break;
//         } else {
//             debug!("Received {} bytes from remote \"{}\"", buffer.len(),
//                    remote_connection.report_filename_prefix
//             );
//             match writer.write_all(&buffer) {
//                 Ok(_) => {}
//                 Err(err) => error!("Cannot write line into file due to: {}", err)
//             };
//         }
//         buffer.clear()
//     }
//     debug!("Finished receiving stdout from \"{}\"",
//            remote_connection.report_filename_prefix
//     );
//     Ok(())
// }