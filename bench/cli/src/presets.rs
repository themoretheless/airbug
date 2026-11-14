//! Curated domain benchmark matrix presets (`#311–400`).
use crate::matrix::Matrix;
use crate::runner::{Plan, Program};
use airbug_bench::{Result, error};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[derive(Clone, Copy)]
pub struct PresetSpec {
    pub slug: &'static str,
    pub category: &'static str,
    pub title: &'static str,
    pub target: &'static str,
    pub description: &'static str,
    pub axes: &'static [(&'static str, &'static [&'static str])],
    pub repetitions: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct PresetSummary {
    pub slug: String,
    pub category: String,
    pub title: String,
    pub target: String,
    pub description: String,
    pub axes: BTreeMap<String, Vec<String>>,
    pub combinations: usize,
}

pub const PRESETS: &[PresetSpec] = &[
    PresetSpec {
        slug: "web-api-latency",
        category: "web-api",
        title: "HTTP router, header & traceparent latency sweep",
        target: "target/release/catalog",
        description: "Evaluates W3C traceparent parsing and consistent routing overhead across thread counts.",
        axes: &[
            (
                "--filter",
                &[
                    "observability/traceparent-parse",
                    "systems/consistent-hash-ratelimit",
                ],
            ),
            ("--threads", &["1", "2", "4"]),
        ],
        repetitions: 12,
    },
    PresetSpec {
        slug: "storage-db-index",
        category: "storage-db",
        title: "Storage bitset, Fenwick & CRC32C durability kernels",
        target: "target/release/catalog",
        description: "Measures Bloom membership, Fenwick prefix indexing, and CRC32C checksum scaling.",
        axes: &[
            (
                "--filter",
                &["algorithms/fenwick-dsu-bloom", "rng-hash/crc32c-sw"],
            ),
            ("--sample-count", &["100", "500"]),
        ],
        repetitions: 12,
    },
    PresetSpec {
        slug: "compiler-parser-frontend",
        category: "compiler-parser",
        title: "String scan, edit distance & varint AST codecs",
        target: "target/release/catalog",
        description: "Covers KMP/BMH lexical matching, Wagner-Fischer diffing, and LEB128 encoding.",
        axes: &[
            (
                "--filter",
                &[
                    "algorithms/kmp-bmh-search",
                    "algorithms/edit-distance",
                    "systems/leb128-base64",
                ],
            ),
            ("--threads", &["1", "2"]),
        ],
        repetitions: 10,
    },
    PresetSpec {
        slug: "concurrency-lockfree-scaling",
        category: "concurrency",
        title: "MPMC queue, Treiber stack & SeqLock contention matrix",
        target: "target/release/catalog",
        description: "Sweeps ABA-tagged MPMC, lock-free stack, SeqLock, TicketLock, and striped counters across core counts.",
        axes: &[
            (
                "--filter",
                &[
                    "concurrency/mpmc-queue",
                    "concurrency/treiber-stack",
                    "concurrency/spsc-ring",
                    "concurrency/seqlock",
                    "concurrency/ticket-lock",
                    "concurrency/striped-counter",
                ],
            ),
            ("--threads", &["1", "2", "4", "8"]),
        ],
        repetitions: 12,
    },
    PresetSpec {
        slug: "simd-numerical-kernels",
        category: "simd-numerical",
        title: "Radix-2 FFT & cache-tiled dense GEMM",
        target: "target/release/catalog",
        description: "Profiles butterfly FFT and blocked GEMM across thread and sample regimes.",
        axes: &[
            (
                "--filter",
                &["algorithms/fft-matmul", "algorithms/radix-sort-u32"],
            ),
            ("--sample-count", &["64", "256"]),
        ],
        repetitions: 12,
    },
    PresetSpec {
        slug: "crypto-pqc-lattice",
        category: "crypto-pqc",
        title: "ML-KEM/Kyber NTT, Ring-LWE & hash-based signatures",
        target: "target/release/catalog",
        description: "Benchmarks q=3329 Kyber NTT, LWE PKE, Lamport OTS, WOTS+, Merkle verify, and constant-time primitives.",
        axes: &[
            (
                "--filter",
                &[
                    "crypto-lattice/kyber-ntt",
                    "crypto-lattice/lwe-pke",
                    "crypto-lattice/lamport-ots",
                    "crypto-lattice/wots-plus",
                    "crypto-lattice/merkle-tree",
                    "crypto-lattice/constant-time-ops",
                ],
            ),
            ("--threads", &["1", "2"]),
        ],
        repetitions: 12,
    },
    PresetSpec {
        slug: "observability-otlp-pipeline",
        category: "observability",
        title: "Histogram, reservoir sampling & W3C trace ingestion",
        target: "target/release/catalog",
        description: "Evaluates OTLP ingest parsing, Log2 tail-quantile tracking, and Vitter reservoir sampling.",
        axes: &[
            (
                "--filter",
                &[
                    "observability/histogram-reservoir",
                    "observability/traceparent-parse",
                ],
            ),
            ("--sample-count", &["100", "400"]),
        ],
        repetitions: 12,
    },
    PresetSpec {
        slug: "ui-frame-pacing",
        category: "ui-graphics",
        title: "Forma vector UI offscreen & GPU timestamp sweep",
        target: "target/release/forma-bench",
        description: "Measures static, hover, animation, scroll, resize, and text UI scenarios across viewport DPIs.",
        axes: &[
            (
                "--filter",
                &["static", "hover", "animation", "scroll", "resize", "text"],
            ),
            ("--dpi", &["1.0", "2.0"]),
        ],
        repetitions: 8,
    },
    PresetSpec {
        slug: "memory-hierarchy-latency",
        category: "systems",
        title: "Pointer-chase cache & TLB latency sweep",
        target: "target/release/catalog",
        description: "Separates L1/L2/DRAM pointer-chase stalls from striped counter updates.",
        axes: &[
            (
                "--filter",
                &["systems/pointer-chase", "concurrency/striped-counter"],
            ),
            ("--sample-count", &["50", "200"]),
        ],
        repetitions: 12,
    },
    PresetSpec {
        slug: "compression-codec-throughput",
        category: "codecs",
        title: "LEB128 varint, Base64 & SHA-256 framing throughput",
        target: "target/release/catalog",
        description: "Compares wire serialization codecs against SHA-256 integrity hashing.",
        axes: &[
            (
                "--filter",
                &["systems/leb128-base64", "rng-hash/sha256-digest"],
            ),
            ("--threads", &["1", "2"]),
        ],
        repetitions: 12,
    },
    PresetSpec {
        slug: "graph-analytics-dijkstra",
        category: "algorithms",
        title: "Shortest-path graph traversal & DSU connectivity",
        target: "target/release/catalog",
        description: "Evaluates priority-queue Dijkstra routing and disjoint-set union latency.",
        axes: &[
            (
                "--filter",
                &[
                    "algorithms/dijkstra-shortest-path",
                    "algorithms/fenwick-dsu-bloom",
                ],
            ),
            ("--sample-count", &["64", "256"]),
        ],
        repetitions: 10,
    },
    PresetSpec {
        slug: "ci-smoke-gate",
        category: "ci",
        title: "Fast deterministic PR gate across PRNG & hash baselines",
        target: "target/release/catalog",
        description: "Lightweight matrix intended for fast automated PR regression checks.",
        axes: &[
            (
                "--filter",
                &["rng-hash/splitmix-xoshiro", "rng-hash/noncrypto-hashes"],
            ),
            ("--sample-count", &["50"]),
        ],
        repetitions: 6,
    },
];

