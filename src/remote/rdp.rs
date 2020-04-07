use crate::remote::{Connector, Computer, Copier, RemoteCopier};
use std::path::{Path, PathBuf};
use std::io;
use crate::process_runner::run_process_blocking;

pub struct Rdp {
    pub nla: bool
}

impl Connector for Rdp {
    fn connect_method_name(&self) -> &'static str {
        return "RDP";
    }

    fn prepare_command(&self,
                       remote_computer: &Computer,
                       command: Vec<String>,
                       output_file_path: Option<String>,
                       elevated: bool,
    ) -> Vec<String> {
        let program_name = "SharpRDP.exe".to_string();
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
                    // "command={} -p.i.p.e- Out-File -FilePath \\\\tsclient\\C\\Users\\Public\\funguj.txt",//\\\\tsclient\\{}\"",
                    "command={} -p.i.p.e- Out-File -FilePath \\\\tsclient\\{}",
                    command_joined,
                    as_remote_path
                )
            }
        };

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
        prepared_command.push(command_as_arg);
        prepared_command
    }
}

pub struct RdpCopy {
    pub computer: Computer,
    pub nla: bool,
}

impl RdpCopy {
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

impl Copier for RdpCopy {
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

impl RemoteCopier for RdpCopy {
    fn computer(&self) -> &Computer {
        &self.computer
    }

    fn copier_impl(&self) -> &dyn Copier {
        self as &dyn Copier
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
        self.copier_impl().copy_file(source, &self.path_to_remote_form(target))
    }
}