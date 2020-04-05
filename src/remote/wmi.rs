use crate::remote::{Connector, Computer};

pub struct Wmi {}

impl Connector for Wmi {

    fn connect_method_name(&self) -> &'static str {
        return "WMI";
    }

    fn prepare_command(&self,
                       remote_computer: &Computer,
                       command: Vec<String>,
                       output_file_path: Option<String>,
    ) -> Vec<String> {
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

pub struct WmiProcess{}

impl Connector for WmiProcess {

    fn connect_method_name(&self) -> &'static str {
        return "WMI";
    }

    fn prepare_command(&self,
                       remote_computer: &Computer,
                       command: Vec<String>,
                       _output_file_path: Option<String>,
    ) -> Vec<String> {
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
