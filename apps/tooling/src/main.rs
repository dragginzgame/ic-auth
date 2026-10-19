//! Local bounded file identities and durable release/publication evidence.
//! Callers own parent paths, locking, artifact limits and effect reconciliation.

use clap::{Parser, Subcommand, ValueEnum};
use ic_build::cargo::packages::verify_external_lock_subset;
use ic_host_fs::{
    durable::{PublicationMode, WriteOptions, write_with},
    read::{hash_file_no_follow, read_file_no_follow},
};
use std::{error::Error, io::Write, path::PathBuf};

mod client_contracts;
#[cfg(unix)]
mod file_hashes;

#[derive(Parser)]
#[command(
    version,
    about = "Bounded IC Auth file, release and client-contract operations"
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
    HashFile {
        #[arg(allow_hyphen_values = true)]
        path: PathBuf,
        max_bytes: usize,
    },
    #[cfg(unix)]
    HashFiles0 { max_bytes: usize },
    CreatePrivate {
        #[arg(allow_hyphen_values = true)]
        source: PathBuf,
        #[arg(allow_hyphen_values = true)]
        destination: PathBuf,
        max_bytes: usize,
    },
    ReplacePrivate {
        #[arg(allow_hyphen_values = true)]
        source: PathBuf,
        #[arg(allow_hyphen_values = true)]
        destination: PathBuf,
        max_bytes: usize,
    },
}

fn run(command: Command) -> Result<(), Box<dyn Error>> {
    match command {
        Command::ClientContracts { mode, directory } => {
            client_contracts::run(
                match mode {
                    ContractMode::Generate => "generate",
                    ContractMode::Check => "check",
                },
                &directory,
            )?;
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
        #[cfg(unix)]
        Command::HashFiles0 { max_bytes } => file_hashes::run(max_bytes)?,
        Command::HashFile { path, max_bytes } => {
            let identity = hash_file_no_follow(&path, max_bytes as u64)?;
            println!("{}", identity.sha256);
        }
        Command::CreatePrivate {
            ref source,
            ref destination,
            max_bytes,
        }
        | Command::ReplacePrivate {
            ref source,
            ref destination,
            max_bytes,
        } => {
            let bytes = read_file_no_follow(source, max_bytes)?;
            let mode = if matches!(command, Command::CreatePrivate { .. }) {
                PublicationMode::CreateNew
            } else {
                PublicationMode::Replace
            };
            write_with(
                destination,
                WriteOptions {
                    mode,
                    permissions: 0o600,
                },
                |file| file.write_all(&bytes),
            )?;
        }
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    let arguments = match Arguments::try_parse() {
        Ok(arguments) => arguments,
        Err(error) => {
            let failure = error.use_stderr();
            let _ = error.print();
            return if failure {
                std::process::ExitCode::FAILURE
            } else {
                std::process::ExitCode::SUCCESS
            };
        }
    };
    match run(arguments.command) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("host file operation refused: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
