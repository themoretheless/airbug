use rbench::{DropPolicy, Suite};
fn main() -> rbench::Result<()> {
    let mut s = Suite::new("sort");
    for size in [32, 128, 512] {
        s.bench_with_input(
            &format!("stable/{size}"),
            move || (0..size).rev().collect::<Vec<u64>>(),
            |v| v.sort(),
            DropPolicy::InsideTiming,
        )
        .parameter("elements", size);
        s.bench_with_input(
            &format!("unstable/{size}"),
            move || (0..size).rev().collect::<Vec<u64>>(),
            |v| v.sort_unstable(),
            DropPolicy::InsideTiming,
        )
        .parameter("elements", size);
    }
    s.main()
}
