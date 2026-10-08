use candid::{Principal, decode_one, encode_args, encode_one};
use ic_agent::{
    Agent, AgentError, Identity,
    identity::{BasicIdentity, DelegatedIdentity, Delegation, SignedDelegation},
};
use ic_auth::canister_signature::{
    CanisterSignaturePolicy, domain_separated_message, verify_canister_signature,
};
use ic_auth_protocol_types::IcCanisterSignatureProofV1;
use ic_canister_sig_creation::{CanisterSigPublicKey, extract_raw_root_pk_from_der, hash_bytes};
use ic_testkit::{
    ic_host_fs::read::read_file_no_follow,
    pic::{CandidCallExt, PocketIc, PocketIcBuilder, PocketIcBuilderExt, PocketIcStartupConfig},
    pocket_ic::Time,
};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, SystemTime},
};

const SEED: &[u8] = b"qualification-identity";
const DOMAIN: &[u8] = b"canic-issuer-delegated-token";
const INGRESS_DOMAIN: &[u8] = b"ic-request-auth-delegation";
static NEXT_INSTANCE: AtomicUsize = AtomicUsize::new(0);

struct Fixture {
    pic: PocketIc,
    signer: Principal,
    resource: Principal,
    controller: Principal,
    owner: Principal,
    wasm: Vec<u8>,
}

impl Fixture {
    fn new() -> Self {
        let server = PathBuf::from(
            std::env::var_os("POCKET_IC_BIN")
                .expect("make test-qualification supplies verified server"),
        );
        let wasm_path = std::env::var_os("IC_AUTH_QUALIFICATION_WASM")
            .expect("make test-qualification builds the fixture Wasm");
        let wasm = read_file_no_follow(Path::new(&wasm_path), 16 * 1024 * 1024).unwrap();
        let state_root = PathBuf::from(
            std::env::var_os("IC_AUTH_QUALIFICATION_STATE_ROOT")
                .expect("make test-qualification supplies a retained state root"),
        );
        let instance = format!("instance-{}", NEXT_INSTANCE.fetch_add(1, Ordering::Relaxed));
        let state_path = state_root.join(&instance);
        let pic = PocketIcBuilder::new()
            .with_state_dir(state_path)
            .with_nns_subnet()
            .with_application_subnet()
            .with_max_request_time_ms(Some(30_000))
            .try_build(
                PocketIcStartupConfig::spawn(server, Duration::from_secs(30))
                    .with_server_output_files(
                        state_root.join(format!("{instance}.stdout.log")),
                        state_root.join(format!("{instance}.stderr.log")),
                    ),
            )
            .expect("bounded testkit startup with the verified server");
        pic.set_time(SystemTime::now().into());
        let controller = Principal::self_authenticating(b"qualification controller");
        let signer = pic.create_canister_with_settings(Some(controller), None);
        let resource = pic.create_canister_with_settings(Some(controller), None);
        let owner = Principal::self_authenticating(
            CanisterSigPublicKey::new(signer, SEED.to_vec()).to_der(),
        );
        for canister in [signer, resource] {
            pic.add_cycles(canister, 2_000_000_000_000);
            pic.install_canister(
                canister,
                wasm.clone(),
                encode_one(owner).unwrap(),
                Some(controller),
            );
        }
        Self {
            pic,
            signer,
            resource,
            controller,
            owner,
            wasm,
        }
    }

    fn now(&self) -> u64 {
        self.pic.get_time().as_nanos_since_unix_epoch()
    }

    fn prepare(&self, domain: &[u8], message: &[u8], ttl: u64) -> Result<u64, String> {
        self.pic.update_candid_as_or_panic(
            self.signer,
            self.controller,
            "prepare",
            (domain.to_vec(), message.to_vec(), ttl),
        )
    }

    fn retrieve(
        &self,
        domain: &[u8],
        message: &[u8],
    ) -> Result<IcCanisterSignatureProofV1, String> {
        // Certificates become available after the update's state is certified.
        self.pic.tick();
        self.pic
            .query_candid_or_panic(self.signer, "retrieve", (domain.to_vec(), message.to_vec()))
    }

    fn verify(&self, domain: &[u8], payload: [u8; 32], proof: &IcCanisterSignatureProofV1) {
        let network = extract_raw_root_pk_from_der(&self.pic.root_key().unwrap()).unwrap();
        let policy = CanisterSignaturePolicy {
            signing_canister: self.signer,
            seed_hash: hash_bytes(SEED),
            ic_root_public_key_raw: &network,
            now_ns: self.now(),
            max_certificate_age_ns: 5_000_000_000,
            max_future_skew_ns: 0,
            max_signature_bytes: 128 * 1024,
            max_message_bytes: 1024,
        };
        assert_eq!(
            verify_canister_signature(
                &domain_separated_message(domain, payload).unwrap(),
                proof,
                &policy
            ),
            Ok(())
        );
        let mut wrong = policy.clone();
        wrong.seed_hash = hash_bytes(b"unapproved seed");
        assert!(
            verify_canister_signature(
                &domain_separated_message(domain, payload).unwrap(),
                proof,
                &wrong
            )
            .is_err()
        );
    }

    fn set_asset(&self, version: u64) {
        let result: Result<(), String> = self.pic.update_candid_as_or_panic(
            self.signer,
            self.controller,
            "set_asset",
            (version,),
        );
        result.unwrap();
    }

    fn asset_version(&self) -> u64 {
        self.pic
            .query_candid_or_panic(self.signer, "asset_version", ())
    }

