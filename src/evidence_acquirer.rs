use std::io::Result;
use std::path::{Path, PathBuf};
use crate::process_runner::{RemoteConnection, run_remote_blocking_and_save};
use std::fs::File;
use crate::remote::{RemoteComputer, Connector};

pub trait EvidenceAcquirer {
    fn remote_computer(&self) -> &RemoteComputer;
    fn store_directory(&self) -> &PathBuf;
    fn remote_connector(&self) -> &dyn Connector;

    fn _run(
        &self,
        command: &[&str],
        report_filename_prefix: &str,
    ) -> Result<()> {
        if command.is_empty() {
            return Ok(());
        }
        let remote_connection = RemoteConnection::new(
            self.remote_computer(),
            self.remote_connector(),
            &command,
            self.store_directory(),
            report_filename_prefix,
        );

        info!("{}: Checking {}",
              self.remote_connector().connect_method_name(),
              report_filename_prefix.replace("_", " ")
        );
        run_remote_blocking_and_save(
            remote_connection
        )
    }

    fn firewall_state_command(&self) -> Vec<&'static str>;
    fn firewall_state(&self) -> Result<()> {
        self._run(
            &self.firewall_state_command(),
            "firewall_status",
        )
    }

    fn network_state_command(&self) -> Vec<&'static str>;
    fn network_state(&self) -> Result<()> {
        self._run(
            &self.network_state_command(),
            "network_status",
        )
    }

    fn logged_users_command(&self) -> Vec<&'static str>;
    fn logged_users(&self) -> Result<()> {
        self._run(
            &self.logged_users_command(),
            "logged_users",
        )
    }

    fn running_processes_command(&self) -> Vec<&'static str>;
    fn running_processes(&self) -> Result<()> {
        self._run(
            &self.running_processes_command(),
            "running_tasks",
        )
    }

    fn active_network_connections_command(&self) -> Vec<&'static str>;
    fn active_network_connections(&self) -> Result<()> {
        self._run(
            &self.active_network_connections_command(),
            "active_network_connections",
        )
    }

    fn run_commands(&self, command_file: &Path) -> Result<()> {
        // UNTESTED
        let file = File::open(command_file)?;
        let reader = std::io::BufReader::new(file);
        use std::io::BufRead;
        for one_command in reader.lines().filter_map(|item| item.ok()) {
            debug!("Running remote command {}", one_command);
            let command = one_command.split(' ').collect::<Vec<&str>>();
            let command_name = command[0];
            let report_filename_prefix = format!("command_{}", command_name);
            let remote_connection = RemoteConnection::new(
                self.remote_computer(),
                self.remote_connector(),
                &command,
                self.store_directory(),
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

    fn system_event_logs_command(&self) -> Vec<&'static str>;
    fn application_event_logs_command(&self) -> Vec<&'static str>;
    fn event_logs(&self) -> Result<()> {
        self._run(
            &self.system_event_logs_command(),
            "events_system",
        )?;
        self._run(
            &self.application_event_logs_command(),
            "events_application",
        )
    }

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
