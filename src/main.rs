extern crate rpassword;

use std::io;
use simplelog::{CombinedLogger, TermLogger, WriteLogger, Config, TerminalMode, LevelFilter};
use std::fs::{File, create_dir_all};
use crate::logo::print_logo;
use crate::arg_parser::Opts;

#[macro_use]
extern crate log;
extern crate simplelog;

use clap::derive::Clap;
use crate::evidence_acquirer::EvidenceAcquirer;
use std::path::{Path, PathBuf};
use crate::remote::{Computer, Cmd, Powershell, WindowsRemoteFileHandler, Rdp, Wmi, Ssh, RemoteFileCopier, ReDownloader};
use crate::memory_acquirer::MemoryAcquirer;
use crate::command_runner::CommandRunner;
use crate::file_acquirer::download_files;
use rpassword::read_password;
use crate::registry_acquirer::RegistryAcquirer;
use std::time::Duration;
use crate::utils::remote_storage_file;

mod process_runner;
mod evidence_acquirer;
mod remote;
mod arg_parser;
mod logo;
mod memory_acquirer;
mod command_utils;
mod utils;
mod file_acquirer;
mod registry_acquirer;
mod command_runner;

fn setup_logger() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Trace, Config::default(), TerminalMode::Mixed).unwrap(),
            WriteLogger::new(LevelFilter::Trace, Config::default(), File::create("gargamel.log").unwrap()),
        ]
    ).unwrap();
}

fn main() -> Result<(), io::Error> {
    setup_logger();
    print_logo();

    let opts: Opts = Opts::parse();
    create_dir_all(&opts.store_directory)?;

    let opts = match &opts.password {
        Some(_) => opts,
        None => {
            println!("Password: ");
            let password = read_password().expect("Error reading password");
            Opts { password: Some(password), ..opts }
        }
    };

    let remote_computer = Computer::from(opts.clone());


    // let local_store_directory_owned = match Path::new(&opts.store_directory).can {
    //     None => std::env::current_dir().unwrap().join(&opts.s),
    //     Some(_) => {},
    // };
    let local_store_directory_owned = dunce::canonicalize(Path::new(&opts.store_directory)).unwrap();
    let local_store_directory = local_store_directory_owned.as_path();
    let key_file = opts.ssh_key.clone().map(|it| PathBuf::from(it));

    if let Some(remote_file) = &opts.re_download {
        let copiers = create_file_copiers(&opts, &remote_computer);
        let remote_file = Path::new(&remote_file);
        for copier in copiers {
            info!("Trying to download {} from {} using method {}", remote_file.display(), remote_computer.address, copier.method_name());
            let re_downloader = ReDownloader{
                copier: copier.as_ref(),
                target_dir: local_store_directory
            };
            re_downloader.retry_download(remote_file);
        }
    }

    if !opts.disable_evidence_download {
        let evidence_acquirers = create_evidence_acquirers(
            &remote_computer,
            local_store_directory,
            &opts,
            key_file.as_ref().map(|it| it.to_path_buf()),
        );
        for acquirer in evidence_acquirers {
            acquirer.run_all();
        }
    }
    if let Some(custom_commands_path) = &opts.custom_command_path {
        let command_runners = create_command_runners(
            &remote_computer,
            local_store_directory,
            &opts,
            key_file.as_ref().map(|it| it.to_path_buf()),
        );
        for command_runner in command_runners {
            info!("Running commands using method {}", command_runner.connector.connect_method_name());
            command_runner.run_commands(
                Path::new(custom_commands_path),
                Some(Duration::from_secs(opts.timeout)),
            );
        }
    }
    if !opts.disable_registry_download {
        let registry_acquirers = create_registry_acquirers(
            &remote_computer,
            local_store_directory,
            &opts,
        );
        for acquirer in registry_acquirers {
            acquirer.acquire();
        }
    }
    if let Some(search_files_path) = &opts.search_files_path {
        let search_files_path = Path::new(search_files_path);
        if opts.ssh {
            let remote_copier = Ssh {
                computer: remote_computer.clone(),
                key_file: key_file.as_ref().map(|it| it.clone()),
            };
            download_files(
                search_files_path,
                local_store_directory,
                &remote_copier,
            )?;
        } else {
            let copiers = create_file_copiers(&opts, &remote_computer);
            for copier in copiers.into_iter() {
                info!("Downloading specified files using {}", copier.copier_impl().method_name());
                let result = download_files(
                    search_files_path,
                    local_store_directory,
                    copier.as_ref(),
                );
                if result.is_ok() {
                    info!("Files in {} successfully transferred.", search_files_path.display());
                    break;
                }
            }
        }
    }
    if let Some(image_memory) = &opts.image_memory {
        let memory_acquirers = create_memory_acquirers(
            &remote_computer,
            local_store_directory,
            &opts,
        );
        for acquirer in memory_acquirers {
            info!("Running memory acquirer using method {}", acquirer.connector.connect_method_name());
            let image_res = acquirer.image_memory(Path::new(image_memory.as_str()));
            if image_res.is_ok() {
                break;
            }
        }
    }

    Ok(())
}

