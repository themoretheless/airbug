//! Sorting, search, string, graph, data-structure, and numerical kernels (`#401–700`, `#761–891`).
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub fn insertion_sort_u64(slice: &mut [u64]) {
    for i in 1..slice.len() {
        let key = slice[i];
        let mut j = i;
        while j > 0 && slice[j - 1] > key {
            slice[j] = slice[j - 1];
            j -= 1;
        }
        slice[j] = key;
    }
}

pub fn radix_sort_u32(slice: &mut [u32]) {
    if slice.len() <= 1 {
        return;
    }
    let mut tmp = vec![0u32; slice.len()];
    for shift in [0u32, 8, 16, 24] {
        let mut counts = [0usize; 256];
        for &v in slice.iter() {
            counts[((v >> shift) & 0xff) as usize] += 1;
        }
        let mut sum = 0usize;
        for c in &mut counts {
            let n = *c;
            *c = sum;
            sum += n;
        }
        for &v in slice.iter() {
            let bucket = ((v >> shift) & 0xff) as usize;
            tmp[counts[bucket]] = v;
            counts[bucket] += 1;
        }
        slice.copy_from_slice(&tmp);
    }
}

pub fn kmp_search(haystack: &[u8], needle: &[u8]) -> usize {
    if needle.is_empty() {
        return 0;
    }
    let mut lps = vec![0usize; needle.len()];
    let mut len = 0usize;
    let mut i = 1usize;
    while i < needle.len() {
        if needle[i] == needle[len] {
            len += 1;
            lps[i] = len;
            i += 1;
        } else if len != 0 {
            len = lps[len - 1];
        } else {
            lps[i] = 0;
            i += 1;
        }
    }
    let mut matches = 0usize;
    let (mut hi, mut ni) = (0usize, 0usize);
    while hi < haystack.len() {
        if haystack[hi] == needle[ni] {
            hi += 1;
            ni += 1;
            if ni == needle.len() {
                matches += 1;
                ni = lps[ni - 1];
            }
        } else if ni != 0 {
            ni = lps[ni - 1];
        } else {
            hi += 1;
        }
    }
    matches
}

pub fn bmh_search(haystack: &[u8], needle: &[u8]) -> usize {
    let m = needle.len();
    if m == 0 || haystack.len() < m {
        return 0;
    }
    let mut skip = [m; 256];
    for i in 0..m - 1 {
        skip[needle[i] as usize] = m - 1 - i;
    }
    let mut i = 0usize;
    let mut count = 0usize;
    while i <= haystack.len() - m {
        if &haystack[i..i + m] == needle {
            count += 1;
        }
        i += skip[haystack[i + m - 1] as usize];
    }
    count
}

