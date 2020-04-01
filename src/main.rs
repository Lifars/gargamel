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
use crate::remote::{PsExec, PsRemote, Local, Computer, Copier, XCopy, PsCopyItem, RemoteCopier, Scp, WindowsRemoteCopier, ScpRemoteCopier};
use crate::memory_acquirer::MemoryAcquirer;
use crate::command_runner::CommandRunner;
use crate::file_acquirer::download_files;
use rpassword::read_password;

mod process_runner;
mod evidence_acquirer;
mod remote;
mod arg_parser;
mod logo;
mod memory_acquirer;
mod command_utils;
mod utils;
mod file_acquirer;
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

    // HRAMSA
    // let mut command = Command::new("wmic");
    // let command_args = vec![
    //     "/OUTPUT:C:\\Users\\viliam\\AppData\\Local\\Temp\\wmi4.txt",
    //     "/NODE:192.168.126.142",
    //     "/USER:IEUser",
    //     "/PASSWORD:trolko",
    //     "COMPUTERSYSTEM", "GET", "USERNAME"
    // ];

    // HRAMSA 2
    // let mut command = Command::new("cmd.exe");
    // let p = "C:\\Users\\viliam\\AppData\\Local\\Temp\\wmi9.txt";
    // let p = Path::new(p);
    // {
    //     File::create(&p);
    // }
    // let p = dunce::canonicalize(p)?;
    // let p = p.to_str().unwrap().to_string();
    // let p = format!("/OUTPUT:{}", p);
    // let command_args = vec![
    //     "/c",
    //     "wmic.exe",
    //     p.as_str(),
    //     "/NODE:192.168.126.142",
    //     "/USER:IEUser",
    //     "/PASSWORD:trolko",
    //     "COMPUTERSYSTEM", "GET", "USERNAME"
    // ];
    // command.args(command_args);
    // let output = command.output()?;
    // println!("{}", String::from_utf8_lossy(&output.stdout));
    // return Ok(());

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
    let local_store_directory = Path::new(&opts.store_directory);
    let evidence_acquirers = create_evidence_acquirers(
        &remote_computer,
        local_store_directory,
        &opts,
    );
    for acquirer in evidence_acquirers {
        acquirer.run_all();
    }
    if opts.image_memory.is_some() {
        let memory_acquirers = create_memory_acquirers(
            &remote_computer,
            local_store_directory,
            &opts,
        );
        let image_memory_remote_store = opts.image_memory.as_ref().unwrap();
        for acquirer in memory_acquirers {
            info!("Running memory acquirer using method {}", acquirer.connector.connect_method_name());
            let image_res = acquirer.image_memory(Path::new(image_memory_remote_store.as_str()));
            if image_res.is_ok() {
                break;
            }
        }
    }
    if opts.custom_command_path.is_some() {
        let command_runners = create_command_runners(
            &remote_computer,
            local_store_directory,
            &opts,
        );
        for command_runner in command_runners {
            info!("Running commands using method {}", command_runner.connector.connect_method_name());
            command_runner.run_commands(Path::new(opts.custom_command_path.as_ref().unwrap()));
        }
    }
    if opts.search_files_path.is_some() {
        let search_files_path = opts.search_files_path.as_ref().unwrap();
        let search_files_path = Path::new(search_files_path);
        if opts.ssh {
            let remote_copier = ScpRemoteCopier::new(
                &remote_computer,
            );
            download_files(
                search_files_path,
                local_store_directory,
                &remote_copier,
            )?;
        } else {
            let copiers = create_windows_file_copiers(&opts, &remote_computer);
            for copier in copiers {
                let remote_copier = WindowsRemoteCopier::new(
                    &remote_computer,
                    copier.as_ref(),
                );
                let result = download_files(
                    search_files_path,
                    local_store_directory,
                    &remote_copier,
                );
                if result.is_ok() {
                    break;
                }
            }
        }
    }

    Ok(())
}

