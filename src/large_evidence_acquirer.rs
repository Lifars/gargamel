use crate::process_runner::create_report_path;
use std::path::Path;
use crate::remote::{Connector, Compression, CompressCopier, RemoteFileCopier, Command};
use std::time::Duration;
use std::thread;

pub(crate) struct LargeEvidenceAcquirer<'a> {
    pub(crate) store_directory: &'a Path,
    pub(crate) connector: &'a dyn Connector,
    pub(crate) compress_timeout: Option<Duration>,
    pub(crate) compression: Compression,
    pub(crate) report_extension: &'a str,
    pub(crate) overwrite_switch: Option<&'a str>
}

impl<'a> LargeEvidenceAcquirer<'a> {
    pub(crate) fn run(
        &self,
        command: &[String],
        report_filename_prefix: &str
    ) {
        if command.is_empty() {
            return;
        }
        let report_path = create_report_path(
            self.connector.computer(),
            self.store_directory,
            report_filename_prefix,
            self.connector.connect_method_name(),
            self.report_extension,
        );

        let remote_report_path = self.connector.remote_temp_storage()
            .join(report_path.file_name().unwrap())
            .to_string_lossy()
            .to_string();
        let mut command = command.to_vec();
        command.push(remote_report_path.clone());
        if let Some(overwrite) = self.overwrite_switch {
            command.push(overwrite.to_string());
        }
        let remote_connection = Command::new(
            command,
            None,
            report_filename_prefix,
            false,
        );

        info!("{}: Checking {}",
              self.connector.connect_method_name(),
              report_filename_prefix.replace("-", " ")
        );

        match self.connector.connect_and_run_command(remote_connection, None) {
            Ok(_) => {}
            Err(err) => {
                error!(
                    "Error running command to acquire {}. Cause: {}",
                    report_filename_prefix,
                    err
                )
            }
        }
        thread::sleep(Duration::from_millis(10_000));

        let _compression_split_copier = CompressCopier::new(self.connector, true, self.compress_timeout.clone());
        let _compression_copier = CompressCopier::new(self.connector, false, self.compress_timeout.clone());
        let copier = match self.compression {
            Compression::No => self.connector.copier(),
            Compression::Yes => &_compression_copier as &dyn RemoteFileCopier,
            Compression::YesSplit => &_compression_split_copier as &dyn RemoteFileCopier,
        };

        match copier.copy_from_remote(Path::new(&remote_report_path), report_path.parent().unwrap()) {
            Ok(_) => {}
            Err(err) => {
                error!("Cannot download {} report from {} using method {} due to {}",
                       report_filename_prefix,
                       self.connector.computer().address,
                       self.connector.connect_method_name(),
                       err
                )
            }
        }
        thread::sleep(Duration::from_secs(2));
        match copier.delete_remote_file(Path::new(&remote_report_path)) {
            Ok(_) => {}
            Err(err) => {
                error!("Cannot delete remote file {} using method {} due to: {}",
                       report_filename_prefix,
                       self.connector.connect_method_name(),
                       err
                )
            }
        }
    }
}