fn create_evidence_acquirers<'a>(
    computer: &'a Computer,
    local_store_directory: &'a Path,
    opts: &Opts,
    key_file: Option<PathBuf>,
) -> Vec<EvidenceAcquirer<'a>> {
    let acquirers: Vec<EvidenceAcquirer<'a>> = if opts.all {
        vec![
            EvidenceAcquirer::psexec(
                computer.clone(),
                local_store_directory,
            ),
            EvidenceAcquirer::wmi(
                computer.clone(),
                local_store_directory,
            ),
            EvidenceAcquirer::psremote(
                computer.clone(),
                local_store_directory,
            ),
            EvidenceAcquirer::rdp(
                computer.clone(),
                local_store_directory,
                opts.nla,
            ),
        ]
    } else {
        let mut acquirers = Vec::<EvidenceAcquirer<'a>>::new();
        if opts.psexec {
            acquirers.push(
                EvidenceAcquirer::psexec(
                    computer.clone(),
                    local_store_directory,
                ),
            );
        }
        if opts.wmi {
            acquirers.push(
                EvidenceAcquirer::wmi(
                    computer.clone(),
                    local_store_directory,
                ),
            );
        }
        if opts.psrem {
            acquirers.push(
                EvidenceAcquirer::psremote(
                    computer.clone(),
                    local_store_directory,
                )
            );
        }
        if opts.local {
            acquirers.push(
                EvidenceAcquirer::local(
                    local_store_directory,
                )
            )
        }
        if opts.ssh {
            acquirers.push(
                EvidenceAcquirer::ssh(
                    computer.clone(),
                    local_store_directory,
                    key_file,
                )
            )
        }
        if opts.rdp {
            acquirers.push(
                EvidenceAcquirer::rdp(
                    computer.clone(),
                    local_store_directory,
                    opts.nla,
                ),
            )
        }
        acquirers
    };
    acquirers
}

fn create_memory_acquirers<'a>(
    computer: &'a Computer,
    local_store_directory: &'a Path,
    opts: &Opts,
) -> Vec<MemoryAcquirer<'a>> {
    let acquirers: Vec<MemoryAcquirer<'a>> = if opts.all {
        vec![
            MemoryAcquirer::psexec(
                computer.clone(),
                local_store_directory,
                opts.no_compression
            ),
            MemoryAcquirer::psremote(
                computer.clone(),
                local_store_directory,
                opts.no_compression
            ),
            MemoryAcquirer::rdp(
                computer.clone(),
                local_store_directory,
                opts.nla,
                Duration::from_secs(opts.timeout),
                Duration::from_secs(opts.compress_timeout),
                opts.no_compression
            ),
            MemoryAcquirer::wmi(
                computer.clone(),
                local_store_directory,
                Duration::from_secs(opts.timeout),
                Duration::from_secs(opts.compress_timeout),
                opts.no_compression
            ),
        ]
    } else {
        let mut acquirers = Vec::<MemoryAcquirer>::new();
        if opts.psexec {
            acquirers.push(
                MemoryAcquirer::psexec(
                    computer.clone(),
                    local_store_directory,
                    opts.no_compression
                )
            );
        }
        if opts.psrem {
            acquirers.push(
                MemoryAcquirer::psremote(
                    computer.clone(),
                    local_store_directory,
                    opts.no_compression
                )
            );
        }
        if opts.rdp {
            acquirers.push(
                MemoryAcquirer::rdp(
                    computer.clone(),
                    local_store_directory,
                    opts.nla,
                    Duration::from_secs(opts.timeout),
                    Duration::from_secs(opts.compress_timeout),
                    opts.no_compression
                )
            );
        }
        if opts.wmi {
            acquirers.push(
                MemoryAcquirer::wmi(
                    computer.clone(),
                    local_store_directory,
                    Duration::from_secs(opts.timeout),
                    Duration::from_secs(opts.compress_timeout),
                    opts.no_compression
                )
            );
        }
        acquirers
    };
    acquirers
}

