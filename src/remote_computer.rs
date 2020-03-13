use std::io::Result;
use crate::process_runner::run_process_blocking;
use uuid::Uuid;
use std::ffi::OsStr;

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
    // fn connect_and_run_command_with_output(
    //     &self,
    //     remote_computer: &RemoteComputer,
    //     command: Vec<String>,
    // ) -> Result<BufReader<R>> {
    //     let prepared = self.prepare_remote_process(remote_computer, command);
    //     run_process_async_with_output(&prepared.program_path, &prepared.all_program_args)
    //             .map(|process| process.stdout_reader)
    //  //       run_process("ipconfig", &["/all"])
    ////         run_process("ping", &["google.com"])
    ////         run_process("netstat", &["-ano"])
    // .map(|process| process.stdout_reader)
    // }

    fn connect_method_name(&self) -> &'static str;

    fn connect_and_run_command(
        &self,
        remote_computer: &RemoteComputer,
        command: Vec<String>,
    ) -> Result<()> {
        debug!("Trying to run command {:?} on {}", command, remote_computer.address);
        let prepared = self.prepare_remote_process(remote_computer, command);
        run_process_blocking(&prepared.program_path, &prepared.all_program_args)
    }

    fn prepare_remote_process(
        &self,
        remote_computer: &RemoteComputer,
        command: Vec<String>,
    ) -> PreparedProgramToRun;
}

pub struct PsExec {}

impl RemoteComputerConnector for PsExec {

    fn connect_method_name(&self) -> &'static str {
        return "paexec";
    }

    fn prepare_remote_process(&self,
                              remote_computer: &RemoteComputer,
                              command: Vec<String>,
    ) -> PreparedProgramToRun {
        let address_for_psexec = format!("\\\\{}", remote_computer.address);
        let program_name = "paexec.exe".to_string();
        let mut all_args: Vec<String> = vec![
            "/c".to_string(),
            program_name,
            address_for_psexec,
            "-u".to_string(),
            remote_computer.username.clone(),
            "-p".to_string(),
            remote_computer.password.clone(),
            // "-s".to_string()
        ];
        all_args.extend(command);

        debug!("Final command to run on {} is \"{} {:?}\"", remote_computer.address, &all_args[0], all_args);
        PreparedProgramToRun {
            program_path: "cmd.exe".to_string(),
            all_program_args: all_args,
        }
    }
}

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