pub fn edit_distance(a: &[u8], b: &[u8]) -> usize {
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for (i, &ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

pub struct FenwickTree {
    bit: Vec<i64>,
}

impl FenwickTree {
    pub fn new(n: usize) -> Self {
        Self {
            bit: vec![0; n + 1],
        }
    }

    pub fn add(&mut self, mut idx: usize, delta: i64) {
        idx += 1;
        while idx < self.bit.len() {
            self.bit[idx] += delta;
            idx += idx & idx.wrapping_neg();
        }
    }

    pub fn sum_prefix(&self, mut idx: usize) -> i64 {
        let mut s = 0i64;
        idx += 1;
        while idx > 0 {
            s += self.bit[idx];
            idx -= idx & idx.wrapping_neg();
        }
        s
    }
}

pub struct DisjointSet {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl DisjointSet {
    pub fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    pub fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    pub fn union(&mut self, a: usize, b: usize) -> bool {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return false;
        }
        if self.rank[ra] < self.rank[rb] {
            self.parent[ra] = rb;
        } else {
            self.parent[rb] = ra;
            if self.rank[ra] == self.rank[rb] {
                self.rank[ra] += 1;
            }
        }
        true
    }
}

pub struct BloomFilter {
    words: Vec<u64>,
}

impl BloomFilter {
    pub fn with_bits(bits: usize) -> Self {
        let words = bits.max(64).div_ceil(64);
        Self {
            words: vec![0; words],
        }
    }

    fn hashes(key: u64, nbits: usize) -> (usize, usize) {
        let h1 = key.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        let h2 = h1.rotate_left(27).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        ((h1 as usize) % nbits, (h2 as usize) % nbits)
    }

    pub fn insert(&mut self, key: u64) {
        let (a, b) = Self::hashes(key, self.words.len() * 64);
        self.words[a >> 6] |= 1u64 << (a & 63);
        self.words[b >> 6] |= 1u64 << (b & 63);
    }

    pub fn contains(&self, key: u64) -> bool {
        let (a, b) = Self::hashes(key, self.words.len() * 64);
        (self.words[a >> 6] >> (a & 63)) & 1 != 0 && (self.words[b >> 6] >> (b & 63)) & 1 != 0
    }
}

pub fn dijkstra_grid(nodes: usize, adj: &[Vec<(usize, u64)>], src: usize) -> Vec<u64> {
    let mut dist = vec![u64::MAX; nodes];
    dist[src] = 0;
    let mut pq = BinaryHeap::new();
    pq.push(Reverse((0u64, src)));
    while let Some(Reverse((d, u))) = pq.pop() {
        if d > dist[u] {
            continue;
        }
        for &(v, w) in &adj[u] {
            let nd = d.saturating_add(w);
            if nd < dist[v] {
                dist[v] = nd;
                pq.push(Reverse((nd, v)));
            }
        }
    }
    dist
}

/// Radix-2 iterative Cooley–Tukey complex FFT (power-of-two length).
pub fn fft_radix2(re: &mut [f64], im: &mut [f64]) {
    let n = re.len();
    assert_eq!(n, im.len());
    assert!(n.is_power_of_two());
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2usize;
    while len <= n {
        let ang = -2.0 * std::f64::consts::PI / (len as f64);
        let (wlen_im, wlen_re) = ang.sin_cos();
        let mut i = 0usize;
        while i < n {
            let (mut w_re, mut w_im) = (1.0, 0.0);
            for k in 0..len / 2 {
                let u_re = re[i + k];
                let u_im = im[i + k];
                let vr = re[i + k + len / 2] * w_re - im[i + k + len / 2] * w_im;
                let vi = re[i + k + len / 2] * w_im + im[i + k + len / 2] * w_re;
                re[i + k] = u_re + vr;
                im[i + k] = u_im + vi;
                re[i + k + len / 2] = u_re - vr;
                im[i + k + len / 2] = u_im - vi;
                let nre = w_re * wlen_re - w_im * wlen_im;
                w_im = w_re * wlen_im + w_im * wlen_re;
                w_re = nre;
            }
            i += len;
        }
        len <<= 1;
    }
}

pub fn dense_matmul_tiled(n: usize, a: &[f64], b: &[f64], c: &mut [f64]) {
    c.fill(0.0);
    const TILE: usize = 8;
    for i0 in (0..n).step_by(TILE) {
        for k0 in (0..n).step_by(TILE) {
            for j0 in (0..n).step_by(TILE) {
                let i_max = (i0 + TILE).min(n);
                let k_max = (k0 + TILE).min(n);
                let j_max = (j0 + TILE).min(n);
                for i in i0..i_max {
                    for k in k0..k_max {
                        let aik = a[i * n + k];
                        for j in j0..j_max {
                            c[i * n + j] += aik * b[k * n + j];
                        }
                    }
                }
            }
        }
    }
}

pub fn kernel_radix_sort(iters: usize) -> u64 {
    let n = (iters * 4).clamp(128, 4096);
    let mut v: Vec<u32> = (0..n)
        .map(|i| (i as u32).wrapping_mul(0x9e37_79b1) ^ 0x5555_5555)
        .collect();
    radix_sort_u32(&mut v);
    u64::from(v[0]) ^ u64::from(v[v.len() - 1])
}

pub fn kernel_kmp_bmh(iters: usize) -> u64 {
    let hay = b"abracadabra_airbug_bench_kernel_abracadabra_kmp_bmh_haystack_repeat".repeat(8);
    let pat = b"abracadabra";
    let mut acc = 0usize;
    for _ in 0..iters.max(8) {
        acc += kmp_search(&hay, pat) + bmh_search(&hay, pat);
    }
    acc as u64
}

pub fn kernel_edit_distance(iters: usize) -> u64 {
    let a = b"kitten_benchmark_sequence_alpha_12345";
    let b = b"sitting_benchmark_sequence_beta_12345";
    let mut acc = 0usize;
    for _ in 0..iters.max(4) {
        acc += edit_distance(a, b);
    }
    acc as u64
}

pub fn kernel_fenwick_dsu_bloom(iters: usize) -> u64 {
    let mut ft = FenwickTree::new(256);
    let mut dsu = DisjointSet::new(256);
    let mut bloom = BloomFilter::with_bits(2048);
    let mut acc = 0u64;
    for i in 0..iters.max(64) {
        let idx = i & 255;
        ft.add(idx, 1);
        dsu.union(idx, (idx + 7) & 255);
        bloom.insert(i as u64);
        acc = acc
            .wrapping_add(ft.sum_prefix(idx) as u64)
            .wrapping_add(dsu.find(idx) as u64)
            .wrapping_add(u64::from(bloom.contains(i as u64)));
    }
    acc
}

pub fn kernel_dijkstra_shortest_path(iters: usize) -> u64 {
    let n = 32usize;
    let mut adj = vec![Vec::new(); n];
    for (u, edges) in adj.iter_mut().enumerate().take(n) {
        edges.push(((u + 1) % n, 1));
        edges.push(((u + 5) % n, 3));
    }
    let mut acc = 0u64;
    for r in 0..iters.max(4) {
        let d = dijkstra_grid(n, &adj, r % n);
        acc = acc.wrapping_add(d[(r + 13) % n]);
    }
    acc
}

pub fn kernel_fft_matmul(iters: usize) -> u64 {
    let mut re = vec![1.0f64; 64];
    let mut im = vec![0.0f64; 64];
    let a = vec![1.0f64; 16 * 16];
    let b = vec![2.0f64; 16 * 16];
    let mut c = vec![0.0f64; 16 * 16];
    let mut acc = 0u64;
    for r in 0..iters.max(4) {
        re[r & 63] += 0.25;
        fft_radix2(&mut re, &mut im);
        dense_matmul_tiled(16, &a, &b, &mut c);
        acc = acc.wrapping_add(re[0].to_bits() ^ c[0].to_bits());
    }
    acc
}
