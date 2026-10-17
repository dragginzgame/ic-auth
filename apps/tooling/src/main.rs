//! Local bounded file identities and durable release/publication evidence.
//! Callers own parent paths, locking, artifact limits and effect reconciliation.

use ic_host_fs::{
    durable::{PublicationMode, WriteOptions, write_with},
    read::{hash_file_no_follow, read_file_no_follow},
};
use std::{env, error::Error, ffi::OsString, io::Write, path::Path};

mod client_contracts;
#[cfg(unix)]
mod file_hashes;

fn run(arguments: &[OsString]) -> Result<(), Box<dyn Error>> {
    if arguments
        .first()
        .is_some_and(|value| value == "client-contracts")
    {
        if arguments.len() != 3 {
            return Err("client-contracts requires generate|check DIRECTORY".into());
        }
        return client_contracts::run(
            arguments[1].to_str().ok_or("invalid mode")?,
            Path::new(&arguments[2]),
        );
    }
    let usage = "usage: ic-auth-tooling hash-file PATH MAX_BYTES | hash-files0 MAX_BYTES (Unix, NUL paths on stdin) | {create-private|replace-private} SOURCE DESTINATION MAX_BYTES";
    let Some(operation) = arguments.first().and_then(|value| value.to_str()) else {
        return Err(usage.into());
    };
    let limit = arguments
        .last()
        .and_then(|value| value.to_str())
        .ok_or(usage)?
        .parse::<usize>()?;
    match (operation, arguments.len()) {
        #[cfg(unix)]
        ("hash-files0", 2) => file_hashes::run(limit)?,
        ("hash-file", 3) => {
            let identity = hash_file_no_follow(Path::new(&arguments[1]), limit as u64)?;
            println!("{}", identity.sha256);
        }
        ("create-private" | "replace-private", 4) => {
            let bytes = read_file_no_follow(Path::new(&arguments[1]), limit)?;
            let mode = if operation == "create-private" {
                PublicationMode::CreateNew
            } else {
                PublicationMode::Replace
            };
            write_with(
                Path::new(&arguments[2]),
                WriteOptions {
                    mode,
                    permissions: 0o600,
                },
                |file| file.write_all(&bytes),
            )?;
        }
        _ => return Err(usage.into()),
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run(&env::args_os().skip(1).collect::<Vec<_>>()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("host file operation refused: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
