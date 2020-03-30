use crate::remote::{Connector, Computer, Command, PsExec, Local, PsRemote, RemoteCopier, Copier, XCopy, PsCopyItem};
use std::path::{PathBuf, Path};
use crate::arg_parser::Opts;
use std::io;
use crate::utils::Quoted;

pub struct MemoryAcquirer<'a> {
    pub computer: &'a Computer,
    pub local_store_directory: &'a Path,
    pub connector: Box<dyn Connector>,
    pub copier: Box<dyn Copier>,
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
            copier: Box::new(XCopy {}),
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
            copier: Box::new(PsCopyItem {}),
        }
    }

    pub fn image_memory(
        &self,
        target_name: &Path,
    ) -> io::Result<()> {
        // self.run_command(target_store);
        // self.extract_file()

        let winpmem = "winpmem.exe";
        let source_winpmem = std::env::current_dir()?.join(winpmem);
        let target_name = match target_name.parent() {
            None => Path::new("C:\\Users\\Public").join(target_name),
            Some(parent) => target_name.to_owned(),
        };
        let target_store = target_name.parent().unwrap();
        let target_winpmem = target_store.join(winpmem);
        let remote_copier = RemoteCopier{
            computer: &self.computer,
            copier_impl: self.copier.as_ref()
        };
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
        remote_copier.copy_from_remote(
            &target_name,
            &self.local_store_directory,
            // &self.local_store_directory.join(target_name.file_name().unwrap()),
        )
    }
}


