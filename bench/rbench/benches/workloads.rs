use rbench::{Fixture, Seeded, Suite};
fn main() -> rbench::Result<()> {
    let mut suite = Suite::new("workloads");
    let data = Fixture::new(|| Seeded::new(42).bytes(4096));
    suite
        .bench_fixture("checksum/4096", data.clone(), |data| {
            data.iter()
                .fold(0u64, |sum, b| sum.wrapping_add(u64::from(*b)))
        })
        .tag("cpu")
        .tag("bytes")
        .work_units("bytes", 4096)
        .seed(42);
    suite
        .bench_fixture("count-zero/4096", data, |data| {
            data.iter().filter(|b| **b == 0).count()
        })
        .tag("cpu")
        .work_units("bytes", 4096)
        .seed(42);
    suite.main()
}
