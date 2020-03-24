use crate::remote::{Connector, Computer};

pub struct Local {}

impl Connector for Local {
    fn connect_method_name(&self) -> &'static str {
        return "LOCAL";
    }

    fn prepare_command(&self,
                       _remote_computer: &Computer,
                       command: Vec<String>,
                       _output_file_path: Option<String>
    ) -> Vec<String> {
        command
    }
}