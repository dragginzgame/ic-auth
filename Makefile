.DEFAULT_GOAL := help

include ci/tool-versions.env
include make/tools.mk

# Keep Cargo artifacts in this checkout, including CI and hook invocations.
export CARGO_TARGET_DIR := $(CURDIR)/target
export RUSTUP_AUTO_INSTALL := 0
RELEASE_REMOTE ?= origin
RELEASE_BRANCH ?= main
export RELEASE_DELIVERY := direct

.PHONY: help install-hooks format-tools-check fmt fmt-check fetch metadata \
        test-types test-protocol check-wasm clippy check-boundaries \
        check-snapshot check-dependency-pins check-doc-links ci \
        release-patch release-minor release-major release-resume \
        release-version release-preflight

help:
	@echo 'Setup: install-tools, tools-check, install-hooks, fetch'
	@echo 'Focused checks: test-types, test-protocol, check-wasm, clippy, metadata'
	@echo 'Formatting: fmt, fmt-check'
	@echo 'Governance: check-snapshot, check-dependency-pins, check-doc-links, check-boundaries'
	@echo 'Complete CI gate (explicit only): ci'
	@echo 'Release entry points: release-patch, release-minor, release-major (bootstrap blocked)'

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

# Explicit network/cache preparation; validation never fetches or unlocks.
fetch:
	cargo fetch --locked

metadata:
	cargo metadata --locked --offline --format-version 1 > /dev/null

test-types:
	cargo test --locked --offline -p ic-auth-types

test-protocol:
	cargo test --locked --offline -p ic-auth

check-wasm:
	cargo check --locked --offline -p ic-auth-types -p ic-auth --target wasm32-unknown-unknown

clippy:
	cargo clippy --locked --offline -p ic-auth-types -p ic-auth --all-targets -- -D warnings

check-boundaries:
	bash scripts/ci/check-auth-boundaries.sh

check-snapshot:
	bash scripts/ci/verify-shared-tooling-snapshot.sh

check-dependency-pins:
	bash scripts/ci/check-dependency-pins.sh --cargo-inheritance

check-doc-links:
	@rg --files -g '*.md' -0 | xargs -0 perl scripts/ci/check-documentation-links.pl --root "$(CURDIR)"

ci:
	+bash scripts/ci/run-validation-targets.sh --fail-fast check-snapshot check-dependency-pins check-doc-links fmt-check metadata check-boundaries test-types test-protocol check-wasm clippy

ifneq ($(word 2,$(filter release-patch release-minor release-major release-resume,$(MAKECMDGOALS))),)
$(error Select exactly one release target)
endif

release-patch release-minor release-major:
	+@bash scripts/ci/run-release.sh "$(@:release-%=%)" "$(RELEASE_REMOTE)" "$(RELEASE_BRANCH)"

release-resume:
	+@bash scripts/ci/run-release.sh resume "$(VERSION)" "$(RELEASE_REMOTE)" "$(RELEASE_BRANCH)"

release-version:
	@bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml

# There is no finalized release base, remote or approved delivery destination.
# Keep the shared entry points fail-closed until that separate release boundary
# supplies and qualifies the remaining metadata/delivery adapters.
release-preflight:
	@echo 'Release unavailable: local bootstrap has no finalized release base or qualified delivery adapters. See docs/development.md.' >&2
	@exit 1
