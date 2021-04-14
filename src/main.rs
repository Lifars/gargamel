extern crate rpassword;

use std::io;
use simplelog::{CombinedLogger, TermLogger, WriteLogger, Config, TerminalMode, LevelFilter, ColorChoice};
use std::fs::{File, create_dir_all};
use crate::logo::print_logo;
use crate::arg_parser::Opts;


#[macro_use]
extern crate log;
extern crate simplelog;

use clap::Clap;
use crate::evidence_acquirer::EvidenceAcquirer;
use std::path::{Path, PathBuf};
use crate::remote::{Computer, Rdp, Wmi, Ssh, RemoteFileCopier, ReDownloader, PsExec, PsRemote, Local, Connector, RevShareConnector, SevenZipCompressCopier, ShadowCopier};
use crate::memory_acquirer::MemoryAcquirer;
use crate::command_runner::CommandRunner;
use crate::file_acquirer::download_files;
use crate::registry_acquirer::RegistryAcquirer;
use std::time::Duration;
use crate::events_acquirer::EventsAcquirer;
use rayon::prelude::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use crate::svi_data_acquirer::SystemVolumeInformationAcquirer;

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
mod kape_handler;
mod svi_data_acquirer;
mod embedded_search_list;

fn setup_logger(disable_log_colors: bool) {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Trace, Config::default(), TerminalMode::Mixed, if disable_log_colors { ColorChoice::Never } else { ColorChoice::Auto }),
            WriteLogger::new(LevelFilter::Trace, Config::default(), File::create("gargamel.log").expect("Cannot create log file")),
        ]
    ).unwrap();
}

fn main() -> Result<(), io::Error> {
    let opts: Opts = Opts::parse();

    setup_logger(opts.disable_log_colors);
    print_logo();

    create_dir_all(&opts.local_store_directory)?;
    debug!("Parsing remote computers.");

    // just the parsing part
    if let Some(kape_path) = &opts.kape_config_path {
        match kape_handler::convert_kape_config(kape_path) {
            Ok(()) => println!("Sucessfull parsed all kape configs!"),
            Err(e) => error!("Parsing kape configs {}", e)
        }
        return Ok(());
    };

    let remote_computers: Vec<Computer> = opts.clone().into();
    trace!("Will connect to {} computers", remote_computers.len());
    let opts = Opts {
        password: remote_computers[0].password.clone(),
        user: Some(remote_computers[0].username.clone()),
        computer: remote_computers[0].address.clone(),
        ..opts
    };

    if opts.par {
        remote_computers.par_iter()
            .map(|remote_computer| handle_remote_computer(&opts, &remote_computer))
            .for_each(|result| if result.is_err() { error!("{}", result.expect_err("")) })
    } else {
        remote_computers.iter()
            .map(|remote_computer| handle_remote_computer(&opts, &remote_computer))
            .for_each(|result| if result.is_err() { error!("{}", result.expect_err("")) })
    }
    Ok(())
}

