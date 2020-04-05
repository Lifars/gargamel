use crate::remote::{Connector, Computer, Copier};
use std::path::Path;
use std::io;
use crate::process_runner::run_process_blocking;

pub struct PsRemote {}

impl Connector for PsRemote {
    fn connect_method_name(&self) -> &'static str {
        return "PSREM";
    }

    fn prepare_command(&self,
                       remote_computer: &Computer,
                       command: Vec<String>,
                       output_file_path: Option<String>,
    ) -> Vec<String> {
        let program_name = "powershell.exe".to_string();
        let prefix = vec![
            program_name,
            "-command".to_string(),
            "Invoke-Command".to_string(),
            "-ComputerName".to_string(),
            remote_computer.address.clone(),
            "-ScriptBlock".to_string(),
            "{".to_string(),
        ];
        let suffix = vec![
            "}".to_string(),
            "-credential".to_string(),
            format!(
                "(New-Object Management.Automation.PSCredential ('{}', (ConvertTo-SecureString '{}' -AsPlainText -Force)))",
                remote_computer.username,
                remote_computer.password
            ),
        ];
        let almost_result = prefix.into_iter()
            .chain(command.into_iter())
            .chain(suffix.into_iter());
        match output_file_path {
            None => almost_result.collect(),
            Some(output_file_path) =>
                almost_result.chain(vec![
                    ">".to_string(),
                    output_file_path
                ].into_iter()).collect(),
        }
    }
}

pub struct PsCopy {}

impl Copier for PsCopy {
    fn copy_file(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        let args = vec![
            "Copy-Item".to_string(),
            format!("'{}'", source.to_string_lossy()),
            format!("'{}'", target.to_string_lossy()),
        ];
        run_process_blocking(
            "powershell.exe",
            &args,
        )
    }

    fn delete_file(&self, target: &Path) -> io::Result<()> {
        let args = vec![
            "Remove-Item".to_string(),
            "-Force".to_string(),
            format!("'{}'", target.to_string_lossy()),
        ];
        run_process_blocking(
            "powershell.exe",
            &args,
        )
    }

    fn method_name(&self) -> &'static str {
        "PowerShell"
    }
}
