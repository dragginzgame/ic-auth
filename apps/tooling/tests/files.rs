use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

fn fixture() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/portable-fixtures/host-tooling-tests")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn invoke(root: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ic-auth-tooling"))
        .current_dir(root)
        .args(arguments)
        .output()
        .unwrap()
}

#[test]
fn hash_stdout_is_exact_and_oversize_has_no_identity() {
    let root = fixture();
    fs::write(root.join("input"), b"abc").unwrap();
    let accepted = invoke(&root, &["hash-file", "input", "3"]);
    assert!(accepted.status.success());
    assert_eq!(
        accepted.stdout,
        b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\n"
    );
    let rejected = invoke(&root, &["hash-file", "input", "2"]);
    assert!(!rejected.status.success());
    assert!(rejected.stdout.is_empty());
    assert!(!invoke(&root, &["hash-file", ".", "3"]).status.success());
}

#[test]
fn intent_creation_never_replaces_saved_bytes() {
    let root = fixture();
    fs::write(root.join("input"), b"original intent").unwrap();
    assert!(
        invoke(&root, &["create-private", "input", "state/intent", "1024"])
            .status
            .success()
    );
    fs::write(root.join("input"), b"conflicting intent").unwrap();
    assert!(
        !invoke(&root, &["create-private", "input", "state/intent", "1024"])
            .status
            .success()
    );
    assert_eq!(
        fs::read(root.join("state/intent")).unwrap(),
        b"original intent"
    );
}

#[test]
fn receipt_replacement_is_bounded_and_private() {
    let root = fixture();
    fs::write(root.join("input"), b"validated receipt").unwrap();
    fs::write(root.join("receipt"), b"old").unwrap();
    assert!(
        !invoke(&root, &["replace-private", "input", "receipt", "1"])
            .status
            .success()
    );
    assert_eq!(fs::read(root.join("receipt")).unwrap(), b"old");
    assert!(
        invoke(&root, &["replace-private", "input", "receipt", "1024"])
            .status
            .success()
    );
    assert_eq!(
        fs::read(root.join("receipt")).unwrap(),
        b"validated receipt"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(root.join("receipt"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}

#[cfg(unix)]
#[test]
fn symlink_inputs_and_existing_intent_links_are_refused() {
    use std::os::unix::fs::symlink;
    let root = fixture();
    fs::write(root.join("input"), b"protected bytes").unwrap();
    symlink("input", root.join("link")).unwrap();
    assert!(
        !invoke(&root, &["hash-file", "link", "1024"])
            .status
            .success()
    );
    assert!(
        !invoke(&root, &["create-private", "link", "new", "1024"])
            .status
            .success()
    );
    assert!(!root.join("new").exists());
    assert!(
        !invoke(&root, &["create-private", "input", "link", "1024"])
            .status
            .success()
    );
    assert!(
        fs::symlink_metadata(root.join("link"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read(root.join("input")).unwrap(), b"protected bytes");
}
