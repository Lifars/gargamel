use std::io::Result;
use std::path::{PathBuf, Path};
use crate::remote_computer::{RemoteComputerConnector, RemoteComputer};
use std::fs::File;
use std::marker::PhantomData;
use crate::arg_parser::Opts;
use crate::process_runner::{RemoteConnection, run_remote_blocking_and_save};

pub struct EvidenceAcquirer<
    C: RemoteComputerConnector
> {
    pub remote_computer: RemoteComputer,
    pub store_directory: PathBuf,
    pub remote_connector: C,
}

impl<
    C: RemoteComputerConnector
> EvidenceAcquirer<C> {
    #[allow(dead_code)]
    pub fn new(remote_computer: RemoteComputer,
               store_directory: PathBuf,
               remote_connector: C,
    ) -> EvidenceAcquirer<C> {
        EvidenceAcquirer {
            remote_computer,
            store_directory,
            remote_connector,
        }
    }

    pub fn from_opts(opts: &Opts,
                     remote_connector: C,
    ) -> EvidenceAcquirer<C> {
        EvidenceAcquirer {
            remote_computer: RemoteComputer {
                address: opts.computer.clone(),
                username: opts.user.clone(),
                password: opts.password.clone(),
            },
            store_directory: PathBuf::from(opts.store_directory.clone()),
            remote_connector,
        }
    }
}

pub trait AnyEvidenceAcquirer {
    fn firewall_state(&self) -> Result<()>;

    fn network_state(&self) -> Result<()>;

    fn logged_users(&self) -> Result<()>;

    fn running_processes(&self) -> Result<()>;

    fn active_network_connections(&self) -> Result<()>;

    fn run_commands(&self, command_file: &Path) -> Result<()>;

    fn files(&self, file_list: &Path) -> Result<()>;

    fn memory_dump(&self) -> Result<()>;

    fn event_logs(&self) -> Result<()>;

    fn run_all(
        &self,
        command_file: Option<&Path>,
        file_list: Option<&Path>,
        include_expensive: bool,
    ) -> Result<()> {
        let mut error_message = String::from("");

        let err = self.firewall_state();
        if err.is_err() {
            error_message.push_str(&format!(":: {} ::", err.unwrap_err()));
        }

        let err = self.network_state();
        if err.is_err() {
            error_message.push_str(&format!(":: {} ::", err.unwrap_err()));
        }

        let err = self.active_network_connections();
        if err.is_err() {
            error_message.push_str(&format!(":: {} ::", err.unwrap_err()));
        }

        let err = self.running_processes();
        if err.is_err() {
            error_message.push_str(&format!(":: {} ::", err.unwrap_err()));
        }

        let err = self.event_logs();
        if err.is_err() {
            error_message.push_str(&format!(":: {} ::", err.unwrap_err()));
        }

        let err = self.logged_users();
        if err.is_err() {
            error_message.push_str(&format!(":: {} ::", err.unwrap_err()));
        }

        if command_file.is_some() {
            let err = self.run_commands(command_file.unwrap());
            if err.is_err() {
                error_message.push_str(&format!(":: {} ::", err.unwrap_err()));
            }
        }

        if include_expensive {
            if file_list.is_some() {
                let err = self.files(file_list.unwrap());
                if err.is_err() {
                    error_message.push_str(&format!(":: {} ::", err.unwrap_err()));
                }
            }

            let err = self.memory_dump();
            if err.is_err() {
                error_message.push_str(&format!(":: {} ::", err.unwrap_err()));
            }
        }

        Ok(())
    }
}

#[cfg(windows)]
impl<
    C: Send + Sync + RemoteComputerConnector
> AnyEvidenceAcquirer for EvidenceAcquirer<C> {
    fn firewall_state(&self) -> Result<()> {
        let remote_connection = RemoteConnection::new(
            &self.remote_computer,
            &self.remote_connector,
            &[
                "netsh",
                "advfirewall",
                "show",
                "allprofiles",
                "state",
            ],
            &self.store_directory,
            "firewall_status",
        );
        info!("Checking firewall state");
        run_remote_blocking_and_save(
            remote_connection
        )
    }

    fn network_state(&self) -> Result<()> {
        let remote_connection = RemoteConnection::new(
            &self.remote_computer,
            &self.remote_connector,
            &["ipconfig", "/all"],
            &self.store_directory,
            "network_status",
        );
        info!("Checking network state");
        run_remote_blocking_and_save(
            remote_connection
        )
    }

    fn logged_users(&self) -> Result<()> {
        let remote_connection = RemoteConnection::new(
            &self.remote_computer,
            &self.remote_connector,
            &["query", "user"],
            &self.store_directory,
            "logged_users",
        );
        info!("Checking logged users");
        run_remote_blocking_and_save(
            remote_connection
        )
    }

    fn running_processes(&self) -> Result<()> {
        let remote_connection = RemoteConnection::new(
            &self.remote_computer,
            &self.remote_connector,
            &["tasklist"],
            &self.store_directory,
            "tasks",
        );
        info!("Checking running tasks");
        run_remote_blocking_and_save(
            remote_connection
        )
    }

    fn active_network_connections(&self) -> Result<()> {
        let remote_connection = RemoteConnection::new(
            &self.remote_computer,
            &self.remote_connector,
            &["netstat", "-ano"],
            &self.store_directory,
            "network_connections",
        );
        info!("Checking open network connections");
        run_remote_blocking_and_save(
            remote_connection
        )
    }

    fn run_commands(&self, command_file: &Path) -> Result<()> {
        let file = File::open(command_file)?;
        let reader = std::io::BufReader::new(file);
        use std::io::BufRead;
        for one_command in reader.lines().filter_map(|item| item.ok()) {
            debug!("Running remote command {}", one_command);
            let command = one_command.split(' ').collect::<Vec<&str>>();
            let command_name = command[0];
            let report_filename_prefix = format!("command_{}", command_name);
            let remote_connection = RemoteConnection::new(
                &self.remote_computer,
                &self.remote_connector,
                &command,
                &self.store_directory,
                &report_filename_prefix,
            );
            match run_remote_blocking_and_save(remote_connection) {
                Ok(_) => {}
                Err(err) => { error!("{}", err) }
            };
        }
        Ok(())
    }

    #[allow(unused_variables)] // DELETE THIS LINE AFTER IMPLEMENTING THE FUNCTION
    fn files(&self, file_list: &Path) -> Result<()> {
        info!("Searching remote files");
        Ok(())
    }

    fn memory_dump(&self) -> Result<()> {
        info!("Creating memory dump");
        Ok(())
    }

    fn event_logs(&self) -> Result<()> {
        let remote_connection = RemoteConnection::new(
            &self.remote_computer,
            &self.remote_connector,
            &["wevtutil", "qe", "system"],
            &self.store_directory,
            "events_system",
        );
        info!("Checking system events");
        run_remote_blocking_and_save(
            remote_connection
        )?;

        let remote_connection = RemoteConnection::new(
            &self.remote_computer,
            &self.remote_connector,
            &["wevtutil", "qe", "application"],
            &self.store_directory,
            "events_application",
        );
        info!("Checking application events");
        run_remote_blocking_and_save(
            remote_connection
        )
    }
}
