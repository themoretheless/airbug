#[airbug_bench::group(name = "external", samples = 2, warmup_ms = 0)]
pub mod cases {
    #[bench(args = [3usize, 7])]
    fn from_file(n: usize) -> usize {
        std::hint::black_box(n * n)
    }

    #[group(name = "nested")]
    mod child {
        #[bench]
        fn leaf() -> usize {
            std::hint::black_box(9)
        }
    }
}
