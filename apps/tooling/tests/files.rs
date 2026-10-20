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
    let mut command = if matches!(
        arguments.first(),
        Some(&"client-contracts" | &"verify-external-lock")
    ) {
        Command::new(env!("CARGO_BIN_EXE_ic-auth-tooling"))
    } else {
        let mut command = Command::new("bash");
        command
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/dev/run-host-file.sh"));
        command
    };
    command.current_dir(root).args(arguments).output().unwrap()
}

#[test]
fn argument_refusals_are_effect_free_and_help_and_version_succeed() {
    let root = fixture();
    fs::write(root.join("input"), b"protected").unwrap();
    for arguments in [
        vec![],
        vec!["unknown"],
        vec!["hash-file", "input"],
        vec!["hash-file", "input", "invalid"],
        vec!["hash-file", "input", "-1"],
        vec!["hash-file", "input", "8", "excess"],
        vec!["client-contracts", "unknown", "."],
        vec!["create-private", "input", "output"],
        vec!["create-private", "input", "output", "8", "excess"],
    ] {
        let result = invoke(&root, &arguments);
        assert_eq!(result.status.code(), Some(1), "{arguments:?}: {result:?}");
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
        assert_eq!(fs::read(root.join("input")).unwrap(), b"protected");
        assert!(!root.join("output").exists());
    }
    for arguments in [
        vec!["--help"],
        vec!["--version"],
        vec!["hash-file", "--help"],
    ] {
        let result = invoke(&root, &arguments);
        assert!(result.status.success(), "{result:?}");
        assert!(!result.stdout.is_empty());
    }
    for name in ["space path", "-input"] {
        fs::write(root.join(name), b"abc").unwrap();
        let result = invoke(&root, &["hash-file", "--", name, "3"]);
        assert!(result.status.success(), "{result:?}");
    }
    #[cfg(target_os = "linux")]
    {
        use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
        let name = OsStr::from_bytes(b"nonutf8\xff");
        fs::write(root.join(name), b"abc").unwrap();
        let result = Command::new("bash")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/dev/run-host-file.sh"))
            .current_dir(&root)
            .arg("hash-file")
            .arg(name)
            .arg("3")
            .output()
            .unwrap();
        assert!(result.status.success(), "{result:?}");
    }
}

#[test]
fn external_lock_admission_bounds_both_inputs_without_mutation() {
    let root = fixture();
    for name in ["original", "candidate"] {
        fs::write(root.join(name), b"package=[]").unwrap();
    }
    let accepted = invoke(
        &root,
        &["verify-external-lock", "original", "candidate", "10"],
    );
    assert!(accepted.status.success(), "{accepted:?}");
    assert!(accepted.stdout.is_empty());
    for name in ["original", "candidate"] {
        fs::write(root.join(name), b"package=[]\n").unwrap();
        let refused = invoke(
            &root,
            &["verify-external-lock", "original", "candidate", "10"],
        );
        assert!(!refused.status.success());
        assert!(refused.stdout.is_empty());
        assert_eq!(fs::read(root.join(name)).unwrap(), b"package=[]\n");
        fs::write(root.join(name), b"package=[]").unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(name, root.join("link")).unwrap();
            let (original, candidate) = if name == "original" {
                ("link", "candidate")
            } else {
                ("original", "link")
            };
            let refused = invoke(&root, &["verify-external-lock", original, candidate, "10"]);
            assert!(!refused.status.success());
            assert!(refused.stdout.is_empty());
            assert!(
                fs::symlink_metadata(root.join("link"))
                    .unwrap()
                    .is_symlink()
            );
            fs::remove_file(root.join("link")).unwrap();
        }
    }
    for name in ["original", "candidate"] {
        assert_eq!(fs::read(root.join(name)).unwrap(), b"package=[]");
    }
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
    let mut child = Command::new("bash")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/dev/run-host-file.sh"))
        .current_dir(root)
        .args(["hash-files0", maximum, "4194304", "4096"])
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
    let odd = b"space tab\tnewline\nutf8-\xc3\xa9";
    fs::write(root.join(OsStr::from_bytes(odd)), b"abc").unwrap();
    fs::write(root.join("plain"), b"abc").unwrap();
    let paths: Vec<&[u8]> = vec![odd.as_slice(), b"plain", odd.as_slice()];
    // macOS rejects invalid UTF-8 names before the hashing protocol runs.
    // Keep raw filesystem coverage on Linux and shared path/ordering coverage
    // on every supported Unix host.
    #[cfg(target_os = "linux")]
    let paths = {
        let non_utf8 = b"nonutf8\xff";
        fs::write(root.join(OsStr::from_bytes(non_utf8)), b"abc").unwrap();
        let mut paths = paths;
        paths.push(non_utf8);
        paths
    };
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
        b"ok\0missing\xff\0".to_vec(),
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
    use ic_auth_wire::{CredentialEnvelope, Logout, wire::Method};

    let vectors = [
        ("credential", CredentialEnvelope::REQUEST_LIMITS.bytes),
        ("logout", Logout::RESPONSE_LIMITS.bytes),
    ];
    let root = fixture();
    let generated = invoke(&root, &["client-contracts", "generate", "."]);
    assert!(generated.status.success(), "{:?}", generated);
    for (name, _) in vectors {
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
    for (name, maximum) in vectors {
        let path = root.join(format!("{name}-from-ts.bin"));
        let original = fs::read(&path).unwrap();
        let mut trailing = original.clone();
        trailing.push(0);
        let other = if name == "credential" {
            "logout"
        } else {
            "credential"
        };
        let wrong_frame = fs::read(root.join(format!("{other}-from-ts.bin"))).unwrap();
        for invalid in [vec![0], trailing, wrong_frame, vec![0; maximum + 1]] {
            fs::write(&path, &invalid).unwrap();
            assert!(
                !invoke(&root, &["client-contracts", "check", "."])
                    .status
                    .success()
            );
            assert_eq!(fs::read(&path).unwrap(), invalid);
        }
        fs::remove_file(&path).unwrap();
        assert!(
            !invoke(&root, &["client-contracts", "check", "."])
                .status
                .success()
        );
        assert!(!path.exists());
        fs::write(path, original).unwrap();
    }
}
