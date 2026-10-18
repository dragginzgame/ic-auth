use ic_auth::canister_signature::{
    CanisterSignatureError, CanisterSignaturePolicy, domain_separated_message,
    verify_canister_signature,
};
use ic_auth_protocol_types::IcCanisterSignatureProofV1;
use ic_auth_qualification_contracts as controls;
use ic_host_fs::read::read_file_no_follow;
use ic_platform::canister_sig_creation::{
    CanisterSigPublicKey, extract_raw_root_pk_from_der, hash_bytes,
};
use ic_platform::{
    Principal,
    agent::{
        Agent, AgentError, AnonymousIdentity, BasicIdentity, DelegatedIdentity, Delegation,
        Identity, SignedDelegation,
    },
};
use ic_testkit::{
    pic::{PocketIc, PocketIcBuilder, PocketIcManagedServer, PocketIcStartupConfig},
    pocket_ic::Time,
};
use ic_wire::{Bytes, Method};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, SystemTime},
};

const SEED: &[u8] = b"qualification-identity";
const DOMAIN: &[u8] = b"canic-issuer-delegated-token";
const INGRESS_DOMAIN: &[u8] = b"ic-request-auth-delegation";
static NEXT_INSTANCE: AtomicUsize = AtomicUsize::new(0);

#[path = "signatures/certified_proofs.rs"]
mod certified_proofs;

#[path = "signatures/profile_composition.rs"]
mod profile_composition;

#[path = "signatures/sessions.rs"]
mod sessions;