fn create_command_runners<'a>(
    computer: &'a Computer,
    local_store_directory: &'a Path,
    opts: &Opts,
    key_file: Option<PathBuf>,
) -> Vec<CommandRunner<'a>> {
    let acquirers: Vec<CommandRunner<'a>> = if opts.all {
        vec![
            CommandRunner::psexec(
                computer.clone(),
                local_store_directory,
            ),
            CommandRunner::psremote(
                computer.clone(),
                local_store_directory,
            ),
            CommandRunner::rdp(
                computer.clone(),
                local_store_directory,
                opts.nla,
            ),
            CommandRunner::wmi(
                computer.clone(),
                local_store_directory,
            ),
        ]
    } else {
        let mut acquirers = Vec::<CommandRunner>::new();
        if opts.psexec {
            acquirers.push(
                CommandRunner::psexec(
                    computer.clone(),
                    local_store_directory,
                )
            );
        }
        if opts.psrem {
            acquirers.push(
                CommandRunner::psremote(
                    computer.clone(),
                    local_store_directory,
                )
            );
        }
        if opts.local {
            acquirers.push(
                CommandRunner::local(
                    local_store_directory,
                )
            )
        }
        if opts.ssh {
            acquirers.push(
                CommandRunner::ssh(
                    computer.clone(),
                    local_store_directory,
                    key_file,
                )
            )
        }
        if opts.wmi {
            acquirers.push(
                CommandRunner::wmi(
                    computer.clone(),
                    local_store_directory,
                )
            )
        }
        if opts.rdp {
            acquirers.push(
                CommandRunner::rdp(
                    computer.clone(),
                    local_store_directory,
                    opts.nla,
                )
            )
        }
        acquirers
    };
    acquirers
}

fn create_registry_acquirers<'a>(
    computer: &'a Computer,
    local_store_directory: &'a Path,
    opts: &Opts,
) -> Vec<RegistryAcquirer<'a>> {
    let acquirers: Vec<RegistryAcquirer<'a>> = if opts.all {
        vec![
            RegistryAcquirer::psexec(
                local_store_directory,
                computer.clone(),
                opts.no_compression
            ),
            RegistryAcquirer::psremote(
                local_store_directory,
                computer.clone(),
                opts.no_compression
            ),
            RegistryAcquirer::wmi(
                local_store_directory,
                computer.clone(),
                Duration::from_secs(opts.timeout),
                opts.no_compression
            ),
            RegistryAcquirer::rdp(
                local_store_directory,
                computer.clone(),
                Duration::from_secs(opts.timeout),
                opts.nla,
                opts.no_compression
            ),
        ]
    } else {
        let mut acquirers = Vec::<RegistryAcquirer<'a>>::new();
        if opts.psexec {
            acquirers.push(
                RegistryAcquirer::psexec(
                    local_store_directory,
                    computer.clone(),
                    opts.no_compression
                ),
            );
        }
        if opts.psrem {
            acquirers.push(
                RegistryAcquirer::psremote(
                    local_store_directory,
                    computer.clone(),
                    opts.no_compression
                ),
            );
        }
        if opts.wmi {
            acquirers.push(
                RegistryAcquirer::wmi(
                    local_store_directory,
                    computer.clone(),
                    Duration::from_secs(opts.timeout),
                    opts.no_compression
                ),
            );
        }
        if opts.rdp {
            acquirers.push(
                RegistryAcquirer::rdp(
                    local_store_directory,
                    computer.clone(),
                    Duration::from_secs(opts.timeout),
                    opts.nla,
                    opts.no_compression
                ),
            )
        }
        acquirers
    };
    acquirers
}

fn create_file_copiers(opts: &Opts, computer: &Computer) -> Vec<Box<dyn RemoteFileCopier>> {
    let copiers: Vec<Box<dyn RemoteFileCopier>> = if opts.all {
        vec![
            Box::new(WindowsRemoteFileHandler::new(
                computer.clone(),
                Box::new(Cmd {}),
            )),
            Box::new(WindowsRemoteFileHandler::new(
                computer.clone(),
                Box::new(Powershell {}),
            )),
            Box::new(Rdp {
                computer: computer.clone(),
                nla: opts.nla,
            }),
            Box::new(Wmi {
                computer: computer.clone(),
            }),
        ]
    } else {
        let mut copiers = Vec::<Box<dyn RemoteFileCopier>>::new();
        if opts.psexec {
            copiers.push(
                Box::new(WindowsRemoteFileHandler::new(
                    computer.clone(),
                    Box::new(Cmd {}),
                ))
            );
        }
        if opts.psrem {
            copiers.push(
                Box::new(WindowsRemoteFileHandler::new(
                    computer.clone(),
                    Box::new(Powershell {}),
                ))
            );
        }
        if opts.rdp {
            copiers.push(
                Box::new(Rdp {
                    computer: computer.clone(),
                    nla: opts.nla,
                })
            );
        }
        if opts.wmi {
            copiers.push(Box::new(Wmi {
                computer: computer.clone(),
            }));
        }
        copiers
    };
    copiers
}
