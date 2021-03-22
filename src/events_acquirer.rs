use std::path::{Path, PathBuf};
use crate::remote::{Computer, Connector, PsExec, PsRemote, Rdp, Wmi, Compression, Local};
use std::time::Duration;
use crate::large_evidence_acquirer::LargeEvidenceAcquirer;

pub struct EventsAcquirer<'a> {
    store_directory: &'a Path,
    connector: Box<dyn Connector>,

    application_event_logs_command: Vec<String>,
    system_event_logs_command: Vec<String>,

    compress_timeout: Option<Duration>,
    compression: Compression,
}

impl<'a> EventsAcquirer<'a> {
    pub fn new(
        store_directory: &'a Path,
        connector: Box<dyn Connector>,
        compress_timeout: Option<Duration>,
        compression: Compression,
    ) -> EventsAcquirer<'a> {
        EventsAcquirer {
            store_directory,
            connector,
            application_event_logs_command: vec![
                "wevtutil".to_string(),
                "epl".to_string(),
                "application".to_string(),
            ],
            system_event_logs_command: vec![
                "wevtutil".to_string(),
                "epl".to_string(),
                "system".to_string(),
            ],
            compress_timeout,
            compression,
        }
    }

    pub fn psexec32(
        store_directory: &'a Path,
        computer: Computer,
        no_7zip: bool,
        remote_temp_storage: PathBuf
    ) -> EventsAcquirer {
        EventsAcquirer::new(
            store_directory,
            Box::new(PsExec::psexec32(computer, remote_temp_storage)),
            None,
            if no_7zip { Compression::No } else { Compression::Yes },
        )
    }

    pub fn psexec64(
        store_directory: &'a Path,
        computer: Computer,
        no_7zip: bool,
        remote_temp_storage: PathBuf
    ) -> EventsAcquirer {
        EventsAcquirer::new(
            store_directory,
            Box::new(PsExec::psexec64(computer, remote_temp_storage)),
            None,
            if no_7zip { Compression::No } else { Compression::Yes },
        )
    }

    pub fn local(
        store_directory: &'a Path,
    ) -> EventsAcquirer {
        EventsAcquirer::new(
            store_directory,
            Box::new(Local::new()),
            None,
            Compression::No,
        )
    }

    pub fn psremote(
        store_directory: &'a Path,
        computer: Computer,
        _no_7zip: bool,
        remote_temp_storage: PathBuf
    ) -> EventsAcquirer {
        EventsAcquirer::new(
            store_directory,
            Box::new(PsRemote::new(computer, remote_temp_storage)),
            None,
            Compression::No,
        )
    }

    pub fn wmi(
        store_directory: &'a Path,
        computer: Computer,
        compress_timeout: Duration,
        no_7zip: bool,
        remote_temp_storage: PathBuf
    ) -> EventsAcquirer {
        EventsAcquirer::new(
            store_directory,
            Box::new(Wmi { computer, remote_temp_storage }),
            Some(compress_timeout),
            if no_7zip { Compression::No } else { Compression::YesSplit }
        )
    }

    pub fn rdp(
        store_directory: &'a Path,
        computer: Computer,
        compress_timeout: Duration,
        nla: bool,
        no_7zip: bool,
        remote_temp_storage: PathBuf
    ) -> EventsAcquirer {
        EventsAcquirer::new(
            store_directory,
            Box::new(Rdp { computer, nla, remote_temp_storage }),
            Some(compress_timeout),
            if no_7zip { Compression::No } else { Compression::YesSplit }
        )
    }

    pub fn acquire(&self) {
        let lea = LargeEvidenceAcquirer{
            store_directory: self.store_directory,
            connector: self.connector.as_ref(),
            compress_timeout: self.compress_timeout,
            compression: self.compression,
            report_extension: "evtx",
            overwrite_switch: Some("/ow:true")
        };
        let command = &self.system_event_logs_command;
        lea.run(
            command,
            "events-system"
        );
        let command = &self.application_event_logs_command;
        lea.run(
            command,
            "events-application"
        );
    }
}