//! Measure instrumentation cost with identical allocation work and explicit variant selection.
use rbench::{alloc::TrackingAllocator, error, Result, Suite};
use std::alloc::{GlobalAlloc, Layout, System};
fn operation<A: GlobalAlloc>(a: &A) {
    let layout = Layout::from_size_align(256, 8).unwrap();
    // SAFETY: same allocator/layout, initialized byte, pointer checked and freed exactly once.
    unsafe {
        let p = a.alloc(layout);
        assert!(!p.is_null());
        std::ptr::write_volatile(p, 42);
        std::hint::black_box(std::ptr::read_volatile(p));
        a.dealloc(p, layout);
    }
}
fn main() -> Result<()> {
    let mode = std::env::var("RBENCH_ALLOC_MODE").unwrap_or_else(|_| "system".into());
    static TRACKED: TrackingAllocator<System> = TrackingAllocator::new(System);
    let tracked = &TRACKED;
    let phase = if mode == "phase" {
        Some(tracked.begin_phase()?)
    } else {
        None
    };
    let mut s = Suite::new("allocator");
    match mode.as_str() {
        "system" => {
            s.bench("alloc_free_256", || operation(&System));
        }
        "tracked" | "phase" => {
            s.bench("alloc_free_256", || operation(tracked));
        }
        _ => return Err(error("RBENCH_ALLOC_MODE=system|tracked|phase")),
    }
    let result = s.main();
    drop(phase);
    result
}
