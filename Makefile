.DEFAULT_GOAL := help

FORMAT_RUST_ARGS := --extra-workspace qualification
AUTH_REFERENCE_REPO ?= ../ic-auth
override TOKO_TOOLING_ROOT := $(CURDIR)/vendor/toko-tooling
include vendor/toko-tooling/make/consumer.mk
include vendor/toko-tooling/make/cargo-dependencies.mk

.PHONY: help snapshot-check
help:
	@printf '%s\n' \
	  'deps-list       List local packages and versions (offline)' \
	  'deps-update     Update workspace dependency locks' \
	  '                Optional: DEPENDENCY_PACKAGE=name DEPENDENCY_PRECISE=version' \
	  'install-tools   Prepare the pinned Toko host and Rust tools' \
	  'tools-check     Verify prepared tools without installing' \
	  'snapshot-check  Verify the installed tooling bytes (offline)' \
	  'toko-tooling-check  Verify Cargo selection, lock and installed tooling' \
	  'test-request    Scope/request contracts and immutable-scope compile-fail case' \
	  'test-claims     Signed-claims bytes, bounds, ordering and role grammar' \
	  'test-delegation-encoding  Root-delegation signing commitments and bounds' \
	  'test-verification  Ed25519 proof/context, work charging and immutable results' \
	  'qualify-ed25519  Adversarial legacy-scalar dependency feature graph' \
	  'qualify-certified-issuer  Real BLS certificates and protected issuer policy' \
	  'lint-certified-qualification  Clippy for independent certificate fixtures' \
	  'lint-ed25519-qualification  Clippy for the adversarial feature check' \
	  'test-wire       All four Wire contracts, proof layouts and bounded refusal' \
	  'qualify-wire    Check current Wire against fixed reference frames and budgets' \
	  'generate-wire   Generate the four browser codecs into qualification/target' \
	  'qualify-wire-browser  Compare old/new browser codecs (Node and reference checkout)' \
	  'lint-wire-qualification  Clippy for the independent codec comparison' \
	  'lint-protocol   Clippy for the implemented protocol packages and tests' \
	  'doc-protocol    Rustdoc for the implemented protocol APIs' \
	  'check-protocol-wasm  Compile the protocol packages for wasm32' \
	  'fmt / fmt-check  Format or check Rust and Cargo manifests' \
	  'toko-cloc       Run this standalone command for classified Rust LOC' \
	  'See docs/development/tooling.md for setup and qualification scope.'

snapshot-check:
	@bash "$(TOKO_TOOLING_ROOT)/scripts/ci/verify-toko-tooling-snapshot.sh" \
	  --consumer "$(CURDIR)" --require-annotations

.PHONY: qualification-tooling-check
toko-tooling-check: qualification-tooling-check
qualification-tooling-check:
	@selection=$$(PATH="$(CURDIR)/.tools/host/bin:$$PATH" perl "$(TOKO_TOOLING_ROOT)/scripts/dev/tooling-lock.pl" "$(CURDIR)/qualification") || exit $$?; \
	printf '%s\n' "$$selection" | "$(CURDIR)/.tools/host/bin/jq" -e \
	  --arg version "$$(awk -F '\t' '$$1 == "# version" { print $$2 }' .toko-tooling.snapshot)" \
	  --arg checksum "$$(awk -F '\t' '$$1 == "revision" { print $$2 }' .toko-tooling.snapshot)" \
	  '.version == $$version and .checksum == $$checksum' >/dev/null

.PHONY: test-request test-claims test-verification qualify-ed25519 lint-ed25519-qualification test-wire qualify-wire generate-wire qualify-wire-browser lint-wire-qualification lint-protocol doc-protocol check-protocol-wasm
test-request: toko-tooling-check
	cargo test --offline --locked -p ic-auth-protocol-types --test scope
	cargo test --offline --locked -p ic-auth --test session_request
	cargo test --offline --locked -p ic-auth-protocol-types --doc

test-claims: toko-tooling-check
	cargo test --offline --locked -p ic-auth-protocol-types --test role
	cargo test --offline --locked -p ic-auth --test claims

.PHONY: test-delegation-encoding
test-delegation-encoding: toko-tooling-check
	cargo test --offline --locked -p ic-auth --test delegation_encoding

test-verification: toko-tooling-check
	cargo test --offline --locked -p ic-auth --features ed25519-verification --test verification --lib
	cargo test --offline --locked -p ic-auth --features ed25519-verification --doc

qualify-ed25519: toko-tooling-check
	cargo test --offline --locked --manifest-path qualification/Cargo.toml -p ic-auth-ed25519-conformance --test legacy_scalar

.PHONY: qualify-certified-issuer lint-certified-qualification
qualify-certified-issuer: toko-tooling-check
	cargo test --offline --locked --manifest-path qualification/Cargo.toml -p ic-auth-certified-conformance --test issuer_signature

lint-certified-qualification: toko-tooling-check
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p ic-auth-certified-conformance --all-targets -- -D warnings

lint-ed25519-qualification: toko-tooling-check
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p ic-auth-ed25519-conformance --all-targets -- -D warnings

test-wire: toko-tooling-check
	cargo test --offline --locked -p ic-auth-wire --test caller_contracts --test admission_contracts

qualify-wire: toko-tooling-check
	cargo test --offline --locked --manifest-path qualification/Cargo.toml -p ic-auth-wire-conformance --test codecs

generate-wire: toko-tooling-check
	cargo run --offline --locked -p ic-auth-wire --features typescript --example generate -- qualification/target/wire-browser

qualify-wire-browser: generate-wire
	node crates/ic-auth-wire/tests/verify-reference.mjs "$(AUTH_REFERENCE_REPO)" qualification/target/wire-browser

lint-wire-qualification: toko-tooling-check
	cargo clippy --offline --locked --manifest-path qualification/Cargo.toml -p ic-auth-wire-conformance --all-targets -- -D warnings

lint-protocol: toko-tooling-check
	cargo clippy --offline --locked -p ic-auth-protocol-types -p ic-auth -p ic-auth-wire --all-targets --features ic-auth-wire/typescript,ic-auth/ed25519-verification,ic-auth/certified-issuer-verification -- -D warnings

doc-protocol: toko-tooling-check
	RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked -p ic-auth-protocol-types -p ic-auth -p ic-auth-wire --features ic-auth/ed25519-verification,ic-auth/certified-issuer-verification --no-deps

check-protocol-wasm: toko-tooling-check
	cargo check --offline --locked -p ic-auth-protocol-types -p ic-auth -p ic-auth-wire --lib --features ic-auth-wire/typescript,ic-auth/ed25519-verification,ic-auth/certified-issuer-verification --target wasm32-unknown-unknown
