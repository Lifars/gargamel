use std::path::{Path, PathBuf};
use crate::remote::Computer;
use std::io;
use crate::process_runner::{run_process_blocking, run_piped_processes_blocking};

pub trait Copier {
    fn copy_file(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()>;
}

pub struct XCopy {}

impl Copier for XCopy {
    fn copy_file(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        let args = vec![
            source.to_string_lossy().to_string(),
            target.to_string_lossy().to_string(),
            "/y".to_string()
        ];
        run_process_blocking(
            "xcopy",
            &args,
        )
    }
}

pub struct PsCopyItem {}

impl Copier for PsCopyItem {
    fn copy_file(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        let args = vec![
            "Copy-Item".to_string(),
            format!("'{}'", source.to_string_lossy()),
            format!("'{}'", target.to_string_lossy()),
        ];
        run_process_blocking(
            "powershell.exe",
            &args,
        )
    }
}

pub struct Scp<'a> {
    pub computer: &'a Computer,
    pub key_file: Option<&'a Path>,
}

impl<'a> Copier for Scp<'a> {
    fn copy_file(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        let mut scp = vec![
            "-l".to_string(),
            self.computer.username.clone(),
            "-pw".to_string(),
            self.computer.password.clone(),
        ];
        if self.key_file.is_some() {
            scp.push("-i".to_string());
            scp.push(self.key_file.unwrap().to_string_lossy().to_string())
        }
        scp.push(format!("{}", source.to_string_lossy()));
        scp.push(format!("{}", target.to_string_lossy()));
        run_piped_processes_blocking(
            "cmd",
            &[
                "/c".to_string(),
                "echo".to_string(),
                "n".to_string()
            ],
            "pscp.exe",
            &scp,
        )
    }
}

pub trait RemoteCopier {
    fn computer(&self) -> &Computer;
    fn copier_impl(&self) -> &dyn Copier;

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

    fn copy_from_remote(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        self.copier_impl().copy_file(&self.path_to_remote_form(source), target)
    }
}

/// Use factory mathods to properly initialize the struct.
pub struct WindowsRemoteCopier<'a> {
    computer: &'a Computer,
    copier_impl: &'a dyn Copier,
}

impl<'a> Drop for WindowsRemoteCopier<'a> {
    fn drop(&mut self) {
        run_process_blocking(
            "NET",
            &[
                "USE".to_string(),
                // format!("\\\\{}\\IPC$", self.computer.address),
                format!("\\\\{}", self.computer.address),
                "/D".to_string()
            ],
        ).expect(&format!(
            "Cannot drop connection using \"net use\" to {}", self.computer.address
        ));
    }
}

impl<'a> WindowsRemoteCopier<'a> {
    pub fn new(
        computer: &'a Computer,
        implementation: &'a dyn Copier,
    ) -> WindowsRemoteCopier<'a> {
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
        WindowsRemoteCopier { computer, copier_impl: implementation }
    }
}

impl<'a> RemoteCopier for WindowsRemoteCopier<'a> {
    fn computer(&self) -> &Computer {
        self.computer
    }

    fn copier_impl(&self) -> &dyn Copier {
        self.copier_impl
    }

    fn path_to_remote_form(
        &self,
        path: &Path,
    ) -> PathBuf {
        PathBuf::from(format!(
            "\\\\{}\\{}",
            self.computer().address,
            path.to_str().unwrap().replacen(":", "$", 1)
        ))
    }
}

pub struct ScpRemoteCopier<'a> {
    copier_impl: Scp<'a>,
}

impl<'a> ScpRemoteCopier<'a> {
    pub fn new(
        computer: &'a Computer,
        key_file: Option<&'a Path>
    ) -> ScpRemoteCopier<'a> {
        ScpRemoteCopier {
            copier_impl: Scp {
                computer,
                key_file
            }
        }
    }
}

impl<'a> RemoteCopier for ScpRemoteCopier<'a> {
    fn computer(&self) -> &Computer {
        self.copier_impl.computer
    }

    fn copier_impl(&self) -> &dyn Copier {
        &self.copier_impl
    }

    fn path_to_remote_form(&self, path: &Path) -> PathBuf {
        PathBuf::from(format!(
            "{}:{}",
            self.computer().address,
            path.to_str().unwrap()
        ))
    }
}

// pub struct Downloader<'a> {
//     remote_copier: &'a RemoteCopier<'a>
// }
//
// pub struct Uploader<'a> {
//     remote_copier: &'a RemoteCopier<'a>
// }
//
// impl<'a> Copier for Downloader<'a> {
//     fn copy_file(&self,
//                  source: &Path,
//                  target: &Path,
//     ) -> io::Result<()> {
//         self.remote_copier.copy_from_remote(source, target)
//     }
// }
//
// impl<'a> Copier for Uploader<'a> {
//     fn copy_file(&self,
//                  source: &Path,
//                  target: &Path,
//     ) -> io::Result<()> {
//         self.remote_copier.copy_to_remote(source, target)
//     }
// }