fn create_evidence_acquirers<'a>(
    computer: &'a Computer,
    local_store_directory: &'a Path,
    opts: &Opts,
) -> Vec<EvidenceAcquirer<'a>> {
    let acquirers: Vec<EvidenceAcquirer<'a>> = if opts.all {
        vec![
            EvidenceAcquirer::psexec(
                computer,
                local_store_directory,
            ),
            EvidenceAcquirer::wmi(
                computer,
                local_store_directory,
            ),
            EvidenceAcquirer::psremote(
                computer,
                local_store_directory,
            )
        ]
    } else {
        let mut acquirers = Vec::<EvidenceAcquirer<'a>>::new();
        if opts.psexec {
            acquirers.push(
                EvidenceAcquirer::psexec(
                    computer,
                    local_store_directory,
                ),
            );
        }
        if opts.wmi {
            acquirers.push(
                EvidenceAcquirer::wmi(
                    computer,
                    local_store_directory,
                ),
            );
        }
        if opts.psrem {
            acquirers.push(
                EvidenceAcquirer::psremote(
                    computer,
                    local_store_directory,
                )
            );
        }
        if opts.local {
            acquirers.push(
                EvidenceAcquirer::local(
                    computer,
                    local_store_directory,
                )
            )
        }
        if opts.ssh {
            acquirers.push(
                EvidenceAcquirer::ssh(
                    computer,
                    local_store_directory,
                )
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
                computer,
                local_store_directory,
            ),
            MemoryAcquirer::psremote(
                computer,
                local_store_directory,
            ),
        ]
    } else {
        let mut acquirers = Vec::<MemoryAcquirer>::new();
        if opts.psexec {
            acquirers.push(
                MemoryAcquirer::psexec(
                    computer,
                    local_store_directory,
                )
            );
        }
        if opts.psrem {
            acquirers.push(
                MemoryAcquirer::psremote(
                    computer,
                    local_store_directory,
                )
            );
        }
        // if opts.local {
        //     acquirers.push(
        //         MemoryAcquirer::local(
        //             Computer::from(opts.clone()),
        //             PathBuf::from(opts.store_directory.clone())
        //         )
        //     )
        // }
        acquirers
    };
    acquirers
}

fn create_command_runners<'a>(
    computer: &'a Computer,
    local_store_directory: &'a Path,
    opts: &Opts,
) -> Vec<CommandRunner<'a>> {
    let acquirers: Vec<CommandRunner<'a>> = if opts.all {
        vec![
            CommandRunner::psexec(
                computer,
                local_store_directory,
            ),
            CommandRunner::psremote(
                computer,
                local_store_directory,
            ),
        ]
    } else {
        let mut acquirers = Vec::<CommandRunner>::new();
        if opts.psexec {
            acquirers.push(
                CommandRunner::psexec(
                    computer,
                    local_store_directory,
                )
            );
        }
        if opts.psrem {
            acquirers.push(
                CommandRunner::psremote(
                    computer,
                    local_store_directory,
                )
            );
        }
        if opts.local {
            acquirers.push(
                CommandRunner::local(
                    computer,
                    local_store_directory,
                )
            )
        }
        if opts.ssh {
            acquirers.push(
                CommandRunner::ssh(
                    computer,
                    local_store_directory,
                )
            )
        }
        acquirers
    };
    acquirers
}

fn create_windows_file_copiers(opts: &Opts, computer: &Computer) -> Vec<Box<dyn Copier>> {
    let acquirers: Vec<Box<dyn Copier>> = if opts.all {
        vec![
            Box::new(XCopy {}),
            Box::new(PsCopyItem {})
        ]
    } else {
        let mut acquirers = Vec::<Box<dyn Copier>>::new();
        if opts.psexec {
            acquirers.push(
                Box::new(XCopy {})
            )
        }
        if opts.psrem {
            acquirers.push(
                Box::new(PsCopyItem {})
            );
        }
        acquirers
    };
    acquirers
}