struct Fixture {
    pic: PocketIc,
    // Drop the instance before terminating its exact owned server.
    _server: Option<PocketIcManagedServer>,
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
        let builder = PocketIcBuilder::new()
            .with_state_dir(state_path)
            .with_nns_subnet()
            .with_application_subnet()
            .with_max_request_time_ms(Some(30_000));
        let owned = PocketIcStartupConfig::Spawn {
            server_binary: server,
            timeout: Duration::from_secs(30),
            server_hard_ttl: None,
            server_idle_ttl: None,
            server_output_files: Some((
                state_root.join(format!("{instance}.stdout.log")),
                state_root.join(format!("{instance}.stderr.log")),
            )),
        }
        .build_instance(builder)
        .expect("bounded testkit startup with the verified server");
        // Retain Testkit's field drop order during setup, including panic paths.
        let pic = &owned.instance;
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
                ic_wire::encode_request::<controls::Initialize>(
                    &Bytes::new(owner.as_slice().to_vec()).unwrap(),
                )
                .unwrap()
                .into_bytes(),
                Some(controller),
            );
        }
        Self {
            pic: owned.instance,
            _server: owned.server,
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
        self.prepare_with_seed(SEED, domain, message, ttl)
    }

    fn prepare_with_seed(
        &self,
        seed: &[u8],
        domain: &[u8],
        message: &[u8],
        ttl: u64,
    ) -> Result<u64, String> {
        update_control::<controls::Prepare, _>(
            &self.pic,
            self.signer,
            self.controller,
            controls::PrepareInput {
                input: controls::SignatureInput {
                    seed: Bytes::new(seed.to_vec()).unwrap(),
                    domain: Bytes::new(domain.to_vec()).unwrap(),
                    message: Bytes::new(message.to_vec()).unwrap(),
                },
                retention_ns: ttl,
            },
        )
    }

    fn retrieve(
        &self,
        domain: &[u8],
        message: &[u8],
    ) -> Result<IcCanisterSignatureProofV1, String> {
        self.retrieve_with_seed(SEED, domain, message)
    }

    fn retrieve_with_seed(
        &self,
        seed: &[u8],
        domain: &[u8],
        message: &[u8],
    ) -> Result<IcCanisterSignatureProofV1, String> {
        // Certificates become available after the update's state is certified.
        self.pic.tick();
        let proof: Result<ic_auth_wire::model::IcCanisterSignatureProofV1, String> =
            query_control::<controls::Retrieve, _>(
                &self.pic,
                self.signer,
                Principal::anonymous(),
                controls::SignatureInput {
                    seed: Bytes::new(seed.to_vec()).unwrap(),
                    domain: Bytes::new(domain.to_vec()).unwrap(),
                    message: Bytes::new(message.to_vec()).unwrap(),
                },
            );
        proof.map(|value| value.try_into().unwrap())
    }

    fn verify(&self, domain: &[u8], payload: [u8; 32], proof: &IcCanisterSignatureProofV1) {
        assert_eq!(self.verify_result(SEED, domain, payload, proof), Ok(()));
        assert_eq!(
            self.verify_result(b"unapproved seed", domain, payload, proof),
            Err(CanisterSignatureError::SeedMismatch)
        );
    }

    fn verify_result(
        &self,
        seed: &[u8],
        domain: &[u8],
        payload: [u8; 32],
        proof: &IcCanisterSignatureProofV1,
    ) -> Result<(), CanisterSignatureError> {
        let network = extract_raw_root_pk_from_der(&self.pic.root_key().unwrap()).unwrap();
        let policy = CanisterSignaturePolicy {
            signing_canister: self.signer,
            seed_hash: hash_bytes(seed),
            ic_root_public_key_raw: &network,
            now_ns: self.now(),
            max_certificate_age_ns: 5_000_000_000,
            max_future_skew_ns: 0,
            max_signature_bytes: 128 * 1024,
            max_message_bytes: 1024,
        };
        verify_canister_signature(&domain_separated_message(domain, payload)?, proof, &policy)
    }

    fn set_asset(&self, version: u64) {
        let result: Result<(), String> = update_control::<controls::SetAsset, _>(
            &self.pic,
            self.signer,
            self.controller,
            version,
        );
        result.unwrap();
    }

    fn asset_version(&self) -> u64 {
        query_wire::<controls::AssetVersion>(&self.pic, self.signer, Principal::anonymous(), ())
    }

    fn upgrade(&self) {
        for canister in [self.signer, self.resource] {
            self.pic
                .upgrade_canister(
                    canister,
                    self.wasm.clone(),
                    ic_wire::encode_request::<controls::Upgrade>(&())
                        .unwrap()
                        .into_bytes(),
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
    // The retired Candid transport, another method's envelope and an oversized
    // request must fail before reaching the controller-installed mutation.
    for bytes in [
        b"DIDL\x00\x00".to_vec(),
        ic_wire::encode_request::<controls::Prune>(&128)
            .unwrap()
            .into_bytes(),
        vec![0; controls::CONTROL_LIMITS.bytes + 1],
    ] {
        assert!(
            fixture
                .pic
                .update_call(
                    fixture.signer,
                    fixture.controller,
                    controls::SetAsset::NAME,
                    bytes
                )
                .is_err()
        );
        assert_eq!(fixture.asset_version(), 1);
    }
    let unauthorized: Result<u64, String> = update_control::<controls::Prepare, _>(
        &fixture.pic,
        fixture.signer,
        fixture.owner,
        controls::PrepareInput {
            input: controls::SignatureInput {
                seed: Bytes::new(SEED.to_vec()).unwrap(),
                domain: Bytes::new(DOMAIN.to_vec()).unwrap(),
                message: Bytes::new(payload.to_vec()).unwrap(),
            },
            retention_ns: 60_000_000_000_u64,
        },
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
    assert!(
        fixture
            .pic
            .upgrade_canister(
                fixture.signer,
                fixture.wasm.clone(),
                ic_wire::encode_request::<controls::AssetVersion>(&())
                    .unwrap()
                    .into_bytes(),
                Some(fixture.controller)
            )
            .is_err(),
        "wrong upgrade envelope must roll back"
    );
    assert_eq!(fixture.asset_version(), 9);
    fixture.upgrade();
    assert_eq!(fixture.asset_version(), 9);
    assert!(fixture.retrieve(DOMAIN, &payload).is_err());
    fixture.prepare(DOMAIN, &payload, 10).unwrap();
    fixture
        .pic
        .set_time(Time::from_nanos_since_unix_epoch(fixture.now() + 10));
    assert!(fixture.retrieve(DOMAIN, &payload).is_err());
    let removed: Result<u32, String> = update_control::<controls::Prune, _>(
        &fixture.pic,
        fixture.signer,
        fixture.controller,
        128_u32,
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
        ingress(&url, &network, AnonymousIdentity, fixture.resource),
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

fn update_wire<M: Method>(
    pic: &PocketIc,
    target: Principal,
    caller: Principal,
    request: &M::Request,
) -> M::Response {
    let arg = ic_wire::encode_request::<M>(request).expect("bounded Wire request");
    let reply = pic
        .update_call(target, caller, M::NAME, arg.into_bytes())
        .expect("qualified Wire update");
    ic_wire::decode_response::<M>(&reply).expect("bounded Wire reply")
}
fn query_wire<M: Method>(
    pic: &PocketIc,
    target: Principal,
    caller: Principal,
    request: M::Request,
) -> M::Response {
    let arg = ic_wire::encode_request::<M>(&request).expect("bounded Wire request");
    let reply = pic
        .query_call(target, caller, M::NAME, arg.into_bytes())
        .expect("qualified Wire query");
    ic_wire::decode_response::<M>(&reply).expect("bounded Wire reply")
}
fn update_control<M, T>(
    pic: &PocketIc,
    target: Principal,
    caller: Principal,
    request: M::Request,
) -> Result<T, String>
where
    M: Method<Response = Result<T, controls::Refusal>>,
{
    update_wire::<M>(pic, target, caller, &request).map_err(|e| e.into_string())
}
fn query_control<M, T>(
    pic: &PocketIc,
    target: Principal,
    caller: Principal,
    request: M::Request,
) -> Result<T, String>
where
    M: Method<Response = Result<T, controls::Refusal>>,
{
    query_wire::<M>(pic, target, caller, request).map_err(|e| e.into_string())
}
