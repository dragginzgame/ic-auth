.DEFAULT_GOAL := help

include ci/tool-versions.env
include make/tools.mk

# Keep Cargo artifacts in this checkout, including CI and hook invocations.
export CARGO_TARGET_DIR := $(CURDIR)/target
export RUSTUP_AUTO_INSTALL := 0
RELEASE_REMOTE ?= origin
RELEASE_BRANCH ?= main
export RELEASE_DELIVERY := direct
export RELEASE_REMOTE RELEASE_BRANCH

.PHONY: help install-hooks format-tools-check fmt fmt-check fetch metadata \
        test-types test-protocol test-signatures test-tokens check-wasm clippy check-boundaries \
        check-snapshot check-dependency-pins check-doc-links ci \
        publish publish-dry-run check-package-licenses test-release-tools \
        test-host-tooling test-release-runner test-evidence-archive \
        release-patch release-minor release-major release-resume \
        release-version release-preflight release-verify release-prepare-version \
        release-prepared-check release-files release-commit-check \
        release-committed-check release-tagged-check release-push-check

help:
	@echo 'Setup: install-tools, tools-check, install-hooks, fetch'
	@echo 'Focused checks: test-types, test-protocol, test-signatures, test-tokens, test-host-tooling, check-wasm, clippy, metadata'
	@echo 'Formatting: fmt, fmt-check'
	@echo 'Governance: check-snapshot, check-dependency-pins, check-doc-links, check-boundaries'
	@echo 'Complete CI gate (explicit only): ci'
	@echo 'Releases: release-patch, release-minor, release-major; recovery: release-resume VERSION=X.Y.Z'
	@echo 'Crates.io: publish; local checks without upload: publish-dry-run, test-release-tools'

install-tools: install-rust-tools
tools-check: rust-tools-check

install-hooks:
	bash scripts/dev/install-git-hooks.sh

format-tools-check:
	bash scripts/ci/check-format-tools.sh "$(SHARED_TOOLING_CARGO_SORT_VERSION)"

fmt: format-tools-check
	cargo sort --workspace
	cargo fmt --all

fmt-check: format-tools-check
	cargo sort --workspace --check
	cargo fmt --all -- --check

# Explicit selected-cache preparation; ordinary Rust checks remain offline.
fetch:
	cargo fetch --locked

metadata:
	cargo metadata --locked --offline --format-version 1 > /dev/null

test-types:
	cargo test --locked --offline -p ic-auth-protocol-types

test-protocol:
	cargo test --locked --offline -p ic-auth

test-signatures:
	cargo test --locked --offline -p ic-auth --features canister-signature-verification --test canister_signature

test-tokens:
	cargo test --locked --offline -p ic-auth --features token-verification --test token

check-wasm:
	cargo check --locked --offline -p ic-auth-protocol-types -p ic-auth --target wasm32-unknown-unknown
	cargo check --locked --offline -p ic-auth --features canister-signature-verification --target wasm32-unknown-unknown
	cargo check --locked --offline -p ic-auth --features token-verification --target wasm32-unknown-unknown

clippy:
	cargo clippy --locked --offline -p ic-auth-protocol-types -p ic-auth -p ic-auth-tooling --all-targets --all-features -- -D warnings

check-boundaries:
	bash scripts/ci/check-auth-boundaries.sh

check-snapshot:
	bash scripts/ci/verify-shared-tooling-snapshot.sh

check-dependency-pins:
	bash scripts/ci/check-dependency-pins.sh --cargo-inheritance

check-doc-links:
	@rg --files -g '*.md' -0 | xargs -0 perl scripts/ci/check-documentation-links.pl --root "$(CURDIR)"

ci:
	+bash scripts/ci/run-validation-targets.sh --fail-fast check-snapshot check-dependency-pins check-doc-links fmt-check metadata check-boundaries test-types test-protocol test-signatures test-tokens test-host-tooling check-wasm clippy publish-dry-run test-release-tools test-release-runner test-evidence-archive

check-package-licenses:
	@cmp LICENSE crates/ic-auth-protocol-types/LICENSE
	@cmp LICENSE crates/ic-auth/LICENSE

publish-dry-run: check-package-licenses
	cargo publish --dry-run --locked --registry crates-io --allow-dirty -p ic-auth-protocol-types -p ic-auth

publish:
	bash scripts/release/publish.sh

test-release-tools:
	bash scripts/release/test-tools.sh

test-host-tooling:
	cargo test --locked --offline -p ic-auth-tooling

test-release-runner:
	bash scripts/ci/test-release-runner.sh

test-evidence-archive:
	@mkdir -p "$(CURDIR)/target/portable-fixtures"
	TMPDIR="$(CURDIR)/target/portable-fixtures" bash scripts/ci/test-evidence-archive.sh

ifneq ($(word 2,$(filter release-patch release-minor release-major release-resume,$(MAKECMDGOALS))),)
$(error Select exactly one release target)
endif

release-patch release-minor release-major:
	+@bash scripts/ci/run-release.sh "$(@:release-%=%)" "$(RELEASE_REMOTE)" "$(RELEASE_BRANCH)"

release-resume:
	+@bash scripts/ci/run-release.sh resume "$(VERSION)" "$(RELEASE_REMOTE)" "$(RELEASE_BRANCH)"

release-version:
	@bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml

release-preflight release-prepare-version release-prepared-check release-files \
release-commit-check release-committed-check release-tagged-check release-push-check:
	@bash scripts/release/metadata.sh "$(@:release-%=%)"

release-verify:
	+@bash scripts/release/metadata.sh verify
