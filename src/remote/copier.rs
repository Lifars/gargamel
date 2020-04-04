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

pub struct RdpCopy {
    pub computer: Computer,
}

impl Copier for RdpCopy {
    fn copy_file(&self, source: &Path, target: &Path) -> io::Result<()> {
        let args = vec![
            format!("computername={}", &self.computer.address),
            format!("username={}", &self.computer.username),
            format!("password={}", &self.computer.password),
            "exec=cmd".to_string(),
            "takeover=true".to_string(),
            "connectdrive=true".to_string(),
            format!(
                "command=xcopy {} {} /y",
                source.to_string_lossy(),
                target.to_string_lossy()
            )
        ];
        run_process_blocking(
            "SharpRDP.exe",
            &args,
        )
    }
}

pub struct Scp {
    pub computer: Computer,
    pub key_file: Option<PathBuf>,
}

impl Copier for Scp {
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
            scp.push(self.key_file.as_ref().unwrap().to_string_lossy().to_string())
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
pub struct WindowsRemoteCopier {
    computer: Computer,
    copier_impl: Box<dyn Copier>,
}

impl Drop for WindowsRemoteCopier {
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

impl WindowsRemoteCopier {
    pub fn new(
        computer: Computer,
        copier_impl: Box<dyn Copier>,
    ) -> WindowsRemoteCopier {
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
        WindowsRemoteCopier { computer, copier_impl }
    }
}

impl RemoteCopier for WindowsRemoteCopier {
    fn computer(&self) -> &Computer {
        &self.computer
    }

    fn copier_impl(&self) -> &dyn Copier {
        self.copier_impl.as_ref()
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

impl RemoteCopier for Scp {
    fn computer(&self) -> &Computer {
        &self.computer
    }

    fn copier_impl(&self) -> &dyn Copier {
        self as &dyn Copier
    }

    fn path_to_remote_form(&self, path: &Path) -> PathBuf {
        PathBuf::from(format!(
            "{}:{}",
            self.computer().address,
            path.to_str().unwrap()
        ))
    }
}

impl RemoteCopier for RdpCopy{
    fn computer(&self) -> &Computer {
        &self.computer
    }

    fn copier_impl(&self) -> &dyn Copier {
        self as &dyn Copier
    }

    fn path_to_remote_form(&self, path: &Path) -> PathBuf {
        trace!("Converting path {}", path.display());
        // let canon_path = dunce::canonicalize(path).unwrap();
        let as_remote_path = path
            .to_string_lossy()
            .replacen(":", "", 1);
        let tsclient_path = format!("\\\\tsclient\\{}", as_remote_path);
        PathBuf::from(tsclient_path)
    }

    fn copy_to_remote(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        self.copier_impl().copy_file(&self.path_to_remote_form(source), target)
    }

    fn copy_from_remote(
        &self,
        source: &Path,
        target: &Path,
    ) -> io::Result<()> {
        self.copier_impl().copy_file(source, &self.path_to_remote_form(target))
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