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
use std::path::Path;
use crate::standard_tools_evidence_acquirer::StandardToolsEvidenceAcquirer;
use crate::wmi_evidence_acquirer::WmiEvidenceAcquirer;
use crate::remote::{PsExec, PowerShell, Local};
use crate::memory_acquirer::MemoryAcquirer;

mod process_runner;
mod evidence_acquirer;
mod remote;
mod arg_parser;
mod logo;
mod standard_tools_evidence_acquirer;
mod wmi_evidence_acquirer;
mod memory_acquirer;
mod command_utils;
mod utils;

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
    // let evidence_acquirers = create_evidence_acquirers(&opts);
    // for acquirer in evidence_acquirers {
    //     acquirer.run_all(
    //         opts.custom_command_path.as_ref().map(|v| Path::new(v)),
    //         opts.search_files_path.as_ref().map(|v| Path::new(v)),
    //         // force_mem_acquire
    //     );
    // }
    if opts.image_memory.is_some() {
        let memory_acquirer = create_memory_acquirers(&opts);
        let image_memory_remote_store = opts.image_memory.unwrap();
        for acquirer in memory_acquirer {
            let image_res = acquirer.image_memory(Path::new(image_memory_remote_store.as_str()));
            if image_res.is_ok() {
                break;
            }
        }
    }

    Ok(())
}

fn create_evidence_acquirers(opts: &Opts) -> Vec<Box<dyn EvidenceAcquirer>> {
    let acquirers: Vec<Box<dyn EvidenceAcquirer>> = if opts.all {
        let mut acquirers = Vec::<Box<dyn EvidenceAcquirer>>::new();
        acquirers.push(
            Box::new(StandardToolsEvidenceAcquirer::from_opts(
                &opts, Box::new(PsExec {}),
            ))
        );
        acquirers.push(
            Box::new(WmiEvidenceAcquirer::from_opts(&opts))
        );
        acquirers.push(
            Box::new(StandardToolsEvidenceAcquirer::from_opts(
                &opts, Box::new(PowerShell {}),
            ))
        );
        acquirers
    } else {
        let mut acquirers = Vec::<Box<dyn EvidenceAcquirer>>::new();
        if opts.psexec {
            acquirers.push(
                Box::new(StandardToolsEvidenceAcquirer::from_opts(
                    &opts, Box::new(PsExec {}),
                ))
            );
        }
        if opts.wmi {
            acquirers.push(
                Box::new(WmiEvidenceAcquirer::from_opts(&opts))
            );
        }
        if opts.psrem {
            acquirers.push(
                Box::new(StandardToolsEvidenceAcquirer::from_opts(
                    &opts, Box::new(PowerShell {}),
                ))
            );
        }
        if opts.local {
            acquirers.push(
                Box::new(StandardToolsEvidenceAcquirer::from_opts(
                    &opts, Box::new(Local {}),
                ))
            )
        }
        acquirers
    };
    acquirers
}


fn create_memory_acquirers(opts: &Opts) -> Vec<MemoryAcquirer> {
    let acquirers: Vec<MemoryAcquirer> = if opts.all {
        let mut acquirers = Vec::<MemoryAcquirer>::new();
        acquirers.push(
            MemoryAcquirer::from_opts(
                &opts, Box::new(PsExec {}),
            )
        );
        acquirers.push(
            MemoryAcquirer::from_opts(
                &opts, Box::new(PowerShell {}),
            )
        );
        acquirers
    } else {
        let mut acquirers = Vec::<MemoryAcquirer>::new();
        if opts.psexec {
            acquirers.push(
                MemoryAcquirer::from_opts(
                    &opts, Box::new(PsExec {}),
                )
            );
        }
        if opts.psrem {
            acquirers.push(
                MemoryAcquirer::from_opts(
                    &opts, Box::new(PowerShell {}),
                )
            );
        }
        if opts.local {
            acquirers.push(
                MemoryAcquirer::from_opts(
                    &opts, Box::new(Local {}),
                )
            )
        }
        acquirers
    };
    acquirers
}
