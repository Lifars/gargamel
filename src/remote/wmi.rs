use crate::remote::{Connector, RemoteComputer};

pub struct Wmi {}

impl Connector for Wmi {
    fn connect_method_name(&self) -> &'static str {
        return "WMI";
    }

    fn prepare_command(&self,
                       remote_computer: &RemoteComputer,
                       command: Vec<String>,
                       output_file_path: String
    ) -> Vec<String> {
        let program_name = "wmic.exe".to_string();
        let output = format!("/OUTPUT:{}", output_file_path);
        let address = format!("/NODE:{}", remote_computer.address);
        let user = format!("/USER:{}", remote_computer.username);
        let password = format!("/PASSWORD:{}", remote_computer.password);
        let prefix = vec![
            program_name,
            output,
            address,
            user,
            password,
        ];
        prefix.into_iter().chain(command.into_iter()).collect()
    }
}

pub static WMI_CONNECTOR: Wmi = Wmi {};
