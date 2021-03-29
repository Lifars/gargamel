use std::io::{Result, BufReader, BufRead};
use crate::process_runner::{run_process_blocking, create_report_path, run_process_blocking_timed};
use std::{iter, thread, io};
use std::path::{Path, PathBuf};
use crate::arg_parser::Opts;
use std::time::Duration;
use crate::remote::{RemoteFileCopier, Local};
use std::fs::File;
use uuid::Uuid;
use rpassword::read_password;

#[derive(Clone)]
pub struct Computer {
    pub address: String,
    pub username: String,
    pub domain: Option<String>,
    pub password: Option<String>,
}

impl Computer {
    pub fn domain_username(&self) -> String {
        match &self.domain {
            None =>
                self.username.clone(),
            Some(domain) =>
                format!("{}\\{}", domain, self.username),
        }
    }
}

pub struct Command<'a> {
    pub command: Vec<String>,
    pub report_store_directory: Option<&'a Path>,
    pub report_filename_prefix: &'a str,
    pub elevated: bool,
}

impl From<Opts> for Computer {
    fn from(opts: Opts) -> Self {
        let (domain, username) = match &opts.user {
            Some(user) => if user.is_empty() {
                (None, "".to_string())
            } else {
                (opts.domain, user.clone())
            },
            None => {
                println!("Domain (optional): ");
                let mut domain = String::new();
                let _ = io::stdin().read_line(&mut domain);

                println!("Username: ");
                let mut user = String::new();
                io::stdin().read_line(&mut user).ok();
                (if domain.trim().is_empty() { None } else { Some(domain) }, user)
            }
        };
        if opts.computer == "127.0.0.1" || opts.computer == "localhost" {
            return Local::new(username, PathBuf::from(opts.remote_store_directory)).computer().clone();
        }
        let password = match &opts.password {
            Some(password) => if password.is_empty() {
                None
            } else {
                Some(password.clone())
            },
            None => {
                println!("Password: ");
                let password = read_password().ok();
                password
            }
        };
        Computer {
            address: opts.computer,
            username,
            domain,
            password,
        }
    }
}

impl Into<Vec<Computer>> for Opts {
    fn into(self) -> Vec<Computer> {
        let file = File::open(&self.computer);
        match file {
            Ok(file) => {
                BufReader::new(file)
                    .lines()
                    .filter_map(|line| line.ok())
                    .filter(|line| !line.trim().is_empty())
                    .filter_map(|line| {
                        let splitted = line
                            .split(" ")
                            .map(|item| item.to_string())
                            .collect::<Vec<String>>();
                        let address = splitted.get(0).cloned();
                        let domain_username = splitted.get(1).cloned();
                        let password = splitted.get(2).cloned();
                        address.map(|address| {
                            let (domain_option, username_option) = match domain_username {
                                None => (self.domain.clone(), self.user.clone()),
                                Some(domain_username_unwrapped) => {
                                    let splitted_du = domain_username_unwrapped
                                        .split("\\")
                                        .map(|item| item.to_string())
                                        .collect::<Vec<String>>();
                                    match splitted_du.len() {
                                        0 => (self.domain.clone(), self.user.clone()),
                                        1 => (self.domain.clone(), Some(splitted_du[0].clone())),
                                        _ => (Some(splitted_du[0].clone()), Some(splitted_du[1].clone()))
                                    }
                                }
                            };
                            let (domain, username) = match username_option {
                                Some(user) => if user.is_empty() {
                                    (domain_option, "".to_string())
                                } else {
                                    (domain_option, user.clone())
                                },
                                None => {
                                    let domain = match domain_option {
                                        None => {
                                            println!("Domain (optional): ");
                                            let mut domain = String::new();
                                            let _ = io::stdin().read_line(&mut domain);
                                            if domain.trim().is_empty() { None } else { Some(domain) }
                                        }
                                        Some(domain) => Some(domain)
                                    };
                                    println!("Username: ");
                                    let mut user = String::new();
                                    io::stdin().read_line(&mut user).ok();
                                    (domain, user)
                                }
                            };
                            let password = match password {
                                None => match self.password.clone() {
                                    Some(password) => if password.trim().is_empty() {
                                        None
                                    } else {
                                        Some(password)
                                    },
                                    None => {
                                        println!("Password for {}: ", address);
                                        let password_user = read_password().ok();
                                        if password_user.is_none() || password_user.as_ref().unwrap().trim().is_empty() {
                                            None
                                        } else {
                                            Some(password_user.unwrap())
                                        }
                                    }
                                },
                                Some(password) => Some(password)
                            };
                            Computer {
                                address: address.clone(),
                                username,
                                domain,
                                password,
                            }
                        })
                    })
                    .collect()
            }
            Err(_) => vec![Computer::from(self)]
        }
    }
}

