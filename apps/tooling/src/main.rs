//! Auth client-contract generation and external Cargo-lock admission.

use clap::{Parser, Subcommand, ValueEnum};
use ic_build::cargo::packages::verify_external_lock_subset;
use ic_host_fs::read::read_file_no_follow;
use std::{error::Error, path::PathBuf};

mod client_contracts;

#[derive(Parser)]
#[command(
    version,
    about = "IC Auth external-lock and client-contract operations"
)]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, ValueEnum)]
enum ContractMode {
    Generate,
    Check,
}

#[derive(Subcommand)]
enum Command {
    ClientContracts {
        #[arg(value_enum)]
        mode: ContractMode,
        #[arg(allow_hyphen_values = true)]
        directory: PathBuf,
    },
    VerifyExternalLock {
        #[arg(allow_hyphen_values = true)]
        original: PathBuf,
        #[arg(allow_hyphen_values = true)]
        candidate: PathBuf,
        max_bytes: usize,
    },
}

fn run(command: Command) -> Result<(), Box<dyn Error>> {
    match command {
        Command::ClientContracts { mode, directory } => {
            client_contracts::run(mode, &directory)?;
        }
        Command::VerifyExternalLock {
            original,
            candidate,
            max_bytes,
        } => {
            let original = read_file_no_follow(&original, max_bytes)?;
            let candidate = read_file_no_follow(&candidate, max_bytes)?;
            verify_external_lock_subset(&original, &candidate, max_bytes)?;
        }
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    let arguments = Arguments::parse();
    match run(arguments.command) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Auth tooling operation refused: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
