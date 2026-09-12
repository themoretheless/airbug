#![cfg(feature = "memory")]
use rbench::memory::{Allocator, Session};
use std::alloc::{GlobalAlloc, Layout, System};
struct RejectRealloc;
// SAFETY: forwards valid layouts/pointers; failure leaves original allocation intact.
unsafe impl GlobalAlloc for RejectRealloc {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        unsafe { System.alloc(l) }
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        unsafe { System.alloc_zeroed(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, _: *mut u8, _: Layout, _: usize) -> *mut u8 {
        std::ptr::null_mut()
    }
}
static A: Allocator<RejectRealloc> = Allocator(RejectRealloc);
#[test]
fn real_allocator_failure_zeroed_and_cross_thread_free() {
    let small = Layout::from_size_align(8, 8).unwrap();
    // Initialize the instrumented allocator before opening the session.
    unsafe {
        let p = A.alloc(small);
        assert!(!p.is_null());
        A.dealloc(p, small);
    }
    let session = Session::start().unwrap();
    unsafe {
        let p = A.alloc(small);
        assert!(!p.is_null());
        p.write(42);
        assert!(A.realloc(p, small, 32).is_null());
        assert_eq!(p.read(), 42);
        A.dealloc(p, small);
        let layout = Layout::from_size_align(32, 16).unwrap();
        let p = A.alloc_zeroed(layout);
        assert!(!p.is_null());
        assert_eq!(p as usize % 16, 0);
        assert!(std::slice::from_raw_parts(p, 32).iter().all(|b| *b == 0));
        let address = p as usize;
        std::thread::spawn(move || A.dealloc(address as *mut u8, layout))
            .join()
            .unwrap();
    }
    let profile = session.finish();
    assert_eq!(profile.dropped_events, 0);
    assert_eq!(
        profile.sites.iter().map(|s| s.allocated_bytes).sum::<u64>(),
        40
    );
    assert_eq!(profile.sites.iter().map(|s| s.live_bytes).sum::<u64>(), 0);
    assert_eq!(profile.peak_tracked_bytes, 32);
    assert!(Session::start().is_err());
}
