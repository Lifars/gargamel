use std::io::{Result, Error};
use std::path::{Path, PathBuf};
use std::fs::File;
use crate::remote::{Computer, Connector, Connection};
use std::ffi::OsStr;

pub trait EvidenceAcquirer {
    fn remote_computer(&self) -> &Computer;
    fn store_directory(&self) -> &PathBuf;
    fn remote_connector(&self) -> &dyn Connector;

    fn _run(
        &self,
        command: &[&str],
        report_filename_prefix: &str,
    ) {
        if command.is_empty() {
            return;
        }
        let remote_connection = Connection::new(
            self.remote_computer(),
            command.iter().map(|it| it.to_string()).collect(),
            Some(self.store_directory()),
            report_filename_prefix,
        );

        info!("{}: Checking {}",
              self.remote_connector().connect_method_name(),
              report_filename_prefix.replace("_", " ")
        );

        match self.remote_connector().connect_and_run_command(remote_connection) {
            Ok(_) => {}
            Err(err) => { error!("Error running command {:?}. Cause: {}", command, err) }
        }
    }

    fn firewall_state_command(&self) -> Vec<&'static str>;
    fn firewall_state(&self) {
        self._run(
             &self.firewall_state_command(),
             "firewall_status",
        )
    }

    fn network_state_command(&self) -> Vec<&'static str>;
    fn network_state(&self) {
        self._run(
             &self.network_state_command(),
             "network_status",
        )
    }

    fn logged_users_command(&self) -> Vec<&'static str>;
    fn logged_users(&self) {
        self._run(
             &self.logged_users_command(),
             "logged_users",
        )
    }

    fn running_processes_command(&self) -> Vec<&'static str>;
    fn running_processes(&self) {
        self._run(
             &self.running_processes_command(),
             "running_tasks",
        )
    }

    fn active_network_connections_command(&self) -> Vec<&'static str>;
    fn active_network_connections(&self) {
        self._run(
             &self.active_network_connections_command(),
             "active_network_connections",
        )
    }

    fn run_commands(&self, command_file: &Path) {
        // UNTESTED
        let file = match File::open(command_file) {
            Ok(file) => file,
            Err(err) => {
                error!("{}", err);
                return;
            },
        };
        let reader = std::io::BufReader::new(file);
        use std::io::BufRead;
        for one_command in reader.lines().filter_map(|item| item.ok()) {
            debug!("Running remote command {}", one_command);
            let command = one_command
                .split(' ')
                .map(|it| it.to_string())
                .collect::<Vec<String>>();
            let command_name = &command[0];
            let report_filename_prefix = format!("command_{}", command_name);
            let remote_connection = Connection::new(
                self.remote_computer(),
                command,
                Some(self.store_directory()),
                &report_filename_prefix,
            );
            match self.remote_connector().connect_and_run_command(remote_connection) {
                Ok(_) => {}
                Err(err) => { error!("{}", err) }
            };
        }
    }


    fn registry(&self);

    #[allow(unused_variables)] // DELETE THIS LINE AFTER IMPLEMENTING THE FUNCTION
    fn files(&self, file_list: &Path) {
        info!("Searching remote files");
    }

    fn memory_dump_command(&self) -> Vec<&'static str>;
    fn memory_dump(&self) {
        info!("Creating memory dump");
    }

    fn system_event_logs_command(&self) -> Vec<&'static str>;
    fn application_event_logs_command(&self) -> Vec<&'static str>;
    fn event_logs(&self) {
        self._run(
             &self.system_event_logs_command(),
             "events_system",
        );
        self._run(
             &self.application_event_logs_command(),
             "events_application",
        );
    }

    fn run_all(
        &self,
        command_file: Option<&Path>,
        file_list: Option<&Path>,
        memory_acquire: bool,
    ) {
        self.firewall_state();
        self.network_state();
        self.active_network_connections();
        self.running_processes();
        self.event_logs();
        self.logged_users();
        if command_file.is_some() {
            self.run_commands(command_file.unwrap());
        }
        if file_list.is_some() {
            self.files(file_list.unwrap());
        }
        if memory_acquire {
            self.memory_dump();
        }
    }
}
