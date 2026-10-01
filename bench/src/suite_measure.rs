//! Hot-path batch helpers for Suite (kept separate from registration API).
use crate::suite::DropPolicy;
use crate::timer::{Timer, TimerStart};
use crate::timing::Measured;
use std::hint::black_box;

pub(crate) fn input_lifecycle(drop: DropPolicy) -> &'static str {
    match drop {
        DropPolicy::InsideTiming => {
            "fresh input per operation; input setup/drop excluded; output drop included; chunks of <=64"
        }
        DropPolicy::OutsideTiming => {
            "fresh input per operation; input setup/drop and output drop excluded; chunks of <=64"
        }
    }
}

pub(crate) fn input_batch<I, O>(
    timer: Timer,
    n: u64,
    setup: &mut impl FnMut() -> I,
    f: &mut impl FnMut(&mut I) -> O,
    drop: DropPolicy,
) -> crate::Result<Measured> {
    input_batch_sized(timer, n, setup, f, drop, 64)
}

pub(crate) fn worker_buffer<T>(workers: usize) -> crate::Result<Vec<T>> {
    let mut values = Vec::new();
    values.try_reserve_exact(workers).map_err(|e| {
        crate::error(format!(
            "cannot allocate worker buffer for {workers} workers: {e}"
        ))
    })?;
    Ok(values)
}

/// Check addressability before invoking any input factory or worker. Heap
/// allocations owned by user values remain the caller's responsibility.
pub(crate) fn validate_batch_capacity<I, O>(count: u64) -> crate::Result<()> {
    let count = usize::try_from(count)
        .map_err(|_| crate::error("batch size exceeds addressable memory"))?;
    std::alloc::Layout::array::<I>(count)
        .map_err(|_| crate::error("batch input capacity exceeds addressable memory"))?;
    std::alloc::Layout::array::<O>(count)
        .map_err(|_| crate::error("batch output capacity exceeds addressable memory"))?;
    Ok(())
}

pub(crate) fn batch_buffers<I, O>(count: u64, drop: DropPolicy) -> crate::Result<(Vec<I>, Vec<O>)> {
    let capacity = usize::try_from(count)
        .map_err(|_| crate::error("batch size exceeds addressable memory"))?;
    let mut inputs = Vec::new();
    inputs.try_reserve_exact(capacity).map_err(|e| {
        crate::error(format!(
            "cannot allocate batch input buffer for {count} operations: {e}"
        ))
    })?;
    let mut outputs = Vec::new();
    if matches!(drop, DropPolicy::OutsideTiming) {
        outputs.try_reserve_exact(capacity).map_err(|e| {
            crate::error(format!(
                "cannot allocate batch output buffer for {count} operations: {e}"
            ))
        })?;
    }
    Ok((inputs, outputs))
}

pub(crate) fn input_batch_sized<I, O>(
    timer: Timer,
    n: u64,
    setup: &mut impl FnMut() -> I,
    f: &mut impl FnMut(&mut I) -> O,
    drop: DropPolicy,
    batch_size: u64,
) -> crate::Result<Measured> {
    if batch_size == 0 {
        return Err(crate::error("batch size must be positive"));
    }
    let mut remaining = n;
    let mut total = Measured::default();
    while remaining > 0 {
        let count = remaining.min(batch_size);
        let (mut inputs, mut outputs) = batch_buffers::<I, O>(count, drop)?;
        inputs.extend((0..count).map(|_| setup()));
        let start = timer.start();
        for input in &mut inputs {
            match drop {
                DropPolicy::InsideTiming => {
                    black_box(f(black_box(input)));
                }
                DropPolicy::OutsideTiming => outputs.push(black_box(f(black_box(input)))),
            }
        }
        total = total.add(start.measure(count)?)?;
        black_box(&outputs);
        remaining -= count;
    }
    Ok(total)
}

// One pool per coordinating thread, reused across calibration, warmup and samples.
// Replacing the worker count replaces the pool; nested work on a worker gets its
// own pool, so blocking rendezvous never occupy the pool they are waiting on.
thread_local! {
    static WORKER_POOL: std::cell::RefCell<Option<crate::worker_pool::Pool>> = const { std::cell::RefCell::new(None) };
}

