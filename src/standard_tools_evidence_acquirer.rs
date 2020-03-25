use std::path::PathBuf;
use crate::arg_parser::Opts;
use crate::evidence_acquirer::EvidenceAcquirer;
use crate::remote::{Computer, Connector, Connection};
use std::ops::Deref;

pub struct StandardToolsEvidenceAcquirer {
    pub remote_computer: Computer,
    pub store_directory: PathBuf,
    pub remote_connector: Box<dyn Connector>,
}

impl StandardToolsEvidenceAcquirer {
    #[allow(dead_code)]
    pub fn new(remote_computer: Computer,
               store_directory: PathBuf,
               remote_connector: Box<dyn Connector>,
    ) -> StandardToolsEvidenceAcquirer {
        StandardToolsEvidenceAcquirer {
            remote_computer,
            store_directory,
            remote_connector,
        }
    }

    pub fn from_opts(opts: &Opts,
                     remote_connector: Box<dyn Connector>,
    ) -> StandardToolsEvidenceAcquirer {
        StandardToolsEvidenceAcquirer {
            remote_computer: Computer {
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
impl StandardToolsEvidenceAcquirer {
    fn export_registry(
        &self,
        root: &str,
        remote_file_exported: &str,
    ) {
        let remote_connection = Connection::new(
            &self.remote_computer,
            vec![
                "reg".to_string(),
                "export".to_string(),
                root.to_string(),
                remote_file_exported.to_string()
            ],
            None,
            "get_registry",
        );

        info!("{}: Checking registry", self.remote_connector.connect_method_name());
        match self.remote_connector.connect_and_run_command(
            remote_connection
        ) {
            Ok(_) => {}
            Err(err) => { error!("Error running command [reg, export]. Cause: {}", err) }
        }
    }
}


#[cfg(windows)]
impl EvidenceAcquirer for StandardToolsEvidenceAcquirer {
    fn remote_computer(&self) -> &Computer {
        &self.remote_computer
    }

    fn store_directory(&self) -> &PathBuf {
        &self.store_directory
    }

    fn remote_connector(&self) -> &dyn Connector {
        self.remote_connector.deref()
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

    fn registry(&self) {
        // self._run(
        //     &vec!["reg", "export", "HKLM", ""],
        //     "registry_HKLM",
        // );
        // self._run(
        //     &self.registry_export_command("HKCU"),
        //     "registry_HKCU",
        // );
        // self._run(
        //     &self.registry_export_command("HKCR"),
        //     "registry_HKCR",
        // );
        // self._run(
        //     &self.registry_export_command("HKU"),
        //     "registry_HKU",
        // );
        // self._run(
        //     &self.registry_export_command("HKCC"),
        //     "registry_HKCC",
        // )
        // // Export protected roots?
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
