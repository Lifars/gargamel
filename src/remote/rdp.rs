use crate::remote::{Connector, Computer};
use std::path::Path;

pub struct Rdp {}

impl Connector for Rdp {
    fn connect_method_name(&self) -> &'static str {
        return "RDP";
    }

    fn prepare_command(&self,
                       remote_computer: &Computer,
                       command: Vec<String>,
                       output_file_path: Option<String>,
    ) -> Vec<String> {
        let program_name = "SharpRDP.exe".to_string();
        let command_joined: String = command.join(" ");
        let command_as_arg = match output_file_path {
            None => format!("command={}", command_joined),
            Some(output_file_path) => {
                let path = Path::new(&output_file_path);
                let canon_path = dunce::canonicalize(path).unwrap();
                let as_remote_path = canon_path
                    .to_string_lossy()
                    .replacen(":", "", 1);
                format!(
                    // "command={} -p.i.p.e- Out-File -FilePath \\\\tsclient\\C\\Users\\Public\\funguj.txt",//\\\\tsclient\\{}\"",
                    "command={} -p.i.p.e- Out-File -FilePath \\\\tsclient\\{}",
                    command_joined,
                    as_remote_path
                )
            },
        };

        vec![
            program_name,
            format!("computername={}", &remote_computer.address),
            format!("username={}", &remote_computer.username),
            format!("password={}", &remote_computer.password),
            "exec=ps".to_string(),
            "takeover=true".to_string(),
            "connectdrive=true".to_string(),
            command_as_arg
        ]
    }
}
