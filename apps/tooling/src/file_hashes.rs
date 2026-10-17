//! One bounded, ordered batch over the existing Host file-identity owner.
use ic_host_fs::read::hash_file_no_follow;
use std::{
    error::Error,
    ffi::OsStr,
    io::{Read, Write},
    os::unix::ffi::OsStrExt,
    path::Path,
};

const MAX_PATH_BYTES: u64 = 4 * 1024 * 1024;
const MAX_FILES: usize = 4096;

pub(crate) fn run(maximum: usize) -> Result<(), Box<dyn Error>> {
    let mut input = Vec::new();
    std::io::stdin()
        .lock()
        .take(MAX_PATH_BYTES + 1)
        .read_to_end(&mut input)?;
    if input.len() as u64 > MAX_PATH_BYTES {
        return Err("file list exceeds 4 MiB".into());
    }
    if input.is_empty() {
        return Ok(());
    }
    let paths = input
        .strip_suffix(&[0])
        .ok_or("file list must be NUL-terminated")?;
    let mut output = Vec::new();
    let paths = paths.split(|&byte| byte == 0);
    if paths.clone().count() > MAX_FILES || paths.clone().any(|path| path.is_empty()) {
        return Err("file list exceeds 4096 paths or contains an empty path".into());
    }
    for path in paths {
        let identity = hash_file_no_follow(Path::new(OsStr::from_bytes(path)), maximum as u64)?;
        output.extend_from_slice(identity.sha256.to_string().as_bytes());
        output.push(b'\t');
        output.extend_from_slice(path);
        output.push(0);
    }
    // A later missing/oversized/linked file cannot leave a successful prefix.
    std::io::stdout().lock().write_all(&output)?;
    Ok(())
}
