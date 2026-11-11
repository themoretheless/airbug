/// Cache Performance Benchmark
///
/// Demonstrates case-level benchmarking with cache miss instrumentation.
/// This example shows how to measure cache effects on different data layouts.
///
/// Run: cargo bench --bench cache-benchmark
///
use airbug_bench::{DropPolicy, Fixture, Seeded, Suite};

/// Simulate L1 cache warm vs cold access patterns
fn bench_cache_patterns(suite: &mut Suite<'_>) {
    // Warm cache: sequential access in working set
    suite
        .bench_with_input(
            "sequential-warm",
            || Seeded::new(42).bytes(65536),
            |data| {
                let sum: u64 = data.iter().fold(0u64, |acc, b| acc.wrapping_add(*b as u64));
                // Prevent optimizer from removing the work
                core::hint::black_box(sum)
            },
            DropPolicy::OutsideTiming,
        )
        .tag("cache")
        .tag("pattern")
        .parameter("cache", "L1")
        .parameter("pattern", "sequential");

    // Cold cache: random access pattern
    suite
        .bench_with_input(
            "random-cold",
            || Seeded::new(123).bytes(65536),
            |data| {
                // Random stride causes cache misses
                let mut total = 0u64;
                for i in (0..data.len()).step_by(64) {
                    total = total.wrapping_add(data[i] as u64);
                }
                core::hint::black_box(total)
            },
            DropPolicy::OutsideTiming,
        )
        .tag("cache")
        .tag("pattern")
        .parameter("cache", "cold")
        .parameter("pattern", "random-stride");

    // Structure of Arrays vs Array of Structures
    const SIZE: usize = 1024 * 1024;

    suite
        .bench_fixture(
            "soa-pattern",
            Fixture::new(|| Seeded::new(456).bytes(SIZE)),
            |data| {
                // Structure of Arrays layout - better cache locality for SIMD
                let (x_sum, y_sum): (u64, u64) = data
                    .as_chunks::<{ SIZE / 2 }>()
                    .0
                    .iter()
                    .map(|chunk| chunk.iter().map(|&b| b as u64).sum::<u64>())
                    .fold(
                        (0u64, 0u64),
                        |(x, y), val| {
                            if x == 0 { (val, y) } else { (x, val) }
                        },
                    );
                core::hint::black_box(x_sum.wrapping_add(y_sum))
            },
        )
        .tag("layout")
        .tag("cache")
        .parameter("layout", "soa");

    suite
        .bench_fixture(
            "aos-pattern",
            Fixture::new(|| Seeded::new(789).bytes(SIZE)),
            |data| {
                // Array of Structures layout - interleaved access
                let result: u64 = data
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|chunk| (chunk[0] as u64) * (chunk[1] as u64))
                    .sum();
                core::hint::black_box(result)
            },
        )
        .tag("layout")
        .tag("cache")
        .parameter("layout", "aos");
}

/// Memory allocation patterns comparison
fn bench_allocation_patterns(suite: &mut Suite<'_>) {
    suite
        .bench_fixture(
            "arena-alloc-small",
            Fixture::new(|| Seeded::new(42).bytes(1024)),
            |data| {
                // Simulate arena allocation with fixed-size chunks
                let mut chunks = Vec::with_capacity(data.len() / 8);
                for chunk in data.as_chunks::<8>().0 {
                    chunks.push(u64::from_ne_bytes(*chunk));
                }
                chunks.iter().sum::<u64>()
            },
        )
        .tag("allocation")
        .tag("arena")
        .parameter("arena", "inline");

    suite
        .bench_fixture(
            "heap-alloc-scattered",
            Fixture::new(|| Seeded::new(42).bytes(8192)),
            |data| {
                // Simulate scattered heap allocations
                let mut total = 0u64;
                for chunk in data.as_chunks::<8>().0 {
                    let val = u64::from_ne_bytes(*chunk);
                    total = total.wrapping_add(val);
                }
                total
            },
        )
        .tag("allocation")
        .tag("heap")
        .parameter("heap", "scattered");
}

/// Hash function performance with cache effects
fn bench_hash_functions(suite: &mut Suite<'_>) {
    suite
        .bench_fixture(
            "hash-fnv1a-small",
            Fixture::new(|| Seeded::new(42).bytes(256)),
            |data| {
                const FNV_PRIME: u64 = 1099511628211u64;
                const FNV_OFFSET: u64 = 14695981039346656037u64;

                let mut hash = FNV_OFFSET;
                for byte in data {
                    hash ^= *byte as u64;
                    hash = hash.wrapping_mul(FNV_PRIME);
                }
                core::hint::black_box(hash)
            },
        )
        .tag("hash")
        .tag("algorithm")
        .parameter("algorithm", "fnv1a");

    suite
        .bench_fixture(
            "hash-crc32-small",
            Fixture::new(|| Seeded::new(42).bytes(256)),
            |data| {
                const POLY: u32 = 0xEDB88320;
                let mut crc: u32 = !0;

                for byte in data {
                    crc ^= *byte as u32;
                    for _ in 0..8 {
                        if (crc & 1) != 0 {
                            crc = (crc >> 1) ^ POLY;
                        } else {
                            crc >>= 1;
                        }
                    }
                }
                core::hint::black_box(!crc)
            },
        )
        .tag("hash")
        .tag("algorithm")
        .parameter("algorithm", "crc32");

    suite
        .bench_fixture(
            "hash-fnv1a-large",
            Fixture::new(|| Seeded::new(42).bytes(4096)),
            |data| {
                const FNV_PRIME: u64 = 1099511628211u64;
                const FNV_OFFSET: u64 = 14695981039346656037u64;

                let mut hash = FNV_OFFSET;
                for byte in data {
                    hash ^= *byte as u64;
                    hash = hash.wrapping_mul(FNV_PRIME);
                }
                core::hint::black_box(hash)
            },
        )
        .tag("hash")
        .tag("algorithm")
        .parameter("algorithm", "fnv1a");
}

fn main() -> airbug_bench::Result<()> {
    let mut suite = Suite::new("cache-benchmark");

    // Hardware-aware benchmarks
    bench_cache_patterns(&mut suite);
    bench_allocation_patterns(&mut suite);
    bench_hash_functions(&mut suite);

    suite.main()
}
