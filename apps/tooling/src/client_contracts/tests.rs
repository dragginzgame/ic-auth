use super::*;
use crate::ContractMode;
use ic_host_fs::testing::FixtureDirectory;

#[test]
fn interop_checks_reject_different_values_and_wrong_contract_frames() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
    let scratch = FixtureDirectory::new(&root, "client-vectors").unwrap();
    let directory = scratch.path();
    run(ContractMode::Generate, directory).unwrap();
    let manifest = std::fs::read_to_string(directory.join("artifacts.tsv")).unwrap();
    let mut vectors = 0;
    for line in manifest.lines() {
        let Some(name) = line.strip_prefix("evidence\t") else {
            continue;
        };
        let Some(stem) = name.strip_suffix("-vector.hex") else {
            continue;
        };
        let hex = std::fs::read_to_string(directory.join(name)).unwrap();
        let bytes: Vec<_> = hex
            .trim()
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect();
        std::fs::write(directory.join(format!("{stem}-from-ts.bin")), bytes).unwrap();
        vectors += 1;
    }
    assert_eq!(vectors, 4);
    run(ContractMode::Check, directory).unwrap();

    let read = directory.join("read-from-ts.bin");
    std::fs::write(
        &read,
        ReadSession::encode_response(&Err(model::SessionReadRefusal::ScopeRejected)).unwrap(),
    )
    .unwrap();
    assert_eq!(
        run(ContractMode::Check, directory).unwrap_err().to_string(),
        "read vector mismatch"
    );
    std::fs::write(
        &read,
        Logout::encode_response(&model::LogoutResult::Revoked(9_007_199_254_740_993)).unwrap(),
    )
    .unwrap();
    assert!(run(ContractMode::Check, directory).is_err());
}
