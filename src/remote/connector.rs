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

pub trait Connector {
    fn connect_method_name(&self) -> &'static str;

    fn connect_and_run_command(
        &self,
        remote_computer: &RemoteComputer,
        output_file_path: String,
        command: Vec<String>,
    ) -> Result<()> {
        debug!("Trying to run command {:?} on {}", command, remote_computer.address);
        // let command_prefix = self.prefix_connector_arguments(
        //     remote_computer,
        //     output_file_path.clone(),
        // );
        // let command_suffix = self.postfix_connector_arguments(remote_computer, output_file_path);

        let processed_command = self.prepare_command(remote_computer,   command, output_file_path);

        let prepared_command = self.prepare_remote_process(
            // command_prefix,
            // command,
            // command_suffix,
            processed_command
        );
        run_process_blocking(&prepared_command.program_path, &prepared_command.all_program_args)
    }

    fn prepare_remote_process(&self,
                              // pre_command: Vec<String>,
                              processed_command: Vec<String>,
                              // post_command: Vec<String>,
    ) -> PreparedProgramToRun {
        let all_args = iter::once("/c".to_string())
            // .chain(pre_command.into_iter())
            .chain(processed_command.into_iter())
            // .chain(post_command.into_iter())
            .collect();
        PreparedProgramToRun {
            program_path: "cmd.exe".to_string(),
            all_program_args: all_args,
        }
    }

    fn prepare_command(&self,
                       remote_computer: &RemoteComputer,
                       command: Vec<String>,
                       output_file_path: String,
    ) -> Vec<String>;
}