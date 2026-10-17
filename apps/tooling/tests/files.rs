use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

fn fixture() -> PathBuf {
    // Wall-clock precision varies by host; parallel tests can observe the same
    // timestamp. A process-local sequence keeps their evidence disjoint.
    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/portable-fixtures/host-tooling-tests")
        .join(format!(
            "{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::create_dir(&path).unwrap();
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

#[cfg(unix)]
fn batch(root: &Path, input: &[u8], maximum: &str) -> Output {
    use std::{io::Write, process::Stdio};
    let mut child = Command::new(env!("CARGO_BIN_EXE_ic-auth-tooling"))
        .current_dir(root)
        .args(["hash-files0", maximum])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[cfg(unix)]
#[test]
fn batch_hashing_preserves_raw_paths_order_duplicates_and_zero_limits() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
    let root = fixture();
    let odd = b"space tab\tnewline\nnonutf8\xff";
    fs::write(root.join(OsStr::from_bytes(odd)), b"abc").unwrap();
    fs::write(root.join("plain"), b"abc").unwrap();
    let paths = [odd.as_slice(), b"plain", odd.as_slice()];
    let input: Vec<_> = paths
        .iter()
        .flat_map(|path| path.iter().copied().chain([0]))
        .collect();
    let result = batch(&root, &input, "3");
    assert!(result.status.success(), "{:?}", result);
    let mut expected = Vec::new();
    for path in paths {
        expected.extend_from_slice(
            b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\t",
        );
        expected.extend_from_slice(path);
        expected.push(0);
    }
    assert_eq!(result.stdout, expected);
    fs::write(root.join("empty"), []).unwrap();
    assert_eq!(
        batch(&root, b"empty\0", "0").stdout,
        b"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\tempty\0"
    );
    assert!(batch(&root, b"", "0").status.success());
}

#[cfg(unix)]
#[test]
fn rejected_hash_batches_emit_no_partial_seal() {
    use std::os::unix::fs::symlink;
    let root = fixture();
    fs::write(root.join("ok"), b"abc").unwrap();
    fs::write(root.join("big"), b"abcd").unwrap();
    symlink("ok", root.join("link")).unwrap();
    for input in [
        b"ok\0missing\0".to_vec(),
        b"ok\0big\0".to_vec(),
        b"ok\0link\0".to_vec(),
        b"ok\0.\0".to_vec(),
        b"ok\0\0".to_vec(),
        b"ok".to_vec(),
        b"ok\0".repeat(4097),
        vec![b'x'; 4 * 1024 * 1024 + 1],
    ] {
        let result = batch(&root, &input, "3");
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
    }
}

#[test]
fn empty_file_hash_accepts_zero_limit_without_admitting_nonempty_bytes() {
    let root = fixture();
    fs::write(root.join("empty"), b"").unwrap();
    let accepted = invoke(&root, &["hash-file", "empty", "0"]);
    assert!(accepted.status.success());
    assert_eq!(
        accepted.stdout,
        b"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n"
    );
    fs::write(root.join("one-byte"), b"x").unwrap();
    let rejected = invoke(&root, &["hash-file", "one-byte", "0"]);
    assert!(!rejected.status.success());
    assert!(rejected.stdout.is_empty());
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

#[test]
fn directory_required_targets_never_publish_or_replace_stripped_names() {
    let root = fixture();
    fs::write(root.join("input"), b"new receipt").unwrap();
    for operation in ["create-private", "replace-private"] {
        for suffix in ["/", "/.", "//"] {
            fs::write(root.join("receipt"), b"saved receipt").unwrap();
            for destination in [format!("receipt{suffix}"), format!("new/receipt{suffix}")] {
                let refused = invoke(&root, &[operation, "input", &destination, "1024"]);
                assert!(!refused.status.success(), "{operation} {destination}");
                assert!(refused.stdout.is_empty());
                assert_eq!(fs::read(root.join("receipt")).unwrap(), b"saved receipt");
                assert!(!root.join("new").exists());
            }
        }
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

#[test]
fn client_contract_bundle_checks_all_vectors_and_refuses_bad_input() {
    let root = fixture();
    let generated = invoke(&root, &["client-contracts", "generate", "."]);
    assert!(generated.status.success(), "{:?}", generated);
    for name in ["request", "credential", "logout"] {
        let hex = fs::read_to_string(root.join(format!("{name}-vector.hex"))).unwrap();
        let bytes: Vec<u8> = hex
            .trim()
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect();
        fs::write(root.join(format!("{name}-from-ts.bin")), bytes).unwrap();
    }
    assert!(
        invoke(&root, &["client-contracts", "check", "."])
            .status
            .success()
    );
    for name in ["request", "credential", "logout"] {
        let path = root.join(format!("{name}-from-ts.bin"));
        let original = fs::read(&path).unwrap();
        for invalid in [vec![0], vec![0; 131_073]] {
            fs::write(&path, &invalid).unwrap();
            assert!(
                !invoke(&root, &["client-contracts", "check", "."])
                    .status
                    .success()
            );
            assert_eq!(fs::read(&path).unwrap(), invalid);
        }
        fs::write(path, original).unwrap();
    }
}
