use std::io::{Result, Error};
use std::path::{Path, PathBuf};
use std::fs::File;
use crate::remote::{Computer, Connector, Command, PsExec, PsRemote, Local, Wmi};
use std::ffi::OsStr;
use crate::command_utils::parse_command;
use std::ops::Deref;

pub struct EvidenceAcquirer<'a> {
    remote_computer: &'a Computer,
    store_directory: &'a Path,
    connector: Box<dyn Connector>,

    firewall_state_command: Option<Vec<&'static str>>,
    network_state_command: Option<Vec<&'static str>>,
    logged_users_command: Option<Vec<&'static str>>,
    running_processes_command: Option<Vec<&'static str>>,
    active_network_connections_command: Option<Vec<&'static str>>,
    registry_hklm_command: Option<Vec<&'static str>>,
    registry_hkcu_command: Option<Vec<&'static str>>,
    registry_hkcr_command: Option<Vec<&'static str>>,
    registry_hku_command: Option<Vec<&'static str>>,
    registry_hkcc_command: Option<Vec<&'static str>>,
    system_event_logs_command: Option<Vec<&'static str>>,
    application_event_logs_command: Option<Vec<&'static str>>,
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
                "netsh",
                "advfirewall",
                "show",
                "allprofiles",
                "state",
            ]),
            network_state_command: Some(vec![
                "ipconfig",
                "/all"
            ]),
            logged_users_command: Some(vec![
                "query",
                "user"
            ]),
            running_processes_command: Some(vec![
                "tasklist"
            ]),
            active_network_connections_command: Some(vec![
                "netstat",
                "-ano"
            ]),
            registry_hklm_command: Some(vec![
                "reg",
                "export",
                "HKLM"
            ]),
            registry_hkcu_command: Some(vec![
                "reg",
                "export",
                "HKCU"
            ]),
            registry_hkcr_command: Some(vec![
                "reg",
                "export",
                "HKCR"
            ]),
            registry_hku_command: Some(vec![
                "reg",
                "export",
                "HKU"
            ]),
            registry_hkcc_command: Some(vec![
                "reg",
                "export",
                "HKCC"
            ]),
            system_event_logs_command: Some(vec![
                "wevtutil",
                "qe",
                "system"
            ]),
            application_event_logs_command: Some(vec![
                "wevtutil",
                "qe",
                "application"
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
                "nic",
                "get",
                "AdapterType,",
                "Name,",
                "Installed,",
                "MACAddress,",
                "PowerManagementSupported,",
                "Speed"
            ]),
            logged_users_command: Some(vec![
                "COMPUTERSYSTEM",
                "GET",
                "USERNAME"
            ]),
            running_processes_command: Some(vec![
                "process"
            ]),
            active_network_connections_command: Some(vec![
                "netuse"
            ]),
            registry_hklm_command: None,
            registry_hkcu_command: None,
            registry_hkcr_command: None,
            registry_hku_command: None,
            registry_hkcc_command: None,
            system_event_logs_command: Some(vec![
                "NTEVENT",
                "WHERE",
                "LogFile='system"
            ]),
            application_event_logs_command: Some(vec![
                "NTEVENT",
                "WHERE",
                "LogFile='application"
            ]),
        }
    }

    fn run(
        &self,
        command: &[&str],
        report_filename_prefix: &str,
    ) {
        if command.is_empty() {
            return;
        }
        let remote_connection = Command::new(
            &self.remote_computer,
            command.iter().map(|it| it.to_string()).collect(),
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