fn broadcast<R: Send>(workers: usize, work: impl Fn(usize) -> R + Sync) -> crate::Result<Vec<R>> {
    WORKER_POOL.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.as_ref().is_none_or(|pool| pool.len() != workers) {
            *slot = Some(crate::worker_pool::Pool::new(workers)?);
        }
        slot.as_mut().unwrap().broadcast(work)
    })
}

/// Prepare all inputs before dispatch so a setup panic cannot strand waiting workers.
/// Pool creation completes before any callback is dispatched.
pub(crate) fn parallel_batch<I: Send>(
    timer: Timer,
    workers: usize,
    prepare: impl FnMut() -> I,
    run: impl Fn(&mut I, TimerStart) -> crate::Result<Measured> + Sync,
) -> crate::Result<Measured> {
    parallel_batch_recorded(timer, workers, prepare, run, None)
}

/// Optional raw/adjusted intervals in worker-slot order. The destination is
/// cleared before work and replaced only after every worker succeeds. Intervals
/// use the selected shared or worker-local start boundary.
pub(crate) fn parallel_batch_recorded<
    I: Send,
    P: FnMut() -> I,
    F: Fn(&mut I, TimerStart) -> crate::Result<Measured> + Sync,
>(
    timer: Timer,
    workers: usize,
    mut prepare: P,
    run: F,
    mut records: Option<&mut Vec<Measured>>,
) -> crate::Result<Measured> {
    use std::sync::{Mutex, OnceLock};
    if let Some(records) = records.as_mut() {
        records.clear();
    }
    if workers == 0 {
        return Err(crate::error("workers must be positive"));
    }
    let mut samples = if records.is_some() {
        worker_buffer(workers)?
    } else {
        Vec::new()
    };
    if workers == 1 {
        let _worker = crate::counters::WorkerScope::enter(0);
        let mut input = prepare();
        let elapsed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run(&mut input, timer.start())
        }))
        .map_err(|_| crate::error("benchmark worker panicked"))??;
        if let Some(records) = records {
            samples.push(elapsed);
            *records = samples;
        }
        return Ok(elapsed);
    }
    let ready = std::sync::Barrier::new(workers);
    let start = OnceLock::new();
    let mut inputs = worker_buffer(workers)?;
    inputs.extend((0..workers).map(|worker| {
        let _worker = crate::counters::WorkerScope::enter(worker as u64);
        Mutex::new(Some(prepare()))
    }));
    let completed = broadcast(workers, |worker| {
        let _worker = crate::counters::WorkerScope::enter(worker as u64);
        let mut input = inputs[worker].lock().unwrap().take().unwrap();
        // All setup is finished before a shared timer is created. A second
        // rendezvous publishes that timestamp to every worker.
        if ready.wait().is_leader() {
            start.set(timer.start()).unwrap();
        }
        ready.wait();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run(&mut input, timer.start_worker(*start.get().unwrap()))
        }));
        (result, input)
    })?;
    let mut elapsed = Measured::default();
    let mut failed = false;
    let mut measurement_error = None;
    // Keep all inputs alive until every timer has stopped, then drop them on
    // the coordinator, including when one operation failed.
    for (result, _) in &completed {
        if let Ok(Ok(duration)) = result {
            elapsed = elapsed.max(*duration);
            if records.is_some() {
                samples.push(*duration);
            }
        }
    }
    for (result, _input) in completed {
        match result {
            Ok(Ok(_)) => {}
            Ok(Err(error)) => {
                measurement_error.get_or_insert(error);
            }
            Err(_) => failed = true,
        }
    }
    if let Some(error) = measurement_error {
        return Err(error);
    }
    if failed {
        return Err(crate::error("benchmark worker panicked"));
    }
    if let Some(records) = records {
        *records = samples;
    }
    Ok(elapsed)
}

/// Worker-owned inputs never cross thread boundaries. Both rendezvous are
/// released on user panics; retained values drop only after all timers stop.
pub(crate) fn parallel_local_batch<I>(
    timer: Timer,
    workers: usize,
    prepare: impl Fn() -> I + Sync,
    run: impl Fn(&mut I, TimerStart) -> crate::Result<Measured> + Sync,
) -> crate::Result<Measured> {
    parallel_local_batch_recorded(timer, workers, prepare, run, None)
}

