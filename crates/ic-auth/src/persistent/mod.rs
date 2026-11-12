//! Auth-owned indexed records in a host-provided governed allocation.
//!
//! No allocator, clock, endpoint, executor or certification root is installed.
//! Mutations require a synchronous host transaction: on IC, message rollback;
//! native hosts must supply equivalent atomicity and discard poisoned runtimes.
//! Returned refusals happen before record writes. Storage faults trap, never
//! acknowledge a partial transaction. Opening reads metadata, not all records.

pub mod issuer;
pub mod receiver;
mod records;

pub use records::StoreError;

/// Two fixed, exclusive host-governed leases: ordered rows and a small metadata
/// cell. Metadata reads/writes never traverse the growing record index. There
/// are no per-user allocations and Auth never registers its own allocator.
pub struct StoreMemory<M: ic_storage::Memory> {
    pub records: ic_storage::RuntimeMemory<M>,
    pub metadata: ic_storage::RuntimeMemory<M>,
}
