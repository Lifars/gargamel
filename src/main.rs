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
use crate::remote::{Computer, Rdp, Wmi, Ssh, RemoteFileCopier, ReDownloader, PsExec, PsRemote, CompressCopierOwned};
use crate::memory_acquirer::MemoryAcquirer;
use crate::command_runner::CommandRunner;
use crate::file_acquirer::download_files;
use crate::registry_acquirer::RegistryAcquirer;
use std::time::Duration;
use crate::events_acquirer::EventsAcquirer;
use rayon::prelude::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

mod process_runner;
mod evidence_acquirer;
mod remote;
mod arg_parser;
mod logo;
mod memory_acquirer;
mod command_utils;
mod utils;
mod large_evidence_acquirer;
mod events_acquirer;
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
    create_dir_all(&opts.local_store_directory)?;
    debug!("Parsing remote computers.");
    let remote_computers: Vec<Computer> = opts.clone().into();
    trace!("Will connect to {} computers", remote_computers.len());
    let opts =  Opts {
        password: remote_computers[0].password.clone(),
        domain: remote_computers[0].domain.clone(),
        user: Some(remote_computers[0].username.clone()),
        computer: remote_computers[0].address.clone(),
        ..opts
    };

    if opts.par {
        remote_computers.par_iter()
            .map(|remote_computer| handle_remote_computer(&opts, &remote_computer))
            .for_each(|result| if result.is_err() { error!("{}", result.expect_err("")) } )
    } else {
        remote_computers.iter()
            .map(|remote_computer| handle_remote_computer(&opts, &remote_computer))
            .for_each(|result| if result.is_err() { error!("{}", result.expect_err("")) } )
    }
    Ok(())
}

