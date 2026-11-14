//! Curated micro-benchmark & verification workload catalog (`#22–1000`).
pub mod algorithms;
pub mod crypto;
pub mod lockfree;
pub mod rng_hash;
pub mod systems;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CatalogCategory {
    Systems,
    RngHash,
    Algorithms,
    Concurrency,
    CryptoLattice,
    Observability,
}

impl CatalogCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Systems => "systems",
            Self::RngHash => "rng-hash",
            Self::Algorithms => "algorithms",
            Self::Concurrency => "concurrency",
            Self::CryptoLattice => "crypto-lattice",
            Self::Observability => "observability",
        }
    }

    pub fn from_str_name(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "systems" => Some(Self::Systems),
            "rng-hash" | "rng" | "hash" => Some(Self::RngHash),
            "algorithms" | "algo" => Some(Self::Algorithms),
            "concurrency" | "lockfree" => Some(Self::Concurrency),
            "crypto-lattice" | "crypto" | "pqc" => Some(Self::CryptoLattice),
            "observability" | "obs" => Some(Self::Observability),
            _ => None,
        }
    }
}

impl std::fmt::Display for CatalogCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy)]
pub struct CatalogKernel {
    pub id: u16,
    pub slug: &'static str,
    pub category: CatalogCategory,
    pub title: &'static str,
    pub run: fn(usize) -> u64,
}

