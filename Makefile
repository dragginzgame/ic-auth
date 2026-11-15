.DEFAULT_GOAL := help

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
	  'lint-protocol   Clippy for both implemented protocol packages and tests' \
	  'doc-protocol    Rustdoc for the implemented protocol APIs' \
	  'check-protocol-wasm  Compile the protocol packages for wasm32' \
	  'fmt / fmt-check  Format or check Rust and Cargo manifests' \
	  'toko-cloc       Run this standalone command for classified Rust LOC' \
	  'See docs/development/tooling.md for setup and qualification scope.'

snapshot-check:
	@bash "$(TOKO_TOOLING_ROOT)/scripts/ci/verify-toko-tooling-snapshot.sh" \
	  --consumer "$(CURDIR)" --require-annotations

.PHONY: test-request test-claims lint-protocol doc-protocol check-protocol-wasm
test-request: toko-tooling-check
	cargo test --offline --locked -p ic-auth-protocol-types --test scope
	cargo test --offline --locked -p ic-auth --test session_request
	cargo test --offline --locked -p ic-auth-protocol-types --doc

test-claims: toko-tooling-check
	cargo test --offline --locked -p ic-auth-protocol-types --test role
	cargo test --offline --locked -p ic-auth --test claims

lint-protocol: toko-tooling-check
	cargo clippy --offline --locked -p ic-auth-protocol-types -p ic-auth --all-targets -- -D warnings

doc-protocol: toko-tooling-check
	RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked -p ic-auth-protocol-types -p ic-auth --no-deps

check-protocol-wasm: toko-tooling-check
	cargo check --offline --locked -p ic-auth-protocol-types -p ic-auth --lib --target wasm32-unknown-unknown
