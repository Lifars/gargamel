use crate::remote::{Connector, Computer};

pub struct PowerShell {}

impl Connector for PowerShell {
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