/// Record worker intervals without transferring worker-local inputs or outputs.
/// Publication happens only after all worker callbacks have completed successfully.
pub(crate) fn parallel_local_batch_recorded<I>(
    timer: Timer,
    workers: usize,
    prepare: impl Fn() -> I + Sync,
    run: impl Fn(&mut I, TimerStart) -> crate::Result<Measured> + Sync,
    mut records: Option<&mut Vec<Measured>>,
) -> crate::Result<Measured> {
    use std::{
        panic::{AssertUnwindSafe, catch_unwind},
        sync::OnceLock,
    };
    if let Some(records) = records.as_mut() {
        records.clear();
    }
    if workers == 0 {
        return Err(crate::error("workers must be positive"));
    }
    let mut samples = if records.is_some() {
        worker_buffer(workers)?
    } else {
        Vec::new()
    };
    if workers == 1 {
        let _worker = crate::counters::WorkerScope::enter(0);
        let elapsed = catch_unwind(AssertUnwindSafe(|| {
            let mut input = prepare();
            run(&mut input, timer.start())
        }))
        .map_err(|_| crate::error("benchmark worker setup or operation panicked"))??;
        if let Some(records) = records {
            samples.push(elapsed);
            *records = samples;
        }
        return Ok(elapsed);
    }
    let ready = std::sync::Barrier::new(workers);
    let finished = std::sync::Barrier::new(workers);
    let setup_failed = std::sync::atomic::AtomicBool::new(false);
    let start = OnceLock::new();
    let completed = broadcast(workers, |worker| {
        let _worker = crate::counters::WorkerScope::enter(worker as u64);
        let input = catch_unwind(AssertUnwindSafe(&prepare));
        if input.is_err() {
            setup_failed.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        if ready.wait().is_leader() {
            start.set(timer.start()).unwrap();
        }
        ready.wait();
        if setup_failed.load(std::sync::atomic::Ordering::Relaxed) {
            return None;
        }
        let mut input = input.unwrap();
        let result = catch_unwind(AssertUnwindSafe(|| {
            run(&mut input, timer.start_worker(*start.get().unwrap()))
        }));
        finished.wait();
        // Local inputs/outputs, including !Send values, are destroyed on their
        // original worker after every measured callback finishes.
        Some(result)
    })?;
    let mut elapsed = Measured::default();
    let mut failed = setup_failed.load(std::sync::atomic::Ordering::Relaxed);
    let mut measurement_error = None;
    for result in completed {
        match result {
            Some(Ok(Ok(duration))) => {
                elapsed = elapsed.max(duration);
                if records.is_some() {
                    samples.push(duration);
                }
            }
            Some(Ok(Err(error))) => {
                measurement_error.get_or_insert(error);
            }
            None => {}
            _ => failed = true,
        }
    }
    if let Some(error) = measurement_error {
        return Err(error);
    }
    if failed {
        return Err(crate::error("benchmark worker setup or operation panicked"));
    }
    if let Some(records) = records {
        *records = samples;
    }
    Ok(elapsed)
}

/// Move every fresh input into the operation exactly once. Setup is excluded;
/// destruction of consumed inputs belongs to the operation and cannot be deferred by us.
pub(crate) fn owned_input_batch<I, O>(
    timer: Timer,
    n: u64,
    setup: &mut impl FnMut() -> I,
    f: &mut impl FnMut(I) -> O,
    drop: DropPolicy,
) -> crate::Result<Measured> {
    owned_input_batch_sized(timer, n, setup, f, drop, 64)
}

pub(crate) fn owned_input_batch_sized<I, O>(
    timer: Timer,
    n: u64,
    setup: &mut impl FnMut() -> I,
    f: &mut impl FnMut(I) -> O,
    drop: DropPolicy,
    batch_size: u64,
) -> crate::Result<Measured> {
    if batch_size == 0 {
        return Err(crate::error("batch size must be positive"));
    }
    let mut remaining = n;
    let mut total = Measured::default();
    while remaining > 0 {
        let count = remaining.min(batch_size);
        let (mut inputs, mut outputs) = batch_buffers::<I, O>(count, drop)?;
        inputs.extend((0..count).map(|_| setup()));
        let start = timer.start();
        for input in inputs {
            match drop {
                DropPolicy::InsideTiming => {
                    black_box(f(black_box(input)));
                }
                DropPolicy::OutsideTiming => outputs.push(black_box(f(black_box(input)))),
            }
        }
        total = total.add(start.measure(count)?)?;
        black_box(&outputs);
        remaining -= count;
    }
    Ok(total)
}

#[cfg(test)]
mod batch_tests {
    use super::*;

    #[test]
    fn pool_reuses_slots_after_panic_and_rejects_impossible_counts() {
        let first = broadcast(2, |_| std::thread::current().id()).unwrap();
        let error = broadcast(2, |worker| {
            assert_ne!(worker, 0, "intentional worker failure");
        })
        .unwrap_err();
        assert!(error.to_string().contains("panicked"));
        let second = broadcast(2, |_| std::thread::current().id()).unwrap();
        assert_eq!(first, second);
        let error = broadcast(usize::MAX, |_| unreachable!()).unwrap_err();
        assert!(error.to_string().contains("cannot allocate worker buffer"));
        let resized = broadcast(3, |_| std::thread::current().id()).unwrap();
        assert_eq!(resized.len(), 3);
        assert!(resized.iter().all(|id| !first.contains(id)));
    }

    #[test]
    fn local_drop_panic_returns_error_and_pool_remains_usable() {
        struct BadDrop;
        impl Drop for BadDrop {
            fn drop(&mut self) {
                panic!("intentional drop failure");
            }
        }
        let error =
            parallel_local_batch(Timer::default(), 2, || BadDrop, |_, _| Ok(Measured::raw(1)))
                .unwrap_err();
        assert!(error.to_string().contains("panicked"));
        assert_eq!(
            parallel_local_batch(Timer::default(), 2, || (), |_, _| Ok(Measured::raw(7)),)
                .unwrap()
                .raw_ns,
            7
        );
    }

    use crate::BatchPolicy;
    use std::time::Instant;
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn worker_capture_keeps_every_interval_and_publishes_only_complete_batches() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        for local in [false, true] {
            for workers in [1, 3] {
                let next = AtomicUsize::new(0);
                let setup = || next.fetch_add(1, Ordering::SeqCst) + 1;
                let run = |n: &mut usize, _: TimerStart| {
                    Ok(Measured {
                        raw_ns: *n as u128 * 10,
                        adjusted_ns: *n as u128 * 5,
                    })
                };
                let mut records = vec![Measured::raw(999)];
                let elapsed = if local {
                    parallel_local_batch_recorded(
                        Timer::default(),
                        workers,
                        setup,
                        run,
                        Some(&mut records),
                    )
                } else {
                    parallel_batch_recorded(
                        Timer::default(),
                        workers,
                        setup,
                        run,
                        Some(&mut records),
                    )
                }
                .unwrap();
                assert_eq!(elapsed.raw_ns, workers as u128 * 10);
                assert_eq!(elapsed.adjusted_ns, workers as u128 * 5);
                let mut values: Vec<_> =
                    records.iter().map(|m| (m.raw_ns, m.adjusted_ns)).collect();
                values.sort_unstable();
                assert_eq!(
                    values,
                    (1..=workers)
                        .map(|n| (n as u128 * 10, n as u128 * 5))
                        .collect::<Vec<_>>()
                );
                let failure = |_: &mut (), _: TimerStart| -> crate::Result<Measured> {
                    Err(crate::error("measurement failed"))
                };
                let result = if local {
                    parallel_local_batch_recorded(
                        Timer::default(),
                        workers,
                        || (),
                        failure,
                        Some(&mut records),
                    )
                } else {
                    parallel_batch_recorded(
                        Timer::default(),
                        workers,
                        || (),
                        failure,
                        Some(&mut records),
                    )
                };
                assert!(result.is_err());
                assert!(records.is_empty(), "failure must clear previous records");
            }
        }
    }

    #[test]
    fn worker_batches_can_execute_above_old_ceiling() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let calls = AtomicUsize::new(0);
        let run = |_: &mut (), start: TimerStart| {
            calls.fetch_add(1, Ordering::Relaxed);
            start.measure(1)
        };
        parallel_batch(Timer::default(), 257, || (), run).unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 257);
        parallel_local_batch(Timer::default(), 257, || (), run).unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 514);
        let result = crate::workloads::parallel(257, |index| index).unwrap();
        assert_eq!(result.outputs, (0..257).collect::<Vec<_>>());
    }

    #[test]
    fn extreme_worker_requests_fail_before_setup_or_work() {
        let result = parallel_batch::<()>(
            Timer::default(),
            usize::MAX,
            || panic!("setup must not run"),
            |_, _| unreachable!(),
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cannot allocate worker buffer")
        );
        let result = parallel_local_batch::<u64>(
            Timer::default(),
            usize::MAX,
            || panic!("setup must not run"),
            |_, _| unreachable!(),
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cannot allocate worker buffer")
        );
        let result = crate::workloads::parallel::<u64>(usize::MAX, |_| panic!("work must not run"));
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cannot allocate worker buffer")
        );
    }

    #[test]
    fn worker_buffer_capacity_errors_are_returned() {
        assert!(
            worker_buffer::<u64>(usize::MAX)
                .unwrap_err()
                .to_string()
                .contains("cannot allocate worker buffer")
        );
        let values = worker_buffer::<u64>(4).unwrap();
        assert!(values.is_empty());
        assert!(values.capacity() >= 4);
    }

    #[test]
    fn single_worker_uses_caller_thread_and_preserves_error_behavior() {
        let caller = std::thread::current().id();
        for local in [false, true] {
            let prepare = || {
                assert_eq!(std::thread::current().id(), caller);
                7
            };
            let run = |input: &mut i32, start: TimerStart| {
                assert_eq!(std::thread::current().id(), caller);
                assert_eq!(*input, 7);
                start.measure(1)
            };
            if local {
                parallel_local_batch(Timer::default(), 1, prepare, run).unwrap();
                assert!(
                    parallel_local_batch::<()>(
                        Timer::default(),
                        1,
                        || panic!("setup"),
                        |_, _| unreachable!()
                    )
                    .is_err()
                );
            } else {
                parallel_batch(Timer::default(), 1, prepare, run).unwrap();
            }
            let failure =
                |_: &mut (), _: TimerStart| -> crate::Result<Measured> { panic!("operation") };
            let result = if local {
                parallel_local_batch(Timer::default(), 1, || (), failure)
            } else {
                parallel_batch(Timer::default(), 1, || (), failure)
            };
            assert!(result.unwrap_err().to_string().contains("panicked"));
        }
    }

    #[test]
    fn impossible_buffers_fail_before_setup_in_both_input_modes() {
        for owned in [false, true] {
            let error = if owned {
                owned_input_batch_sized::<u64, u64>(
                    Timer::default(),
                    u64::MAX,
                    &mut || panic!("setup must not run"),
                    &mut |_| 0,
                    DropPolicy::OutsideTiming,
                    u64::MAX,
                )
            } else {
                input_batch_sized::<u64, u64>(
                    Timer::default(),
                    u64::MAX,
                    &mut || panic!("setup must not run"),
                    &mut |_| 0,
                    DropPolicy::OutsideTiming,
                    u64::MAX,
                )
            }
            .unwrap_err();
            assert!(error.to_string().contains("batch input buffer"));
        }
        assert!(
            batch_buffers::<(), u64>(u64::MAX, DropPolicy::OutsideTiming)
                .unwrap_err()
                .to_string()
                .contains("batch output buffer")
        );
        // Outputs dropped inside timing need no retained output buffer.
        assert!(
            batch_buffers::<(), u64>(1, DropPolicy::InsideTiming)
                .unwrap()
                .1
                .capacity()
                == 0
        );
    }

    struct Value(Rc<RefCell<Vec<&'static str>>>, &'static str);
    impl Drop for Value {
        fn drop(&mut self) {
            self.0.borrow_mut().push(self.1);
        }
    }
    #[test]
    fn output_destructor_intervals_are_inside_or_outside_the_reported_duration() {
        use std::{cell::Cell, time::Duration};
        struct SlowDrop<'a>(&'a Cell<u128>);
        impl Drop for SlowDrop<'_> {
            fn drop(&mut self) {
                let start = Instant::now();
                std::thread::sleep(Duration::from_millis(2));
                self.0.set(self.0.get() + start.elapsed().as_nanos());
            }
        }
        for owned in [false, true] {
            for policy in [DropPolicy::InsideTiming, DropPolicy::OutsideTiming] {
                let destruction = Cell::new(0);
                let start = Instant::now();
                let measured = if owned {
                    owned_input_batch_sized(
                        Timer::os(),
                        5,
                        &mut || (),
                        &mut |_| SlowDrop(&destruction),
                        policy,
                        2,
                    )
                } else {
                    input_batch_sized(
                        Timer::os(),
                        5,
                        &mut || (),
                        &mut |_| SlowDrop(&destruction),
                        policy,
                        2,
                    )
                }
                .unwrap();
                let measured = measured.raw_ns;
                let wall = start.elapsed().as_nanos();
                let drops = destruction.get();
                assert!(drops >= Duration::from_millis(10).as_nanos());
                match policy {
                    DropPolicy::InsideTiming => assert!(
                        measured >= drops,
                        "destructors must be nested within measured intervals: {measured} < {drops}"
                    ),
                    DropPolicy::OutsideTiming => assert!(
                        wall >= measured + drops,
                        "destructor and measurement intervals must not overlap: {wall} < {measured} + {drops}"
                    ),
                }
            }
        }
    }
    #[test]
    fn panic_drops_retained_outputs_and_every_prepared_input_once() {
        use std::cell::Cell;
        struct CountDrop<'a>(&'a Cell<usize>);
        impl Drop for CountDrop<'_> {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        for owned in [false, true] {
            let input_drops = Cell::new(0);
            let output_drops = Cell::new(0);
            let calls = Cell::new(0);
            let mut setup = || CountDrop(&input_drops);
            let operation = || {
                calls.set(calls.get() + 1);
                assert_ne!(calls.get(), 3, "intentional workload panic");
                CountDrop(&output_drops)
            };
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if owned {
                    owned_input_batch_sized(
                        Timer::os(),
                        9,
                        &mut setup,
                        &mut |_| operation(),
                        DropPolicy::OutsideTiming,
                        5,
                    )
                } else {
                    input_batch_sized(
                        Timer::os(),
                        9,
                        &mut setup,
                        &mut |_| operation(),
                        DropPolicy::OutsideTiming,
                        5,
                    )
                }
            }));
            assert!(result.is_err());
            assert_eq!(calls.get(), 3);
            assert_eq!(
                input_drops.get(),
                5,
                "also drop inputs whose operation never started"
            );
            assert_eq!(
                output_drops.get(),
                2,
                "drop exactly the outputs created before failure"
            );
        }
    }
    #[test]
    fn explicit_batches_can_exceed_the_default_limit() {
        use std::cell::Cell;
        let setups = Cell::new(0);
        let calls = Cell::new(0);
        input_batch_sized(
            Timer::os(),
            130,
            &mut || {
                setups.set(setups.get() + 1);
            },
            &mut |_| {
                let called = calls.get();
                assert_eq!(setups.get(), if called < 100 { 100 } else { 130 });
                calls.set(called + 1);
            },
            DropPolicy::OutsideTiming,
            100,
        )
        .unwrap();
        assert_eq!(calls.get(), 130);
    }
    #[test]
    fn policies_preserve_remainders_and_lifetimes() {
        for owned in [false, true] {
            for drop in [DropPolicy::InsideTiming, DropPolicy::OutsideTiming] {
                let events = Rc::new(RefCell::new(Vec::new()));
                let mut setup = || {
                    events.borrow_mut().push("setup");
                    Value(events.clone(), "input drop")
                };
                let operation = || {
                    events.borrow_mut().push("call");
                    Value(events.clone(), "output drop")
                };
                if owned {
                    owned_input_batch_sized(
                        Timer::os(),
                        5,
                        &mut setup,
                        &mut |_| operation(),
                        drop,
                        3,
                    )
                    .unwrap();
                } else {
                    input_batch_sized(Timer::os(), 5, &mut setup, &mut |_| operation(), drop, 3)
                        .unwrap();
                }
                let mut expected = vec![];
                for count in [3, 2] {
                    expected.extend(std::iter::repeat_n("setup", count));
                    for _ in 0..count {
                        expected.push("call");
                        if owned {
                            expected.push("input drop");
                        }
                        if matches!(drop, DropPolicy::InsideTiming) {
                            expected.push("output drop");
                        }
                    }
                    if matches!(drop, DropPolicy::OutsideTiming) {
                        expected.extend(std::iter::repeat_n("output drop", count));
                    }
                    if !owned {
                        expected.extend(std::iter::repeat_n("input drop", count));
                    }
                }
                assert_eq!(*events.borrow(), expected);
            }
        }
        assert_eq!(BatchPolicy::SmallInput.size(101), 11);
        assert_eq!(BatchPolicy::LargeInput.size(1001), 2);
        assert_eq!(BatchPolicy::PerIteration.size(1000), 1);
        assert_eq!(BatchPolicy::Batches(3.try_into().unwrap()).size(10), 4);
        assert_eq!(
            BatchPolicy::Iterations(100.try_into().unwrap()).size(5),
            100
        );
        assert_eq!(
            BatchPolicy::Batches(u64::MAX.try_into().unwrap()).size(u64::MAX),
            1
        );
    }
}

