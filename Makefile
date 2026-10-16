.DEFAULT_GOAL := help

include ci/tool-versions.env
# Extend the shared ordered aggregates after host, IC and Rust admission.
LOCAL_TOOL_INSTALL_TARGETS := install-msrv install-testkit-tools
LOCAL_TOOL_CHECK_TARGETS := msrv-tools-check testkit-tools-check
override SHARED_TOOLING_ROOT := $(CURDIR)
include make/consumer.mk

# Keep Cargo artifacts in this checkout, including CI and hook invocations.
export CARGO_TARGET_DIR := $(CURDIR)/target
export RUSTUP_AUTO_INSTALL := 0
export RELEASE_DELIVERY := direct
export RELEASE_REMOTE RELEASE_BRANCH

.PHONY: help install-hooks fetch metadata test-formatting-hooks \
        install-testkit-tools testkit-tools-check test-testkit-tools \
        test-types test-protocol test-portable-signatures test-chain-key-signatures test-signatures test-signature-store test-tokens test-sessions check-wasm clippy check-boundaries \
        check-snapshot check-dependency-pins check-doc-links check-msrv install-msrv msrv-tools-check tasks ci \
        client-tools-check install-client-dependencies generate-client-contracts check-client-contracts test-client test-client-tools \
        publish publish-dry-run check-package-licenses test-release-tools \
        test-host-tooling test-qualification build-qualification-canister test-release-runner test-evidence-archive test-tools-evidence test-make-jobserver \
        release-version release-preflight release-verify release-prepare-version \
        release-prepared-check release-files release-commit-check \
        release-committed-check release-tagged-check release-push-check

help:
	@echo 'Setup: install-tools, tools-check, install-hooks, fetch; minimum compiler: install-msrv, msrv-tools-check'
	@echo 'Testkit server: install-testkit-tools (explicit setup), testkit-tools-check (offline admission)'
	@echo 'Focused checks: test-types, test-protocol, test-portable-signatures, test-chain-key-signatures, test-signatures, test-signature-store, test-tokens, test-sessions, test-host-tooling, test-qualification, check-wasm, clippy, metadata'
	@echo 'Browser client: install-client-dependencies, client-tools-check, generate-client-contracts, check-client-contracts, test-client'
	@echo 'Formatting: fmt, fmt-check, test-formatting-hooks'
	@echo 'Governance: check-snapshot, check-dependency-pins, check-doc-links, check-boundaries, check-msrv; maintenance catalog: tasks'
	@echo 'Complete CI gate (explicit only): ci'
	@echo 'Releases: release-patch, release-minor, release-major; recovery: release-resume VERSION=X.Y.Z'
	@echo 'Crates.io: publish; local checks without upload: publish-dry-run, test-release-tools'

install-testkit-tools: install-host-tools
	+bash scripts/dev/testkit-tools.sh install

testkit-tools-check:
	+@bash scripts/dev/testkit-tools.sh check

test-testkit-tools:
	bash scripts/ci/test-testkit-tools.sh

