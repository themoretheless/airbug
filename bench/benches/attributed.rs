#[airbug_bench::suite]
mod collections {
    use std::hint::black_box;

    #[bench]
    fn sort() -> Vec<u64> {
        let mut values = black_box(vec![5, 2, 4, 1, 3]);
        values.sort_unstable();
        values
    }

    #[airbug_bench::bench]
    fn sum() -> u64 {
        black_box(0..100).sum()
    }

    // Disabled functions must also be excluded from generated registration.
    #[bench]
    #[cfg(any())]
    fn disabled() {
        panic!("disabled benchmark executed");
    }
}
