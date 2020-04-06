use std::path::{Path, PathBuf};
use crate::remote::Computer;
use std::io;
use crate::process_runner::run_process_blocking;

pub trait Copier {
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

pub struct XCopy {}

impl Copier for XCopy {
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
            "/f".to_string(),
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
                format!("\\\\{}\\IPC$", self.computer.address),
                // format!("\\\\{}", self.computer.address),
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