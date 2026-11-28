# Receiver policy ownership

Receiver policy is private to `ic-auth`. Hosts use `ic_auth::receiver_memory` for
operations, `SessionAuthority` and admission inputs, limits, errors, receipts,
usage and inspection results. `LoginBinding` and `SessionFacts` remain public
because inspection returns them; they establish no authority and no operation
accepts them as a prepared admission.

`ReceiverSnapshot`, `LoginSnapshot`, `LoginFacts`, `DeviceFacts`,
`AdmissionFootprint`, retry/login decisions and their assessment functions are
internal. There is no public backend extension interface or test-only facade.
The policy module is compiled for the memory feature or unit tests, so codec-only
and standalone-verifier builds do not carry an unused receiver policy surface.
This intentionally changes the unpublished Rust API; Wire and signed bytes do
not change. No compatibility re-export of `ic_auth::receiver_policy` is retained.

## Rules and state custody

The memory receiver selects the caller, session, login and device facts from its
own records and indexes. The host still supplies authenticated caller/time and
current authority; untrusted fields cannot establish their provenance. A future
durable backend must admit persisted corruption before invoking the private rules.
The facts borrow/project authoritative state rather than introducing another
login ledger or generic storage framework.

Read and exact retry share current liveness, denial, generation, authority,
single-role scope and lease checks. Changed request identity conflicts; exact retry
retains the original deadline without new proof work. Expired proof bytes cannot
revive an expired lease, denied login or stale authority. Request normalization,
canonical grant ordering and independent required-scope limits remain unchanged.

Login preflight checks the immutable issuer-qualified account/device/login binding
and at most two selected rows. Same-browser renewal preserves that binding and its
absolute deadline. A different browser needs a strictly higher slot generation;
replacing a known denied slot still requires complete proof verification. Live
unknown denial blocks the new browser. Known fences survive session expiry until bounded pruning reaches their safe reclamation deadline, and
replacement/renewal reuse a full known-login pool without erasing replay history.

Capacity policy validates physical session/replay/account counts and checked byte
arithmetic. Expired rows count until pruning. Replacement subtracts only the old
session's size and adds a new replay; it refunds no consumed replay quota. The
reference 2048/512 session/replay record ceilings and independently checked logical
byte accounting remain intact. They do not select a durable representation.

The operation owner retains clock observations and attempted-work charges on
normal refusals, verifies the exact canonical claims and publishes affected state.
Pure decisions are neither reservations nor portable write tokens. Closed-state
handling, storage/host rollback and complete recovery remain distinct obligations.
Stable receivers reuse this policy and retain their state through [upgrade checkpoints](../design/DESIGN.md#stable-upgrade-checkpoints).

## Coverage after API narrowing

All thirteen read/retry/login scenarios moved unchanged from the integration-test
boundary into `src/receiver_policy/tests/mod.rs`; all six literal capacity cases
moved to its `capacity.rs`. Only imports and module wiring changed. Expected hashes,
refusals, counts, deadlines and adversarial fact mutations remain independently
specified. `make test-receiver-policy` runs all nineteen; the narrower
`make test-admission-capacity` runs the six capacity cases. Login-only selection is
`cargo test --locked -p ic-auth --lib receiver_policy::tests::login_`.

Two synthetic proof-to-facts qualification drivers were removed after mapping
all their assertions to the established policy/verifier tests and real receivers:

| Former driver or assertion | Retained evidence |
| --- | --- |
| `receiver_retry_uses_retained_proof_evidence_after_admission_expiry`: live retry/read after proof expiry; original deadline; refusal at lease expiry | `memory_admission_retry_read_and_exclusive_lease`; certified `memory_certified_accounts_devices_and_receivers_preserve_independent_lifecycles`; private literal retry/lease cases |
| Fresh proof verification refuses expired evidence | Existing verifier context/refusal cases and certified `common_context_and_certificate_bindings_refuse_without_observing_time` |
| `login_replacement_preflight_stays_separate_from_both_complete_proof_checks`: greater generation replaces a denied slot at full known capacity | Private `login_replacement_requires_new_browser_and_higher_generation_even_after_logout_or_expiry`; Ed25519 `memory_logout_pruning_and_device_fences_survive_expiry`; new certified `memory_certified_replacement_of_denied_slot_still_requires_complete_proof` |
| Invalid proof retains attempted-work charge and cannot change the selected identity/fence | Ed25519 `memory_full_pool_replacement_is_verified_before_displacement`, independent sequence model, certified full-pool and denied-slot replacement cases; existing component tests retain precise cryptographic refusal variants |
| Verified caller/claims reflect the independent signature fixture | Existing positive Ed25519 and complete certified verifier cases |

The added certified denied-slot case uses already independent signed fixtures,
not a new source of expected decisions. It checks failed root and issuer proofs
against real retained state, then successful displacement with unaffected devices.
It supplies stronger operation-boundary evidence for that scenario without
exposing internal policy to qualification. No installed-host rollback is inferred.

`make test-memory` keeps the public Ed25519 and certified operation tests, the
independent sequence model, publication faults, inspection limits and four
compile-fail cases. Two prevent borrowed authorization receipts escaping; two
prevent importing the private policy module/snapshot through the public API.
There is no test-only public feature. Standalone verification remains independently
usable without the receiver feature.

Contract provenance remains reference `e52a40be75dd9809d0528189d84b43fe96952ad9`,
particularly `session/mod.rs`, `session/model.rs`, `portable/revocation/mod.rs`
and `docs/protocol/sessions.md`, recorded in `reference.sha256`. The scenarios
were newly written from those contracts; moving them does not change their origin
or turn them into a complete differential or installed-host qualification suite.
