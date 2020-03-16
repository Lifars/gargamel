use std::io::Result;
use crate::process_runner::run_process_blocking;
use std::iter;

pub struct RemoteComputer {
    pub address: String,
    pub username: String,
    pub password: String,
}

pub struct PreparedProgramToRun {
    pub program_path: String,
    pub all_program_args: Vec<String>,
}

pub trait RemoteComputerConnector {
    fn connect_method_name(&self) -> &'static str;

    fn connect_and_run_command(
        &self,
        remote_computer: &RemoteComputer,
        output_file_path: String,
        command: Vec<String>,
    ) -> Result<()> {
        debug!("Trying to run command {:?} on {}", command, remote_computer.address);
        let prepared = self.prepare_remote_process(
            self.prefix_connector_arguments(remote_computer, output_file_path.clone()),
            command,
            self.postfix_connector_arguments(remote_computer, output_file_path)
        );
        run_process_blocking(&prepared.program_path, &prepared.all_program_args)
    }

    fn prepare_remote_process(&self,
                              pre_command: Vec<String>,
                              command: Vec<String>,
                              post_command: Vec<String>,
    ) -> PreparedProgramToRun {
        let all_args = iter::once("/c".to_string())
            .chain(pre_command.into_iter())
            .chain(command.into_iter())
            .chain(post_command.into_iter())
            .collect();
        PreparedProgramToRun {
            program_path: "cmd.exe".to_string(),
            all_program_args: all_args,
        }
    }

    fn prefix_connector_arguments(&self,
                                  remote_computer: &RemoteComputer,
                                  output_file_path: String,
    ) -> Vec<String>;

    fn postfix_connector_arguments(&self,
                                   remote_computer: &RemoteComputer,
                                   output_file_path: String,
    ) -> Vec<String>;
}

pub struct Local {}

impl RemoteComputerConnector for Local {
    fn connect_method_name(&self) -> &'static str {
        return "LOCAL";
    }

    fn prefix_connector_arguments(&self,
                                  remote_computer: &RemoteComputer,
                                  output_file_path: String) -> Vec<String> {
        vec![]
    }

    fn postfix_connector_arguments(&self,
                                   remote_computer: &RemoteComputer,
                                   output_file_path: String,
    ) -> Vec<String> {
        unimplemented!()
    }
}

pub static LOCAL_CONNECTOR: Local = Local {};

pub struct PsExec {}

impl RemoteComputerConnector for PsExec {
    fn connect_method_name(&self) -> &'static str {
        return "PAEXEC";
    }

    fn prefix_connector_arguments(&self,
                                  remote_computer: &RemoteComputer,
                                  output_file_path: String,
    ) -> Vec<String> {
        let address = format!("\\\\{}", remote_computer.address);
        let program_name = "paexec.exe".to_string();
        vec![
            program_name,
            address,
            "-u".to_string(),
            remote_computer.username.clone(),
            "-p".to_string(),
            remote_computer.password.clone(),
            // "-s".to_string()
        ]
    }

    fn postfix_connector_arguments(&self,
                                   remote_computer: &RemoteComputer,
                                   output_file_path: String,
    ) -> Vec<String> {
        vec![
            ">".to_string(),
            output_file_path
        ]
    }
}

pub struct Wmi {}

impl RemoteComputerConnector for Wmi {
    fn connect_method_name(&self) -> &'static str {
        return "WMI";
    }

    fn prefix_connector_arguments(&self,
                                  remote_computer: &RemoteComputer,
                                  output_file_path: String,
    ) -> Vec<String> {
        let address = format!("/NODE:{}", remote_computer.address);
        let user = format!("/USER:{}", remote_computer.username);
        let password = format!("/PASSWORD:{}", remote_computer.password);
        let output = format!("/OUTPUT:{}", output_file_path);
        let program_name = "wmic.exe".to_string();
        vec![
            program_name,
            output,
            address,
            user,
            password,
        ]
    }

    fn postfix_connector_arguments(&self, remote_computer: &RemoteComputer, output_file_path: String) -> Vec<String> {
        vec![
            // ">>".to_string(),
            // output_file_path
        ]
    }
}

pub static WMI_CONNECTOR: Wmi = Wmi {};

// pub struct Powershell {}
//
// impl RemoteComputerConnector for Powershell {
//     fn connect_and_run_command(&self,
//                                remote_computer: &RemoteComputer,
//                                command: &[&str],
//     ) -> Result<BufReader<ChildStdout>> {
//         let address_for_psexec = format!("\\\\{}", remote_computer.address);
//         let mut all_args: Vec<&str> = vec![
//             &address_for_psexec,
//             "-u",
//             remote_computer.username,
//             "-p",
//             remote_computer.password,
//             "-accepteula"
//         ];
//         all_args.extend_from_slice(command);
//         run_process("PsExec.exe", &all_args)
//             //     .map(|process| process.stdout_reader)
//             // run_process("ipconfig", &["/all"])
//             // run_process("ping", &["google.com"])
//             // run_process("netstat", &["-ano"])
//             .map(|process| process.stdout_reader)
//     }
// }


// pub struct Wmi {}
//
// impl RemoteComputerConnector<ChildStdout> for Wmi {
//     fn run_remote_command(&self,
//                           address: &str,
//                           user: &str,
//                           password: &str,
//                           command: &[&str],
//     ) -> Result<BufReader<ChildStdout>> {
//         let address_for_wmi = format!("/node:\"{}\"", address);
//         let user_for_wmi = format!("/user:\"{}\"", user);
//         let password_for_wmi = format!("/password:\"{}\"", password);
//
//         let remote_output_filename = format!("{}.txt", Uuid::new_v4());
//         let remote_command_path = {
//             let joined_command: String = command.join(" ");
//             format!(
//                 "\"cmd /C > C:\\Users\\{}\\AppData\\Local\\Temp\\{} 2>&1 {}\"",
//                 user,
//                 remote_output_filename,
//                 joined_command
//             )
//         };
//         let mut all_args: Vec<&str> = vec![
//             &address_for_wmi,
//             &user_for_wmi,
//             &password_for_wmi,
//             "process",
//             "call",
//             "remote",
//             &remote_command_path
//         ];
//         all_args.extend_from_slice(command);
//         run_process("wmic.exe", &all_args)
//             //     .map(|process| process.stdout_reader)
//             // run_process("ipconfig", &["/all"])
//             // run_process("ping", &["google.com"])
//             // run_process("netstat", &["-ano"])
//             .map(|process| process.stdout_reader)
//     }
// }