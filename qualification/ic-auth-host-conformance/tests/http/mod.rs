//! Actual SDK HTTP certificate acceptance through a single-subnet PocketIC gateway.
use super::support;

use ic_auth_host_fixture::{CONTROL, Command, Fault, REQUEST, RESPONSE, Refusal, browser};
use ic_host_process::{Input, Ownership, Policy, Stderr, run};
use std::{path::PathBuf, process::Command as Process, sync::atomic::AtomicBool, time::Duration};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[tokio::test]
#[ignore = "requires prepared Node/Chromium, build tools and PocketIC 16.x; run make qualify-client-http"]
async fn certified_http_acceptance_and_tamper_refusal() {
    let wasm = support::wasm();
    let mut owned = support::start().await;
    let pic = &owned;
    assert_eq!(pic.topology().await.subnet_configs.len(), 1);
    let canister = support::install(pic, &wasm).await;
    let root = pic.root_key().await.unwrap();
    let gateway = owned.make_live(None).await;
    let request = |command: Command| {
        ic_wire::encode(
            &command,
            ic_wire::Encoding::Frame {
                method: CONTROL,
                direction: ic_wire::Direction::Request,
            },
            REQUEST,
        )
        .unwrap()
    };
    let expected = ic_wire::encode(
        &Ok::<(), Refusal>(()),
        ic_wire::Encoding::Frame {
            method: CONTROL,
            direction: ic_wire::Direction::Response,
        },
        RESPONSE,
    )
    .unwrap();
    let mut command = Process::new("node");
    command
        .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/client"))
        .arg("tests/browser/journal.mjs")
        .env("IC_AUTH_CLIENT_SUITE", "installed-http")
        .env("AUTH_HTTP_GATEWAY", gateway.as_str())
        .env("AUTH_HTTP_ROOT", hex(&root))
        .env("AUTH_HTTP_CANISTER", canister.to_text())
        .env("AUTH_HTTP_REQUEST", hex(&request(Command::Prepare)))
        .env(
            "AUTH_HTTP_OTHER_REQUEST",
            hex(&request(Command::Complete(Fault::None))),
        )
        .env("AUTH_HTTP_EXPECTED", hex(&expected));
    let result = run_browser(&mut command);
    owned.drop().await;
    println!("{}", result.unwrap());
}

fn run_browser(command: &mut Process) -> Result<String, String> {
    let mut evidence = Vec::new();
    let report = run(
        command,
        Policy {
            // Playwright owns Chromium's separate process group and closes it in finally.
            ownership: Ownership::Direct,
            input: Input::Null,
            stdout_bytes: 64 * 1024,
            stderr: Stderr::Capture(64 * 1024),
            elapsed: Duration::from_secs(90),
        },
        &AtomicBool::new(false),
        |_stream, bytes| {
            evidence.extend_from_slice(bytes);
            Ok(())
        },
    );
    let output = String::from_utf8_lossy(&evidence).into_owned();
    if report.failure.is_none()
        && report.cleanup.is_empty()
        && report.status.is_some_and(|status| status.success())
    {
        Ok(output)
    } else {
        Err(format!(
            "browser qualification failed: {report:?}\n{output}"
        ))
    }
}

#[tokio::test]
#[ignore = "requires prepared Node/Chromium, build tools and PocketIC 16.x; run make qualify-client-http"]
async fn browser_journey_reconciles_admits_authorizes_and_revokes() {
    let wasm = support::wasm();
    let mut owned = support::start().await;
    assert_eq!(owned.topology().await.subnet_configs.len(), 1);
    // Installation lifetimes and the live gateway must start in the same clock domain.
    owned
        .set_time(ic_testkit::pocket_ic::Time::from_nanos_since_unix_epoch(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .try_into()
                .unwrap(),
        ))
        .await;
    let canister = support::install(&owned, &wasm).await;
    let root = owned.root_key().await.unwrap();
    let gateway = owned.make_live(None).await;
    let frame = |method, direction| ic_wire::Encoding::Frame { method, direction };
    let mut command = Process::new("node");
    command
        .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/client"))
        .arg("tests/browser/journal.mjs")
        .env("IC_AUTH_CLIENT_SUITE", "installed-flow")
        .env("AUTH_HTTP_GATEWAY", gateway.as_str())
        .env("AUTH_HTTP_ROOT", hex(&root))
        .env("AUTH_HTTP_CANISTER", canister.to_text())
        .env(
            "AUTH_FLOW_ISSUE",
            hex(&ic_wire::encode(
                &false,
                frame(browser::CREDENTIAL, ic_wire::Direction::Request),
                REQUEST,
            )
            .unwrap()),
        )
        .env(
            "AUTH_FLOW_LOOKUP",
            hex(&ic_wire::encode(
                &true,
                frame(browser::CREDENTIAL, ic_wire::Direction::Request),
                REQUEST,
            )
            .unwrap()),
        )
        .env(
            "AUTH_FLOW_WRITE",
            hex(&ic_wire::encode(
                &(),
                frame(browser::WRITE, ic_wire::Direction::Request),
                REQUEST,
            )
            .unwrap()),
        )
        .env(
            "AUTH_FLOW_WRITE_OK",
            hex(&ic_wire::encode(
                &Ok::<u32, ()>(1),
                frame(browser::WRITE, ic_wire::Direction::Response),
                RESPONSE,
            )
            .unwrap()),
        )
        .env(
            "AUTH_FLOW_WRITE_DENIED",
            hex(&ic_wire::encode(
                &Err::<u32, ()>(()),
                frame(browser::WRITE, ic_wire::Direction::Response),
                RESPONSE,
            )
            .unwrap()),
        )
        .env(
            "AUTH_FLOW_REVOKED",
            hex(&ic_auth_wire::EstablishSession::encode_response(&Err(
                ic_auth_wire::model::AdmissionRefusal::Revoked,
            ))
            .unwrap()),
        );
    let result = run_browser(&mut command);
    let observation = owned
        .query_call(
            canister,
            ic_testkit::pocket_ic::CanisterId::anonymous(),
            "browser_fixture_observe",
            ic_wire::encode(
                &(),
                frame(browser::OBSERVE, ic_wire::Direction::Request),
                REQUEST,
            )
            .unwrap(),
        )
        .await;
    // Close the instance before checking either browser or host assertions.
    owned.drop().await;
    println!("{}", result.unwrap());
    let observed: browser::Observation = ic_wire::decode(
        &observation.unwrap(),
        frame(browser::OBSERVE, ic_wire::Direction::Response),
        RESPONSE,
    )
    .unwrap();
    assert_eq!(
        (
            observed.signatures,
            observed.writes,
            observed.sessions,
            observed.logins,
            observed.operations
        ),
        (1, 1, 0, 1, 1)
    );
}
