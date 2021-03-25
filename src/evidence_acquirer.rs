use std::path::{Path, PathBuf};
use crate::remote::{Computer, Connector, Command, PsExec, PsRemote, Ssh, Rdp, Wmi, Local, RevShareConnector};

pub struct EvidenceAcquirer<'a> {
    store_directory: &'a Path,
    connector: Box<dyn Connector>,

    firewall_state_command: Option<Vec<String>>,
    network_state_command: Option<Vec<String>>,
    logged_users_command: Option<Vec<String>>,
    running_processes_command: Option<Vec<String>>,
    active_network_connections_command: Option<Vec<String>>,
}

impl<'a> EvidenceAcquirer<'a> {
    pub fn new(
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
        }
    }

    pub fn local(
        store_directory: &'a Path,
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer::new(
            store_directory,
            Box::new(Local::new()),
        )
    }

    pub fn psexec(
        remote_computer: Computer,
        store_directory: &'a Path,
        remote_temp_storage: PathBuf,
        custom_share_folder: Option<String>,
        reversed: bool,
    ) -> EvidenceAcquirer<'a> {
        let connector = Box::new(PsExec::paexec(remote_computer, remote_temp_storage, custom_share_folder));
        EvidenceAcquirer::new(
            store_directory,
            if reversed { Box::new(RevShareConnector::new(connector)) } else { connector },
        )
    }

    pub fn psremote(
        remote_computer: Computer,
        store_directory: &'a Path,
        remote_temp_storage: PathBuf,
        custom_share_folder: Option<String>,
        reversed: bool,
    ) -> EvidenceAcquirer<'a> {
        let connector = Box::new(PsRemote::new(remote_computer, remote_temp_storage, custom_share_folder));
        EvidenceAcquirer::new(
            store_directory,
            if reversed { Box::new(RevShareConnector::new(connector)) } else { connector },
        )
    }

    pub fn wmi(
        remote_computer: Computer,
        store_directory: &'a Path,
        remote_temp_storage: PathBuf,
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer::new(
            store_directory,
            Box::new(Wmi {
                computer: remote_computer,
                remote_temp_storage,
            }),
        )
    }

    pub fn rdp(
        remote_computer: Computer,
        store_directory: &'a Path,
        nla: bool,
        remote_temp_storage: PathBuf,
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer::new(
            store_directory,
            Box::new(Rdp {
                nla,
                computer: remote_computer,
                remote_temp_storage,
            }),
        )
    }

    pub fn ssh(
        remote_computer: Computer,
        store_directory: &'a Path,
        key_file: Option<PathBuf>,
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer {
            store_directory,
            connector: Box::new(Ssh { key_file, computer: remote_computer.clone() }),
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
            true,
        );

        info!("{}: Checking {}",
              self.connector.connect_method_name(),
              report_filename_prefix.replace("_", " ")
        );

        match self.connector.connect_and_run_command(remote_connection, None) {
            Ok(_) => {}
            Err(err) => { error!("Error running command {:?}. Cause: {}", command, err) }
        }
    }

    pub fn firewall_state(&self) {
        match &self.firewall_state_command {
            None => {}
            Some(command) => {
                self.run(
                    command,
                    "firewall-status",
                )
            }
        }
    }

    pub fn network_state(&self) {
        match &self.network_state_command {
            None => {}
            Some(command) => {
                self.run(
                    command,
                    "network-status",
                )
            }
        }
    }

    pub fn logged_users(&self) {
        match &self.logged_users_command {
            None => {}
            Some(command) => {
                self.run(
                    command,
                    "logged-users",
                )
            }
        }
    }

    pub fn running_processes(&self) {
        match &self.running_processes_command {
            None => {}
            Some(command) => {
                self.run(
                    command,
                    "running-processes",
                )
            }
        }
    }

    pub fn active_network_connections(&self) {
        match &self.active_network_connections_command {
            None => {}
            Some(command) => {
                self.run(
                    command,
                    "active-network-connections",
                )
            }
        }
    }

    pub fn run_all(
        &self,
    ) {
        self.firewall_state();
        self.network_state();
        self.active_network_connections();
        self.running_processes();
        self.logged_users();
    }
}