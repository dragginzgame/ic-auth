//! Shared native-only allocation observation; callers retain their assertions.
use serde_json::{Value, json};
use std::fmt::Debug;

pub(super) fn measure<T: Debug + Eq>(
    iterations: u16,
    mut operation: impl FnMut() -> T,
) -> (T, Value) {
    let expected = operation();
    let profiler = dhat::Profiler::builder()
        .testing()
        .trim_backtraces(Some(0))
        .build();
    for _ in 0..iterations {
        assert_eq!(operation(), expected);
    }
    let stats = dhat::HeapStats::get();
    dhat::assert_eq!(stats.curr_bytes, 0);
    drop(profiler);
    (
        expected,
        json!({
            "iterations": iterations,
            "total_blocks": stats.total_blocks,
            "total_bytes": stats.total_bytes,
            "peak_live_bytes": stats.max_bytes,
            "blocks_per_operation": stats.total_blocks as f64 / f64::from(iterations),
            "bytes_per_operation": stats.total_bytes as f64 / f64::from(iterations),
        }),
    )
}
