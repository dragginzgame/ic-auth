//! Native index prototype, not production authorization or a Wasm benchmark.
//! Payloads, crypto, persistence and replay are deliberately outside this model.
use clap::Parser;
use ic_auth_protocol_types::Principal;
use std::{collections::BTreeMap, hint::black_box, time::Instant};

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

#[derive(Parser)]
struct Arguments {
    #[arg(long = "bench", hide = true)]
    _bench: bool,
}

#[derive(Clone, Copy)]
struct Slot {
    generation: u64,
    revoked: bool,
}

fn principal(n: u64) -> Principal {
    let mut bytes = [0_u8; 29];
    bytes[..8].copy_from_slice(&n.to_be_bytes());
    bytes[28] = 2;
    Principal::from_slice(&bytes)
}

fn median(mut operation: impl FnMut(usize)) -> u128 {
    let mut samples = [0; 7];
    for sample in &mut samples {
        let start = Instant::now();
        for n in 0..100_000 {
            operation(n);
        }
        *sample = start.elapsed().as_nanos() / 100_000;
    }
    samples.sort_unstable();
    samples[3]
}

fn main() {
    let _args = Arguments::parse();
    for count in [1_000_usize, 10_000, 100_000] {
        let profiler = dhat::Profiler::builder().testing().build();
        let issuer = principal(u64::MAX);
        let before = dhat::HeapStats::get().curr_bytes;
        // Identical caller index in both paths. Real session payload is omitted.
        let callers: BTreeMap<_, _> = (0..count * 8)
            .map(|n| (principal(n as u64), (principal((n / 8) as u64), n % 8)))
            .collect();
        let caller_bytes = dhat::HeapStats::get().curr_bytes - before;
        let before = dhat::HeapStats::get().curr_bytes;
        let mut accounts: BTreeMap<_, _> = (0..count)
            .map(|n| {
                (
                    (issuer, principal(n as u64)),
                    [Slot {
                        generation: 1,
                        revoked: false,
                    }; 8],
                )
            })
            .collect();
        let account_bytes = dhat::HeapStats::get().curr_bytes - before;
        let lookup = median(|n| {
            let key = principal(((n * 7919) % (count * 8)) as u64);
            black_box(callers.get(black_box(&key)).unwrap());
        });
        let checked = median(|n| {
            let key = principal(((n * 7919) % (count * 8)) as u64);
            let (account, slot) = callers.get(black_box(&key)).unwrap();
            let state = accounts.get(black_box(&(issuer, *account))).unwrap()[*slot];
            assert!(black_box(state.generation == 1 && !state.revoked));
        });
        let replace = median(|n| {
            let account = principal(((n * 7919) % count) as u64);
            let slot = &mut accounts.get_mut(black_box(&(issuer, account))).unwrap()[n % 8];
            slot.generation = slot.generation.checked_add(1).unwrap();
            slot.revoked = false;
            black_box(slot);
        });
        println!(
            "accounts={count} devices={} caller_index_bytes={caller_bytes} additional_slot_index_bytes={account_bytes} caller_lookup_ns={lookup} lookup_plus_slot_ns={checked} slot_update_ns={replace} slot_struct_bytes={}",
            count * 8,
            size_of::<Slot>()
        );
        drop(accounts);
        drop(callers);
        drop(profiler);
    }
}
