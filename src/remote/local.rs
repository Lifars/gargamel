use crate::remote::{Connector, Computer, FileCopier, RemoteFileCopier, Command, DEFAULT_REMOTE_PATH_STORAGE, copy_from_remote_wildcards, Cmd};
use std::path::{Path, PathBuf};
use std::{io, fs};
use std::time::Duration;
use fs_extra::dir::CopyOptions;
use std::io::ErrorKind;

pub struct Local {
    localhost: Computer,
    temp_storage: PathBuf,
}

impl Local {
    pub fn new(username: String, temp_storage: PathBuf) -> Local {
        Local {
            localhost: Computer {
                address: String::from("127.0.0.1"),
                username,
                password: None,
                domain: None,
            },
            temp_storage
        }
    }

    pub fn new_default(username: String) -> Local {
        Local::new(username, PathBuf::from(DEFAULT_REMOTE_PATH_STORAGE))
    }
}

impl Connector for Local {
    fn connect_method_name(&self) -> &'static str {
        return "LOCAL";
    }

    fn computer(&self) -> &Computer {
        &self.localhost
    }

    fn copier(&self) -> &dyn RemoteFileCopier {
        self as &dyn RemoteFileCopier
    }

    fn remote_temp_storage(&self) -> &Path {
        self.temp_storage.as_path()
    }

    fn connect_and_run_local_program(
        &self,
        command_to_run: Command<'_>,
        timeout: Option<Duration>
    ) -> io::Result<Option<PathBuf>> {
        self.connect_and_run_command(command_to_run, timeout)
    }

    fn prepare_command(&self,
                       command: Vec<String>,
                       output_file_path: Option<&str>,
                       _elevated: bool,
    ) -> Vec<String> {
        match output_file_path {
            None => command,
            Some(output_file_path) => {
                let mut result : Vec<String> = command.into();
                result.push(">".to_string());
                result.push(output_file_path.to_string());
                result
            }
        }
    }
}

impl FileCopier for Local {
    fn copy_file(&self, source: &Path, target: &Path) -> io::Result<()> {
        Cmd{}.copy_file(source, target)
    }

    fn delete_file(&self, target: &Path) -> io::Result<()> {
        Cmd{}.delete_file(target)
    }

    fn method_name(&self) -> &'static str {
        "XCopy (local)"
    }
}

impl RemoteFileCopier for Local {
    fn remote_computer(&self) -> &Computer {
        &self.localhost
    }

    fn copier_impl(&self) -> &dyn FileCopier {
        self
    }

    fn path_to_remote_form(&self, path: &Path) -> PathBuf {
        path.to_path_buf()
    }

    fn copy_from_remote(&self, source: &Path, target: &Path) -> io::Result<()> {
        copy_from_remote_wildcards(
            source,
            target,
            self,
            |s, t| self.copier_impl().copy_file(&self.path_to_remote_form(s), t),
        )
    }
}