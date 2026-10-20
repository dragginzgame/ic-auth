use super::*;
use ic_host_process::testing::{
    http::{HttpContentLength, HttpFixture, HttpFixtureLimits, capture_http_request},
    net::accept_tcp_until,
};
use ic_platform::agent::{AgentError, AnonymousIdentity, DelegatedIdentity};
use std::{io::Read as _, net::TcpListener, time::Instant};

async fn ingress(
    url: &str,
    root: &[u8],
    identity: impl Identity + 'static,
    target: Principal,
) -> Result<Result<Principal, String>, AgentError> {
    // Reuse the caller runtime, but bind a fresh Agent to this explicit identity
    // and root on every call. Identity changes must not reuse old authority.
    let agent = Agent::builder()
        .with_url(url)
        .with_identity(identity)
        .with_ingress_expiry(Duration::from_secs(30))
        .build()
        .unwrap();
    agent.set_root_key(root.to_vec());
    let bytes = agent
        .update(&target.into(), "owner_checked_operation")
        .with_arg(
            ic_wire::encode_request::<controls::OwnerOperation>(&())
                .unwrap()
                .into_bytes(),
        )
        .call_and_wait()
        .await?;
    Ok(ic_wire::decode_response::<controls::OwnerOperation>(&bytes)
        .unwrap()
        .map(|p| Principal::try_from_slice(p.as_slice()).unwrap())
        .map_err(|e| e.into_string()))
}

fn assert_replica_rejection(
    result: Result<Result<Principal, String>, AgentError>,
    expected_reason: &str,
) {
    let error = result.expect_err("invalid delegation must not reach the application");
    assert!(
        matches!(&error, AgentError::HttpError(response)
            if matches!(response.status, 400 | 403)
                && String::from_utf8_lossy(&response.content).to_lowercase().contains(expected_reason)),
        "expected replica authentication refusal, received {error:?}"
    );
}

#[test]
fn ingress_protocol_failure_remains_distinct_from_application_rejection() {
    let fixture = HttpFixture::spawn(
        b"\xff",
        "application/cbor",
        HttpFixtureLimits {
            timeout: Duration::from_secs(10),
            max_header_bytes: 65_536,
            max_body_bytes: 65_536,
        },
    )
    .unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let result = runtime.block_on(async {
        tokio::time::timeout(
            Duration::from_secs(10),
            ingress(
                fixture.endpoint(),
                &[],
                AnonymousIdentity,
                Principal::anonymous(),
            ),
        )
        .await
        .expect("malformed response must fail within the caller's deadline")
    });
    assert!(matches!(result, Err(AgentError::InvalidCborData(_))));
    let request = fixture.finish().unwrap();
    assert!(request.starts_with(b"POST "));
    assert!(
        request
            .windows(b"owner_checked_operation".len())
            .any(|bytes| bytes == b"owner_checked_operation")
    );
}

#[test]
fn ingress_deadline_and_abort_release_pending_transport() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    for use_deadline in [true, false] {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let (sent, received) = tokio::sync::oneshot::channel();
        let server = std::thread::spawn(move || {
            let (mut stream, _) =
                accept_tcp_until(&listener, Instant::now() + Duration::from_secs(10)).unwrap();
            let request = capture_http_request(
                &mut stream,
                Duration::from_secs(10),
                65_536,
                65_536,
                HttpContentLength::Required,
            )
            .into_result()
            .unwrap();
            // The complete update reached the server. Withhold its response so
            // cancellation is tested after dispatch, not against an idle future.
            sent.send(()).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(10)))
                .unwrap();
            assert_eq!(stream.read(&mut [0]).unwrap(), 0, "client must disconnect");
            assert_eq!(
                listener.accept().unwrap_err().kind(),
                std::io::ErrorKind::WouldBlock
            );
            request
        });
        let mut request = runtime.spawn(async move {
            ingress(&url, &[], AnonymousIdentity, Principal::anonymous()).await
        });
        runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(10), received)
                .await
                .expect("Agent must send its request")
                .unwrap();
            if use_deadline {
                assert!(
                    tokio::time::timeout(Duration::from_millis(100), &mut request)
                        .await
                        .is_err()
                );
            }
            // Timing out a JoinHandle does not cancel its task. Explicitly abort
            // and await it so there is no detached ingress work on this runtime.
            request.abort();
            assert!(
                tokio::time::timeout(Duration::from_secs(10), request)
                    .await
                    .unwrap()
                    .unwrap_err()
                    .is_cancelled()
            );
        });
        let dispatched = server.join().unwrap();
        assert!(dispatched.starts_with(b"POST "));
        assert!(
            dispatched
                .windows(b"owner_checked_operation".len())
                .any(|bytes| bytes == b"owner_checked_operation")
        );
        // This proves local transport release only. A server that accepted an
        // update may still execute it; cancellation never authorizes resubmission.
    }
}

