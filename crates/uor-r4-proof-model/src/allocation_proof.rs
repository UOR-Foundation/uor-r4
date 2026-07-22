//! Executable proof module: Zero-allocation step contract verification.

use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static ALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);

pub fn reset_alloc_counters() {
    ALLOC_COUNT.store(0, Ordering::SeqCst);
    ALLOC_BYTES.store(0, Ordering::SeqCst);
}

pub fn current_alloc_count() -> usize {
    ALLOC_COUNT.load(Ordering::SeqCst)
}

pub fn current_alloc_bytes() -> usize {
    ALLOC_BYTES.load(Ordering::SeqCst)
}

/// Verify that executing closure `f` performs zero heap allocations.
pub fn verify_zero_allocation<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce() -> R,
{
    let count_before = current_alloc_count();
    let bytes_before = current_alloc_bytes();
    let result = f();
    let count_after = current_alloc_count();
    let bytes_after = current_alloc_bytes();

    let diff_count = count_after.saturating_sub(count_before);
    let diff_bytes = bytes_after.saturating_sub(bytes_before);

    if diff_count != 0 {
        Err(format!(
            "Allocation proof failed: detected {} allocations ({} bytes)",
            diff_count, diff_bytes
        ))
    } else {
        Ok(result)
    }
}