impl<'a> Command<'a> {
    pub fn new(
        command: Vec<String>,
        store_directory: Option<&'a Path>,
        report_filename_prefix: &'a str,
        elevated: bool,
    ) -> Command<'a> {
        Command {
            command,
            report_store_directory: store_directory,
            report_filename_prefix,
            elevated,
        }
    }
}

pub trait Connector {
    fn connect_method_name(&self) -> &'static str;

    fn computer(&self) -> &Computer;

    fn copier(&self) -> &dyn RemoteFileCopier;

    fn remote_temp_storage(&self) -> &Path;

    fn mkdir(&self, path: &Path) {
        let command = Command::new(
            vec![
                "cmd.exe".to_string(),
                "/c".to_string(),
                "md".to_string(),
                path.to_str().unwrap_or_default().to_string(),
            ],
            None,
            "",
            true,
        );
        if let Err(err) = self.connect_and_run_command(command, Some(Duration::from_secs(10))) {
            error!("{}", err);
        }
    }

    fn connect_and_run_local_program_in_current_directory(
        &self,
        command_to_run: Command<'_>,
        timeout: Option<Duration>,
    ) -> Result<Option<PathBuf>> {
        let mut command = command_to_run.command;
        command[0] = std::env::current_dir().unwrap()
            .join(Path::new(&command[0]).file_name().unwrap())
            .to_string_lossy().to_string();
        let command_to_run = Command {
            command,
            ..command_to_run
        };
        self.connect_and_run_local_program(
            command_to_run,
            timeout,
        )
    }

    fn connect_and_run_local_program(
        &self,
        command_to_run: Command<'_>,
        timeout: Option<Duration>,
    ) -> Result<Option<PathBuf>> {
        let local_program_path = Path::new(command_to_run.command.first().unwrap());
        let remote_storage = self.remote_temp_storage();
        let copier = self.copier();
        copier.copy_to_remote(&local_program_path, &remote_storage)?;
        thread::sleep(Duration::from_millis(20_000));
        let remote_program_path = remote_storage.join(local_program_path
            .file_name()
            .expect(&format!("Must specify file instead of {}", local_program_path.display())
            )
        );
        let mut command = command_to_run.command;
        command[0] = remote_program_path.to_string_lossy().to_string();
        let command_to_run = Command {
            command,
            ..command_to_run
        };
        let result = self.connect_and_run_command(command_to_run, timeout)?;
        thread::sleep(Duration::from_millis(10_000));
        copier.delete_remote_file(&remote_program_path)?;
        Ok(result)
    }

    fn connect_and_run_command(
        &self,
        command_to_run: Command<'_>,
        timeout: Option<Duration>,
    ) -> Result<Option<PathBuf>> {
        debug!("Trying to run command {:?} on {}",
               command_to_run.command,
               &self.computer().address
        );
        let output_file_path = match command_to_run.report_store_directory {
            None => None,
            Some(store_directory) => {
                let file_path = create_report_path(
                    self.computer(),
                    store_directory,
                    &command_to_run.report_filename_prefix,
                    self.connect_method_name(),
                    "txt",
                );
                Some(file_path.to_str().unwrap().to_string())
            }
        };

        let processed_command = self.prepare_command(
            command_to_run.command,
            output_file_path.as_deref(),
            command_to_run.elevated,
        );

        let prepared_command = self.prepare_remote_process(processed_command);
        match timeout {
            None =>
                run_process_blocking(
                    "cmd.exe",
                    &prepared_command,
                ),
            Some(timeout) =>
                run_process_blocking_timed(
                    "cmd.exe",
                    &prepared_command,
                    timeout.clone(),
                ),
        }?;
        Ok(output_file_path.map(|it| PathBuf::from(it)))
    }

