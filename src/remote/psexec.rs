use crate::remote::{Connector, RemoteComputer};

pub struct PsExec {}

impl Connector for PsExec {
    fn connect_method_name(&self) -> &'static str {
        return "PAEXEC";
    }

        fn prepare_command(&self, remote_computer: &RemoteComputer, command: Vec<String>, output_file_path: String) -> Vec<String> {
        let address = format!("\\\\{}", remote_computer.address);
        let program_name = "paexec.exe".to_string();
        let prefix = vec![
            program_name,
            address,
            "-u".to_string(),
            remote_computer.username.clone(),
            "-p".to_string(),
            remote_computer.password.clone(),
            // "-s".to_string()
        ];
        let suffix = vec![
            ">".to_string(),
            output_file_path
        ];

        prefix.into_iter()
            .chain(command.into_iter())
            .chain(suffix.into_iter())
            .collect()
    }
}
