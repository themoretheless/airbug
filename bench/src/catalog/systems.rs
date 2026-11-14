//! Systems, memory hierarchy, serialization, parsing, and observability kernels (`#22–100`, `#129–170`, `#207–300`, `#901–1000`).

/// Encode `u64` into unsigned LEB128 varint bytes, returning bytes written.
pub fn leb128_encode(mut v: u64, out: &mut Vec<u8>) -> usize {
    let start = out.len();
    loop {
        let mut b = (v & 0x7f) as u8;
        v >>= 7;
        if v != 0 {
            b |= 0x80;
        }
        out.push(b);
        if v == 0 {
            return out.len() - start;
        }
    }
}

pub fn leb128_decode(bytes: &[u8]) -> Option<(u64, usize)> {
    let mut out = 0u64;
    let mut shift = 0u32;
    for (i, &b) in bytes.iter().enumerate() {
        out |= u64::from(b & 0x7f).checked_shl(shift)?;
        if b & 0x80 == 0 {
            return Some((out, i + 1));
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }
    None
}

const B64_ALPHA: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = chunk.get(1).copied().map(u32::from).unwrap_or(0);
        let b2 = chunk.get(2).copied().map(u32::from).unwrap_or(0);
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64_ALPHA[((triple >> 18) & 0x3f) as usize]);
        out.push(B64_ALPHA[((triple >> 12) & 0x3f) as usize]);
        out.push(if chunk.len() > 1 {
            B64_ALPHA[((triple >> 6) & 0x3f) as usize]
        } else {
            b'='
        });
        out.push(if chunk.len() > 2 {
            B64_ALPHA[(triple & 0x3f) as usize]
        } else {
            b'='
        });
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceContext {
    pub version: u8,
    pub trace_id: [u8; 16],
    pub parent_id: [u8; 8],
    pub flags: u8,
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn parse_hex_exact<const N: usize>(src: &[u8]) -> Option<[u8; N]> {
    if src.len() != N * 2 {
        return None;
    }
    let mut out = [0u8; N];
    for i in 0..N {
        let hi = hex_nibble(src[2 * i])?;
        let lo = hex_nibble(src[2 * i + 1])?;
        out[i] = (hi << 4) | lo;
    }
    Some(out)
}

/// `#250` Parse W3C `traceparent` header (`00-<32hex>-<16hex>-<2hex>`).
pub fn parse_traceparent(header: &str) -> Option<TraceContext> {
    let mut parts = header.trim().split('-');
    let ver = parse_hex_exact::<1>(parts.next()?.as_bytes())?[0];
    if ver == 0xff {
        return None;
    }
    let trace_id = parse_hex_exact::<16>(parts.next()?.as_bytes())?;
    let parent_id = parse_hex_exact::<8>(parts.next()?.as_bytes())?;
    let flags = parse_hex_exact::<1>(parts.next()?.as_bytes())?[0];
    if parts.next().is_some()
        || trace_id.iter().all(|&b| b == 0)
        || parent_id.iter().all(|&b| b == 0)
    {
        return None;
    }
    Some(TraceContext {
        version: ver,
        trace_id,
        parent_id,
        flags,
    })
}

/// Google Jump Consistent Hash assigning `key` to bucket in `0..num_buckets`.
pub fn chash_jump(mut key: u64, num_buckets: i32) -> i32 {
    assert!(num_buckets > 0);
    let mut b: i64 = -1;
    let mut j: i64 = 0;
    while j < i64::from(num_buckets) {
        b = j;
        key = key.wrapping_mul(2_862_933_555_777_941_757).wrapping_add(1);
        j = (((b + 1) as f64) * (((1i64 << 31) as f64) / (((key >> 33) + 1) as f64))) as i64;
    }
    b as i32
}

/// Fixed-bucket logarithmic latency histogram (powers of 2 nanoseconds).
pub struct Log2Histogram {
    buckets: [u64; 64],
    total: u64,
}

impl Default for Log2Histogram {
    fn default() -> Self {
        Self::new()
    }
}

impl Log2Histogram {
    pub const fn new() -> Self {
        Self {
            buckets: [0; 64],
            total: 0,
        }
    }

    pub fn record(&mut self, value: u64) {
        let b = if value == 0 {
            0
        } else {
            (63 - value.leading_zeros() as usize).min(63)
        };
        self.buckets[b] += 1;
        self.total += 1;
    }

    pub fn quantile_upper_bound(&self, q: f64) -> u64 {
        if self.total == 0 {
            return 0;
        }
        let target = ((self.total as f64) * q.clamp(0.0, 1.0)).ceil() as u64;
        let target = target.max(1);
        let mut seen = 0u64;
        for (idx, &count) in self.buckets.iter().enumerate() {
            seen += count;
            if seen >= target {
                return if idx == 63 {
                    u64::MAX
                } else {
                    1u64 << (idx + 1)
                };
            }
        }
        u64::MAX
    }
}

/// Deterministic integer token-bucket rate limiter.
pub struct TokenBucket {
    capacity: u64,
    tokens: u64,
    rate_per_tick: u64,
    last_tick: u64,
}

impl TokenBucket {
    pub fn new(capacity: u64, rate_per_tick: u64) -> Self {
        Self {
            capacity,
            tokens: capacity,
            rate_per_tick,
            last_tick: 0,
        }
    }

    pub fn allow(&mut self, tick: u64, cost: u64) -> bool {
        let elapsed = tick.saturating_sub(self.last_tick);
        self.last_tick = tick;
        self.tokens = self
            .tokens
            .saturating_add(elapsed.saturating_mul(self.rate_per_tick))
            .min(self.capacity);
        if self.tokens >= cost {
            self.tokens -= cost;
            true
        } else {
            false
        }
    }
}

/// Vitter Algorithm R reservoir sampler for streaming cardinality `k`.
pub fn reservoir_sample(stream: &[u64], k: usize, mut seed: u64) -> Vec<u64> {
    if k == 0 {
        return Vec::new();
    }
    let mut res = Vec::with_capacity(k.min(stream.len()));
    for (i, &item) in stream.iter().enumerate() {
        if i < k {
            res.push(item);
        } else {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let j = ((seed >> 16) as usize) % (i + 1);
            if j < k {
                res[j] = item;
            }
        }
    }
    res
}

pub fn kernel_pointer_chase(iters: usize) -> u64 {
    let len = 512usize;
    let mut ring = vec![0usize; len];
    for (i, slot) in ring.iter_mut().enumerate() {
        *slot = (i + 17) % len;
    }
    let mut cur = 0usize;
    for _ in 0..iters.max(512) {
        cur = ring[cur];
    }
    cur as u64
}

pub fn kernel_leb128_base64(iters: usize) -> u64 {
    let mut buf = Vec::with_capacity(32);
    let mut acc = 0u64;
    for i in 0..iters.max(32) {
        buf.clear();
        leb128_encode((i as u64).wrapping_mul(123_457), &mut buf);
        if let Some((v, _)) = leb128_decode(&buf) {
            let b64 = base64_encode(&buf);
            acc = acc.wrapping_add(v ^ (b64.len() as u64));
        }
    }
    acc
}

pub fn kernel_traceparent_parse(iters: usize) -> u64 {
    let hdr = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
    let mut acc = 0u64;
    for _ in 0..iters.max(32) {
        if let Some(ctx) = parse_traceparent(hdr) {
            acc = acc.wrapping_add(u64::from(ctx.flags));
        }
    }
    acc
}

pub fn kernel_histogram_reservoir(iters: usize) -> u64 {
    let mut hist = Log2Histogram::new();
    let stream: Vec<u64> = (0..iters.max(128)).map(|i| (i as u64) * 19 + 5).collect();
    for &x in &stream {
        hist.record(x);
    }
    let p99 = hist.quantile_upper_bound(0.99);
    let sample = reservoir_sample(&stream, 16, 42);
    p99.wrapping_add(sample.iter().sum::<u64>())
}

pub fn kernel_consistent_hash_ratelimit(iters: usize) -> u64 {
    let mut tb = TokenBucket::new(100, 10);
    let mut acc = 0u64;
    for i in 0..iters.max(64) {
        let allowed = tb.allow(i as u64 / 4, 3);
        let bucket = chash_jump(i as u64, 64) as u64;
        acc = acc.wrapping_add(bucket + u64::from(allowed));
    }
    acc
}