    fn prepare_remote_process(&self,
                              // pre_command: Vec<String>,
                              processed_command: Vec<String>,
                              // post_command: Vec<String>,
    ) -> Vec<String> {
        let all_args = iter::once("/c".to_string())
            // .chain(pre_command.into_iter())
            .chain(processed_command.into_iter())
            // .chain(post_command.into_iter())
            .collect();
        all_args
    }

    fn prepare_command(&self,
                       command: Vec<String>,
                       output_file_path: Option<&str>,
                       elevated: bool,
    ) -> Vec<String>;

    fn list_dirs(&self, path: &Path, store_directory: &Path) -> Vec<String> {
        debug!("Listing dirs in remote path {}", path.display());
        let prefix = format!("--TEMP_LIST_DIR_{}", Uuid::new_v4());
        let command = Command::new(
            vec![
                "cmd.exe".to_string(),
                "/c".to_string(),
                "dir".to_string(),
                path.to_str().unwrap_or_default().to_string(),
                "/Ad".to_string(),
                "/B".to_string()
            ],
            Some(store_directory),
            &prefix,
            true,
        );
        if let Err(err) = self.connect_and_run_command(command, Some(Duration::from_secs(10))) {
            error!("{}", err);
        }
        let result_file_name = match store_directory.read_dir() {
            Ok(dir_entry_iter) => {
                dir_entry_iter
                    .filter_map(|item| item.ok())
                    .map(|item| item.file_name())
                    .find(|item| item.to_string_lossy().contains(&prefix))
            }
            _ => None,
        };
        if result_file_name.is_none() {
            error!("Cannot find dir result file");
            debug!("Remote path {} has dirs []", path.display());
            return vec![];
        }
        let result_file_path = store_directory.join(&result_file_name.unwrap());
        let result_file = File::open(&result_file_path);
        if result_file.is_err() {
            error!("Cannot open file {} due to {}", result_file_path.display(), result_file.err().unwrap());
            debug!("Remote path {} has dirs []", path.display());
            return vec![];
        }
        let result = BufReader::new(result_file.unwrap()).lines()
            .filter_map(|line| line.ok())
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect::<Vec<String>>();
        debug!("Remote path {} has dirs {:?}", path.display(), result);
        if let Err(err) = std::fs::remove_file(&result_file_path) {
            error!("{}", err);
        }
        result
    }

    fn acquire_perms(&self, path: &Path) {
        debug!("Acquiring ownership");
        let grant_svi = Command {
            command: vec![
                "cmd.exe".to_string(),
                "/c".to_string(),
                "icacls.exe".to_string(),
                path.to_string_lossy().to_string(),
                "/grant".to_string(),
                format!("{}:F", self.computer().username)
            ],
            report_store_directory: None,
            report_filename_prefix: "GRANT_VSI",
            elevated: true,
        };

        if let Err(err) = self.connect_and_run_command(
            grant_svi,
            None,
        ) {
            warn!("Cannot acquire ownership: {}", err)
        }
        thread::sleep(Duration::from_secs(5));
    }

    fn release_perms(&self, path: &Path) {
        thread::sleep(Duration::from_secs(5));
        debug!("Releasing ownership");
        let grant_svi = Command {
            command: vec![
                "cmd.exe".to_string(),
                "/c".to_string(),
                "icacls.exe".to_string(),
                path.to_string_lossy().to_string(),
                "/deny".to_string(),
                format!("{}:F", self.computer().username)
            ],
            report_store_directory: None,
            report_filename_prefix: "DENY_VSI",
            elevated: true,
        };

        if let Err(err) = self.connect_and_run_command(
            grant_svi,
            None,
        ) {
            warn!("Cannot release ownership: {}", err)
        }
    }
}