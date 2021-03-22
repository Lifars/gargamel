use crate::remote::{Computer, Connector, Command, PsExec, PsRemote, Ssh, Rdp, Wmi, Local};
use std::path::{Path, PathBuf};
use std::fs::File;
use crate::command_utils::parse_command;
use std::time::Duration;

pub struct CommandRunner<'a> {
    local_store_directory: &'a Path,
    pub(crate) connector: Box<dyn Connector>,
    run_implicit: bool,
}

impl<'a> CommandRunner<'a> {
    pub fn psexec(
        remote_computer: Computer,
        local_store_directory: &'a Path,
        remote_temp_storage: PathBuf
    ) -> CommandRunner<'a> {
        CommandRunner {
            local_store_directory,
            connector: Box::new(PsExec::paexec(remote_computer, remote_temp_storage)),
            run_implicit: true,
        }
    }

    pub fn wmi(
        remote_computer: Computer,
        local_store_directory: &'a Path,
        remote_temp_storage: PathBuf
    ) -> CommandRunner<'a> {
        CommandRunner {
            local_store_directory,
            connector: Box::new(Wmi { computer: remote_computer, remote_temp_storage}),
            run_implicit: true,
        }
    }

    pub fn local(
        local_store_directory: &'a Path,
    ) -> CommandRunner<'a> {
        CommandRunner {
            local_store_directory,
            connector: Box::new(Local::new()),
            run_implicit: true,
        }
    }

    pub fn psremote(
        remote_computer: Computer,
        local_store_directory: &'a Path,
        remote_temp_storage: PathBuf
    ) -> CommandRunner<'a> {
        CommandRunner {
            local_store_directory,
            connector: Box::new(PsRemote::new(remote_computer, remote_temp_storage)),
            run_implicit: true,
        }
    }

    pub fn rdp(
        remote_computer: Computer,
        local_store_directory: &'a Path,
        nla: bool,
        remote_temp_storage: PathBuf
    ) -> CommandRunner<'a> {
        CommandRunner {
            local_store_directory,
            connector: Box::new(Rdp {
                computer: remote_computer,
                nla,
                remote_temp_storage
            }),
            run_implicit: true,
        }
    }

    pub fn ssh(
        remote_computer: Computer,
        local_store_directory: &'a Path,
        key_file: Option<PathBuf>,
    ) -> CommandRunner<'a> {
        CommandRunner {
            local_store_directory,
            connector: Box::new(Ssh { key_file, computer: remote_computer }),
            run_implicit: false,
        }
    }

    pub fn run_commands(
        &self,
        command_file: &Path,
        timeout: Option<Duration>
    ) {
        let file = match File::open(command_file) {
            Ok(file) => file,
            Err(err) => {
                error!("{}", err);
                return;
            }
        };
        let reader = std::io::BufReader::new(file);
        use std::io::BufRead;
        for one_command in reader.lines().filter_map(|item| item.ok()) {
            if one_command.starts_with("#") {
                continue;
            }
            if one_command.is_empty() {
                continue;
            }
            debug!("Running remote command {}", one_command);
            let command = parse_command(&one_command);

            let first_arg = command[0].to_ascii_lowercase();
            let command = if first_arg.starts_with(":") {
                let method_name = self.connector.connect_method_name().to_ascii_lowercase();
                if first_arg.contains(&method_name) {
                    command[1..].to_vec()
                } else {
                    continue;
                }
            } else if self.run_implicit {
                command
            } else {
                continue;
            };

            let elevated =
                first_arg.contains(":admin") || first_arg.contains(":sudo");

            let command_joined: String = command.join("-");
            let command_joined = if command_joined.len() > 100 {
                command_joined[..100].to_string()
            } else {
                command_joined
            };
            let command_joined = command_joined
                .replace(" ", "-")
                .replace("\"", "")
                .replace("/", "")
                .replace("\\", "")
                .replace(":", "-");
            let report_filename_prefix = format!("custom-{}", command_joined);

            let remote_connection = Command::new(
                command,
                Some(&self.local_store_directory),
                &report_filename_prefix,
                elevated,
            );
            if let Err(err) = self.connector.connect_and_run_command(
                remote_connection,
                timeout
            ) {
                error!("{}", err)
            };
        }
    }
}