pub const KERNELS: &[CatalogKernel] = &[
    // Systems & memory hierarchy (#22–170)
    CatalogKernel {
        id: 22,
        slug: "pointer-chase",
        category: CatalogCategory::Systems,
        title: "Cache/TLB latency pointer-chase traversal",
        run: systems::kernel_pointer_chase,
    },
    CatalogKernel {
        id: 129,
        slug: "leb128-base64",
        category: CatalogCategory::Systems,
        title: "LEB128 varint & Base64 codec roundtrip",
        run: systems::kernel_leb128_base64,
    },
    CatalogKernel {
        id: 250,
        slug: "traceparent-parse",
        category: CatalogCategory::Observability,
        title: "W3C traceparent header validation & decode",
        run: systems::kernel_traceparent_parse,
    },
    CatalogKernel {
        id: 260,
        slug: "histogram-reservoir",
        category: CatalogCategory::Observability,
        title: "Log2 latency histogram & Vitter Algorithm R reservoir sampling",
        run: systems::kernel_histogram_reservoir,
    },
    CatalogKernel {
        id: 910,
        slug: "consistent-hash-ratelimit",
        category: CatalogCategory::Systems,
        title: "Jump consistent hashing & token-bucket rate limiter",
        run: systems::kernel_consistent_hash_ratelimit,
    },
    // RNG & hashing (#171–196)
    CatalogKernel {
        id: 171,
        slug: "splitmix-xoshiro",
        category: CatalogCategory::RngHash,
        title: "SplitMix64 + Xoshiro256** PRNG throughput",
        run: rng_hash::kernel_splitmix_xoshiro,
    },
    CatalogKernel {
        id: 175,
        slug: "pcg64-wyrand",
        category: CatalogCategory::RngHash,
        title: "PCG64-DXSM + WyRand stream generation",
        run: rng_hash::kernel_pcg64_wyrand,
    },
    CatalogKernel {
        id: 180,
        slug: "chacha8-stream",
        category: CatalogCategory::RngHash,
        title: "ChaCha8 quarter-round keystream block",
        run: rng_hash::kernel_chacha8_stream,
    },
    CatalogKernel {
        id: 185,
        slug: "noncrypto-hashes",
        category: CatalogCategory::RngHash,
        title: "FNV-1a, FxHash64 & SipHash-2-4 digest mix",
        run: rng_hash::kernel_noncrypto_hashes,
    },
    CatalogKernel {
        id: 190,
        slug: "crc32c-sw",
        category: CatalogCategory::RngHash,
        title: "Castagnoli CRC32C checksum",
        run: rng_hash::kernel_crc32c,
    },
    CatalogKernel {
        id: 196,
        slug: "sha256-digest",
        category: CatalogCategory::RngHash,
        title: "SHA-256 512-byte message hashing",
        run: rng_hash::kernel_sha256_digest,
    },
    // Algorithms, strings, data structures & numerical (#401–700, #761–891)
    CatalogKernel {
        id: 401,
        slug: "radix-sort-u32",
        category: CatalogCategory::Algorithms,
        title: "LSB 8-bit 4-pass radix sort on u32 arrays",
        run: algorithms::kernel_radix_sort,
    },
    CatalogKernel {
        id: 450,
        slug: "kmp-bmh-search",
        category: CatalogCategory::Algorithms,
        title: "KMP & Boyer-Moore-Horspool substring matching",
        run: algorithms::kernel_kmp_bmh,
    },
    CatalogKernel {
        id: 480,
        slug: "edit-distance",
        category: CatalogCategory::Algorithms,
        title: "Wagner-Fischer Levenshtein edit distance DP",
        run: algorithms::kernel_edit_distance,
    },
    CatalogKernel {
        id: 520,
        slug: "fenwick-dsu-bloom",
        category: CatalogCategory::Algorithms,
        title: "Fenwick BIT, Union-Find DSU & double-hash Bloom filter",
        run: algorithms::kernel_fenwick_dsu_bloom,
    },
    CatalogKernel {
        id: 610,
        slug: "dijkstra-shortest-path",
        category: CatalogCategory::Algorithms,
        title: "Binary-heap Dijkstra shortest path traversal",
        run: algorithms::kernel_dijkstra_shortest_path,
    },
    CatalogKernel {
        id: 800,
        slug: "fft-matmul",
        category: CatalogCategory::Algorithms,
        title: "Radix-2 Cooley-Tukey FFT & cache-tiled dense GEMM",
        run: algorithms::kernel_fft_matmul,
    },
    // Concurrency & lock-free (#701–760)
    CatalogKernel {
        id: 701,
        slug: "mpmc-queue",
        category: CatalogCategory::Concurrency,
        title: "ABA-tagged arena Michael-Scott MPMC lock-free queue",
        run: lockfree::kernel_mpmc_queue,
    },
    CatalogKernel {
        id: 710,
        slug: "treiber-stack",
        category: CatalogCategory::Concurrency,
        title: "ABA-tagged Treiber lock-free stack",
        run: lockfree::kernel_treiber_stack,
    },
    CatalogKernel {
        id: 720,
        slug: "spsc-ring",
        category: CatalogCategory::Concurrency,
        title: "Wait-free bounded SPSC ring buffer",
        run: lockfree::kernel_spsc_ring,
    },
    CatalogKernel {
        id: 730,
        slug: "seqlock",
        category: CatalogCategory::Concurrency,
        title: "Versioned SeqLock optimistic reader-writer",
        run: lockfree::kernel_seqlock,
    },
    CatalogKernel {
        id: 740,
        slug: "ticket-lock",
        category: CatalogCategory::Concurrency,
        title: "Fair FIFO ticket spinlock",
        run: lockfree::kernel_ticket_lock,
    },
    CatalogKernel {
        id: 750,
        slug: "striped-counter",
        category: CatalogCategory::Concurrency,
        title: "64-byte cache-line padded striped atomic counter",
        run: lockfree::kernel_striped_counter,
    },
    // Post-quantum & lattice crypto (#892–900)
    CatalogKernel {
        id: 892,
        slug: "kyber-ntt",
        category: CatalogCategory::CryptoLattice,
        title: "ML-KEM/Kyber Number-Theoretic Transform (N=256, q=3329)",
        run: crypto::kernel_kyber_ntt,
    },
    CatalogKernel {
        id: 895,
        slug: "lwe-pke",
        category: CatalogCategory::CryptoLattice,
        title: "Ring-LWE keygen, encrypt & decrypt over NTT domain",
        run: crypto::kernel_lwe_pke,
    },
    CatalogKernel {
        id: 896,
        slug: "lamport-ots",
        category: CatalogCategory::CryptoLattice,
        title: "Lamport SHA-256 one-time signature sign & verify",
        run: crypto::kernel_lamport_ots,
    },
    CatalogKernel {
        id: 897,
        slug: "wots-plus",
        category: CatalogCategory::CryptoLattice,
        title: "Winternitz WOTS+ (w=16) chain sign & verify",
        run: crypto::kernel_wots_plus,
    },
    CatalogKernel {
        id: 898,
        slug: "merkle-tree",
        category: CatalogCategory::CryptoLattice,
        title: "SHA-256 Merkle root & authentication path verify",
        run: crypto::kernel_merkle_tree,
    },
    CatalogKernel {
        id: 899,
        slug: "constant-time-ops",
        category: CatalogCategory::CryptoLattice,
        title: "Branchless constant-time ct_eq & ct_select_u64",
        run: crypto::kernel_constant_time_ops,
    },
];

pub fn all_kernels() -> &'static [CatalogKernel] {
    KERNELS
}

pub fn find_kernel(slug_or_id: &str) -> Option<&'static CatalogKernel> {
    let trimmed = slug_or_id.trim();
    if let Ok(id) = trimmed.parse::<u16>() {
        return KERNELS.iter().find(|k| k.id == id);
    }
    KERNELS
        .iter()
        .find(|k| k.slug.eq_ignore_ascii_case(trimmed))
}

pub fn by_category(cat: CatalogCategory) -> Vec<&'static CatalogKernel> {
    KERNELS.iter().filter(|k| k.category == cat).collect()
}
