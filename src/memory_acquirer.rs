use crate::remote::{Connector, Computer, Connection};
use std::path::{PathBuf, Path};
use crate::arg_parser::Opts;
use std::io;
use crate::remote::Copier;
use crate::utils::Quoted;

pub struct MemoryAcquirer {
    pub remote_computer: Computer,
    pub local_store_directory: PathBuf,
    pub connector: Box<dyn Connector>,
}

impl MemoryAcquirer {
    pub fn from_opts(
        opts: &Opts,
        connector: Box<dyn Connector>,
    ) -> MemoryAcquirer {
        MemoryAcquirer {
            remote_computer: Computer {
                address: opts.computer.clone(),
                username: opts.user.clone(),
                password: opts.password.clone(),
            },
            local_store_directory: PathBuf::from(&opts.store_directory),
            connector,
        }
    }

    pub fn image_memory(
        &self,
        target_name: &Path,
    ) -> io::Result<()> {
        // self.run_command(target_store);
        // self.extract_file()
        let current_dir = std::env::current_dir()?;
        let winpmem = "winpmem.exe";
        let copier = Copier::new(&self.remote_computer);
        let target_store_path = match target_name.parent(){
            None => Path::new("C:\\Users\\Public"),
            Some(parent) => parent,
        };
        copier.copy_to_remote(
            &current_dir,
            target_store_path,
            Some(winpmem)
        )?;
        let target_winpmem = target_store_path
            .join(winpmem);
        trace!("Winpmem target path: {:#?}", target_winpmem);
        let connection = Connection {
            remote_computer: &self.remote_computer,
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
        trace!("Tu nedejdzem");
        self.connector.connect_and_run_command(connection)?;

        copier.copy_from_remote(
            target_store_path,
            &self.local_store_directory,
            target_name
                .file_name()
                .map(|it| it.to_str().expect("Specified filename for memory image is not Unicode")),
        )
    }
}


