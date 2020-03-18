use crate::remote::{Connector, RemoteComputer};

pub struct PowerShell {}

impl Connector for PowerShell {
    fn connect_method_name(&self) -> &'static str {
        return "PS";
    }

    fn prepare_command(&self,
                       remote_computer: &RemoteComputer,
                       command: Vec<String>,
                       output_file_path: String,
    ) -> Vec<String> {
        let program_name = "powershell.exe".to_string();
        let joined_command = command.join(" ");
        // vec![
        //     program_name,
        //     "-command".to_string(),
        //     format!(
        //         "Invoke-Command -ComputerName {} -ScriptBlock {{ {} }} -credential (New-Object Management.Automation.PSCredential ('{}', (ConvertTo-SecureString '{}' -AsPlainText -Force)))",
        //         remote_computer.address,
        //         joined_command,
        //         remote_computer.username,
        //         remote_computer.password
        //     ),
        //     ">".to_string(),
        //     output_file_path
        // ]

        // vec![
        //     program_name,
        //     "-command".to_string(),
        //     "Invoke-Command".to_string(),
        //     "-ComputerName".to_string(),
        //     remote_computer.address.clone(),
        //     "-ScriptBlock".to_string(),
        //     format!(
        //         "{{ {} }}",
        //         joined_command,
        //     ),
        //     "-credential".to_string(),
        //     format!(
        //         "(New-Object Management.Automation.PSCredential ('{}', (ConvertTo-SecureString '{}' -AsPlainText -Force)))",
        //         remote_computer.username,
        //         remote_computer.password
        //     ),
        //     ">".to_string(),
        //     output_file_path
        // ]

        // let prefix = vec![
        //     program_name,
        //     "-command".to_string(),
        //     "Invoke-Command".to_string(),
        //     "-ComputerName".to_string(),
        //     remote_computer.address.clone(),
        //     "-ScriptBlock".to_string(),
        //     "{".to_string(),
        // ];
        // let suffix = vec![
        //         "}".to_string(),
        //         "-credential".to_string(),
        //         format!(
        //             "(New-Object Management.Automation.PSCredential ('{}', (ConvertTo-SecureString '{}' -AsPlainText -Force)))",
        //             remote_computer.username,
        //             remote_computer.password
        //         ),
        //         ">".to_string(),
        //         output_file_path
        //     ];
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
            ">".to_string(),
            output_file_path
        ];
        prefix.into_iter()
            .chain(command.into_iter())
            .chain(suffix.into_iter())
            .collect()
    }
}
