use std::io::{Result, Error};
use std::path::{Path, PathBuf};
use std::fs::File;
use crate::remote::{Computer, Connector, Command, PsExec, PsRemote, Local, Wmi, Ssh};
use std::ffi::OsStr;
use crate::command_utils::parse_command;
use std::ops::Deref;

pub struct EvidenceAcquirer<'a> {
    remote_computer: &'a Computer,
    store_directory: &'a Path,
    connector: Box<dyn Connector>,

    firewall_state_command: Option<Vec<String>>,
    network_state_command: Option<Vec<String>>,
    logged_users_command: Option<Vec<String>>,
    running_processes_command: Option<Vec<String>>,
    active_network_connections_command: Option<Vec<String>>,
    registry_hklm_command: Option<Vec<String>>,
    registry_hkcu_command: Option<Vec<String>>,
    registry_hkcr_command: Option<Vec<String>>,
    registry_hku_command: Option<Vec<String>>,
    registry_hkcc_command: Option<Vec<String>>,
    system_event_logs_command: Option<Vec<String>>,
    application_event_logs_command: Option<Vec<String>>,
}

impl<'a> EvidenceAcquirer<'a> {
    fn new_standard_acquirer(
        remote_computer: &'a Computer,
        store_directory: &'a Path,
        remote_connector: Box<dyn Connector>,
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer {
            remote_computer,
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
            registry_hklm_command: Some(vec![
                "reg".to_string(),
                "export".to_string(),
                "HKLM".to_string(),
            ]),
            registry_hkcu_command: Some(vec![
                "reg".to_string(),
                "export".to_string(),
                "HKCU".to_string(),
            ]),
            registry_hkcr_command: Some(vec![
                "reg".to_string(),
                "export".to_string(),
                "HKCR".to_string(),
            ]),
            registry_hku_command: Some(vec![
                "reg".to_string(),
                "export".to_string(),
                "HKU".to_string(),
            ]),
            registry_hkcc_command: Some(vec![
                "reg".to_string(),
                "export".to_string(),
                "HKCC".to_string(),
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
        remote_computer: &'a Computer,
        store_directory: &'a Path,
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer::new_standard_acquirer(
            remote_computer,
            store_directory,
            Box::new(PsExec {}),
        )
    }

    pub fn psremote(
        remote_computer: &'a Computer,
        store_directory: &'a Path,
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer::new_standard_acquirer(
            remote_computer,
            store_directory,
            Box::new(PsRemote {}),
        )
    }

    pub fn local(
        remote_computer: &'a Computer,
        store_directory: &'a Path,
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer::new_standard_acquirer(
            remote_computer,
            store_directory,
            Box::new(Local {}),
        )
    }

    pub fn wmi(
        remote_computer: &'a Computer,
        store_directory: &'a Path,
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer {
            remote_computer,
            store_directory,
            connector: Box::new(Wmi{}),
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
            registry_hklm_command: None,
            registry_hkcu_command: None,
            registry_hkcr_command: None,
            registry_hku_command: None,
            registry_hkcc_command: None,
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

    pub fn ssh(
        remote_computer: &'a Computer,
        store_directory: &'a Path,
    ) -> EvidenceAcquirer<'a> {
        EvidenceAcquirer {
            remote_computer,
            store_directory,
            connector: Box::new(Ssh{}),
            firewall_state_command: Some(vec![
                format!("echo {} | sudo -S iptables -L", remote_computer.password),
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
            registry_hklm_command: None,
            registry_hkcu_command: None,
            registry_hkcr_command: None,
            registry_hku_command: None,
            registry_hkcc_command: None,
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
            &self.remote_computer,
            command.to_vec(),
            Some(&self.store_directory),
            report_filename_prefix,
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
                    "firewall_status",
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
                    "network_status",
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
                    "logged_users",
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
                    "running_processes",
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
                    "active_network_connections",
                )
            },
        }
    }

    pub fn registry(&self) {
        match &self.registry_hklm_command {
            None => {},
            Some(command) => {
                self.run(
                    command,
                    "registry_hklm",
                )
            },
        }
        match &self.registry_hkcu_command {
            None => {},
            Some(command) => {
                self.run(
                    command,
                    "registry_hkcu",
                )
            },
        }
        match &self.registry_hku_command {
            None => {},
            Some(command) => {
                self.run(
                    command,
                    "registry_hku",
                )
            },
        }
        match &self.registry_hkcr_command {
            None => {},
            Some(command) => {
                self.run(
                    command,
                    "registry_hkcr",
                )
            },
        }
        match &self.registry_hkcc_command {
            None => {},
            Some(command) => {
                self.run(
                    command,
                    "registry_hkcc",
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
                    "events_system",
                )
            },
        };
        match &self.application_event_logs_command {
            None => {},
            Some(command) => {
                self.run(
                    command,
                    "events_application",
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