    fn upgrade(&self) {
        for canister in [self.signer, self.resource] {
            self.pic
                .upgrade_canister(
                    canister,
                    self.wasm.clone(),
                    encode_args(()).unwrap(),
                    Some(self.controller),
                )
                .unwrap();
        }
    }

    fn delegation(
        &self,
        key: u8,
        expiration: u64,
        targets: Vec<Principal>,
    ) -> (Vec<u8>, SignedDelegation) {
        let delegation = Delegation {
            pubkey: BasicIdentity::from_raw_key(&[key; 32])
                .public_key()
                .unwrap(),
            expiration,
            targets: Some(targets),
            permissions: None,
        };
        // The IC agent owns request-ID encoding. Only split its standard domain
        // prefix to pass the raw payload hash into the signature store.
        let signable = delegation.signable();
        let payload: [u8; 32] = signable
            .strip_prefix(b"\x1Aic-request-auth-delegation")
            .unwrap()
            .try_into()
            .unwrap();
        self.prepare(INGRESS_DOMAIN, &payload, 60_000_000_000)
            .unwrap();
        let proof = self.retrieve(INGRESS_DOMAIN, &payload).unwrap();
        self.verify(INGRESS_DOMAIN, payload, &proof);
        (
            proof.public_key_der,
            SignedDelegation {
                delegation,
                signature: proof.signature_cbor,
            },
        )
    }
}

#[test]
fn real_certificates_composition_cleanup_and_upgrade_obey_the_host_contract() {
    let fixture = Fixture::new();
    let payload = [7; 32];
    let unauthorized: Result<u64, String> = fixture.pic.update_candid_as_or_panic(
        fixture.signer,
        fixture.owner,
        "prepare",
        (DOMAIN.to_vec(), payload.to_vec(), 60_000_000_000_u64),
    );
    assert!(unauthorized.is_err());
    let deadline = fixture.prepare(DOMAIN, &payload, 60_000_000_000).unwrap();
    assert_eq!(
        fixture.prepare(DOMAIN, &payload, 60_000_000_000),
        Ok(deadline)
    );
    let proof = fixture.retrieve(DOMAIN, &payload).unwrap();
    fixture.verify(DOMAIN, payload, &proof);
    fixture.set_asset(9);
    assert_eq!(fixture.asset_version(), 9);
    fixture.verify(
        DOMAIN,
        payload,
        &fixture.retrieve(DOMAIN, &payload).unwrap(),
    );
    fixture.upgrade();
    assert_eq!(fixture.asset_version(), 9);
    assert!(fixture.retrieve(DOMAIN, &payload).is_err());
    fixture.prepare(DOMAIN, &payload, 10).unwrap();
    fixture
        .pic
        .set_time(Time::from_nanos_since_unix_epoch(fixture.now() + 10));
    assert!(fixture.retrieve(DOMAIN, &payload).is_err());
    let removed: Result<u32, String> = fixture.pic.update_candid_as_or_panic(
        fixture.signer,
        fixture.controller,
        "prune",
        (128_u32,),
    );
    assert_eq!(removed, Ok(1));
    fixture.prepare(DOMAIN, &payload, 60_000_000_000).unwrap();
    let restored = fixture.retrieve(DOMAIN, &payload).unwrap();
    assert_eq!(proof.public_key_der, restored.public_key_der);
    fixture.verify(DOMAIN, payload, &restored);
    assert_eq!(fixture.asset_version(), 9);
}

fn ingress(
    url: &str,
    root: &[u8],
    identity: impl Identity + 'static,
    target: Principal,
) -> Result<Result<Principal, String>, AgentError> {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let agent = Agent::builder()
            .with_url(url)
            .with_identity(identity)
            .with_ingress_expiry(Duration::from_secs(30))
            .build()
            .unwrap();
        agent.set_root_key(root.to_vec());
        let bytes = agent
            .update(&target, "owner_checked_operation")
            .with_arg(encode_args(()).unwrap())
            .call_and_wait()
            .await?;
        Ok(decode_one::<Result<Principal, String>>(&bytes).unwrap())
    })
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
fn replica_checks_real_ingress_delegations_and_fresh_browser_keys_keep_identity_after_upgrade() {
    let mut fixture = Fixture::new();
    let url = fixture.pic.make_live(None).to_string();
    let network = fixture.pic.root_key().unwrap();
    // Valid ingress authentication still grants no resource ownership. Keep
    // application denial separate from transport/replica signature rejection.
    assert!(matches!(
        ingress(
            &url,
            &network,
            BasicIdentity::from_raw_key(&[6; 32]),
            fixture.resource
        ),
        Ok(Err(_))
    ));
    assert!(matches!(
        ingress(
            &url,
            &network,
            ic_agent::identity::AnonymousIdentity,
            fixture.resource
        ),
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
                ingress(&url, &network, old, fixture.resource),
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
            ingress(&url, &network, identity, fixture.resource),
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
            ingress(&url, &network, identity, fixture.signer),
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
            ingress(&url, &network, identity, fixture.resource),
            "signature",
        );
        let identity = DelegatedIdentity::new_unchecked(
            public_key.clone(),
            Box::new(BasicIdentity::from_raw_key(&[key + 1; 32])),
            vec![chain.clone()],
        );
        assert_replica_rejection(
            ingress(&url, &network, identity, fixture.resource),
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
        ingress(&url, &network, identity, fixture.resource),
        "expired",
    );
}
