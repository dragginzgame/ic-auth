//! Local bounded file identities and durable release/publication evidence.
//! Callers own parent paths, locking, artifact limits and effect reconciliation.

use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_fs::{
    durable::{PublicationMode, WriteOptions, write_typed_with},
    read::read_file_no_follow,
};
use std::{env, error::Error, ffi::OsString, io::Write, path::Path};

fn run(arguments: &[OsString]) -> Result<(), Box<dyn Error>> {
    let usage = "usage: ic-auth-tooling hash-file PATH MAX_BYTES | {create-private|replace-private} SOURCE DESTINATION MAX_BYTES";
    let Some(operation) = arguments.first().and_then(|value| value.to_str()) else {
        return Err(usage.into());
    };
    let limit = arguments
        .last()
        .and_then(|value| value.to_str())
        .ok_or(usage)?
        .parse::<usize>()?;
    match (operation, arguments.len()) {
        ("hash-file", 3) => {
            let bytes = read_file_no_follow(Path::new(&arguments[1]), limit)?;
            println!("{}", Sha256Digest::compute(&bytes));
        }
        ("create-private" | "replace-private", 4) => {
            let bytes = read_file_no_follow(Path::new(&arguments[1]), limit)?;
            let mode = if operation == "create-private" {
                PublicationMode::CreateNew
            } else {
                PublicationMode::Replace
            };
            write_typed_with(
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