test-formatting-hooks:
	@mkdir -p target/portable-fixtures
	+@TMPDIR="$(CURDIR)/target/portable-fixtures" bash scripts/ci/check-formatting-hooks.sh \
		"$(CURDIR)" crates/ic-auth/src/lib.rs Cargo.toml --no-dependency-tables \
		make/consumer.mk make/release.mk make/rust-format.mk ci/tool-versions.env \
		crates/ic-auth/src/canonical.rs \
		crates/ic-auth/src/portable.rs \
		crates/ic-auth/src/portable/rules.rs \
		crates/ic-auth/src/portable/revocation.rs \
		crates/ic-auth/src/portable/verified.rs \
		crates/ic-auth/tests/portable_rules.rs \
		crates/ic-auth/tests/portable_revocation.rs \
		crates/ic-auth/src/portable_signature.rs \
		crates/ic-auth/src/portable_certified.rs \
		crates/ic-auth/src/portable_credential.rs \
		crates/ic-auth/tests/portable_certified/mod.rs \
		crates/ic-auth/tests/portable_signature.rs \
		crates/ic-auth/tests/portable.rs \
		crates/ic-auth-protocol-types/src/lib.rs \
		crates/ic-auth-protocol-types/src/portable.rs \
		crates/ic-auth-protocol-types/src/logout.rs \
		crates/ic-auth/src/certificate_issuance.rs \
		crates/ic-auth/src/session_request.rs \
		crates/ic-auth/src/session/mod.rs \
		crates/ic-auth/src/admission_budget.rs \
		crates/ic-auth/src/session/model.rs \
		crates/ic-auth/Cargo.toml \
		crates/ic-auth/src/chain_key_signature.rs \
		crates/ic-auth/src/token/root.rs \
		crates/ic-auth/src/token/rules.rs \
		crates/ic-auth/src/token_preparation.rs \
		crates/ic-auth/tests/chain_key_signature.rs \
		crates/ic-auth/tests/certificate_issuance.rs \
		crates/ic-auth/tests/session_request.rs \
		crates/ic-auth/tests/sessions.rs \
		crates/ic-auth/tests/token.rs \
		crates/ic-auth/tests/root_bindings/mod.rs \
		crates/ic-auth/tests/token_preparation.rs \
		apps/qualification/runner/Cargo.toml \
		apps/qualification/runner/tests/signatures/application_tokens.rs \
		apps/qualification/runner/tests/signatures/sessions.rs \
		apps/qualification/runner/tests/signatures/sessions/costs.rs \
		apps/qualification/sessions.rs \
		apps/qualification/canister/Cargo.toml \
		apps/qualification/canister/src/sessions.rs \
		apps/qualification/canister/src/lib.rs \
		apps/qualification/runner/tests/signatures.rs \
		apps/qualification/runner/tests/signatures/profile_composition.rs

install-msrv: install-host-tools
	bash scripts/dev/msrv-tools.sh install

msrv-tools-check:
	+bash scripts/dev/msrv-tools.sh check

check-msrv:
	+bash scripts/dev/check-msrv.sh

tasks:
	@cat tasks/README.md

# Explicit selected-cache preparation; ordinary Rust checks remain offline.
fetch:
	+cargo fetch --locked

metadata:
	+cargo metadata --locked --offline --format-version 1 > /dev/null

test-types:
	+cargo test --locked --offline -p ic-auth-protocol-types

test-protocol:
	+cargo test --locked --offline -p ic-auth

test-portable-signatures:
	+cargo test --locked --offline -p ic-auth --features portable-signature-verification --test portable_signature --test portable_revocation
	+cargo test --locked --offline -p ic-auth --features portable-signature-verification,ed25519-dalek/legacy_compatibility --test portable_signature --test portable_revocation
	+cargo test --locked --offline -p ic-auth --features portable-signature-verification,token-verification --test token portable_certified::

test-chain-key-signatures:
	+cargo test --locked --offline -p ic-auth --features chain-key-verification --test chain_key_signature

test-signatures:
	+cargo test --locked --offline -p ic-auth --features canister-signature-verification --test canister_signature

test-signature-store:
	+cargo test --locked --offline -p ic-auth --features canister-signature-preparation,canister-signature-verification --lib --test signature_store

test-tokens:
	+cargo test --locked --offline -p ic-auth --features token-verification --test token

test-sessions:
	+cargo test --locked --offline -p ic-auth --features sessions,token-verification,portable-signature-verification --test token portable_certified::
	+cargo test --locked --offline -p ic-auth --features sessions,portable-signature-verification --test sessions
	+cargo test --locked --offline -p ic-auth --features sessions --doc
	+cargo test --locked --offline -p ic-auth --features session-restore,portable-signature-verification --test sessions --test portable_revocation

check-wasm:
	+cargo check --locked --offline -p ic-auth --features portable-signature-verification --target wasm32-unknown-unknown
	+cargo check --locked --offline -p ic-auth-protocol-types -p ic-auth --target wasm32-unknown-unknown
	+cargo check --locked --offline -p ic-auth --features chain-key-verification --target wasm32-unknown-unknown
	+cargo check --locked --offline -p ic-auth --features canister-signature-verification --target wasm32-unknown-unknown
	+cargo check --locked --offline -p ic-auth --features canister-signature-preparation --target wasm32-unknown-unknown
	+cargo check --locked --offline -p ic-auth --features token-verification --target wasm32-unknown-unknown
	+cargo check --locked --offline -p ic-auth --features sessions --target wasm32-unknown-unknown
	+cargo check --locked --offline -p ic-auth --features session-restore --target wasm32-unknown-unknown

