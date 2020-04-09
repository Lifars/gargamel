use std::path::{Path, PathBuf};
use crate::remote::{Computer, Connector, Command, PsExec, PsRemote, Local, Wmic, Ssh, Rdp, WmiImplant};

pub struct EvidenceAcquirer<'a> {
    store_directory: &'a Path,
    connector: Box<dyn Connector>,

    firewall_state_command: Option<Vec<String>>,
    network_state_command: Option<Vec<String>>,
    logged_users_command: Option<Vec<String>>,
    running_processes_command: Option<Vec<String>>,
    active_network_connections_command: Option<Vec<String>>,
    system_event_logs_command: Option<Vec<String>>,
    application_event_logs_command: Option<Vec<String>>,
}

impl<'a> EvidenceAcquirer<'a> {
    fn new_standard_acquirer(
        store_directory: &'a Path,
        remote_connector: Box<dyn Connector>,
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer {
            store_directory,
            connector: remote_connector,
            firewall_state_command: Some(vec![
                "netsh".to_string(),
                "advfirewall".to_string(),
                "show".to_string(),
                "allprofiles".to_string(),
                "state".to_string(),
            ]),
            network_state_command: Some(vec![
                "ipconfig".to_string(),
                "/all".to_string(),
            ]),
            logged_users_command: Some(vec![
                "query".to_string(),
                "user".to_string(),
            ]),
            running_processes_command: Some(vec![
                "tasklist".to_string(),
            ]),
            active_network_connections_command: Some(vec![
                "netstat".to_string(),
                "-ano".to_string(),
            ]),
            system_event_logs_command: Some(vec![
                "wevtutil".to_string(),
                "qe".to_string(),
                "system".to_string(),
            ]),
            application_event_logs_command: Some(vec![
                "wevtutil".to_string(),
                "qe".to_string(),
                "application".to_string(),
            ]),
        }
    }

    pub fn psexec(
        remote_computer: Computer,
        store_directory: &'a Path,
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer::new_standard_acquirer(
            store_directory,
            Box::new(PsExec { computer: remote_computer }),
        )
    }

    pub fn psremote(
        remote_computer: Computer,
        store_directory: &'a Path,
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer::new_standard_acquirer(
            store_directory,
            Box::new(PsRemote { computer: remote_computer }),
        )
    }

    pub fn local(
        store_directory: &'a Path,
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer::new_standard_acquirer(
            store_directory,
            Box::new(Local::new()),
        )
    }

    pub fn wmi(
        remote_computer: Computer,
        store_directory: &'a Path,
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer::new_standard_acquirer(
            store_directory,
            Box::new(WmiImplant { computer: remote_computer }),
        )
    }

    pub fn wmic(
        remote_computer: Computer,
        store_directory: &'a Path,
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer {
            store_directory,
            connector: Box::new(Wmic { computer: remote_computer }),
            firewall_state_command: None,
            network_state_command: Some(vec![
                "nic".to_string(),
                "get".to_string(),
                "AdapterType,".to_string(),
                "Name,".to_string(),
                "Installed,".to_string(),
                "MACAddress,".to_string(),
                "PowerManagementSupported,".to_string(),
                "Speed".to_string(),
            ]),
            logged_users_command: Some(vec![
                "COMPUTERSYSTEM".to_string(),
                "GET".to_string(),
                "USERNAME".to_string(),
            ]),
            running_processes_command: Some(vec![
                "process".to_string(),
            ]),
            active_network_connections_command: Some(vec![
                "netuse".to_string(),
            ]),
            system_event_logs_command: Some(vec![
                "NTEVENT".to_string(),
                "WHERE".to_string(),
                "LogFile='system".to_string(),
            ]),
            application_event_logs_command: Some(vec![
                "NTEVENT".to_string(),
                "WHERE".to_string(),
                "LogFile='application".to_string(),
            ]),
        }
    }

    pub fn rdp(
        remote_computer: Computer,
        store_directory: &'a Path,
        nla: bool,
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer::new_standard_acquirer(
            store_directory,
            Box::new(Rdp {
                nla,
                connection_time: None,
                computer: remote_computer
            }),
        )
    }

    pub fn ssh(
        remote_computer: Computer,
        store_directory: &'a Path,
        key_file: Option<PathBuf>
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer {
            store_directory,
            connector: Box::new(Ssh{ key_file, computer: remote_computer.clone() }),
            firewall_state_command: Some(vec![
                format!("echo {} | sudo -S iptables -L", remote_computer.password.unwrap_or_default()),
            ]),
            network_state_command: Some(vec![
                "ifconfig".to_string(),
            ]),
            logged_users_command: Some(vec![
                "who".to_string(),
            ]),
            running_processes_command: Some(vec![
                "ps aux".to_string(),
            ]),
            active_network_connections_command: Some(vec![
                "netstat -natp".to_string(),
            ]),
            system_event_logs_command: None,
            application_event_logs_command: Some(vec![
                "lsof".to_string(),
            ]),
        }
    }

    fn run(
        &self,
        command: &[String],
        report_filename_prefix: &str,
    ) {
        if command.is_empty() {
            return;
        }
        let remote_connection = Command::new(
            command.to_vec(),
            Some(&self.store_directory),
            report_filename_prefix,
            false,
            None
        );

        info!("{}: Checking {}",
              self.connector.connect_method_name(),
              report_filename_prefix.replace("_", " ")
        );

        match self.connector.connect_and_run_command(remote_connection) {
            Ok(_) => {}
            Err(err) => { error!("Error running command {:?}. Cause: {}", command, err) }
        }
    }

    pub fn firewall_state(&self) {
        match &self.firewall_state_command {
            None => {},
            Some(command) => {
                self.run(
                    command,
                    "firewall-status",
                )
            },
        }
    }

    pub fn network_state(&self) {
        match &self.network_state_command {
            None => {},
            Some(command) => {
                self.run(
                    command,
                    "network-status",
                )
            },
        }
    }

    pub fn logged_users(&self) {
        match &self.logged_users_command {
            None => {},
            Some(command) => {
                self.run(
                    command,
                    "logged-users",
                )
            },
        }
    }

    pub fn running_processes(&self) {
        match &self.running_processes_command {
            None => {},
            Some(command) => {
                self.run(
                    command,
                    "running-processes",
                )
            },
        }
    }

    pub fn active_network_connections(&self) {
        match &self.active_network_connections_command {
            None => {},
            Some(command) => {
                self.run(
                    command,
                    "active-network-connections",
                )
            },
        }
    }

    pub fn event_logs(&self) {
        match &self.system_event_logs_command {
            None => {},
            Some(command) => {
                self.run(
                    command,
                    "events-system",
                )
            },
        };
        match &self.application_event_logs_command {
            None => {},
            Some(command) => {
                self.run(
                    command,
                    "events-application",
                )
            },
        };
    }

    pub fn run_all(
        &self,
    ) {
        self.firewall_state();
        self.network_state();
        self.active_network_connections();
        self.running_processes();
        self.event_logs();
        self.logged_users();
    }
}