#[cfg(test)]
mod local_worker_tests {
    use super::*;
    use std::{
        rc::Rc,
        sync::atomic::{AtomicUsize, Ordering},
    };
    #[test]
    fn local_values_drop_on_origin_thread_after_all_workers_complete() {
        struct Local<'a> {
            _not_send: Rc<()>,
            origin: std::thread::ThreadId,
            completed: &'a AtomicUsize,
            drops: &'a AtomicUsize,
        }
        impl Drop for Local<'_> {
            fn drop(&mut self) {
                assert_eq!(self.origin, std::thread::current().id());
                assert_eq!(self.completed.load(Ordering::SeqCst), 4);
                self.drops.fetch_add(1, Ordering::SeqCst);
            }
        }
        let completed = AtomicUsize::new(0);
        let drops = AtomicUsize::new(0);
        let coordinator = std::thread::current().id();
        let duration = parallel_local_batch(
            Timer::os(),
            4,
            || {
                assert_ne!(coordinator, std::thread::current().id());
                Local {
                    _not_send: Rc::new(()),
                    origin: std::thread::current().id(),
                    completed: &completed,
                    drops: &drops,
                }
            },
            |local, _| {
                assert_eq!(local.origin, std::thread::current().id());
                completed.fetch_add(1, Ordering::SeqCst);
                Ok(Measured::raw(123))
            },
        )
        .unwrap();
        assert_eq!(duration.raw_ns, 123);
        assert_eq!(drops.load(Ordering::SeqCst), 4);
    }
    #[test]
    fn worker_setup_and_operation_panics_release_peers() {
        let (tx, rx) = std::sync::mpsc::channel();
        let handle = std::thread::spawn(move || {
            for setup_fails in [true, false] {
                let initialized = AtomicUsize::new(0);
                let calls = AtomicUsize::new(0);
                let result = parallel_local_batch(
                    Timer::os(),
                    4,
                    || {
                        let id = initialized.fetch_add(1, Ordering::SeqCst);
                        assert!(!(setup_fails && id == 0), "setup failure");
                        Rc::new(id)
                    },
                    |id, _| {
                        calls.fetch_add(1, Ordering::SeqCst);
                        assert!(**id != 0, "operation failure");
                        Ok(Measured::raw(1))
                    },
                );
                assert!(result.is_err());
                assert_eq!(initialized.load(Ordering::SeqCst), 4);
                assert_eq!(
                    calls.load(Ordering::SeqCst),
                    if setup_fails { 0 } else { 4 }
                );
            }
            tx.send(()).unwrap();
        });
        rx.recv_timeout(std::time::Duration::from_secs(5))
            .expect("workers must not deadlock on user panic");
        handle.join().unwrap();
        assert!(
            parallel_local_batch(
                Timer::os(),
                0,
                || panic!("must not prepare"),
                |_: &mut (), _| Ok(Measured::raw(0))
            )
            .is_err()
        );
    }
}
