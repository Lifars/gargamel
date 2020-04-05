use crate::remote::{Connector, Computer};

pub struct PsExec {}

impl Connector for PsExec {
    fn connect_method_name(&self) -> &'static str {
        return "PSEXEC";
    }

    fn prepare_command(&self,
                       remote_computer: &Computer,
                       command: Vec<String>,
                       output_file_path: Option<String>,
    ) -> Vec<String> {
        let address = format!("\\\\{}", remote_computer.address);
        let program_name = "paexec.exe".to_string();
        let mut prepared_command = vec![
            program_name,
            address,
            "-u".to_string(),
            remote_computer.domain_username(),
        ];
        if let Some(password) = &remote_computer.password {
            prepared_command.push("-p".to_string());
            prepared_command.push(password.clone());
        }
        prepared_command.extend(command.into_iter());
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
