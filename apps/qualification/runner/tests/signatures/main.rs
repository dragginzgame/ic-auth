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
    agent::{Agent, BasicIdentity, Delegation, Identity, SignedDelegation},
};
use ic_testkit::pic::{
    PocketIcInstance, PocketIcStartupConfig, WireCallError, WireCallSpec, query_wire, update_wire,
};
use ic_testkit::pocket_ic::{PocketIc, PocketIcBuilder, Time};
use ic_wire::Bytes;
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, SystemTime},
};

// Select the upstream transport spec from the canonical contract constants.
macro_rules! spec {
    ($method:path) => {
        ic_testkit::pic::WireCallSpec {
            method: <$method>::NAME,
            method_id: <$method>::ID,
            request_limits: <$method>::REQUEST_LIMITS,
            response_limits: <$method>::RESPONSE_LIMITS,
        }
    };
}

const SEED: &[u8] = b"qualification-identity";
const DOMAIN: &[u8] = b"canic-issuer-delegated-token";
const INGRESS_DOMAIN: &[u8] = b"ic-request-auth-delegation";
static NEXT_INSTANCE: AtomicUsize = AtomicUsize::new(0);

mod certified_proofs;

mod ingress;

mod profile_composition;

mod sessions;

struct Fixture {
    pic: PocketIcInstance<PocketIc>,
    signer: Principal,
    resource: Principal,
    controller: Principal,
    owner: Principal,
    wasm: Vec<u8>,
}

impl Fixture {
    async fn new() -> Self {
        Self::with_application_subnets(1).await
    }

    async fn with_application_subnets(count: usize) -> Self {
        assert!(matches!(count, 1 | 2));
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
        let mut builder = PocketIcBuilder::new(Duration::from_secs(30))
            .with_state_dir(state_path)
            .with_nns_subnet()
            .with_application_subnet();
        if count == 2 {
            builder = builder.with_application_subnet();
        }
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
        .await
        .expect("bounded testkit startup with the verified server");
        // Keep the managed server alive through setup and explicit async close.
        // On panic, its Drop still stops/reaps the process and all its instances.
        let pic = owned.instance();
        pic.set_time(SystemTime::now().into()).await.unwrap();
        let controller = Principal::self_authenticating(b"qualification controller");
        let subnets = pic.topology().await.unwrap().get_app_subnets();
        assert_eq!(subnets.len(), count);
        let signer = pic
            .create_canister_on_subnet(Some(controller.into()), None, subnets[0])
            .await
            .unwrap();
        let resource = pic
            .create_canister_on_subnet(Some(controller.into()), None, subnets[count - 1])
            .await
            .unwrap();
        assert_eq!(pic.get_subnet(signer).await.unwrap(), Some(subnets[0]));
        assert_eq!(
            pic.get_subnet(resource).await.unwrap(),
            Some(subnets[count - 1])
        );
        let owner = Principal::self_authenticating(
            CanisterSigPublicKey::new(signer.into(), SEED.to_vec()).to_der(),
        );
        for canister in [signer, resource] {
            pic.add_cycles(canister, 2_000_000_000_000).await.unwrap();
            pic.install_canister(
                canister,
                wasm.clone(),
                controls::Initialize::encode_request(
                    &Bytes::new(owner.as_slice().to_vec()).unwrap(),
                )
                .unwrap(),
                Some(controller.into()),
            )
            .await
            .unwrap();
        }
        Self {
            pic: owned,
            signer: signer.into(),
            resource: resource.into(),
            controller,
            owner,
            wasm,
        }
    }

    async fn now(&self) -> u64 {
        self.pic
            .instance()
            .get_time()
            .await
            .unwrap()
            .as_nanos_since_unix_epoch()
    }

    async fn root_key(&self) -> Vec<u8> {
        self.pic
            .instance()
            .root_key()
            .await
            .expect("PocketIC root key response")
            .expect("fixture NNS root key")
    }

    async fn prepare(
        &self,
        domain: &[u8],
        message: &[u8],
        ttl: u64,
    ) -> Result<u64, controls::Refusal> {
        self.prepare_with_seed(SEED, domain, message, ttl).await
    }

    async fn prepare_with_seed(
        &self,
        seed: &[u8],
        domain: &[u8],
        message: &[u8],
        ttl: u64,
    ) -> Result<u64, controls::Refusal> {
        update_control::<controls::PrepareInput, u64>(
            self.pic.instance(),
            self.signer,
            self.controller,
            spec!(controls::Prepare),
            controls::PrepareInput {
                input: controls::SignatureInput {
                    seed: Bytes::new(seed.to_vec()).unwrap(),
                    domain: Bytes::new(domain.to_vec()).unwrap(),
                    message: Bytes::new(message.to_vec()).unwrap(),
                },
                retention_ns: ttl,
            },
        )
        .await
    }

