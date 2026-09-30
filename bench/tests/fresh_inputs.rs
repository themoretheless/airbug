use airbug_bench::{Config, DropPolicy, Sampling, Suite, workloads::LocalExecutor};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

type Spans = Arc<Mutex<Vec<(Instant, Instant)>>>;
fn record(spans: &Spans) {
    let start = Instant::now();
    std::thread::sleep(Duration::from_micros(100));
    let end = Instant::now();
    spans.lock().unwrap().push((start, end));
}
fn union(spans: &Spans) -> u128 {
    let mut spans = spans.lock().unwrap().clone();
    spans.sort_unstable();
    let (mut begin, mut end) = spans[0];
    let mut total = 0;
    for (next, last) in spans.into_iter().skip(1) {
        if next <= end {
            end = end.max(last);
        } else {
            total += end.duration_since(begin).as_nanos();
            (begin, end) = (next, last);
        }
    }
    total + end.duration_since(begin).as_nanos()
}
struct Input {
    used: bool,
    external: Spans,
    dropped: Arc<AtomicUsize>,
}
impl Drop for Input {
    fn drop(&mut self) {
        assert!(self.used, "each generated input must be used");
        record(&self.external);
        self.dropped.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn fresh_borrowed_input_setup_and_drop_are_excluded_in_all_execution_modes() {
    for mode in 0..5 {
        let external: Spans = Arc::default();
        let work: Spans = Arc::default();
        let created = AtomicUsize::new(0);
        let dropped = Arc::new(AtomicUsize::new(0));
        let setup = || {
            record(&external);
            created.fetch_add(1, Ordering::SeqCst);
            Input {
                used: false,
                external: external.clone(),
                dropped: dropped.clone(),
            }
        };
        let operation = |input: &mut Input| {
            assert!(!input.used, "operation must receive a fresh input");
            input.used = true;
            record(&work);
        };
        let mut suite = Suite::new("fresh");
        match mode {
            0 => {
                suite.bench_with_input("sync", setup, operation, DropPolicy::OutsideTiming);
            }
            1 => {
                suite.bench_async_with_input(
                    "async",
                    || LocalExecutor,
                    setup,
                    async |input| operation(input),
                    DropPolicy::OutsideTiming,
                );
            }
            2 => {
                suite.bench_threads_with_input(
                    "coordinator",
                    2,
                    setup,
                    operation,
                    DropPolicy::OutsideTiming,
                );
            }
            3 => {
                suite.bench_threads_with_local_input(
                    "local",
                    2,
                    setup,
                    operation,
                    DropPolicy::OutsideTiming,
                );
            }
            _ => {
                suite.bench_async_threads_with_input(
                    "async-workers",
                    2,
                    || LocalExecutor,
                    setup,
                    async |input| operation(input),
                    DropPolicy::OutsideTiming,
                );
            }
        }
        suite.sampling(Sampling {
            iterations: Some(65),
            ..Default::default()
        });
        suite.config(Config {
            samples: 2,
            warmup: Duration::ZERO,
            sample_time: Duration::from_nanos(1),
            max_iterations: 65,
        });
        assert_eq!(suite.list("").len(), 1);
        assert_eq!(created.load(Ordering::SeqCst), 0);
        let start = Instant::now();
        let run = suite.run("").unwrap();
        let wall = start.elapsed().as_nanos();
        let measured: u128 = run
            .observations
            .iter()
            .map(|o| o.value.as_ref().unwrap().parse::<u128>().unwrap())
            .sum();
        let expected = if mode >= 2 { 260 } else { 130 };
        assert_eq!(created.load(Ordering::SeqCst), expected);
        assert_eq!(dropped.load(Ordering::SeqCst), expected);
        assert!(
            measured >= union(&work),
            "mode {mode}: workload must be timed"
        );
        assert!(
            wall >= measured + union(&external),
            "mode {mode}: setup/drop must be outside timing"
        );
    }
}
