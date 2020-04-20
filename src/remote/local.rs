use crate::remote::{Connector, Computer, FileCopier, RemoteFileCopier, Command};
use std::path::{Path, PathBuf};
use std::{io, fs};
use std::time::Duration;
use fs_extra::dir::CopyOptions;
use std::io::ErrorKind;

pub struct Local {
    localhost: Computer
}

impl Local {
    pub fn new() -> Local {
        Local {
            localhost: Computer {
                address: String::from("127.0.0.1"),
                username: String::new(),
                password: None,
                domain: None,
            }
        }
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

    fn prepare_command(&self,
                       command: Vec<String>,
                       _output_file_path: Option<String>,
                       _elevated: bool,
    ) -> Vec<String> {
        command
    }

    fn connect_and_run_local_program(
        &self,
        command_to_run: Command<'_>,
        timeout: Option<Duration>
    ) -> io::Result<()> {
        self.connect_and_run_command(command_to_run, timeout)
    }

    fn remote_temp_storage(&self) -> &Path {
        Path::new("C:\\Users\\Public")
    }
}

impl FileCopier for Local {
    fn copy_file(&self, source: &Path, target: &Path) -> io::Result<()> {
        if source.is_file() {
            let target = if target.is_file() {
                target.to_path_buf()
            } else {
                target.join(source.file_name().unwrap())
            };
            fs::copy(source, &target)?;
        } else {
            if !target.exists() {
                fs::create_dir_all(target)?;
            }
            let mut options = CopyOptions::new();
            options.copy_inside = true;
            options.overwrite = true;
            fs_extra::dir::copy(source, target, &options).map_err(|err| io::Error::new(ErrorKind::Other, err))?;
        }
        Ok(())
    }

    fn delete_file(&self, target: &Path) -> io::Result<()> {
        if target.is_file() {
            fs::remove_file(target)
        } else {
            fs::remove_dir_all(target)
        }
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
}