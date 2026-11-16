//! Shared installed-fixture setup; Build and Testkit retain their ownership boundaries.
use ic_build::{Artifact, BuildRequest, Limits, build};
use ic_testkit::{
    pic::{InstallSpec, PocketIcInstance, PocketIcStartupConfig, create_and_install},
    pocket_ic::{CanisterId, PocketIcBuilder, nonblocking::PocketIc},
};
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    sync::{OnceLock, atomic::AtomicBool},
    time::Duration,
};

pub(crate) fn wasm() -> Vec<u8> {
    // One build per harness process. Every case installs the same owned bytes;
    // there is no prebuilt path, stale fallback or retained mutable artifact.
    static WASM: OnceLock<Artifact> = OnceLock::new();
    WASM.get_or_init(|| {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();
        let root = ic_host_fs::Directory::open(&workspace, 0).unwrap().unwrap();
        let _scratch = root
            .prepare_directories(&["target".as_ref(), "host-build".as_ref()], 0o700, 0)
            .unwrap();
        let request = BuildRequest {
            manifest: workspace.join("ic-auth-host-fixture/Cargo.toml"),
            package: "ic-auth-host-fixture".into(),
            package_id: std::env::var("AUTH_HOST_PACKAGE_ID").expect("run make qualify-host"),
            library: "ic_auth_host_fixture".into(),
            cargo: PathBuf::from(
                std::env::var_os("AUTH_HOST_CARGO").expect("run make qualify-host"),
            ),
            rustc: PathBuf::from(
                std::env::var_os("AUTH_HOST_RUSTC").expect("run make qualify-host"),
            ),
            scratch_parent: workspace.join("target/host-build"),
            profile: "release".into(),
            features: Vec::new(),
            default_features: true,
            // Prepared tool/cache locations only; Build owns RUSTC and target-dir.
            environment: ["HOME", "PATH", "CARGO_HOME", "RUSTUP_HOME", "TMPDIR"]
                .into_iter()
                .filter_map(|name| std::env::var_os(name).map(|value| (name.into(), value)))
                .collect(),
            limits: Limits {
                stdout_bytes: 16 * 1024 * 1024,
                stderr_bytes: 4 * 1024 * 1024,
                record_bytes: 1024 * 1024,
                candidates: 4,
                artifact_bytes: 16 * 1024 * 1024,
                elapsed: Duration::from_secs(300),
            },
        };
        let artifact = build(&request, &AtomicBool::new(false), &mut std::io::sink())
            .unwrap_or_else(|failure| panic!("owned fixture build failed: {failure:?}"));
        let digest: [u8; 32] = Sha256::digest(artifact.bytes()).into();
        assert_eq!(artifact.sha256(), &digest);
        artifact
    })
    .bytes()
    .to_vec()
}

pub(crate) async fn start() -> PocketIcInstance {
    PocketIcStartupConfig::from_env(Duration::from_secs(30))
        .unwrap()
        .build_instance(
            PocketIcBuilder::new()
                .with_max_request_time_ms(Some(30_000))
                // A single root subnet supplies the certificate trust key too.
                .with_nns_subnet(),
        )
        .await
        .unwrap()
}
pub(crate) async fn install(pic: &PocketIc, wasm: &[u8]) -> CanisterId {
    create_and_install(
        pic,
        InstallSpec {
            wasm: wasm.to_vec(),
            init_bytes: vec![],
            cycles: 5_000_000_000_000,
            install_sender: None,
        },
    )
    .await
}
