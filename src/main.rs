use std::io;
use simplelog::{CombinedLogger, TermLogger, WriteLogger, Config, TerminalMode, LevelFilter};
use std::fs::{File, create_dir_all};
use crate::logo::print_logo;
use crate::arg_parser::Opts;

#[macro_use]
extern crate log;
extern crate simplelog;

use clap::derive::Clap;
use crate::evidence_acquirer::{EvidenceAcquirer, AnyEvidenceAcquirer};
use std::path::Path;
use crate::remote_computer::PsExec;

mod process_runner;
mod interactive_process;
mod evidence_acquirer;
mod remote_computer;
mod arg_parser;
mod logo;
// mod path_utils;

fn setup_logger() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Trace, Config::default(), TerminalMode::Mixed).unwrap(),
            WriteLogger::new(LevelFilter::Trace, Config::default(), File::create("gargamel.log").unwrap()),
        ]
    ).unwrap();
}

//
fn main() -> Result<(), io::Error> {
    setup_logger();
    print_logo();
    let opts: Opts = Opts::parse();
    create_dir_all(&opts.store_directory)?;
    let acquirers = create_evidence_acquirers(&opts);
    for acquirer in acquirers {
        let result = acquirer.run_all(
            opts.custom_command_path.as_ref().map(|v| Path::new(v)),
            opts.search_files_path.as_ref().map(|v| Path::new(v)),
            opts.fast_mode,
        );
        if result.is_err() {
            error!("{}", result.unwrap_err())
        }
    }

    Ok(())
}

fn create_evidence_acquirers(opts: &Opts) -> Vec<Box<dyn AnyEvidenceAcquirer>> {
    let acquirers: Vec<Box<dyn AnyEvidenceAcquirer>> = if opts.all {
        let mut acquirers = Vec::<Box<dyn AnyEvidenceAcquirer>>::new();
        acquirers.push(
            Box::new(EvidenceAcquirer::from_opts(
                &opts, PsExec {},
            ))
        );

        acquirers
    } else {
        let mut acquirers = Vec::<Box<dyn AnyEvidenceAcquirer>>::new();
        if opts.psexec {
            acquirers.push(
                Box::new(EvidenceAcquirer::from_opts(
                    &opts, PsExec {},
                ))
            );
        }
        acquirers
    };
    acquirers
}