use crate::remote::{Connector, Computer, Copier, RemoteCopier};
use std::path::{Path, PathBuf};
use std::io;
use crate::process_runner::run_process_blocking;

pub struct Wmic {
    pub computer: Computer
}

impl Connector for Wmic {
    fn connect_method_name(&self) -> &'static str {
        return "WMI";
    }

    fn computer(&self) -> &Computer {
        &self.computer
    }

    fn prepare_command(&self,
                       command: Vec<String>,
                       output_file_path: Option<String>,
                       _elevated: bool,
    ) -> Vec<String> {
        let remote_computer = self.computer();
        let program_name = "wmic.exe".to_string();

        let address = format!("/NODE:{}", remote_computer.address);
        let username = remote_computer.domain_username();
        let user = format!("/USER:{}", username);

        let mut final_command = vec![program_name];
        if let Some(output_file_path) = output_file_path {
            final_command.push(format!("/OUTPUT:{}", output_file_path));
        }
        final_command.push(address);
        final_command.push(user);
        if let Some(password) = &remote_computer.password {
            final_command.push(format!("/PASSWORD:{}", password));
        }
        final_command.extend(command.into_iter());
        final_command
    }
}

pub struct WmiProcess {
    pub computer: Computer
}

impl Connector for WmiProcess {
    fn connect_method_name(&self) -> &'static str {
        return "WMI";
    }

    fn computer(&self) -> &Computer {
        &self.computer
    }

    fn prepare_command(&self,
                       command: Vec<String>,
                       _output_file_path: Option<String>,
                       _elevated: bool,
    ) -> Vec<String> {
        let remote_computer = self.computer();

        let program_name = "wmic.exe".to_string();

        let address = format!("/NODE:{}", remote_computer.address);
        let username = remote_computer.domain_username();
        let user = format!("/USER:{}", username);

        let mut final_command = vec![program_name];
        final_command.push(address);
        final_command.push(user);
        if let Some(password) = &remote_computer.password {
            final_command.push(format!("/PASSWORD:{}", password));
        }
        final_command.push("process".to_string());
        final_command.push("call".to_string());
        final_command.push("create".to_string());

        final_command.extend(command.into_iter());
        final_command
    }
}

pub struct WmiImplant {
    pub computer: Computer
}

impl Connector for WmiImplant {
    fn connect_method_name(&self) -> &'static str {
        "WMI"
    }

    fn computer(&self) -> &Computer {
        &self.computer
    }

    fn prepare_command(&self,
                       command: Vec<String>,
                       output_file_path: Option<String>,
                       _elevated: bool,
    ) -> Vec<String> {
        let remote_computer = self.remote_computer();
        let program_name = "powershell.exe".to_string();
        // let wmi_implant = std::env::current_dir()
        //     .expect("Cannot get current working directory")
        //     .join("WMImplant.ps1")
        //     .to_string_lossy().to_string();
        let command_joined: String = command.join(" ");
        let mut prepared_command = vec![
            program_name,
            "-File".to_string(),
            "WMImplant.ps1".to_string(),
            "-CommandExec".to_string(),
            "-RemoteCommand".to_string(),
            command_joined,
            "-ComputerName".to_string(),
            remote_computer.address.clone(),
            "-RemoteUser".to_string(),
            remote_computer.domain_username()
        ];

        if let Some(password) = &remote_computer.password {
            prepared_command.push("-RemotePass".to_string());
            prepared_command.push(password.clone());
        }
        match output_file_path {
            None => prepared_command,
            Some(output_file_path) => {
                prepared_command.push(">".to_string());
                prepared_command.push(output_file_path);
                prepared_command
            }
        }
    }
}

impl WmiImplant {
    fn copy_impl(&self,
                 source: &Path,
                 target: &Path,
                 method_name: &str,
                 target_is_remote: bool,
    ) -> io::Result<()>{
        let remote_computer = self.remote_computer();

        let target = match source.file_name() {
            None => target.to_path_buf(),
            Some(file_name) => target.join(file_name),
        };


        let mut prepared_command = vec![
            "-File".to_string(),
            "WMImplant.ps1".to_string(),
            method_name.to_string(),
            "-ComputerName".to_string(),
            remote_computer.address.clone(),
            "-RemoteUser".to_string(),
            remote_computer.domain_username()
        ];

        if target_is_remote {
            prepared_command.push("-RemoteFile".to_string());
            prepared_command.push(target.to_string_lossy().to_string());
            prepared_command.push("-LocalFile".to_string());
            prepared_command.push(source.to_string_lossy().to_string());
        } else {
            prepared_command.push("-RemoteFile".to_string());
            prepared_command.push(source.to_string_lossy().to_string());
            prepared_command.push("-LocalFile".to_string());
            prepared_command.push(target.to_string_lossy().to_string());
        }

        if let Some(password) = &remote_computer.password {
            prepared_command.push("-RemotePass".to_string());
            prepared_command.push(password.clone());
        }

        run_process_blocking("powershell.exe", &prepared_command)
    }
}

impl Copier for WmiImplant {
    fn copy_file(&self, source: &Path, target: &Path) -> io::Result<()> {
        self.copy_impl(source, target, "-Copy", true)
    }

    fn delete_file(&self, target: &Path) -> io::Result<()> {
        let remote_computer = self.remote_computer();

        let mut prepared_command = vec![
            "-File".to_string(),
            "WMImplant.ps1".to_string(),
            "-Delete".to_string(),
            "-LocalFile".to_string(),
            target.to_string_lossy().to_string(),
            "-ComputerName".to_string(),
            remote_computer.address.clone(),
            "-RemoteUser".to_string(),
            remote_computer.domain_username()
        ];

        if let Some(password) = &remote_computer.password {
            prepared_command.push("-RemotePass".to_string());
            prepared_command.push(password.clone());
        }

        run_process_blocking("powershell.exe", &prepared_command)
    }

    fn method_name(&self) -> &'static str {
        self.connect_method_name()
    }
}

impl RemoteCopier for WmiImplant {
    fn remote_computer(&self) -> &Computer {
        self.computer()
    }

    fn copier_impl(&self) -> &dyn Copier {
        self as &dyn Copier
    }

    fn path_to_remote_form(&self, path: &Path) -> PathBuf {
        path.to_path_buf()
    }

    fn copy_to_remote(&self, source: &Path, target: &Path) -> io::Result<()> {
        self.copy_impl(source, target, "-Upload", true)
    }

    fn delete_remote_file(&self, target: &Path) -> io::Result<()> {
        self.delete_file(target)
    }

    fn copy_from_remote(&self, source: &Path, target: &Path) -> io::Result<()> {
        self.copy_impl(source, target, "-Download", false)
    }
}