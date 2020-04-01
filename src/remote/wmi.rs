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
        let user = format!("/USER:{}", remote_computer.username);
        let password = format!("/PASSWORD:{}", remote_computer.password);

        let mut final_command = vec![program_name];
        if output_file_path.is_some() {
            final_command.push(format!("/OUTPUT:{}", output_file_path.unwrap()));
        }
        final_command.extend_from_slice(&[
            address,
            user,
            password,
        ]);
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
        let user = format!("/USER:{}", remote_computer.username);
        let password = format!("/PASSWORD:{}", remote_computer.password);

        let mut final_command = vec![program_name];
        // if output_file_path.is_some() {
        //     final_command.push(format!("/OUTPUT:{}", output_file_path.unwrap()));
        // }
        final_command.extend_from_slice(&[
            address,
            user,
            password,
            "process".to_string(),
            "call".to_string(),
            "create".to_string()
        ]);
        final_command.extend(command.into_iter());
        final_command
    }
}
