#[path = "support/external.rs"]
mod external;

#[airbug_bench::suite(groups = [crate::external::cases])]
mod collections {
    use std::hint::black_box;

    fn guarded_executor() -> airbug_bench::workloads::LocalExecutor {
        assert!(
            std::env::var_os("AIRBUG_TEST_NO_EXECUTION").is_none(),
            "executor created during preview"
        );
        airbug_bench::workloads::LocalExecutor
    }

    #[bench(custom = true, args = [String::from("input")], executor = guarded_executor())]
    async fn async_manual(iterations: u64, input: &str) -> std::time::Duration {
        let started = std::time::Instant::now();
        for _ in 0..iterations {
            black_box(std::future::ready(input.len()).await);
        }
        started.elapsed()
    }

    #[bench]
    fn sort() -> Vec<u64> {
        let mut values = black_box(vec![5, 2, 4, 1, 3]);
        values.sort_unstable();
        values
    }

    #[airbug_bench::bench]
    fn sum() -> u64 {
        assert!(
            std::env::var_os("AIRBUG_TEST_NO_EXECUTION").is_none(),
            "workload executed during preview"
        );
        black_box(0..100).sum()
    }

    #[bench(args = [64usize, 1024], setup = |n| (0..n).rev().collect::<Vec<_>>(), input_items = |v: &Vec<usize>| v.len() as u64)]
    fn sort_only(values: &mut [usize]) {
        assert!(values[0] > values[1], "each operation needs fresh input");
        values.sort_unstable();
    }

    #[bench(args = [8u64, 32], name = "parameterized_sum")]
    fn sum_n(n: u64) -> u64 {
        (0..n).sum()
    }

    #[bench(setup = || vec![3u64, 1, 2], drop_output = "outside")]
    fn clone_sorted(values: &mut [u64]) -> Vec<u64> {
        values.sort_unstable();
        values.to_vec()
    }

    // Listing and selecting other cases must never invoke setup or the workload.
    fn guarded_input() -> Vec<u64> {
        assert!(
            std::env::var_os("AIRBUG_TEST_NO_SETUP").is_none(),
            "unselected setup ran"
        );
        vec![3, 1, 2]
    }

    #[bench(setup = guarded_input)]
    fn unselected(values: &mut [u64]) {
        values.sort_unstable();
    }

    #[bench(threads = &[1usize, 2][..], setup_thread = "worker",
        setup = || std::rc::Rc::new(std::cell::Cell::new(0u64)), drop_output = "outside", input_items = |_| 1)]
    fn worker_local(
        input: &mut std::rc::Rc<std::cell::Cell<u64>>,
    ) -> std::rc::Rc<std::cell::Cell<u64>> {
        input.set(input.get() + 1);
        input.clone()
    }

    #[bench(threads = [1, 2], setup = || std::rc::Rc::new(42u64), input_items = |_| 1)]
    async fn async_workers(input: &mut std::rc::Rc<u64>) -> u64 {
        std::future::ready(**input).await
    }

    #[bench(types = [String], args = [String::from("input")])]
    fn lifetime_conversion<'input, T: From<&'input str>>(input: &'input str) -> T {
        T::from(input)
    }

    #[cfg(feature = "async-tokio")]
    #[bench(executor = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap(),
        threads = [2])]
    async fn tokio_timer() {
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }

    const ARRAY_SIZES: &[usize] = &[4, 16];
    #[bench(types = [u32, u64], consts = ARRAY_SIZES, items = N as u64)]
    fn generic_fill<T: Default + Copy, const N: usize>() -> [T; N] {
        [black_box(T::default()); N]
    }

    #[bench(types = [Vec<u32>, std::collections::BTreeSet<u32>], args = [4u32, 8], items = |n| n as u64)]
    fn collect<T: FromIterator<u32>>(n: u32) -> T {
        (0..n).collect()
    }

    #[bench(args = [String::from("abc"), String::from("defg")], bytes = |s: &str| s.len() as u64)]
    fn string_len(value: &str) -> usize {
        value.len()
    }

    #[bench(args = [4u64, 8], items = |n| n, samples = 3, warmup_ms = 0, sample_ms = 1)]
    async fn async_sum(n: u64) -> u64 {
        std::future::ready((0..n).sum()).await
    }

    #[bench(args = [4usize, 8], setup = |n| (0..n).rev().collect::<Vec<_>>(), items = |n| n as u64, batch = airbug_bench::BatchPolicy::LargeInput)]
    async fn async_sort(values: &mut [usize]) {
        assert!(values[0] > values[1]);
        std::future::ready(()).await;
        values.sort_unstable();
    }

    static SHARED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    #[bench(threads = [1, 2], items = 1)]
    fn contended() -> u64 {
        SHARED.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    #[bench(args = [4usize, 8], threads = [1, 2], setup = |n| (0..n).rev().collect::<Vec<_>>(), items = |n| n as u64)]
    fn threaded_sort(values: &mut [usize]) {
        assert!(values[0] > values[1]);
        values.sort_unstable();
    }

    #[bench(custom = true, args = [4u64, 8], items = |n| n)]
    fn manual(iterations: u64, size: u64) -> std::time::Duration {
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            black_box((0..black_box(size)).sum::<u64>());
        }
        start.elapsed()
    }

    #[bench(drop_output = "outside", batch = airbug_bench::BatchPolicy::LargeInput)]
    fn allocate() -> Vec<u8> {
        vec![black_box(7); 128]
    }

    #[bench(args = [String::from("hello")], bytes = |s: &str| s.len() as u64)]
    async fn async_borrowed(value: &str) -> usize {
        std::future::ready(value.len()).await
    }

    #[bench(args = [String::from("hello")], threads = [1, 2], bytes = |s: &str| s.len() as u64)]
    fn threaded_borrowed(value: &str) -> usize {
        value.len()
    }

    #[bench(types = [u16, u32], args = [4usize], threads = [1, 2],
        setup = |n| vec![T::default(); n], items = |n| n as u64)]
    fn typed_setup<T: Default + Copy + Ord + Send>(values: &mut [T]) {
        values.sort_unstable();
    }

    #[group(samples = 3, items = 1)]
    mod nested {
        #[bench]
        fn plain() -> usize {
            std::hint::black_box(1)
        }
        #[group(name = "child", bytes = 128)]
        mod deeper {
            #[bench(items = 2, chars = 4, cycles = 10)]
            fn multiple() -> usize {
                std::hint::black_box(2)
            }
        }
        #[bench]
        #[ignore]
        fn ignored() -> usize {
            std::hint::black_box(3)
        }
    }
    #[bench(setup = || String::from("owned"), drop_output = "outside")]
    fn owned_string(mut value: String) -> String {
        value.push('!');
        value
    }
    #[bench(setup = || String::from("async-owned"), drop_output = "outside", batch = airbug_bench::BatchPolicy::PerIteration)]
    async fn async_owned_string(mut value: String) -> String {
        std::future::ready(()).await;
        value.push('!');
        value
    }

    #[bench(args = [1.1f64, -2.0, 1.01, -10.0, 0.0])]
    fn numeric_order(value: f64) -> f64 {
        black_box(value)
    }

    #[bench(args = [String::new(), String::from("é🦀")],
        bytes = |s: &str| airbug_bench::counters::bytes_of_str(s),
        chars = |s: &str| airbug_bench::counters::chars_of_str(s))]
    fn unicode_text(value: &str) -> usize {
        black_box(value).chars().count()
    }

    // Disabled functions must also be excluded from generated registration.
    #[bench]
    #[cfg(any())]
    fn disabled() {
        panic!("disabled benchmark executed");
    }
}
