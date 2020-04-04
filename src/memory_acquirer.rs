use crate::remote::{Connector, Computer, Command, PsExec, PsRemote, RemoteCopier, XCopy, PsCopyItem, WindowsRemoteCopier, RdpCopy, Rdp};
use std::path::Path;
use std::{io, thread};
use std::time::Duration;

pub struct MemoryAcquirer<'a> {
    pub computer: &'a Computer,
    pub local_store_directory: &'a Path,
    pub connector: Box<dyn Connector>,
    pub copier_factory: Box<dyn Fn(Computer) -> Box<dyn RemoteCopier>>,
    pub manual_wait: Option<Duration>
}

impl<'a> MemoryAcquirer<'a> {
    pub fn psexec(
        remote_computer: &'a Computer,
        local_store_directory: &'a Path,
    ) -> MemoryAcquirer<'a> {
        MemoryAcquirer {
            computer: remote_computer,
            local_store_directory,
            connector: Box::new(PsExec {}),
            copier_factory: Box::new(|computer: Computer|
                Box::new(WindowsRemoteCopier::new(
                    computer,
                    Box::new(XCopy {}),
                ))),
            manual_wait: None
        }
    }

    // pub fn wmi(
    //     remote_computer: Computer,
    //     local_store_directory: PathBuf,
    // )-> MemoryAcquirer{
    //     MemoryAcquirer{
    //         remote_computer,
    //         local_store_directory,
    //         connector: Box::new(WmiProcess {})
    //     }
    // }

    pub fn psremote(
        remote_computer: &'a Computer,
        local_store_directory: &'a Path,
    ) -> MemoryAcquirer<'a> {
        MemoryAcquirer {
            computer: remote_computer,
            local_store_directory,
            connector: Box::new(PsRemote {}),
            copier_factory: Box::new(|computer: Computer|
                Box::new(WindowsRemoteCopier::new(
                    computer,
                    Box::new(PsCopyItem {}),
                ))
            ),
            manual_wait: None
        }
    }

    pub fn rdp(
        remote_computer: &'a Computer,
        local_store_directory: &'a Path,
    ) -> MemoryAcquirer<'a> {
        MemoryAcquirer {
            computer: remote_computer,
            local_store_directory,
            connector: Box::new(Rdp {}),
            copier_factory: Box::new(|computer: Computer|
                Box::new(RdpCopy {
                    computer,
                })
            ),
            manual_wait: Some(Duration::from_secs(60 * 5))
        }
    }

    pub fn image_memory(
        &self,
        target_name: &Path,
    ) -> io::Result<()> {
        let winpmem = "winpmem.exe";
        let source_winpmem = std::env::current_dir()?.join(winpmem);
        let target_name = match target_name.parent() {
            None => Path::new("C:\\Users\\Public").join(target_name),
            Some(_) => target_name.to_owned(),
        };
        let target_store = target_name.parent().unwrap();
        let target_winpmem = target_store.join(winpmem);
        let remote_copier = self.copier_factory.as_ref()(self.computer.clone());
        remote_copier.copy_to_remote(
            &source_winpmem,
            &target_store,
        )?;
        trace!("Winpmem target path: {:#?}", target_winpmem);
        let connection = Command {
            remote_computer: &self.computer,
            command: vec![
                target_winpmem.to_string_lossy().to_string(),
                "--format".to_string(),
                "map".to_string(),
                "-o".to_string(),
                target_name.to_string_lossy().to_string(),
            ],
            store_directory: None,
            report_filename_prefix: "mem-ack-log",
        };
        self.connector.connect_and_run_command(connection)?;
        if self.manual_wait.is_some() {
            thread::sleep(self.manual_wait.unwrap());
        }
        remote_copier.copy_from_remote(
            &target_name,
            &self.local_store_directory,
            // &self.local_store_directory.join(target_name.file_name().unwrap()),
        )
    }
}


