use crate::remote::{Connector, Computer};

pub struct PsExec {}

impl Connector for PsExec {
    fn connect_method_name(&self) -> &'static str {
        return "PAEXEC";
    }

    fn prepare_command(&self,
                       remote_computer: &Computer,
                       command: Vec<String>,
                       output_file_path: Option<String>,
    ) -> Vec<String> {
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
        let almost_result = prefix.into_iter()
            .chain(command.into_iter());
        match output_file_path {
            None => almost_result.collect(),
            Some(output_file_path) =>
                almost_result
                    .chain(vec![
                        ">".to_string(),
                        output_file_path
                    ]).collect()
        }
    }
}
