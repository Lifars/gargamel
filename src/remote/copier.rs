use std::path::{Path, PathBuf, Component};
use crate::remote::{Computer, Connector};
use std::io;
use crate::process_runner::run_process_blocking;
use std::env::temp_dir;
use wildmatch::WildMatch;

pub trait FileCopier {
    fn copy_file(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()>;

    fn delete_file(&self,
                   target: &Path,
    ) -> io::Result<()>;

    fn method_name(&self) -> &'static str;
}

pub struct Cmd {}

impl FileCopier for Cmd {
    fn copy_file(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        let args = vec![
            "/y".to_string(),
            "/i".to_string(),
            "/c".to_string(),
            source.to_string_lossy().to_string(),
            target.to_string_lossy().to_string(),
        ];
        run_process_blocking(
            "xcopy",
            &args,
        )
    }

    fn delete_file(&self, target: &Path) -> io::Result<()> {
        let args = vec![
            "/F".to_string(),
            "/Q".to_string(),
            target.to_string_lossy().to_string(),
        ];
        run_process_blocking(
            "del",
            &args,
        )
    }

    fn method_name(&self) -> &'static str {
        "XCopy"
    }
}

pub trait RemoteFileCopier {
    fn remote_computer(&self) -> &Computer;
    fn copier_impl(&self) -> &dyn FileCopier;

    fn path_to_remote_form(
        &self,
        path: &Path,
    ) -> PathBuf;

    fn copy_to_remote(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        self.copier_impl().copy_file(source, &self.path_to_remote_form(target))
    }

    fn delete_remote_file(&self, target: &Path) -> io::Result<()> {
        self.copier_impl().delete_file(&self.path_to_remote_form(target))
    }

    fn copy_from_remote(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        self.copier_impl().copy_file(&self.path_to_remote_form(source), target)
    }

    fn method_name(&self) -> &'static str {
        self.copier_impl().method_name()
    }
}

pub fn copy_from_remote_wildcards<F>(
    source: &Path,
    target: &Path,
    connector: &dyn Connector,
    copy_fn: F,
) -> io::Result<()>
    where F: Fn(&Path, &Path) -> io::Result<()> {
    trace!("Copier supports wildcards");
    let dir = source
        .components()
        .take_while(|item| !item.as_os_str().to_str().unwrap_or_default().contains("*"))
        .map(|item| item.as_os_str())
        .collect::<PathBuf>();

    let wildcarded = source
        .components()
        .skip_while(|item| !item.as_os_str().to_str().unwrap_or_default().contains("*"))
        .take(1)
        .collect::<Vec<Component>>()
        .get(0)
        .map(|it| it.as_os_str().to_string_lossy());

    let rem = source
        .components()
        .skip_while(|item| !item.as_os_str().to_str().unwrap_or_default().contains("*"))
        .skip(1)
        .map(|item| item.as_os_str())
        .collect::<PathBuf>();

    if dir.components().count() >= source.components().count() - 1 {
        copy_fn(source, target)
    } else {
        let wildcarded = wildcarded.unwrap();
        connector
            .list_dirs(&dir, &temp_dir())
            .iter()
            .filter(|path_item| {
                trace!("Matching {} with {}", &wildcarded, path_item);
                WildMatch::new(&wildcarded).matches(path_item)
            }
            )
            .for_each(|item| {
                let src = dir.join(item).join(&rem);
                debug!("Copying wildcarded path {} to {}", src.display(), target.display());
                if copy_fn(&src, target).is_err() {
                    error!("Error remote {} copying from {} to {}", connector.computer().address, src.display(), target.display())
                }
            });
        Ok(())
    }
}

pub struct WindowsRemoteFileHandler {
    computer: Computer,
    copier_impl: Box<dyn FileCopier>,
    pub custom_share_folder: Option<String>,
}

impl Drop for WindowsRemoteFileHandler {
    fn drop(&mut self) {
        run_process_blocking(
            "NET",
            &[
                "USE".to_string(),
                format!("\\\\{}", self.computer.address),
                // format!("\\\\{}", self.computer.address),
                "/D".to_string()
            ],
        ).expect(&format!(
            "Cannot drop connection using \"net use\" to {}", self.computer.address
        ));
    }
}

impl WindowsRemoteFileHandler {
    pub fn new(
        computer: Computer,
        copier_impl: Box<dyn FileCopier>,
        custom_share_folder: Option<String>,
    ) -> WindowsRemoteFileHandler {
        let mut args = vec![
            "USE".to_string(),
            format!("\\\\{}", computer.address),
        ];
        let username = computer.domain_username();
        args.push(format!("/u:{}", username));
        if let Some(password) = &computer.password {
            args.push(password.clone());
        }
        run_process_blocking(
            "NET",
            &args,
        ).expect(&format!(
            "Cannot establish connection using \"net use\" to {}", &computer.address
        ));
        WindowsRemoteFileHandler { computer, copier_impl, custom_share_folder }
    }

    fn open_connection(
        &self
    ) {
        let mut args = vec![
            "USE".to_string(),
            format!("\\\\{}", self.computer.address),
        ];
        let username = self.computer.domain_username();
        args.push(format!("/u:{}", username));
        if let Some(password) = &self.computer.password {
            args.push(password.clone());
        }
        run_process_blocking(
            "NET",
            &args,
        ).expect(&format!(
            "Cannot establish connection using \"net use\" to {}", &self.computer.address
        ));
    }
}

impl RemoteFileCopier for WindowsRemoteFileHandler {
    fn remote_computer(&self) -> &Computer {
        &self.computer
    }

    fn copier_impl(&self) -> &dyn FileCopier {
        self.copier_impl.as_ref()
    }

    fn path_to_remote_form(
        &self,
        path: &Path,
    ) -> PathBuf {
        match self.custom_share_folder.as_ref() {
            None => {
                self.open_connection();
                PathBuf::from(format!(
                    "\\\\{}\\{}",
                    self.remote_computer().address,
                    path.to_str().unwrap().replacen(":", "$", 1)
                ))
            }
            // Some(custom_share) => {
            //     self.open_connection();
            //     PathBuf::from(format!(
            //         "\\\\{}\\{}",
            //         self.remote_computer().address,
            //         path.to_str().unwrap().replacen(":", "$", 1).replace("C", custom_share)
            //     ))
            // }

            Some(custom_share) => {
                if custom_share.len() == 1 {
                    PathBuf::from(
                        path.to_str().unwrap().replace("C", custom_share)
                    )
                } else {
                    PathBuf::from(format!(
                        "\\\\{}\\{}",
                        self.remote_computer().address,
                        path.to_str().unwrap().replace("C:", custom_share)
                    ))
                }
            }
        }
    }
}
