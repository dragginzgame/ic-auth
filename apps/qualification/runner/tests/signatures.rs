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
    pocket_ic::{PocketIc, PocketIcBuilder, Time},
};
use ic_testkit::pic::{
    PocketIcManagedServer, PocketIcStartupConfig, WireCallError, WireCallSpec, query_wire,
    update_wire,
};
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

#[path = "signatures/certified_proofs.rs"]
mod certified_proofs;

#[path = "signatures/ingress.rs"]
mod ingress;

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
        Self::with_application_subnets(1)
    }

    fn with_application_subnets(count: usize) -> Self {
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
        let mut builder = PocketIcBuilder::new()
            .with_state_dir(state_path)
            .with_nns_subnet()
            .with_application_subnet()
            .with_max_request_time_ms(Some(30_000));
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
        .expect("bounded testkit startup with the verified server");
        // Retain Testkit's field drop order during setup, including panic paths.
        let pic = owned.instance();
        pic.set_time(SystemTime::now().into());
        let controller = Principal::self_authenticating(b"qualification controller");
        let subnets = pic.topology().get_app_subnets();
        assert_eq!(subnets.len(), count);
        let signer = pic.create_canister_on_subnet(Some(controller.into()), None, subnets[0]);
        let resource =
            pic.create_canister_on_subnet(Some(controller.into()), None, subnets[count - 1]);
        assert_eq!(pic.get_subnet(signer), Some(subnets[0]));
        assert_eq!(pic.get_subnet(resource), Some(subnets[count - 1]));
        let owner = Principal::self_authenticating(
            CanisterSigPublicKey::new(signer, SEED.to_vec()).to_der(),
        );
        for canister in [signer, resource] {
            pic.add_cycles(canister, 2_000_000_000_000);
            pic.install_canister(
                canister,
                wasm.clone(),
                controls::Initialize::encode_request(
                    &Bytes::new(owner.as_slice().to_vec()).unwrap(),
                )
                .unwrap(),
                Some(controller.into()),
            );
        }
        // Fixture fields retain client-before-server teardown after this transfer.
        let (pic, server) = owned.into_parts();
        Self {
            pic,
            _server: server,
            signer: signer.into(),
            resource: resource.into(),
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
        update_control::<controls::PrepareInput, u64>(
            &self.pic,
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
        let proof: Result<ic_auth_wire::model::IcCanisterSignatureProofV1, String> = query_control::<
            controls::SignatureInput,
            ic_auth_wire::model::IcCanisterSignatureProofV1,
        >(
            &self.pic,
            self.signer,
            Principal::anonymous(),
            spec!(controls::Retrieve),
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
        let result: Result<(), String> = update_control::<u64, ()>(
            &self.pic,
            self.signer,
            self.controller,
            spec!(controls::SetAsset),
            version,
        );
        result.unwrap();
    }

    fn asset_version(&self) -> u64 {
        query_wire::<(), u64>(
            &self.pic,
            self.signer.into(),
            Principal::anonymous().into(),
            spec!(controls::AssetVersion),
            &(),
        )
        .expect("qualified Wire query")
    }

    fn upgrade(&self) {
        for canister in [self.signer, self.resource] {
            self.pic
                .upgrade_canister(
                    canister.into(),
                    self.wasm.clone(),
                    controls::Upgrade::encode_request(&()).unwrap(),
                    Some(self.controller.into()),
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
        controls::Prune::encode_request(&128).unwrap(),
        vec![0; controls::CONTROL_LIMITS.bytes + 1],
    ] {
        assert!(
            fixture
                .pic
                .update_call(
                    fixture.signer.into(),
                    fixture.controller.into(),
                    controls::SetAsset::NAME,
                    bytes
                )
                .is_err()
        );
        assert_eq!(fixture.asset_version(), 1);
    }
    // A query cannot dispatch the controller mutation. Keep transport refusal
    // distinct from an encoded application refusal returned by a valid update.
    let refused = query_wire::<u64, Result<(), controls::Refusal>>(
        &fixture.pic,
        fixture.signer.into(),
        fixture.controller.into(),
        spec!(controls::SetAsset),
        &99,
    );
    assert!(matches!(refused, Err(WireCallError::Call { context, .. })
        if Principal::from(context.canister_id) == fixture.signer
            && Principal::from(context.caller) == fixture.controller
            && context.method == controls::SetAsset::NAME));
    assert_eq!(fixture.asset_version(), 1);
    let unauthorized = update_wire::<controls::PrepareInput, Result<u64, controls::Refusal>>(
        &fixture.pic,
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
    );
    assert!(matches!(unauthorized, Ok(Err(_))));
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
                fixture.signer.into(),
                fixture.wasm.clone(),
                controls::AssetVersion::encode_request(&()).unwrap(),
                Some(fixture.controller.into())
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
    let removed: Result<u32, String> = update_control::<u32, u32>(
        &fixture.pic,
        fixture.signer,
        fixture.controller,
        spec!(controls::Prune),
        128_u32,
    );
    assert_eq!(removed, Ok(1));
    fixture.prepare(DOMAIN, &payload, 60_000_000_000).unwrap();
    let restored = fixture.retrieve(DOMAIN, &payload).unwrap();
    assert_eq!(proof.public_key_der, restored.public_key_der);
    fixture.verify(DOMAIN, payload, &restored);
    assert_eq!(fixture.asset_version(), 9);
}

fn update_control<Request, T>(
    pic: &PocketIc,
    target: Principal,
    caller: Principal,
    spec: WireCallSpec,
    request: Request,
) -> Result<T, String>
where
    Request: serde::Serialize,
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    update_wire::<Request, Result<T, controls::Refusal>>(
        pic,
        target.into(),
        caller.into(),
        spec,
        &request,
    )
    .expect("qualified Wire update")
    .map_err(|e| e.into_string())
}
fn query_control<Request, T>(
    pic: &PocketIc,
    target: Principal,
    caller: Principal,
    spec: WireCallSpec,
    request: Request,
) -> Result<T, String>
where
    Request: serde::Serialize,
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    query_wire::<Request, Result<T, controls::Refusal>>(
        pic,
        target.into(),
        caller.into(),
        spec,
        &request,
    )
    .expect("qualified Wire query")
    .map_err(|e| e.into_string())
}