fn handle_remote_computer(opts: &Opts, remote_computer: &Computer) -> Result<(), io::Error> {
    info!("Connecting to {} with user {}", remote_computer.address, remote_computer.domain_username());
    let local_store_directory_owned = dunce::canonicalize(Path::new(&opts.local_store_directory)).unwrap();
    let local_store_directory = local_store_directory_owned.as_path();
    let remote_temp_storage = Path::new(&opts.remote_store_directory);
    let key_file = opts.ssh_key.clone().map(|it| PathBuf::from(it));

    if let Some(remote_file) = &opts.re_download {
        let copiers = create_copiers(
            &opts,
            &remote_computer,
            remote_temp_storage,
            true,
            false,
            false,
            None
        );
        let remote_file = Path::new(&remote_file);
        for copier in copiers {
            info!("Trying to download {} from {} using method {}", remote_file.display(), remote_computer.address, copier.method_name());
            let re_downloader = ReDownloader {
                copier: copier.as_ref(),
                target_dir: local_store_directory,
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
            remote_temp_storage,
        );
        for acquirer in evidence_acquirers {
            acquirer.run_all();
        }
    }

    if !opts.disable_event_download {
        let event_acquirers = create_events_acquirers(
            &remote_computer,
            local_store_directory,
            &opts,
            remote_temp_storage,
        );
        for acquirer in event_acquirers {
            acquirer.acquire();
        }
    }

    if let Some(custom_commands_path) = &opts.custom_command_path {
        let command_runners = create_command_runners(
            &remote_computer,
            local_store_directory,
            &opts,
            key_file.as_ref().map(|it| it.to_path_buf()),
            remote_temp_storage,
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
            remote_temp_storage,
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
            let copiers = create_copiers(
                &opts,
                &remote_computer,
                remote_temp_storage,
                true,
                !opts.no_compression,
                false,
                Some(Duration::from_secs(opts.timeout))
            );
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
    if opts.image_memory {
        let memory_acquirers = create_memory_acquirers(
            &remote_computer,
            local_store_directory,
            &opts,
            remote_temp_storage,
        );
        for acquirer in memory_acquirers {
            info!("Running memory acquirer using method {}", acquirer.connector.connect_method_name());
            let image_res = acquirer.image_memory();
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
    remote_temp_storage: &Path,
) -> Vec<EvidenceAcquirer<'a>> {
    let acquirers: Vec<EvidenceAcquirer<'a>> = if opts.all {
        vec![
            EvidenceAcquirer::psexec(
                computer.clone(),
                local_store_directory,
                remote_temp_storage.to_path_buf(),
            ),
            EvidenceAcquirer::wmi(
                computer.clone(),
                local_store_directory,
                remote_temp_storage.to_path_buf(),
            ),
            EvidenceAcquirer::psremote(
                computer.clone(),
                local_store_directory,
                remote_temp_storage.to_path_buf(),
            ),
            EvidenceAcquirer::rdp(
                computer.clone(),
                local_store_directory,
                opts.nla,
                remote_temp_storage.to_path_buf(),
            ),
        ]
    } else {
        let mut acquirers = Vec::<EvidenceAcquirer<'a>>::new();
        if opts.psexec64 || opts.psexec32 {
            acquirers.push(
                EvidenceAcquirer::psexec(
                    computer.clone(),
                    local_store_directory,
                    remote_temp_storage.to_path_buf(),
                ),
            );
        }
        if opts.wmi {
            acquirers.push(
                EvidenceAcquirer::wmi(
                    computer.clone(),
                    local_store_directory,
                    remote_temp_storage.to_path_buf(),
                ),
            );
        }
        if opts.psrem {
            acquirers.push(
                EvidenceAcquirer::psremote(
                    computer.clone(),
                    local_store_directory,
                    remote_temp_storage.to_path_buf(),
                )
            );
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
                    remote_temp_storage.to_path_buf(),
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
    remote_temp_storage: &Path,
) -> Vec<MemoryAcquirer<'a>> {
    let acquirers: Vec<MemoryAcquirer<'a>> = if opts.all {
        vec![
            MemoryAcquirer::psexec64(
                computer.clone(),
                local_store_directory,
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
            ),
            MemoryAcquirer::psremote(
                computer.clone(),
                local_store_directory,
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
            ),
            MemoryAcquirer::rdp(
                computer.clone(),
                local_store_directory,
                opts.nla,
                Duration::from_secs(opts.timeout),
                Duration::from_secs(opts.timeout),
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
            ),
            MemoryAcquirer::wmi(
                computer.clone(),
                local_store_directory,
                Duration::from_secs(opts.timeout),
                Duration::from_secs(opts.timeout),
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
            ),
        ]
    } else {
        let mut acquirers = Vec::<MemoryAcquirer>::new();
        if opts.psexec32 {
            acquirers.push(
                MemoryAcquirer::psexec32(
                    computer.clone(),
                    local_store_directory,
                    opts.no_compression,
                    remote_temp_storage.to_path_buf(),
                )
            );
        }
        if opts.psexec64 {
            acquirers.push(
                MemoryAcquirer::psexec64(
                    computer.clone(),
                    local_store_directory,
                    opts.no_compression,
                    remote_temp_storage.to_path_buf(),
                )
            );
        }
        if opts.psrem {
            acquirers.push(
                MemoryAcquirer::psremote(
                    computer.clone(),
                    local_store_directory,
                    opts.no_compression,
                    remote_temp_storage.to_path_buf(),
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
                    Duration::from_secs(opts.timeout),
                    opts.no_compression,
                    remote_temp_storage.to_path_buf(),
                )
            );
        }
        if opts.wmi {
            acquirers.push(
                MemoryAcquirer::wmi(
                    computer.clone(),
                    local_store_directory,
                    Duration::from_secs(opts.timeout),
                    Duration::from_secs(opts.timeout),
                    opts.no_compression,
                    remote_temp_storage.to_path_buf(),
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
    remote_temp_storage: &Path,
) -> Vec<CommandRunner<'a>> {
    let acquirers: Vec<CommandRunner<'a>> = if opts.all {
        vec![
            CommandRunner::psexec(
                computer.clone(),
                local_store_directory,
                remote_temp_storage.to_path_buf(),
            ),
            CommandRunner::psremote(
                computer.clone(),
                local_store_directory,
                remote_temp_storage.to_path_buf(),
            ),
            CommandRunner::rdp(
                computer.clone(),
                local_store_directory,
                opts.nla,
                remote_temp_storage.to_path_buf(),
            ),
            CommandRunner::wmi(
                computer.clone(),
                local_store_directory,
                remote_temp_storage.to_path_buf(),
            ),
        ]
    } else {
        let mut acquirers = Vec::<CommandRunner>::new();
        if opts.psexec64 || opts.psexec32 {
            acquirers.push(
                CommandRunner::psexec(
                    computer.clone(),
                    local_store_directory,
                    remote_temp_storage.to_path_buf(),
                )
            );
        }
        if opts.psrem {
            acquirers.push(
                CommandRunner::psremote(
                    computer.clone(),
                    local_store_directory,
                    remote_temp_storage.to_path_buf(),
                )
            );
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
                    remote_temp_storage.to_path_buf(),
                )
            )
        }
        if opts.rdp {
            acquirers.push(
                CommandRunner::rdp(
                    computer.clone(),
                    local_store_directory,
                    opts.nla,
                    remote_temp_storage.to_path_buf(),
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
    remote_temp_storage: &Path,
) -> Vec<RegistryAcquirer<'a>> {
    let acquirers: Vec<RegistryAcquirer<'a>> = if opts.all {
        vec![
            RegistryAcquirer::psexec64(
                local_store_directory,
                computer.clone(),
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
            ),
            RegistryAcquirer::psremote(
                local_store_directory,
                computer.clone(),
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
            ),
            RegistryAcquirer::wmi(
                local_store_directory,
                computer.clone(),
                Duration::from_secs(opts.timeout),
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
            ),
            RegistryAcquirer::rdp(
                local_store_directory,
                computer.clone(),
                Duration::from_secs(opts.timeout),
                opts.nla,
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
            ),
        ]
    } else {
        let mut acquirers = Vec::<RegistryAcquirer<'a>>::new();
        if opts.psexec32 {
            acquirers.push(
                RegistryAcquirer::psexec32(
                    local_store_directory,
                    computer.clone(),
                    opts.no_compression,
                    remote_temp_storage.to_path_buf(),
                ),
            );
        }
        if opts.psexec64 {
            acquirers.push(
                RegistryAcquirer::psexec64(
                    local_store_directory,
                    computer.clone(),
                    opts.no_compression,
                    remote_temp_storage.to_path_buf(),
                ),
            );
        }
        if opts.psrem {
            acquirers.push(
                RegistryAcquirer::psremote(
                    local_store_directory,
                    computer.clone(),
                    opts.no_compression,
                    remote_temp_storage.to_path_buf(),
                ),
            );
        }
        if opts.wmi {
            acquirers.push(
                RegistryAcquirer::wmi(
                    local_store_directory,
                    computer.clone(),
                    Duration::from_secs(opts.timeout),
                    opts.no_compression,
                    remote_temp_storage.to_path_buf(),
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
                    opts.no_compression,
                    remote_temp_storage.to_path_buf(),
                ),
            )
        }
        acquirers
    };
    acquirers
}

fn create_events_acquirers<'a>(
    computer: &'a Computer,
    local_store_directory: &'a Path,
    opts: &Opts,
    remote_temp_storage: &Path,
) -> Vec<EventsAcquirer<'a>> {
    let acquirers: Vec<EventsAcquirer<'a>> = if opts.all {
        vec![
            EventsAcquirer::psexec64(
                local_store_directory,
                computer.clone(),
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
            ),
            EventsAcquirer::psremote(
                local_store_directory,
                computer.clone(),
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
            ),
            EventsAcquirer::wmi(
                local_store_directory,
                computer.clone(),
                Duration::from_secs(opts.timeout),
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
            ),
            EventsAcquirer::rdp(
                local_store_directory,
                computer.clone(),
                Duration::from_secs(opts.timeout),
                opts.nla,
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
            ),
        ]
    } else {
        let mut acquirers = Vec::<EventsAcquirer<'a>>::new();
        if opts.psexec32 {
            acquirers.push(
                EventsAcquirer::psexec32(
                    local_store_directory,
                    computer.clone(),
                    opts.no_compression,
                    remote_temp_storage.to_path_buf(),
                ),
            );
        }
        if opts.psexec64 {
            acquirers.push(
                EventsAcquirer::psexec64(
                    local_store_directory,
                    computer.clone(),
                    opts.no_compression,
                    remote_temp_storage.to_path_buf(),
                ),
            );
        }
        if opts.psrem {
            acquirers.push(
                EventsAcquirer::psremote(
                    local_store_directory,
                    computer.clone(),
                    opts.no_compression,
                    remote_temp_storage.to_path_buf(),
                ),
            );
        }
        if opts.wmi {
            acquirers.push(
                EventsAcquirer::wmi(
                    local_store_directory,
                    computer.clone(),
                    Duration::from_secs(opts.timeout),
                    opts.no_compression,
                    remote_temp_storage.to_path_buf(),
                ),
            );
        }
        if opts.rdp {
            acquirers.push(
                EventsAcquirer::rdp(
                    local_store_directory,
                    computer.clone(),
                    Duration::from_secs(opts.timeout),
                    opts.nla,
                    opts.no_compression,
                    remote_temp_storage.to_path_buf(),
                ),
            )
        }
        acquirers
    };
    acquirers
}


fn create_copiers(
    opts: &Opts,
    computer: &Computer,
    remote_temp_storage: &Path,
    allowed_ssh: bool,
    compression: bool,
    uncompress_downloaded: bool,
    compress_timeout: Option<Duration>
) -> Vec<Box<dyn RemoteFileCopier>> {
    let mut copiers = Vec::<Box<dyn RemoteFileCopier>>::new();
    if opts.psexec32 {
        trace!("Creating psexec32 copier");
        let copier = Box::new(PsExec::psexec32(computer.clone(), remote_temp_storage.to_path_buf()));
        copiers.push(
            if compression {
                trace!("Enabling compression for psexec");
                Box::new(CompressCopierOwned::new(copier, false, None, uncompress_downloaded))
            } else {
                copier
            }
        );
    }
    if opts.psexec64 || opts.all {
        trace!("Creating psexec64 copier");
        let copier = Box::new(PsExec::psexec64(computer.clone(), remote_temp_storage.to_path_buf()));
        copiers.push(
            if compression {
                trace!("Enabling compression for psexec");
                Box::new(CompressCopierOwned::new(copier, false, None, uncompress_downloaded))
            } else {
                copier
            }
        );
    }
    if opts.psrem || opts.all {
        copiers.push(
            Box::new(PsRemote::new(computer.clone(), remote_temp_storage.to_path_buf()))
        );
    }
    if opts.rdp || opts.all {
        let copier = Box::new(Rdp {
            computer: computer.clone(),
            nla: opts.nla,
            remote_temp_storage: remote_temp_storage.to_path_buf(),
        });
        copiers.push(
            if compression {
                Box::new(CompressCopierOwned::new(copier, true, compress_timeout.clone(), uncompress_downloaded))
            } else {
                copier
            }
        );
    }
    if opts.wmi || opts.all {
        let copier = Box::new(Wmi {
            computer: computer.clone(),
            remote_temp_storage: remote_temp_storage.to_path_buf(),
        });
        copiers.push(if compression {
            Box::new(CompressCopierOwned::new(copier, true, compress_timeout.clone(), uncompress_downloaded))
        } else {
            copier
        });
    }
    if opts.ssh && allowed_ssh {
        copiers.push(Box::new(Ssh {
            computer: computer.clone(),
            key_file: opts.ssh_key.clone().map(|key_file| PathBuf::from(key_file)),
        }));
    }
    copiers
}