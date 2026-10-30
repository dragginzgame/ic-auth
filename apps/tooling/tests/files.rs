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
fn argument_refusals_are_effect_free_and_help_and_version_succeed() {
    let root = fixture();
    fs::write(root.join("input"), b"protected").unwrap();
    for arguments in [
        vec![],
        vec!["unknown"],
        vec!["client-contracts", "unknown", "."],
        vec!["client-contracts", "generate"],
        vec!["verify-external-lock", "input", "output", "invalid"],
        vec!["verify-external-lock", "input", "output", "-1"],
        vec!["verify-external-lock", "input", "output", "8", "excess"],
    ] {
        let result = invoke(&root, &arguments);
        assert_eq!(result.status.code(), Some(2), "{arguments:?}: {result:?}");
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
        assert_eq!(fs::read(root.join("input")).unwrap(), b"protected");
        assert!(!root.join("output").exists());
    }
    for arguments in [
        vec!["--help"],
        vec!["--version"],
        vec!["client-contracts", "--help"],
        vec!["verify-external-lock", "--help"],
    ] {
        let result = invoke(&root, &arguments);
        assert!(result.status.success(), "{result:?}");
        assert!(!result.stdout.is_empty());
    }
    let version = invoke(&root, &["--version"]);
    assert!(
        String::from_utf8(version.stdout)
            .unwrap()
            .contains("ic-auth-tooling")
    );
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

// Host owns the exhaustive file-command tests. Keep Auth's actual adapter
// boundary: offline selection, preserved cwd/argv, exact output and refusal.
#[test]
fn host_file_adapter_preserves_working_directory_arguments_and_refusal() {
    let root = fixture();
    let call = |arguments: &[&str]| {
        Command::new("bash")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/dev/run-host-file.sh"))
            .current_dir(&root)
            .args(arguments)
            .output()
            .unwrap()
    };
    fs::write(root.join("-input with spaces"), b"abc").unwrap();
    let accepted = call(&["hash-file", "--", "-input with spaces", "3"]);
    assert!(accepted.status.success(), "{accepted:?}");
    assert_eq!(
        accepted.stdout,
        b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\n"
    );
    let refused = call(&["hash-file", "--", "-input with spaces", "2"]);
    assert!(!refused.status.success());
    assert!(refused.stdout.is_empty());
    assert_eq!(fs::read(root.join("-input with spaces")).unwrap(), b"abc");
    assert!(
        call(&["create-private", "--", "-input with spaces", "intent", "3"])
            .status
            .success()
    );
    fs::write(root.join("-input with spaces"), b"new").unwrap();
    assert!(
        !call(&["create-private", "--", "-input with spaces", "intent", "3"])
            .status
            .success()
    );
    assert_eq!(fs::read(root.join("intent")).unwrap(), b"abc");
}

#[test]
fn client_contract_bundle_checks_all_vectors_and_refuses_bad_input() {
    use ic_auth_wire::{CredentialEnvelope, EstablishSession, Logout, ReadSession};

    let vectors = [
        ("credential", CredentialEnvelope::REQUEST_LIMITS.bytes),
        ("logout", Logout::RESPONSE_LIMITS.bytes),
        ("admission", EstablishSession::RESPONSE_LIMITS.bytes),
        ("read", ReadSession::RESPONSE_LIMITS.bytes),
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