    async fn retrieve(
        &self,
        domain: &[u8],
        message: &[u8],
    ) -> Result<IcCanisterSignatureProofV1, controls::Refusal> {
        self.retrieve_with_seed(SEED, domain, message).await
    }

    async fn retrieve_with_seed(
        &self,
        seed: &[u8],
        domain: &[u8],
        message: &[u8],
    ) -> Result<IcCanisterSignatureProofV1, controls::Refusal> {
        // Certificates become available after the update's state is certified.
        self.pic.instance().tick().await.unwrap();
        let proof: Result<ic_auth_wire::model::IcCanisterSignatureProofV1, controls::Refusal> =
            query_control::<
                controls::SignatureInput,
                ic_auth_wire::model::IcCanisterSignatureProofV1,
            >(
                self.pic.instance(),
                self.signer,
                Principal::anonymous(),
                spec!(controls::Retrieve),
                controls::SignatureInput {
                    seed: Bytes::new(seed.to_vec()).unwrap(),
                    domain: Bytes::new(domain.to_vec()).unwrap(),
                    message: Bytes::new(message.to_vec()).unwrap(),
                },
            )
            .await;
        proof.map(|value| value.try_into().unwrap())
    }

    async fn verify(&self, domain: &[u8], payload: [u8; 32], proof: &IcCanisterSignatureProofV1) {
        assert_eq!(
            self.verify_result(SEED, domain, payload, proof).await,
            Ok(())
        );
        assert_eq!(
            self.verify_result(b"unapproved seed", domain, payload, proof)
                .await,
            Err(CanisterSignatureError::SeedMismatch)
        );
    }

    async fn verify_result(
        &self,
        seed: &[u8],
        domain: &[u8],
        payload: [u8; 32],
        proof: &IcCanisterSignatureProofV1,
    ) -> Result<(), CanisterSignatureError> {
        let network = extract_raw_root_pk_from_der(&self.root_key().await).unwrap();
        let policy = CanisterSignaturePolicy {
            signing_canister: self.signer,
            seed_hash: hash_bytes(seed),
            ic_root_public_key_raw: &network,
            now_ns: self.now().await,
            max_certificate_age_ns: 5_000_000_000,
            max_future_skew_ns: 0,
            max_signature_bytes: 128 * 1024,
            max_message_bytes: 1024,
        };
        verify_canister_signature(&domain_separated_message(domain, payload)?, proof, &policy)
    }

    async fn set_asset(&self, version: u64) {
        let result: Result<(), controls::Refusal> = update_control::<u64, ()>(
            self.pic.instance(),
            self.signer,
            self.controller,
            spec!(controls::SetAsset),
            version,
        )
        .await;
        result.unwrap();
    }

    async fn asset_version(&self) -> u64 {
        query_wire::<(), u64>(
            self.pic.instance(),
            self.signer.into(),
            Principal::anonymous().into(),
            spec!(controls::AssetVersion),
            &(),
        )
        .await
        .expect("qualified Wire query")
    }

    async fn upgrade(&self) {
        for canister in [self.signer, self.resource] {
            self.pic
                .instance()
                .upgrade_canister(
                    canister.into(),
                    self.wasm.clone(),
                    controls::Upgrade::encode_request(&()).unwrap(),
                    Some(self.controller.into()),
                )
                .await
                .unwrap();
        }
    }