impl PresetSpec {
    pub fn to_matrix(self) -> Matrix {
        let mut axes = BTreeMap::new();
        for (flag, values) in self.axes {
            axes.insert(
                (*flag).to_string(),
                values.iter().map(|v| (*v).to_string()).collect(),
            );
        }
        let mut contract = BTreeMap::new();
        contract.insert("preset.slug".into(), self.slug.into());
        contract.insert("preset.category".into(), self.category.into());

        Matrix {
            plan: Plan {
                privacy: None,
                variants: BTreeMap::new(),
                start_pair: 0,
                candidate: Program {
                    path: PathBuf::from(self.target),
                    args: vec!["--json".into()],
                    env: BTreeMap::new(),
                    cwd: None,
                },
                baseline: None,
                repetitions: self.repetitions,
                timeout_ms: 60_000,
                protocol: true,
                fixtures: Vec::new(),
                contract,
                provenance: BTreeMap::new(),
            },
            axes,
        }
    }

    pub fn summary(&self) -> PresetSummary {
        let matrix = self.to_matrix();
        let combinations = matrix
            .axes
            .values()
            .fold(1usize, |acc, vals| acc.saturating_mul(vals.len()));
        PresetSummary {
            slug: self.slug.into(),
            category: self.category.into(),
            title: self.title.into(),
            target: self.target.into(),
            description: self.description.into(),
            axes: matrix.axes,
            combinations,
        }
    }
}

