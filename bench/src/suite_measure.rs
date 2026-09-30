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

pub(crate) fn input_batch_sized<I, O>(
    timer: Timer,
    n: u64,
    setup: &mut impl FnMut() -> I,
    f: &mut impl FnMut(&mut I) -> O,
    drop: DropPolicy,
    batch_size: u64,
) -> crate::Result<Measured> {
    let mut remaining = n;
    let mut total = Measured::default();
    while remaining > 0 {
        let count = remaining.min(batch_size);
        let mut inputs: Vec<I> = (0..count).map(|_| setup()).collect();
        let mut outputs = Vec::with_capacity(count as usize);
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

/// Prepare all inputs before spawning so a setup panic cannot strand waiting workers.
/// Failed spawns release the gate in cancelled state and join every started worker.
pub(crate) fn parallel_batch<
    I: Send,
    P: FnMut() -> I,
    F: Fn(&mut I, TimerStart) -> crate::Result<Measured> + Sync,
>(
    timer: Timer,
    workers: usize,
    mut prepare: P,
    run: F,
) -> crate::Result<Measured> {
    use std::sync::{Condvar, Mutex, OnceLock};
    if workers == 0 || workers > 256 {
        return Err(crate::error("workers must be 1..256"));
    }
    let inputs: Vec<_> = (0..workers).map(|_| prepare()).collect();
    let gate = (Mutex::new((0usize, None::<bool>)), Condvar::new());
    let start = OnceLock::new();
    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        let mut spawn_error = None;
        for mut input in inputs {
            let (gate, start, run) = (&gate, &start, &run);
            match std::thread::Builder::new().spawn_scoped(scope, move || {
                let mut state = gate.0.lock().unwrap();
                state.0 += 1;
                gate.1.notify_all();
                while state.1.is_none() {
                    state = gate.1.wait(state).unwrap();
                }
                let proceed = state.1.unwrap();
                drop(state);
                if proceed {
                    let elapsed = run(&mut input, *start.get().unwrap());
                    Some((elapsed, input))
                } else {
                    None
                }
            }) {
                Ok(handle) => handles.push(handle),
                Err(error) => {
                    spawn_error = Some(error);
                    break;
                }
            }
        }
        let mut state = gate.0.lock().unwrap();
        while spawn_error.is_none() && state.0 != handles.len() {
            state = gate.1.wait(state).unwrap();
        }
        start.set(timer.start()).unwrap();
        state.1 = Some(spawn_error.is_none());
        drop(state);
        gate.1.notify_all();
        let mut completed = Vec::new();
        let mut elapsed = Measured::default();
        let mut panicked = false;
        let mut measurement_error = None;
        for handle in handles {
            match handle.join() {
                Ok(Some((duration, input))) => {
                    match duration {
                        Ok(duration) => elapsed = elapsed.max(duration),
                        Err(error) => {
                            measurement_error.get_or_insert(error);
                        }
                    }
                    completed.push(input);
                }
                Ok(None) => {}
                Err(_) => panicked = true,
            }
        }
        if let Some(error) = spawn_error {
            return Err(error.into());
        }
        if let Some(error) = measurement_error {
            return Err(error);
        }
        if panicked {
            return Err(crate::error("benchmark worker panicked"));
        }
        Ok(elapsed)
    })
}

/// Worker-owned inputs never cross thread boundaries. Both rendezvous are
/// released on user panics; retained values drop only after all timers stop.
pub(crate) fn parallel_local_batch<I>(
    timer: Timer,
    workers: usize,
    prepare: impl Fn() -> I + Sync,
    run: impl Fn(&mut I, TimerStart) -> crate::Result<Measured> + Sync,
) -> crate::Result<Measured> {
    use std::{
        panic::{AssertUnwindSafe, catch_unwind},
        sync::{Condvar, Mutex, OnceLock},
    };
    if workers == 0 || workers > 256 {
        return Err(crate::error("workers must be 1..256"));
    }
    let ready = (Mutex::new((0usize, false, None::<bool>)), Condvar::new());
    let finished = (Mutex::new(0usize), Condvar::new());
    let start = OnceLock::new();
    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        let mut spawn_error = None;
        for _ in 0..workers {
            let (prepare, run, ready, finished, start) =
                (&prepare, &run, &ready, &finished, &start);
            match std::thread::Builder::new().spawn_scoped(scope, move || {
                let input = catch_unwind(AssertUnwindSafe(prepare));
                let mut state = ready.0.lock().unwrap();
                state.0 += 1;
                state.1 |= input.is_err();
                ready.1.notify_all();
                while state.2.is_none() {
                    state = ready.1.wait(state).unwrap();
                }
                let proceed = state.2.unwrap();
                drop(state);
                if !proceed {
                    return None;
                }
                let mut input = input.unwrap();
                let result =
                    catch_unwind(AssertUnwindSafe(|| run(&mut input, *start.get().unwrap())));
                let mut count = finished.0.lock().unwrap();
                *count += 1;
                finished.1.notify_all();
                while *count != workers {
                    count = finished.1.wait(count).unwrap();
                }
                drop(count);
                // input (including any buffered outputs) stays on this worker.
                Some(result)
            }) {
                Ok(handle) => handles.push(handle),
                Err(error) => {
                    spawn_error = Some(error);
                    break;
                }
            }
        }
        let mut state = ready.0.lock().unwrap();
        while spawn_error.is_none() && state.0 != handles.len() {
            state = ready.1.wait(state).unwrap();
        }
        let setup_failed = state.1;
        start.set(timer.start()).unwrap();
        state.2 = Some(spawn_error.is_none() && !setup_failed);
        drop(state);
        ready.1.notify_all();
        let mut elapsed = Measured::default();
        let mut failed = setup_failed;
        let mut measurement_error = None;
        for handle in handles {
            match handle.join() {
                Ok(Some(Ok(Ok(duration)))) => elapsed = elapsed.max(duration),
                Ok(Some(Ok(Err(error)))) => {
                    measurement_error.get_or_insert(error);
                }
                Ok(None) => {}
                _ => failed = true,
            }
        }
        if let Some(error) = spawn_error {
            return Err(error.into());
        }
        if let Some(error) = measurement_error {
            return Err(error);
        }
        if failed {
            return Err(crate::error("benchmark worker setup or operation panicked"));
        }
        Ok(elapsed)
    })
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
    let mut remaining = n;
    let mut total = Measured::default();
    while remaining > 0 {
        let count = remaining.min(batch_size);
        let inputs: Vec<I> = (0..count).map(|_| setup()).collect();
        let mut outputs = Vec::with_capacity(count as usize);
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
    use crate::BatchPolicy;
    use std::time::Instant;
    use std::{cell::RefCell, rc::Rc};

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
