use airbug_bench::alloc::TrackingAllocator;
use std::alloc::{GlobalAlloc, Layout, System};

#[test]
fn thread_phases_isolate_workers_and_handle_cross_thread_frees() {
    let allocator = TrackingAllocator::new(System);
    let other = TrackingAllocator::new(System);
    unsafe {
        let layout = Layout::from_size_align(64, 8).unwrap();
        let pointer = allocator.alloc(layout);
        assert!(!pointer.is_null());
        let address = pointer as usize;
        let phase = allocator.begin_thread_phase().unwrap();
        let worker = std::thread::scope(|scope| {
            let handle = scope.spawn(|| {
                let phase = allocator.begin_thread_phase().unwrap();
                allocator.dealloc(address as *mut u8, layout);
                let p = allocator.alloc(layout);
                assert!(!p.is_null());
                let p = allocator.realloc(p, layout, 128);
                assert!(!p.is_null());
                let p = allocator.realloc(p, Layout::from_size_align(128, 8).unwrap(), 32);
                assert!(!p.is_null());
                allocator.dealloc(p, Layout::from_size_align(32, 8).unwrap());
                phase.finish()
            });
            let small = Layout::from_size_align(16, 8).unwrap();
            let p = allocator.alloc_zeroed(small);
            assert!(!p.is_null());
            allocator.dealloc(p, small);
            let p = other.alloc(small);
            assert!(!p.is_null());
            other.dealloc(p, small);
            handle.join().unwrap()
        });
        let parent = phase.finish();
        assert_eq!(
            (
                parent.allocations,
                parent.deallocations,
                parent.reallocations
            ),
            (1, 1, 0)
        );
        assert_eq!(parent.allocated_bytes, 16);
        assert_eq!(parent.peak_above_start_bytes, 16);
        assert_eq!(parent.net_live_bytes, 0);
        assert_eq!(parent.net_live_count, 0);
        assert_eq!(parent.peak_above_start_count, 1);
        assert_eq!(
            (
                worker.allocations,
                worker.deallocations,
                worker.reallocations
            ),
            (1, 2, 2)
        );
        assert_eq!((worker.grow_operations, worker.shrink_operations), (1, 1));
        assert_eq!((worker.grown_bytes, worker.shrunk_bytes), (64, 96));
        assert_eq!(worker.net_live_bytes, -64);
        assert_eq!(worker.net_live_count, -1);
        assert_eq!(worker.peak_above_start_count, 0);
        assert_eq!(worker.peak_above_start_bytes, 64);
        assert!(!parent.overflowed && !worker.overflowed);
        assert_eq!(allocator.snapshot().live_bytes, 0);
    }
}

#[test]
fn nested_phases_reject_and_unwind_releases_the_thread_slot() {
    let allocator = TrackingAllocator::new(System);
    let phase = allocator.begin_thread_phase().unwrap();
    assert!(allocator.begin_thread_phase().is_err());
    drop(phase);
    let result = std::panic::catch_unwind(|| {
        let _phase = allocator.begin_thread_phase().unwrap();
        panic!("intentional panic");
    });
    assert!(result.is_err());
    assert_eq!(
        allocator.begin_thread_phase().unwrap().finish(),
        Default::default()
    );
}

#[test]
fn successful_same_size_realloc_is_zero_byte_growth_in_all_accounting_scopes() {
    let allocator = TrackingAllocator::new(System);
    unsafe {
        let layout = Layout::from_size_align(32, 8).unwrap();
        let pointer = allocator.alloc(layout);
        assert!(!pointer.is_null());
        let phase = allocator.begin_phase().unwrap();
        let thread = allocator.begin_thread_phase().unwrap();
        let pointer = allocator.realloc(pointer, layout, layout.size());
        assert!(!pointer.is_null());
        let thread = thread.finish();
        let phase = phase.finish();
        let snapshot = allocator.snapshot();
        assert_eq!(
            (
                thread.reallocations,
                thread.grow_operations,
                thread.shrink_operations
            ),
            (1, 1, 0)
        );
        assert_eq!((thread.grown_bytes, thread.shrunk_bytes), (0, 0));
        assert_eq!((thread.net_live_count, thread.net_live_bytes), (0, 0));
        assert_eq!(
            (thread.peak_above_start_count, thread.peak_above_start_bytes),
            (0, 0)
        );
        assert_eq!(
            (
                phase.reallocations,
                phase.grow_operations,
                phase.shrink_operations
            ),
            (1, 1, 0)
        );
        assert_eq!((phase.grown_bytes, phase.shrunk_bytes), (0, 0));
        assert_eq!((phase.live_start_bytes, phase.live_end_bytes), (32, 32));
        assert_eq!(phase.peak_above_start_bytes, 0);
        assert_eq!(
            (
                snapshot.reallocations,
                snapshot.grow_operations,
                snapshot.shrink_operations
            ),
            (1, 1, 0)
        );
        assert_eq!((snapshot.grown_bytes, snapshot.shrunk_bytes), (0, 0));
        assert_eq!(snapshot.live_bytes, 32);
        allocator.dealloc(pointer, layout);
        assert_eq!(allocator.snapshot().live_bytes, 0);
    }
}