fn handle_remote_computer(opts: &Opts, remote_computer: &Computer) -> Result<(), io::Error> {
    info!("Connecting to {} with user {}", remote_computer.address, remote_computer.username);
    let local_store_directory_owned = dunce::canonicalize(Path::new(&opts.local_store_directory)).unwrap();
    let local_store_directory = local_store_directory_owned.as_path();
    let remote_temp_storage = Path::new(&opts.remote_store_directory);
    let key_file = opts.ssh_key.clone().map(|it| PathBuf::from(it));
    let local = opts.computer == "127.0.0.1" || opts.computer == "localhost";

    if let Some(remote_file) = &opts.re_download {
        let connectors = create_connectors(
            &opts,
            &remote_computer,
            remote_temp_storage,
            true,
            local,
            opts.reverse_share,
        );
        let remote_file = Path::new(&remote_file);
        for connector in connectors {
            let _compress_copier = SevenZipCompressCopier::new(connector.as_ref(), false, None, false);
            let mut _shadow_copier = ShadowCopier::new(connector.as_ref(), local_store_directory, None);
            let compression = !opts.no_compression;
            let shadow = opts.shadow;
            let copier = if compression && shadow {
                _shadow_copier.copier_impl = &_compress_copier;
                &_shadow_copier as &dyn RemoteFileCopier
            } else if compression {
                &_compress_copier as &dyn RemoteFileCopier
            } else if shadow {
                &_shadow_copier as &dyn RemoteFileCopier
            } else {
                connector.copier()
            };

            info!("Trying to download {} from {} using method {}", remote_file.display(), remote_computer.address, copier.method_name());
            let re_downloader = ReDownloader {
                copier,
                target_dir: local_store_directory,
            };
            re_downloader.retry_download(remote_file);
        }
    }

    if !opts.disable_evidence_download && !opts.disable_predefined_download {
        let evidence_acquirers = create_evidence_acquirers(
            &remote_computer,
            local_store_directory,
            &opts,
            key_file.as_ref().map(|it| it.to_path_buf()),
            remote_temp_storage,
            local,
        );
        for acquirer in evidence_acquirers {
            acquirer.run_all();
        }
    }

    if !opts.disable_event_download && !opts.disable_predefined_download {
        let event_acquirers = create_events_acquirers(
            &remote_computer,
            local_store_directory,
            &opts,
            remote_temp_storage,
            local,
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
            local,
        );
        for command_runner in command_runners {
            info!("Running commands using method {}", command_runner.connector.connect_method_name());
            command_runner.run_commands(
                Path::new(custom_commands_path),
                Some(Duration::from_secs(opts.timeout)),
            );
        }
    }
    if !opts.disable_registry_download && !opts.disable_predefined_download {
        let registry_acquirers = create_registry_acquirers(
            &remote_computer,
            local_store_directory,
            &opts,
            remote_temp_storage,
            local,
        );
        for acquirer in registry_acquirers {
            acquirer.acquire();
        }
    }
    if let Some(search_files_path) = &opts.search_files_path {
        if opts.ssh {
            let remote_copier = Ssh {
                computer: remote_computer.clone(),
                key_file: key_file.as_ref().map(|it| it.clone()),
            };
            download_files(
                search_files_path,
                local_store_directory,
                &remote_copier,
                opts.no_compression,
            )?;
        } else {
            let connectors = create_connectors(
                &opts,
                &remote_computer,
                remote_temp_storage,
                true,
                local,
                opts.reverse_share,
            );
            for connector in connectors.into_iter() {
                let _compress_copier = SevenZipCompressCopier::new(connector.as_ref(), false, None, false);
                let mut _shadow_copier = ShadowCopier::new(connector.as_ref(), local_store_directory, None);
                let compression = !opts.no_compression;
                let shadow = opts.shadow;
                let copier = if compression && shadow {
                    _shadow_copier.copier_impl = &_compress_copier;
                    &_shadow_copier as &dyn RemoteFileCopier
                } else if compression {
                    &_compress_copier as &dyn RemoteFileCopier
                } else if shadow {
                    &_shadow_copier as &dyn RemoteFileCopier
                } else {
                    connector.copier()
                };
                info!("Downloading specified files using {}", copier.method_name());
                let result = download_files(
                    search_files_path,
                    local_store_directory,
                    copier,
                    opts.no_compression,
                );
                match result {
                    Err(err) => error!("{}", err),
                    Ok(_) => {
                        info!("Files in {} successfully transferred.", search_files_path);
                        break;
                    }
                };
            }
        }
    }
    if opts.image_memory {
        let memory_acquirers = create_memory_acquirers(
            &remote_computer,
            local_store_directory,
            &opts,
            remote_temp_storage,
            local,
        );
        for acquirer in memory_acquirers {
            info!("Running memory acquirer using method {}", acquirer.connector.connect_method_name());
            let image_res = acquirer.image_memory();
            match image_res {
                Err(err) => error!("{}", err),
                Ok(_) => break
            };
        }
    }

    if opts.acquire_svi_data {
        let svi_acquirers = create_svi_acquirers(
            &remote_computer,
            local_store_directory,
            &opts,
            remote_temp_storage,
            local,
        );
        for acquirer in svi_acquirers {
            info!("Running svi acquirer using method {}", acquirer.connector.connect_method_name());
            let svi_res = acquirer.download_data();
            match svi_res {
                Err(err) => error!("{}", err),
                Ok(_) => break
            };
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
    local: bool,
) -> Vec<EvidenceAcquirer<'a>> {
    if local {
        return vec![EvidenceAcquirer::local(computer.username.clone(), local_store_directory, remote_temp_storage.to_path_buf())];
    }

    let mut acquirers = Vec::<EvidenceAcquirer<'a>>::new();
    if opts.psexec64 || opts.psexec32 || opts.all {
        acquirers.push(
            EvidenceAcquirer::psexec(
                computer.clone(),
                local_store_directory,
                remote_temp_storage.to_path_buf(),
                opts.share.clone(),
                opts.reverse_share,
            ),
        );
    }
    if opts.wmi || opts.all {
        acquirers.push(
            EvidenceAcquirer::wmi(
                computer.clone(),
                local_store_directory,
                remote_temp_storage.to_path_buf(),
            ),
        );
    }
    if opts.psrem || opts.all {
        acquirers.push(
            EvidenceAcquirer::psremote(
                computer.clone(),
                local_store_directory,
                remote_temp_storage.to_path_buf(),
                opts.share.clone(),
                opts.reverse_share,
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
    if opts.rdp || opts.all {
        acquirers.push(
            EvidenceAcquirer::rdp(
                computer.clone(),
                local_store_directory,
                opts.nla,
                remote_temp_storage.to_path_buf(),
            ),
        )
    }
    if opts.local {
        acquirers.push(
            EvidenceAcquirer::local(
                computer.username.clone(),
                local_store_directory,
                remote_temp_storage.to_path_buf(),
            ),
        )
    }
    acquirers
}

fn create_memory_acquirers<'a>(
    computer: &'a Computer,
    local_store_directory: &'a Path,
    opts: &Opts,
    remote_temp_storage: &Path,
    local: bool,
) -> Vec<MemoryAcquirer<'a>> {
    if local {
        return vec![MemoryAcquirer::local(computer.username.clone(), local_store_directory, remote_temp_storage.to_path_buf())];
    }

    let mut acquirers = Vec::<MemoryAcquirer>::new();
    if opts.psexec32 {
        acquirers.push(
            MemoryAcquirer::psexec32(
                computer.clone(),
                local_store_directory,
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
                opts.share.clone(),
                opts.reverse_share,
            )
        );
    }
    if opts.psexec64 || opts.all {
        acquirers.push(
            MemoryAcquirer::psexec64(
                computer.clone(),
                local_store_directory,
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
                opts.share.clone(),
                opts.reverse_share,
            )
        );
    }
    if opts.psrem || opts.all {
        acquirers.push(
            MemoryAcquirer::psremote(
                computer.clone(),
                local_store_directory,
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
                opts.share.clone(),
                opts.reverse_share,
            )
        );
    }
    if opts.rdp || opts.all {
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
    if opts.wmi || opts.all {
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
    if opts.local {
        acquirers.push(
            MemoryAcquirer::local(computer.username.clone(), local_store_directory, remote_temp_storage.to_path_buf())
        );
    }
    acquirers
}

fn create_svi_acquirers<'a>(
    computer: &'a Computer,
    local_store_directory: &'a Path,
    opts: &Opts,
    remote_temp_storage: &Path,
    local: bool,
) -> Vec<SystemVolumeInformationAcquirer<'a>> {
    if local {
        return vec![SystemVolumeInformationAcquirer::local(computer.username.clone(), local_store_directory, remote_temp_storage.to_path_buf())];
    }

    let mut acquirers = Vec::<SystemVolumeInformationAcquirer>::new();
    if opts.psexec32 {
        acquirers.push(
            SystemVolumeInformationAcquirer::psexec32(
                computer.clone(),
                local_store_directory,
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
                opts.share.clone(),
                opts.reverse_share,
            )
        );
    }
    if opts.psexec64 || opts.all {
        acquirers.push(
            SystemVolumeInformationAcquirer::psexec64(
                computer.clone(),
                local_store_directory,
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
                opts.share.clone(),
                opts.reverse_share,
            )
        );
    }
    if opts.psrem || opts.all {
        acquirers.push(
            SystemVolumeInformationAcquirer::psremote(
                computer.clone(),
                local_store_directory,
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
                opts.share.clone(),
                opts.reverse_share,
            )
        );
    }
    if opts.rdp || opts.all {
        acquirers.push(
            SystemVolumeInformationAcquirer::rdp(
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
    if opts.wmi || opts.all {
        acquirers.push(
            SystemVolumeInformationAcquirer::wmi(
                computer.clone(),
                local_store_directory,
                Duration::from_secs(opts.timeout),
                Duration::from_secs(opts.timeout),
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
            )
        );
    }
    if opts.local {
        acquirers.push(
            SystemVolumeInformationAcquirer::local(computer.username.clone(), local_store_directory, remote_temp_storage.to_path_buf())
        );
    }
    acquirers
}

fn create_command_runners<'a>(
    computer: &'a Computer,
    local_store_directory: &'a Path,
    opts: &Opts,
    key_file: Option<PathBuf>,
    remote_temp_storage: &Path,
    local: bool,
) -> Vec<CommandRunner<'a>> {
    if local {
        return vec![CommandRunner::local(
            computer.username.clone(), local_store_directory,
        )];
    }

    let mut acquirers = Vec::<CommandRunner>::new();
    if opts.psexec64 || opts.psexec32 || opts.all {
        acquirers.push(
            CommandRunner::psexec(
                computer.clone(),
                local_store_directory,
                remote_temp_storage.to_path_buf(),
                opts.share.clone(),
            )
        );
    }

    if opts.psrem || opts.all {
        acquirers.push(
            CommandRunner::psremote(
                computer.clone(),
                local_store_directory,
                remote_temp_storage.to_path_buf(),
                opts.share.clone(),
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
    if opts.wmi || opts.all {
        acquirers.push(
            CommandRunner::wmi(
                computer.clone(),
                local_store_directory,
                remote_temp_storage.to_path_buf(),
            )
        )
    }
    if opts.rdp || opts.all {
        acquirers.push(
            CommandRunner::rdp(
                computer.clone(),
                local_store_directory,
                opts.nla,
                remote_temp_storage.to_path_buf(),
            )
        )
    }
    if opts.local {
        acquirers.push(
            CommandRunner::local(computer.username.clone(), local_store_directory)
        );
    }
    acquirers
}

fn create_registry_acquirers<'a>(
    computer: &'a Computer,
    local_store_directory: &'a Path,
    opts: &Opts,
    remote_temp_storage: &Path,
    local: bool,
) -> Vec<RegistryAcquirer<'a>> {
    if local {
        return vec![
            RegistryAcquirer::local(
                computer.username.clone(), local_store_directory, remote_temp_storage.to_path_buf(),
            )
        ];
    }

    let mut acquirers = Vec::<RegistryAcquirer<'a>>::new();
    if opts.psexec32 {
        acquirers.push(
            RegistryAcquirer::psexec32(
                local_store_directory,
                computer.clone(),
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
                opts.share.clone(),
                opts.reverse_share,
            ),
        );
    }
    if opts.psexec64 || opts.all {
        acquirers.push(
            RegistryAcquirer::psexec64(
                local_store_directory,
                computer.clone(),
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
                opts.share.clone(),
                opts.reverse_share,
            ),
        );
    }
    if opts.psrem || opts.all {
        acquirers.push(
            RegistryAcquirer::psremote(
                local_store_directory,
                computer.clone(),
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
                opts.share.clone(),
                opts.reverse_share,
            ),
        );
    }
    if opts.wmi || opts.all {
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
    if opts.rdp || opts.all {
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
    if opts.local {
        acquirers.push(
            RegistryAcquirer::local(
                computer.username.clone(), local_store_directory, remote_temp_storage.to_path_buf(),
            ),
        )
    }
    acquirers
}

fn create_events_acquirers<'a>(
    computer: &'a Computer,
    local_store_directory: &'a Path,
    opts: &Opts,
    remote_temp_storage: &Path,
    local: bool,
) -> Vec<EventsAcquirer<'a>> {
    if local {
        return vec![EventsAcquirer::local(computer.username.clone(), local_store_directory, remote_temp_storage.to_path_buf())];
    }

    let mut acquirers = Vec::<EventsAcquirer<'a>>::new();
    if opts.psexec32 {
        acquirers.push(
            EventsAcquirer::psexec32(
                local_store_directory,
                computer.clone(),
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
                opts.share.clone(),
                opts.reverse_share,
            ),
        );
    }
    if opts.psexec64 || opts.all {
        acquirers.push(
            EventsAcquirer::psexec64(
                local_store_directory,
                computer.clone(),
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
                opts.share.clone(),
                opts.reverse_share,
            ),
        );
    }
    if opts.psrem || opts.all {
        acquirers.push(
            EventsAcquirer::psremote(
                local_store_directory,
                computer.clone(),
                opts.no_compression,
                remote_temp_storage.to_path_buf(),
                opts.share.clone(),
                opts.reverse_share,
            ),
        );
    }
    if opts.wmi || opts.all {
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
    if opts.rdp || opts.all {
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
    if opts.local {
        acquirers.push(
            EventsAcquirer::local(
                computer.username.clone(),
                local_store_directory,
                remote_temp_storage.to_path_buf(),
            ),
        )
    }
    acquirers
}


fn create_connectors(
    opts: &Opts,
    computer: &Computer,
    remote_temp_storage: &Path,
    allowed_ssh: bool,
    local: bool,
    reverse_share: bool,
) -> Vec<Box<dyn Connector>> {
    if local {
        return vec![Box::new(Local::new(computer.username.clone(), remote_temp_storage.to_path_buf()))];
    }

    let mut copiers = Vec::<Box<dyn Connector>>::new();
    if opts.psexec32 {
        trace!("Creating psexec32 copier");
        let _copier = Box::new(PsExec::psexec32(computer.clone(), remote_temp_storage.to_path_buf(), opts.share.clone()));
        let copier: Box<dyn Connector> = if reverse_share { Box::new(RevShareConnector::new(_copier)) } else { _copier };
        copiers.push(copier);
    }
    if opts.psexec64 || opts.all {
        trace!("Creating psexec64 copier");
        let _copier = Box::new(PsExec::psexec64(computer.clone(), remote_temp_storage.to_path_buf(), opts.share.clone()));
        let copier: Box<dyn Connector> = if reverse_share { Box::new(RevShareConnector::new(_copier)) } else { _copier };
        copiers.push(copier);
    }
    if opts.psrem || opts.all {
        let _copier = Box::new(PsRemote::new(computer.clone(), remote_temp_storage.to_path_buf(), opts.share.clone()));
        let copier: Box<dyn Connector> = if reverse_share { Box::new(RevShareConnector::new(_copier)) } else { _copier };
        copiers.push(copier);
    }
    if opts.rdp || opts.all {
        let copier = Box::new(Rdp {
            computer: computer.clone(),
            nla: opts.nla,
            remote_temp_storage: remote_temp_storage.to_path_buf(),
        });
        copiers.push(
            copier
        );
    }
    if opts.wmi || opts.all {
        let copier = Box::new(Wmi {
            computer: computer.clone(),
            remote_temp_storage: remote_temp_storage.to_path_buf(),
        });
        copiers.push(
            copier
        );
    }
    if opts.ssh && allowed_ssh {
        copiers.push(Box::new(Ssh {
            computer: computer.clone(),
            key_file: opts.ssh_key.clone().map(|key_file| PathBuf::from(key_file)),
        }));
    }
    if opts.local {
        let copier = Box::new(Local::new(computer.username.clone(), remote_temp_storage.to_path_buf()));
        copiers.push(
            copier
        );
    }
    copiers
}