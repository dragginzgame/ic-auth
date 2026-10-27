.DEFAULT_GOAL := help

include ci/tool-versions.env
# Extend the shared ordered aggregates after common host and Rust admission.
LOCAL_TOOL_INSTALL_TARGETS := install-msrv install-testkit-tools install-host-file
LOCAL_TOOL_CHECK_TARGETS := msrv-tools-check testkit-tools-check host-file-check
RELEASE_DELIVERY_POLICY := $(CURDIR)/ci/release-delivery.txt
override SHARED_TOOLING_ROOT := $(CURDIR)
include make/consumer.mk

# Keep Cargo artifacts in this checkout, including CI and hook invocations.
export CARGO_TARGET_DIR := $(CURDIR)/target
export RUSTUP_AUTO_INSTALL := 0
export RELEASE_DELIVERY := direct
export RELEASE_REMOTE RELEASE_BRANCH

.PHONY: help install-hooks fetch metadata test-formatting-hooks \
        install-testkit-tools testkit-tools-check test-testkit-tools install-release-tools release-tools-check \
        test-types test-protocol test-portable-signatures test-chain-key-signatures test-signatures test-signature-store test-certified-proofs test-sessions check-wasm clippy check-boundaries \
        check-snapshot check-dependency-pins check-doc-links check-msrv install-msrv msrv-tools-check tasks ci \
        client-tools-check install-client-dependencies generate-client-contracts check-client-contracts check-client-boundary test-client test-client-tools \
        publish publish-dry-run check-package-licenses test-release-tools \
        test-host-tooling test-qualification build-qualification-canister test-release-runner test-evidence-archive test-tools-evidence test-make-jobserver \
        release-version release-preflight release-verify release-prepare-version \
        release-prepared-check release-files release-commit-check \
        release-committed-check release-tagged-check release-push-check

help:
	@echo 'Setup: install-tools, tools-check, install-hooks, fetch; minimum compiler: install-msrv, msrv-tools-check'
	@echo 'Release editor: install-release-tools (explicit setup), release-tools-check (offline path admission)'
	@echo 'File CLI: install-host-file (explicit setup), host-file-check (offline path admission)'
	@echo 'Testkit server: install-testkit-tools (explicit setup), testkit-tools-check (offline admission)'
	@echo 'Select by changed contract: test-types, test-protocol, test-portable-signatures, test-chain-key-signatures, test-signatures, test-signature-store, test-certified-proofs, test-sessions, test-host-tooling, metadata'
	@echo 'Aggregate qualification: clippy, check-wasm, check-msrv; live fixture: test-qualification'
	@echo 'Named Cargo package/test selections: see docs/development.md#validation'
	@echo 'Browser client: install-client-dependencies, client-tools-check, generate-client-contracts, check-client-contracts, check-client-boundary, test-client'
	@echo 'Formatting: fmt, fmt-check, test-formatting-hooks'
	@echo 'Governance: check-snapshot, check-dependency-pins, check-doc-links, check-boundaries, check-msrv; maintenance catalog: tasks'
	@echo 'Complete CI gate (explicit only): ci'
	@echo 'Releases: release-patch, release-minor, release-major; recovery: release-resume VERSION=X.Y.Z'
	@echo 'Crates.io: publish; local checks without upload: publish-dry-run, test-release-tools'

install-testkit-tools: install-host-tools
	+bash scripts/dev/testkit-tools.sh install

testkit-tools-check:
	+@bash scripts/dev/testkit-tools.sh check

install-release-tools: install-host-tools
	+bash scripts/dev/release-tools.sh install

.PHONY: install-host-file host-file-check
install-host-file: install-host-tools
	+bash scripts/dev/host-file-tools.sh install

host-file-check:
	+@bash scripts/dev/host-file-tools.sh check

release-tools-check:
	+@bash scripts/dev/release-tools.sh check

test-testkit-tools:
	bash scripts/ci/test-testkit-tools.sh

test-formatting-hooks:
	@mkdir -p target/portable-fixtures
	+@set -e; fixture="$$(mktemp "$(CURDIR)/target/portable-fixtures/unsorted-manifest.XXXXXX")"; \
		trap 'rm -f "$$fixture"' EXIT; \
		perl -0777 -pe 's/^(serde_bytes = [^\n]*\n)(serde_cbor = [^\n]*\n)/$$2$$1/m or die "missing selected dependency pair"' Cargo.toml > "$$fixture"; \
		TMPDIR="$(CURDIR)/target/portable-fixtures" bash scripts/ci/check-formatting-hooks.sh \
		"$(CURDIR)" crates/ic-auth/src/lib.rs Cargo.toml "$$fixture" --rust-sources \
		make/consumer.mk make/release.mk make/rust-format.mk ci/tool-versions.env

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
	+cargo test --locked --offline -p ic-auth-protocol-types -p ic-auth-wire -p ic-auth-qualification-contracts

test-protocol:
	+cargo test --locked --offline -p ic-auth

