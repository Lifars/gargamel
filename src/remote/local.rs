use crate::remote::{Connector, RemoteComputer};

pub struct Local {}

impl Connector for Local {
    fn connect_method_name(&self) -> &'static str {
        return "LOCAL";
    }

    fn prepare_command(&self,
                       _remote_computer: &RemoteComputer,
                       command: Vec<String>,
                       _output_file_path: String
    ) -> Vec<String> {
        command
    }
}