pub fn list(category: Option<&str>) -> Vec<PresetSummary> {
    PRESETS
        .iter()
        .filter(|p| {
            category
                .map(|c| p.category.eq_ignore_ascii_case(c.trim()))
                .unwrap_or(true)
        })
        .map(PresetSpec::summary)
        .collect()
}

pub fn export_json(slug: &str) -> Result<String> {
    let trimmed = slug.trim();
    let spec = PRESETS
        .iter()
        .find(|p| p.slug.eq_ignore_ascii_case(trimmed))
        .ok_or_else(|| error(format!("unknown preset {slug:?}")))?;
    let matrix = spec.to_matrix();
    validate_matrix_shape(&matrix)?;
    Ok(serde_json::to_string_pretty(&matrix)?)
}

pub fn format_text(items: &[PresetSummary]) -> String {
    let mut out = String::new();
    for p in items {
        out.push_str(&format!(
            "{} [{}] ({} combos) — {}\n  target: {}\n",
            p.slug, p.category, p.combinations, p.title, p.target
        ));
    }
    out
}

pub fn validate_matrix_shape(m: &Matrix) -> Result<()> {
    if m.axes.is_empty() || m.axes.len() > 8 {
        return Err(error("preset matrix must define 1..=8 axes"));
    }
    let mut combos = 1usize;
    for (key, vals) in &m.axes {
        if !key.starts_with("--")
            || key.len() < 3
            || !key[2..]
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || vals.is_empty()
            || vals.iter().any(|v| v.is_empty() || v.len() > 256)
            || vals.iter().collect::<BTreeSet<_>>().len() != vals.len()
        {
            return Err(error(format!("invalid axis {key:?}")));
        }
        if m.plan
            .candidate
            .args
            .iter()
            .any(|a| a == key || a.starts_with(&format!("{key}=")))
        {
            return Err(error(format!("axis {key:?} conflicts with candidate args")));
        }
        combos = combos
            .checked_mul(vals.len())
            .ok_or_else(|| error("combination overflow"))?;
        if combos > 256 {
            return Err(error("preset exceeds 256 matrix combinations"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_presets_roundtrip_and_satisfy_matrix_validation() {
        assert!(PRESETS.len() >= 10);
        let mut seen = BTreeSet::new();
        for p in PRESETS {
            assert!(seen.insert(p.slug), "duplicate preset slug {}", p.slug);
            let json = export_json(p.slug).unwrap();
            let parsed: Matrix = serde_json::from_str(&json).unwrap();
            validate_matrix_shape(&parsed).unwrap();
        }
        assert!(export_json("does-not-exist").is_err());
        assert!(!list(Some("crypto-pqc")).is_empty());
    }
}