#[test]
fn replica_checks_real_ingress_delegations_and_fresh_browser_keys_keep_identity_after_upgrade() {
    let ingress_runtime = tokio::runtime::Runtime::new().unwrap();
    let mut fixture = Fixture::new();
    let url = fixture.pic.make_live(None).to_string();
    let network = fixture.pic.root_key().unwrap();
    let mut wrong_root = network.clone();
    *wrong_root.last_mut().unwrap() ^= 1;
    assert!(matches!(
        ingress_runtime.block_on(ingress(
            &url,
            &wrong_root,
            BasicIdentity::from_raw_key(&[6; 32]),
            fixture.resource,
        )),
        Err(AgentError::CertificateVerificationFailed())
    ));
    // Trust refusal does not prove that an update was never executed. The fixture
    // operation is read-only; a fresh Agent with the real root must still work.
    // Valid ingress authentication still grants no resource ownership. Keep
    // application denial separate from transport/replica signature rejection.
    assert!(matches!(
        ingress_runtime.block_on(ingress(
            &url,
            &network,
            BasicIdentity::from_raw_key(&[6; 32]),
            fixture.resource
        )),
        Ok(Err(_))
    ));
    assert!(matches!(
        ingress_runtime.block_on(ingress(&url, &network, AnonymousIdentity, fixture.resource)),
        Ok(Err(_))
    ));
    let mut previous: Option<(Vec<u8>, SignedDelegation)> = None;
    for key in [7, 8] {
        if key == 8 {
            fixture.upgrade();
            // Clearing pending signature leaves is not revocation of an already
            // issued ingress delegation. The receiver's stored owner survives.
            let (public_key, delegation) = previous.as_ref().unwrap();
            let old = DelegatedIdentity::new_with_root_key(
                public_key.clone(),
                Box::new(BasicIdentity::from_raw_key(&[7; 32])),
                vec![delegation.clone()],
                &network,
            )
            .unwrap();
            assert_eq!(
                ingress_runtime.block_on(ingress(&url, &network, old, fixture.resource)),
                Ok(Ok(fixture.owner))
            );
        }
        let (public_key, chain) =
            fixture.delegation(key, fixture.now() + 120_000_000_000, vec![fixture.resource]);
        let identity = DelegatedIdentity::new_with_root_key(
            public_key.clone(),
            Box::new(BasicIdentity::from_raw_key(&[key; 32])),
            vec![chain.clone()],
            &network,
        )
        .unwrap();
        assert_eq!(
            ingress_runtime.block_on(ingress(&url, &network, identity, fixture.resource)),
            Ok(Ok(fixture.owner))
        );
        // Unchecked identities intentionally bypass the client's early validation
        // only in these negative tests, so the real replica must reject the bytes.
        let identity = DelegatedIdentity::new_unchecked(
            public_key.clone(),
            Box::new(BasicIdentity::from_raw_key(&[key; 32])),
            vec![chain.clone()],
        );
        assert_replica_rejection(
            ingress_runtime.block_on(ingress(&url, &network, identity, fixture.signer)),
            "not one of the delegation targets",
        );
        let mut changed = chain.clone();
        changed.delegation.expiration += 1;
        assert!(
            DelegatedIdentity::new_with_root_key(
                public_key.clone(),
                Box::new(BasicIdentity::from_raw_key(&[key; 32])),
                vec![changed.clone()],
                &network
            )
            .is_err()
        );
        let identity = DelegatedIdentity::new_unchecked(
            public_key.clone(),
            Box::new(BasicIdentity::from_raw_key(&[key; 32])),
            vec![changed],
        );
        assert_replica_rejection(
            ingress_runtime.block_on(ingress(&url, &network, identity, fixture.resource)),
            "signature",
        );
        let identity = DelegatedIdentity::new_unchecked(
            public_key.clone(),
            Box::new(BasicIdentity::from_raw_key(&[key + 1; 32])),
            vec![chain.clone()],
        );
        assert_replica_rejection(
            ingress_runtime.block_on(ingress(&url, &network, identity, fixture.resource)),
            "signature",
        );
        previous = Some((public_key, chain));
    }
    let (public_key, expired) = fixture.delegation(9, fixture.now() - 1, vec![fixture.resource]);
    let identity = DelegatedIdentity::new_with_root_key(
        public_key,
        Box::new(BasicIdentity::from_raw_key(&[9; 32])),
        vec![expired],
        &network,
    )
    .unwrap();
    assert_replica_rejection(
        ingress_runtime.block_on(ingress(&url, &network, identity, fixture.resource)),
        "expired",
    );
}