    async fn delegation(
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
            targets: Some(targets.into_iter().map(Into::into).collect()),
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
            .await
            .unwrap();
        let proof = self.retrieve(INGRESS_DOMAIN, &payload).await.unwrap();
        self.verify(INGRESS_DOMAIN, payload, &proof).await;
        (
            proof.public_key_der,
            SignedDelegation {
                delegation,
                signature: proof.signature_cbor,
            },
        )
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_certificates_composition_cleanup_and_upgrade_obey_the_host_contract() {
    let fixture = Fixture::new().await;
    let payload = [7; 32];
    // The retired Candid transport, another method's envelope and an oversized
    // request must fail before reaching the controller-installed mutation.
    for bytes in [
        b"DIDL\x00\x00".to_vec(),
        controls::Prune::encode_request(&128).unwrap(),
        vec![0; controls::CONTROL_LIMITS.bytes + 1],
    ] {
        assert!(
            fixture
                .pic
                .instance()
                .update_call(
                    fixture.signer.into(),
                    fixture.controller.into(),
                    controls::SetAsset::NAME,
                    bytes
                )
                .await
                .is_err()
        );
        assert_eq!(fixture.asset_version().await, 1);
    }
    // A query cannot dispatch the controller mutation. Keep transport refusal
    // distinct from an encoded application refusal returned by a valid update.
    let refused = query_wire::<u64, Result<(), controls::Refusal>>(
        fixture.pic.instance(),
        fixture.signer.into(),
        fixture.controller.into(),
        spec!(controls::SetAsset),
        &99,
    )
    .await;
    assert!(matches!(refused, Err(WireCallError::Call { context, .. })
        if Principal::from(context.canister_id) == fixture.signer
            && Principal::from(context.caller) == fixture.controller
            && context.method == controls::SetAsset::NAME));
    assert_eq!(fixture.asset_version().await, 1);
    let unauthorized = update_wire::<controls::PrepareInput, Result<u64, controls::Refusal>>(
        fixture.pic.instance(),
        fixture.signer.into(),
        fixture.owner.into(),
        spec!(controls::Prepare),
        &controls::PrepareInput {
            input: controls::SignatureInput {
                seed: Bytes::new(SEED.to_vec()).unwrap(),
                domain: Bytes::new(DOMAIN.to_vec()).unwrap(),
                message: Bytes::new(payload.to_vec()).unwrap(),
            },
            retention_ns: 60_000_000_000_u64,
        },
    )
    .await;
    assert!(matches!(unauthorized, Ok(Err(_))));
    let deadline = fixture
        .prepare(DOMAIN, &payload, 60_000_000_000)
        .await
        .unwrap();
    assert_eq!(
        fixture.prepare(DOMAIN, &payload, 60_000_000_000).await,
        Ok(deadline)
    );
    let proof = fixture.retrieve(DOMAIN, &payload).await.unwrap();
    fixture.verify(DOMAIN, payload, &proof).await;
    fixture.set_asset(9).await;
    assert_eq!(fixture.asset_version().await, 9);
    fixture
        .verify(
            DOMAIN,
            payload,
            &fixture.retrieve(DOMAIN, &payload).await.unwrap(),
        )
        .await;
    assert!(
        fixture
            .pic
            .instance()
            .upgrade_canister(
                fixture.signer.into(),
                fixture.wasm.clone(),
                controls::AssetVersion::encode_request(&()).unwrap(),
                Some(fixture.controller.into())
            )
            .await
            .is_err(),
        "wrong upgrade envelope must roll back"
    );
    assert_eq!(fixture.asset_version().await, 9);
    fixture.upgrade().await;
    assert_eq!(fixture.asset_version().await, 9);
    assert!(fixture.retrieve(DOMAIN, &payload).await.is_err());
    fixture.prepare(DOMAIN, &payload, 10).await.unwrap();
    fixture
        .pic
        .instance()
        .set_time(Time::from_nanos_since_unix_epoch(fixture.now().await + 10))
        .await
        .unwrap();
    assert!(fixture.retrieve(DOMAIN, &payload).await.is_err());
    let removed: Result<u32, controls::Refusal> = update_control::<u32, u32>(
        fixture.pic.instance(),
        fixture.signer,
        fixture.controller,
        spec!(controls::Prune),
        128_u32,
    )
    .await;
    assert_eq!(removed, Ok(1));
    fixture
        .prepare(DOMAIN, &payload, 60_000_000_000)
        .await
        .unwrap();
    let restored = fixture.retrieve(DOMAIN, &payload).await.unwrap();
    assert_eq!(proof.public_key_der, restored.public_key_der);
    fixture.verify(DOMAIN, payload, &restored).await;
    assert_eq!(fixture.asset_version().await, 9);
    fixture
        .pic
        .close()
        .await
        .expect("delete qualification instance before stopping its server");
}

async fn update_control<Request, T>(
    pic: &PocketIc,
    target: Principal,
    caller: Principal,
    spec: WireCallSpec,
    request: Request,
) -> Result<T, controls::Refusal>
where
    Request: serde::Serialize,
    T: serde::de::DeserializeOwned,
{
    update_wire::<Request, Result<T, controls::Refusal>>(
        pic,
        target.into(),
        caller.into(),
        spec,
        &request,
    )
    .await
    .expect("qualified Wire update")
}
async fn query_control<Request, T>(
    pic: &PocketIc,
    target: Principal,
    caller: Principal,
    spec: WireCallSpec,
    request: Request,
) -> Result<T, controls::Refusal>
where
    Request: serde::Serialize,
    T: serde::de::DeserializeOwned,
{
    query_wire::<Request, Result<T, controls::Refusal>>(
        pic,
        target.into(),
        caller.into(),
        spec,
        &request,
    )
    .await
    .expect("qualified Wire query")
}
