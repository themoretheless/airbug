//! Benchmark runner for curated catalog workloads (`#22–1000`).
use airbug_bench::{Fixture, Suite, catalog};

fn main() -> airbug_bench::Result<()> {
    let mut suite = Suite::new("catalog");
    for k in catalog::all_kernels() {
        let run = k.run;
        let id = format!("{}/{}", k.category.as_str(), k.slug);
        suite
            .bench_fixture(&id, Fixture::new(|| 64usize), move |iters| run(*iters))
            .tag(k.category.as_str());
    }
    suite.main()
}
