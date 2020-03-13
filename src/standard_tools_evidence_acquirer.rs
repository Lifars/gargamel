use crate::remote_computer::{RemoteComputerConnector, RemoteComputer};
use std::path::{PathBuf, Path};
use crate::arg_parser::Opts;
use crate::evidence_acquirer::EvidenceAcquirer;
use std::io::Result;
use crate::process_runner::{RemoteConnection, run_remote_blocking_and_save};
use std::fs::File;

pub struct StandardToolsEvidenceAcquirer<
    C: RemoteComputerConnector
> {
    pub remote_computer: RemoteComputer,
    pub store_directory: PathBuf,
    pub remote_connector: C,
}

impl<
    C: RemoteComputerConnector
> StandardToolsEvidenceAcquirer<C> {
    #[allow(dead_code)]
    pub fn new(remote_computer: RemoteComputer,
               store_directory: PathBuf,
               remote_connector: C,
    ) -> StandardToolsEvidenceAcquirer<C> {
        StandardToolsEvidenceAcquirer {
            remote_computer,
            store_directory,
            remote_connector,
        }
    }

    pub fn from_opts(opts: &Opts,
                     remote_connector: C,
    ) -> StandardToolsEvidenceAcquirer<C> {
        StandardToolsEvidenceAcquirer {
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


#[cfg(windows)]
impl<
    C: Send + Sync + RemoteComputerConnector
> EvidenceAcquirer for StandardToolsEvidenceAcquirer<C> {
    fn remote_computer(&self) -> &RemoteComputer {
        &self.remote_computer
    }

    fn store_directory(&self) -> &PathBuf {
        &self.store_directory
    }

    fn remote_connector(&self) -> &dyn RemoteComputerConnector {
        &self.remote_connector
    }

    fn firewall_state_command(&self) -> Vec<&'static str> {
        vec![
            "netsh",
            "advfirewall",
            "show",
            "allprofiles",
            "state",
        ]
    }

    fn network_state_command(&self) -> Vec<&'static str> {
        vec![
            "ipconfig",
            "/all"
        ]
    }

    fn logged_users_command(&self) -> Vec<&'static str> {
        vec![
            "query",
            "user"
        ]
    }

    fn running_processes_command(&self) -> Vec<&'static str> {
        vec![
            "tasklist"
        ]
    }

    fn active_network_connections_command(&self) -> Vec<&'static str> {
        vec![
            "netstat",
            "-ano"
        ]
    }


    fn system_event_logs_command(&self) -> Vec<&'static str> {
        vec![
            "wevtutil",
            "qe",
            "system"
        ]
    }

    fn application_event_logs_command(&self) -> Vec<&'static str> {
        vec![
            "wevtutil",
            "qe",
            "application"
        ]
    }
}