test-portable-signatures:
	+cargo test --locked --offline -p ic-auth --features portable-signature-verification --test portable_signature --test portable_revocation
	+cargo test --locked --offline -p ic-auth-feature-unification --test portable_signature --test portable_revocation
	+cargo test --locked --offline -p ic-auth --features portable-signature-verification,certified-proof-verification --test certified portable_certified::

test-chain-key-signatures:
	+cargo test --locked --offline -p ic-auth --features chain-key-verification --test chain_key_signature

test-signatures:
	+cargo test --locked --offline -p ic-auth --features canister-signature-verification --test canister_signature

test-signature-store:
	+cargo test --locked --offline -p ic-auth --features canister-signature-preparation,canister-signature-verification --lib --test signature_store

test-certified-proofs:
	+cargo test --locked --offline -p ic-auth --features certified-proof-verification --test certified

test-sessions:
	+cargo test --locked --offline -p ic-auth --features sessions,certified-proof-verification,portable-signature-verification --test admission_refusals
	+cargo test --locked --offline -p ic-auth --features sessions,certified-proof-verification,portable-signature-verification --test certified portable_certified::
	+cargo test --locked --offline -p ic-auth --features sessions,portable-signature-verification --test sessions
	+cargo test --locked --offline -p ic-auth --features sessions --doc
	+cargo test --locked --offline -p ic-auth --features session-restore,portable-signature-verification --test sessions --test portable_revocation

check-wasm:
	+bash scripts/dev/check-wasm.sh

clippy:
	+cargo clippy --locked --offline -p ic-auth-protocol-types -p ic-auth -p ic-auth-wire -p ic-auth-tooling -p ic-auth-qualification -p ic-auth-qualification-contracts -p ic-auth-feature-unification --all-targets --all-features -- -D warnings
	+cargo clippy --locked --offline -p ic-auth-qualification-canister --target wasm32-unknown-unknown -- -D warnings

build-qualification-canister:
	+cargo build --locked --offline -p ic-auth-qualification-canister --target wasm32-unknown-unknown --release

test-qualification: testkit-tools-check client-tools-check
	+cargo test --locked --offline -p ic-auth-qualification-canister --lib
	+$(MAKE) --no-print-directory build-qualification-canister
	+bash scripts/dev/test-qualification.sh

check-boundaries:
	+bash scripts/ci/check-cargo-dependency-boundaries.sh --consumer "$(CURDIR)" --external-ic
	+bash scripts/ci/test-auth-boundaries.sh
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

check-client-boundary: client-tools-check
	npm --prefix packages/client run check:sdk

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

# Sequential validation also holds under parallel Make: admit tools, then run
# the complete existing lint gate before product tests, builds or packaging.
ci:
	+bash scripts/ci/run-validation-targets.sh --fail-fast \
	  --input Cargo.toml --input Cargo.lock --input rust-toolchain.toml \
	  --input ci/tool-versions.env --input ci/ic-tools.tsv --input ci/release-delivery.txt \
	  --input ci/cargo-dependency-boundaries.json --input ci/external-ic-dependencies.txt \
	  --input ci/host-file.env --input scripts/dev/host-file-tools.sh --input scripts/dev/run-host-file.sh \
	  --input packages/client/package.json --input packages/client/package-lock.json \
	  --input packages/client/.nvmrc \
	  check-snapshot tools-check clippy check-dependency-pins check-doc-links test-make-jobserver fmt-check test-formatting-hooks metadata check-boundaries check-msrv test-tools-evidence test-testkit-tools test-client-tools test-client test-types test-protocol test-portable-signatures test-chain-key-signatures test-signatures test-signature-store test-certified-proofs test-sessions test-host-tooling test-qualification check-wasm publish-dry-run test-release-tools

check-package-licenses:
	@cmp LICENSE crates/ic-auth-protocol-types/LICENSE
	@cmp LICENSE crates/ic-auth/LICENSE
	@cmp LICENSE crates/ic-auth-wire/LICENSE

# A reused temporary-registry path can retain extracted same-version source
# across edits. Give each dry run a new registry identity and retain its evidence.
publish-dry-run: check-package-licenses
	+@set -eu; \
	  mkdir -p "$(CURDIR)/target/publication-dry-runs"; \
	  staging="$$(mktemp -d "$(CURDIR)/target/publication-dry-runs/run.XXXXXX")"; \
	  printf 'Publication dry-run evidence retained: %s\n' "$$staging"; \
	  cargo publish --dry-run --locked --registry crates-io --allow-dirty \
	    --target-dir "$$staging" -p ic-auth-protocol-types -p ic-auth -p ic-auth-wire

publish:
	+bash scripts/release/publish.sh

test-release-tools:
	+bash scripts/release/test-tools.sh

test-host-tooling: host-file-check
	+cargo test --locked --offline -p ic-auth-tooling

# Generic producer matrices remain opt-in; CI keeps Auth adapter/caller proofs.
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
