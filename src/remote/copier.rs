use std::path::Path;
use crate::remote::Computer;
use std::io;
use crate::process_runner::run_process_blocking;

/// Use Copier::new(...) to properly initialize the struct.
pub struct Copier<'a> {
    computer: &'a Computer
}

impl<'a> Drop for Copier<'a> {
    fn drop(&mut self) {
        run_process_blocking(
            "NET",
            &[
                "USE".to_string(),
                format!("\\\\{}\\IPC$", self.computer.address),
                "/D".to_string()
            ],
        ).expect(&format!(
            "Cannot drop connection using \"net use\" to {}", self.computer.address
        ));
    }
}

impl Copier<'_> {
    pub fn new(computer: &Computer) -> Copier {
        run_process_blocking(
            "NET",
            &[
                "USE".to_string(),
                format!("\\\\{}\\IPC$", computer.address),
                format!("/u:{}", computer.username),
                format!("{}", computer.password),
            ],
        ).expect(&format!(
            "Cannot establish connection using \"net use\" to {}", &computer.address
        ));
        Copier { computer }
    }

    pub fn copy_to_remote(
        &self,
        source: &Path,
        target: &Path,
        filename: Option<&str>,
    ) -> io::Result<()> {
        self.copy_file(source, target, filename, true)
    }

    pub fn copy_from_remote(
        &self,
        source: &Path,
        target: &Path,
        filename: Option<&str>,
    ) -> io::Result<()> {
        self.copy_file(source, target, filename, false)
    }

    pub fn copy_file(
        &self,
        source: &Path,
        target: &Path,
        filename: Option<&str>,
        target_is_remote: bool,
    ) -> io::Result<()> {
        let mut args = self.source_target(source, target, target_is_remote);
        match filename {
            None => {}
            Some(filename) => args.push(filename.to_string()),
        }
        args.push("/R:1".to_string());
        args.push("/W:1".to_string());
        args.push("/V".to_string());
        args.push("/TEE".to_string());
        run_process_blocking(
            "robocopy",
            &args,
        )
    }

    fn source_target(&self,
                     source: &Path,
                     target: &Path,
                     target_is_remote: bool,
    ) -> Vec<String> {
        if target_is_remote {
            vec![
                source.to_str().unwrap().to_string(),
                self.path_to_remote_form(target),
            ]
        } else {
            vec![
                self.path_to_remote_form(source),
                target.to_str().unwrap().to_string(),
            ]
        }
    }

    fn path_to_remote_form(
        &self,
        path: &Path,
    ) -> String {
        format!(
            "\\\\{}\\{}",
            self.computer.address,
            path.to_str().unwrap().replacen(":", "$", 1)
        )
    }
}

