//! Executable proof module: Zero-allocation step contract verification.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    /// Counters are per-thread: libtest runs each test on its own thread, so
    /// a parallel test's allocations can never leak into another test's
    /// measured section (the previous process-global counters raced and made
    /// the harness flaky under parallel execution).
    static ALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
    static ALLOC_BYTES: Cell<usize> = const { Cell::new(0) };
}

struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_COUNT.with(|c| c.set(c.get() + 1));
        ALLOC_BYTES.with(|c| c.set(c.get() + layout.size()));
        // SAFETY: forwarding to the system allocator with the same layout.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: forwarding to the system allocator with the same layout.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

pub fn reset_alloc_counters() {
    ALLOC_COUNT.with(|c| c.set(0));
    ALLOC_BYTES.with(|c| c.set(0));
}

pub fn current_alloc_count() -> usize {
    ALLOC_COUNT.with(|c| c.get())
}

pub fn current_alloc_bytes() -> usize {
    ALLOC_BYTES.with(|c| c.get())
}

/// Verify that executing closure `f` performs zero heap allocations on the
/// calling thread. (Allocations performed by threads `f` may spawn are out of
/// scope — the step contract being proven is about the calling path.)
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
