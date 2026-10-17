//! Local bounded file identities and durable release/publication evidence.
//! Callers own parent paths, locking, artifact limits and effect reconciliation.

use ic_build::cargo::packages::verify_external_lock_subset;
use ic_host_fs::{
    durable::{PublicationMode, WriteOptions, write_with},
    read::{hash_file_no_follow, read_file_no_follow},
};
use std::{env, error::Error, ffi::OsString, io::Write, path::Path};

mod client_contracts;

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
    let usage = "usage: ic-auth-tooling hash-file PATH MAX_BYTES | verify-external-lock ORIGINAL CANDIDATE MAX_BYTES | {create-private|replace-private} SOURCE DESTINATION MAX_BYTES";
    let Some(operation) = arguments.first().and_then(|value| value.to_str()) else {
        return Err(usage.into());
    };
    let limit = arguments
        .last()
        .and_then(|value| value.to_str())
        .ok_or(usage)?
        .parse::<usize>()?;
    match (operation, arguments.len()) {
        ("verify-external-lock", 4) => {
            let original = read_file_no_follow(Path::new(&arguments[1]), limit)?;
            let candidate = read_file_no_follow(Path::new(&arguments[2]), limit)?;
            verify_external_lock_subset(&original, &candidate, limit)?;
        }
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