clippy:
	+cargo clippy --locked --offline -p ic-auth-protocol-types -p ic-auth -p ic-auth-tooling -p ic-auth-qualification --all-targets --all-features -- -D warnings
	+cargo clippy --locked --offline -p ic-auth-qualification-canister --target wasm32-unknown-unknown -- -D warnings

build-qualification-canister:
	+cargo build --locked --offline -p ic-auth-qualification-canister --target wasm32-unknown-unknown --release

test-qualification: testkit-tools-check
	+cargo test --locked --offline -p ic-auth-qualification-canister --lib
	+$(MAKE) --no-print-directory build-qualification-canister
	+bash scripts/dev/test-qualification.sh

check-boundaries:
	+bash scripts/ci/check-auth-boundaries.sh

check-snapshot: shared-tooling-check

client-tools-check:
	bash scripts/dev/client-tools.sh check

install-client-dependencies:
	bash scripts/dev/client-tools.sh install

generate-client-contracts:
	+bash scripts/dev/client-contracts.sh generate

check-client-contracts:
	+bash scripts/dev/client-contracts.sh check

test-client:
	+bash scripts/dev/test-client.sh

test-client-tools:
	+bash scripts/ci/test-client-tools.sh

check-dependency-pins:
	+bash scripts/ci/check-dependency-pins.sh --cargo-inheritance --npm-root packages/client \
	  --node-version "$$(cat packages/client/.nvmrc)" \
	  --npm-version "$$(jq -er '.packageManager | sub("^npm@"; "")' packages/client/package.json)"

check-doc-links:
	@rg --files -g '*.md' -0 | xargs -0 perl scripts/ci/check-documentation-links.pl --root "$(CURDIR)"

ci:
	+bash scripts/ci/run-validation-targets.sh --fail-fast \
	  --input Cargo.toml --input Cargo.lock --input rust-toolchain.toml \
	  --input ci/tool-versions.env --input ci/ic-tools.tsv \
	  --input packages/client/package.json --input packages/client/package-lock.json \
	  --input packages/client/.nvmrc \
	  check-snapshot tools-check check-dependency-pins check-doc-links test-make-jobserver fmt-check test-formatting-hooks metadata check-boundaries check-msrv test-tools-evidence test-testkit-tools test-client-tools test-client test-types test-protocol test-portable-signatures test-chain-key-signatures test-signatures test-signature-store test-tokens test-sessions test-host-tooling test-qualification check-wasm clippy publish-dry-run test-release-tools test-release-runner test-evidence-archive

check-package-licenses:
	@cmp LICENSE crates/ic-auth-protocol-types/LICENSE
	@cmp LICENSE crates/ic-auth/LICENSE

publish-dry-run: check-package-licenses
	+cargo publish --dry-run --locked --registry crates-io --allow-dirty -p ic-auth-protocol-types -p ic-auth

publish:
	+bash scripts/release/publish.sh

test-release-tools:
	+bash scripts/release/test-tools.sh

test-host-tooling:
	+cargo test --locked --offline -p ic-auth-tooling

test-release-runner:
	bash scripts/ci/test-release-runner.sh

test-tools-evidence:
	bash scripts/ci/test-tools-evidence.sh

test-make-jobserver:
	bash scripts/ci/test-make-jobserver.sh

test-evidence-archive:
	@mkdir -p "$(CURDIR)/target/portable-fixtures"
	TMPDIR="$(CURDIR)/target/portable-fixtures" bash scripts/ci/test-evidence-archive.sh

release-version:
	+@bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml

release-preflight release-prepare-version release-prepared-check release-files \
release-commit-check release-committed-check release-tagged-check release-push-check:
	+@bash scripts/release/metadata.sh "$(@:release-%=%)"

release-verify:
	+@bash scripts/release/metadata.sh verify
