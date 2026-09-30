use airbug_bench::{Config, DropPolicy, Sampling, Suite, workloads::LocalExecutor};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

static CREATED: AtomicUsize = AtomicUsize::new(0);
static DROPPED: AtomicUsize = AtomicUsize::new(0);
struct Output;
impl Drop for Output {
    fn drop(&mut self) {
        DROPPED.fetch_add(1, Ordering::SeqCst);
    }
}
fn produce() -> Output {
    CREATED.fetch_add(1, Ordering::SeqCst);
    Output
}
async fn produce_async() -> Output {
    let mut pending = true;
    std::future::poll_fn(|cx| {
        if std::mem::take(&mut pending) {
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
        } else {
            std::task::Poll::Ready(produce())
        }
    })
    .await
}

#[test]
fn zero_sized_results_are_destroyed_once_across_samples_and_worker_waves() {
    assert_eq!(std::mem::size_of::<Output>(), 0);
    assert!(std::mem::needs_drop::<Output>());
    for policy in [DropPolicy::InsideTiming, DropPolicy::OutsideTiming] {
        for mode in 0..4 {
            CREATED.store(0, Ordering::SeqCst);
            DROPPED.store(0, Ordering::SeqCst);
            let mut suite = Suite::new("zst");
            match mode {
                0 => {
                    suite.bench_with_input("sync", || (), |_| produce(), policy);
                }
                1 => {
                    suite.bench_async_factory("async", || LocalExecutor, produce_async, policy);
                }
                2 => {
                    suite.bench_threads_with_input("workers", 2, || (), |_| produce(), policy);
                }
                _ => {
                    suite.bench_async_threads(
                        "async-workers",
                        2,
                        || LocalExecutor,
                        async || produce_async().await,
                        policy,
                    );
                }
            }
            suite.sampling(Sampling {
                iterations: Some(130),
                ..Default::default()
            });
            suite.config(Config {
                samples: 2,
                warmup: Duration::ZERO,
                sample_time: Duration::from_nanos(1),
                max_iterations: 130,
            });
            assert_eq!(suite.list("").len(), 1);
            assert_eq!(CREATED.load(Ordering::SeqCst), 0);
            let run = suite.run("").unwrap();
            let operations = if mode >= 2 { 260 } else { 130 };
            assert_eq!(run.observations.len(), 2);
            assert!(run.observations.iter().all(|o| o.operations == operations));
            assert_eq!(CREATED.load(Ordering::SeqCst), operations as usize * 2);
            assert_eq!(DROPPED.load(Ordering::SeqCst), operations as usize * 2);
        }
    }
}

#[test]
fn async_output_drop_respects_the_reported_timer_boundary() {
    use std::{cell::Cell, rc::Rc, time::Instant};
    struct TimedDrop(Rc<Cell<u128>>);
    impl Drop for TimedDrop {
        fn drop(&mut self) {
            let start = Instant::now();
            std::thread::sleep(Duration::from_millis(2));
            self.0.set(self.0.get() + start.elapsed().as_nanos());
        }
    }
    for policy in [DropPolicy::InsideTiming, DropPolicy::OutsideTiming] {
        let drops = Rc::new(Cell::new(0));
        let counter = drops.clone();
        let mut suite = Suite::new("async-drop");
        suite.bench_async_factory(
            "case",
            || LocalExecutor,
            move || {
                let counter = counter.clone();
                async move { TimedDrop(counter) }
            },
            policy,
        );
        suite.sampling(Sampling {
            iterations: Some(5),
            ..Default::default()
        });
        suite.config(Config {
            samples: 1,
            warmup: Duration::ZERO,
            sample_time: Duration::from_nanos(1),
            max_iterations: 5,
        });
        let start = Instant::now();
        let run = suite.run("").unwrap();
        let wall = start.elapsed().as_nanos();
        let measured: u128 = run.observations[0].value.as_ref().unwrap().parse().unwrap();
        assert!(drops.get() >= Duration::from_millis(10).as_nanos());
        match policy {
            DropPolicy::InsideTiming => assert!(measured >= drops.get()),
            DropPolicy::OutsideTiming => assert!(wall >= measured + drops.get()),
        }
    }
}

#[test]
fn threaded_destructor_intervals_respect_the_shared_timer_boundary() {
    use std::{
        sync::{Arc, Mutex},
        time::Instant,
    };
    type Intervals = Arc<Mutex<Vec<(Instant, Instant)>>>;
    struct TimedDrop(Intervals);
    impl Drop for TimedDrop {
        fn drop(&mut self) {
            let start = Instant::now();
            std::thread::sleep(Duration::from_micros(100));
            let end = Instant::now();
            self.0.lock().unwrap().push((start, end));
        }
    }
    for policy in [DropPolicy::InsideTiming, DropPolicy::OutsideTiming] {
        for mode in 0..3 {
            let intervals: Intervals = Arc::default();
            let mut suite = Suite::new("thread-drop");
            match mode {
                0 => {
                    suite.bench_threads_with_input(
                        "coordinator",
                        2,
                        || (),
                        |_| TimedDrop(intervals.clone()),
                        policy,
                    );
                }
                1 => {
                    suite.bench_threads_with_local_input(
                        "local",
                        2,
                        || (),
                        |_| TimedDrop(intervals.clone()),
                        policy,
                    );
                }
                _ => {
                    suite.bench_async_threads(
                        "async",
                        2,
                        || LocalExecutor,
                        async || TimedDrop(intervals.clone()),
                        policy,
                    );
                }
            }
            suite.sampling(Sampling {
                iterations: Some(65),
                ..Default::default()
            });
            suite.config(Config {
                samples: 1,
                warmup: Duration::ZERO,
                sample_time: Duration::from_nanos(1),
                max_iterations: 65,
            });
            let start = Instant::now();
            let run = suite.run("").unwrap();
            let wall = start.elapsed().as_nanos();
            let measured: u128 = run.observations[0].value.as_ref().unwrap().parse().unwrap();
            let mut intervals = intervals.lock().unwrap().clone();
            assert_eq!(intervals.len(), 130);
            intervals.sort_unstable();
            // Concurrent destructor intervals overlap. Compare their union,
            // rather than double-counting time spent by two workers at once.
            let (mut begin, mut end) = intervals[0];
            let mut destruction = 0;
            for (next_begin, next_end) in intervals.into_iter().skip(1) {
                if next_begin <= end {
                    end = end.max(next_end);
                } else {
                    destruction += end.duration_since(begin).as_nanos();
                    (begin, end) = (next_begin, next_end);
                }
            }
            destruction += end.duration_since(begin).as_nanos();
            match policy {
                DropPolicy::InsideTiming => assert!(
                    measured >= destruction,
                    "mode {mode}: measured {measured}, drops {destruction}"
                ),
                DropPolicy::OutsideTiming => assert!(
                    wall >= measured + destruction,
                    "mode {mode}: wall {wall}, measured {measured}, drops {destruction}"
                ),
            }
        }
    }
}
