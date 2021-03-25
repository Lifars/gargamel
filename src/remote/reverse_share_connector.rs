use crate::remote::{Connector, Computer, Command, RemoteFileCopier, Cmd, WindowsRemoteFileHandler, FileCopier, copy_from_remote_wildcards, PsExec, GARGAMEL_SHARED_FOLDER_NAME, RemoteCmd};
use std::time::Duration;
use std::io::Error;
use std::path::{PathBuf, Path};
use std::io;
use crate::process_runner::run_process_blocking;


pub struct RevShareConnector {
    connector_impl: Box<dyn Connector>
}

impl RevShareConnector {
    pub fn new(connector_impl: Box<dyn Connector>) -> RevShareConnector {
        let result = RevShareConnector { connector_impl };
        result.open_connection();
        result
    }

    fn open_connection(
        &self
    ) {
        let mut args = vec![
            "share".to_string(),
            format!("{}=C:", GARGAMEL_SHARED_FOLDER_NAME),
            "/GRANT:Everyone,FULL".to_string()
        ];
        run_process_blocking(
            "NET",
            &args,
        ).expect(&format!(
            "Cannot establish share using \"net share\" for {}=C:", GARGAMEL_SHARED_FOLDER_NAME
        ));
    }
}

impl FileCopier for RevShareConnector {
    fn copy_file(&self, source: &Path, target: &Path) -> Result<(), Error> {
        RemoteCmd::new(self).copy_file(source, target)
    }

    fn delete_file(&self, target: &Path) -> Result<(), Error> {
        RemoteCmd::new(self).delete_file(target)
    }

    fn method_name(&self) -> &'static str {
        RemoteCmd::new(self).method_name()
    }
}

impl Connector for RevShareConnector {
    fn connect_method_name(&self) -> &'static str {
        self.connector_impl.connect_method_name()
    }

    fn computer(&self) -> &Computer {
        self.connector_impl.computer()
    }

    fn copier(&self) -> &dyn RemoteFileCopier {
        self as &dyn RemoteFileCopier
    }

    fn remote_temp_storage(&self) -> &Path {
        self.connector_impl.remote_temp_storage()
    }

    fn connect_and_run_local_program(&self,
                                     command_to_run: Command<'_>,
                                     timeout: Option<Duration>,
    ) -> Result<Option<PathBuf>, Error> {
        let local_program = &command_to_run.command[0];
        let local_program_path = Path::new(local_program);
        if let Err(err) = self.copy_to_remote(local_program_path, self.remote_temp_storage()){
            error!("{}", err);
        }
        let local_program_on_target_path = self.remote_temp_storage().join(local_program_path.file_name().unwrap());
        let mut command = command_to_run.command;
        command[0] = local_program_on_target_path.to_string_lossy().to_string();

        let command_to_run = Command {
            command,
            ..command_to_run
        };
        let result = self.connect_and_run_command(command_to_run, timeout);
        let _ = self.delete_remote_file(&local_program_on_target_path);
        result
    }

    fn prepare_command(&self,
                       command: Vec<String>,
                       output_file_path: Option<&str>,
                       elevated: bool,
    ) -> Vec<String> {
        self.connector_impl.prepare_command(command, output_file_path, elevated)
    }
}

impl RemoteFileCopier for RevShareConnector {
    fn remote_computer(&self) -> &Computer {
        self.computer()
    }

    fn copier_impl(&self) -> &dyn FileCopier {
        self as &dyn FileCopier
    }

    fn path_to_remote_form(
        &self,
        path: &Path,
    ) -> PathBuf {
        PathBuf::from(format!(
            "\\\\{}\\{}",
            gethostname::gethostname().to_string_lossy(),
            path.to_str().unwrap().replace("C:", GARGAMEL_SHARED_FOLDER_NAME)
        ))
    }

    fn copy_to_remote(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        self.copier_impl().copy_file(&self.path_to_remote_form(source), target)
    }

    fn delete_remote_file(&self, target: &Path) -> io::Result<()> {
        self.copier_impl().delete_file(target)
    }

    fn copy_from_remote(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        self.copier_impl().copy_file(source, &self.path_to_remote_form(target))
    }
}

impl Drop for RevShareConnector {
    fn drop(&mut self) {
        run_process_blocking(
            "NET",
            &[
                "share".to_string(),
                "/Y".to_string(),
                "/D".to_string(),
                GARGAMEL_SHARED_FOLDER_NAME.to_string()
            ],
        ).expect(&format!(
            "Cannot drop connection using \"net share\" to {}", GARGAMEL_SHARED_FOLDER_NAME
        ));
    }
}
