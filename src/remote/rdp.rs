use crate::remote::{Connector, Computer, FileCopier, Command, RemoteFileCopier, copy_from_remote_wildcards};
use std::path::{Path, PathBuf};
use std::io;
use crate::process_runner::{run_process_blocking, create_report_path};
use std::time::Duration;

#[derive(Clone)]
pub struct Rdp {
    pub computer: Computer,
    pub nla: bool,
    pub remote_temp_storage: PathBuf,
}

impl Connector for Rdp {
    fn connect_method_name(&self) -> &'static str {
        return "RDP";
    }

    fn computer(&self) -> &Computer {
        &self.computer
    }

    fn copier(&self) -> &dyn RemoteFileCopier {
        self as &dyn RemoteFileCopier
    }

    fn remote_temp_storage(&self) -> &Path {
        self.remote_temp_storage.as_path()
    }

    fn connect_and_run_command(
        &self,
        remote_connection: Command<'_>,
        timeout: Option<Duration>,
    ) -> io::Result<Option<PathBuf>> {
        debug!("Trying to run command {:?} on {}",
               remote_connection.command,
               &self.computer().address
        );
        let output_file_path = match remote_connection.report_store_directory {
            None => None,
            Some(store_directory) => {
                let file_path = create_report_path(
                    self.computer(),
                    store_directory,
                    &remote_connection.report_filename_prefix,
                    self.connect_method_name(),
                    "txt",
                );
                Some(file_path.to_str().unwrap().to_string())
            }
        };

        let processed_command = self.prepare_command(
            remote_connection.command,
            output_file_path.as_deref(),
            remote_connection.elevated,
        );

        let prepared_command = self.prepare_remote_process(processed_command);
        let result = run_process_blocking(
            "cmd.exe",
            &prepared_command,
        );
        if let Some(timeout) = timeout {
            std::thread::sleep(timeout);
        }
        result.map(|_| output_file_path.map(|it| PathBuf::from(it)))
    }

    fn prepare_command(&self,
                       command: Vec<String>,
                       output_file_path: Option<&str>,
                       elevated: bool,
    ) -> Vec<String> {
        let remote_computer = self.remote_computer();
        let program_name = "SharpRDP.exe".to_string();

        let mut prepared_command = vec![
            program_name,
            format!("computername={}", &remote_computer.address),
        ];

        let username = match &remote_computer.domain {
            None =>
                remote_computer.username.clone(),
            Some(domain) =>
                format!("{}\\{}", domain, remote_computer.username),
        };
        prepared_command.push(format!("username={}", username));
        if let Some(password) = &remote_computer.password {
            prepared_command.push(format!("password={}", password));
        }

        if self.nla {
            prepared_command.push("nla=true".to_string());
        }

        if elevated {
            prepared_command.push("elevated=taskmgr".to_string());
        }

        prepared_command.push("exec=ps".to_string());
        prepared_command.push("takeover=true".to_string());
        prepared_command.push("connectdrive=true".to_string());

//        if let Some(time) = self.timeout {
//            prepared_command.push(format!("time={}", time.as_secs() * 60));
//        }

        let command_joined: String = command.join(" ");
        let command_as_arg = match output_file_path {
            None => format!("command={}", command_joined),
            Some(output_file_path) => {
                let path = Path::new(&output_file_path);
                let canon_path = dunce::canonicalize(path).unwrap();
                let as_remote_path = canon_path
                    .to_string_lossy()
                    .replacen(":", "", 1);
                format!(
//                     "command={} -p.i.p.e- Out-File -FilePath \\\\tsclient\\{}",
"command=cmd.exe /c {} -p.i.p.e- Out-File -FilePath \\\\tsclient\\{}",
command_joined,
as_remote_path
                )
            }
        };
        prepared_command.push(command_as_arg);
        prepared_command
    }
}

impl Rdp {
    fn run_command(&self, command: String) -> io::Result<()> {
        let mut args = vec![
            format!("computername={}", &self.computer.address),
            "exec=cmd".to_string(),
            "takeover=true".to_string(),
            "connectdrive=true".to_string(),
        ];
        let username = self.computer.domain_username();
        args.push(format!("username={}", username));
        if let Some(password) = &self.computer.password {
            args.push(format!("password={}", password));
        }
        if self.nla {
            args.push("nla=true".to_string());
        }
        args.push(command);
        run_process_blocking(
            "SharpRDP.exe",
            &args,
        )
    }
}

impl FileCopier for Rdp {
    fn copy_file(&self, source: &Path, target: &Path) -> io::Result<()> {
        self.run_command(format!(
            "command=xcopy {} {} /y",
            source.to_string_lossy(),
            target.to_string_lossy()
        ))
    }

    fn delete_file(&self, target: &Path) -> io::Result<()> {
        self.run_command(format!(
            "command=del /f {}",
            target.to_string_lossy()
        ))
    }

    fn method_name(&self) -> &'static str {
        "RDP"
    }
}

impl RemoteFileCopier for Rdp {
    fn remote_computer(&self) -> &Computer {
        &self.computer
    }

    fn copier_impl(&self) -> &dyn FileCopier {
        self as &dyn FileCopier
    }

    fn path_to_remote_form(&self, path: &Path) -> PathBuf {
        trace!("Converting path {}", path.display());
        // let canon_path = dunce::canonicalize(path).unwrap();
        let as_remote_path = path
            .to_string_lossy()
            .replacen(":", "", 1);
        let tsclient_path = format!("\\\\tsclient\\{}", as_remote_path);
        PathBuf::from(tsclient_path)
    }

    fn copy_to_remote(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        self.copier_impl().copy_file(&self.path_to_remote_form(source), target)
    }

    fn delete_remote_file(&self, target: &Path) -> io::Result<()> {
        self.copier_impl().delete_file(target)
    }

    fn copy_from_remote(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        copy_from_remote_wildcards(
            source,
            target,
            self,
            |source, target| self.copier_impl().copy_file(source, &self.path_to_remote_form(target)),
        )
    }
}

