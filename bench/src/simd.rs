//! SIMD capability detection and vectorized efficiency baselines (`#7`).
use serde::{Deserialize, Serialize};
use std::hint::black_box;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SimdCapabilities {
    pub arch: String,
    pub max_vector_bits: u16,
    pub compile_features: Vec<String>,
    pub runtime_features: Vec<String>,
}

/// Detect compile-time and runtime SIMD extensions supported on the host CPU.
pub fn detect() -> SimdCapabilities {
    let mut compile_features = Vec::new();
    let mut runtime_features = Vec::new();
    #[allow(unused_mut)]
    let mut max_bits: u16 = 64;

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        for (name, enabled) in [
            ("sse2", cfg!(target_feature = "sse2")),
            ("sse4.2", cfg!(target_feature = "sse4.2")),
            ("avx", cfg!(target_feature = "avx")),
            ("avx2", cfg!(target_feature = "avx2")),
            ("fma", cfg!(target_feature = "fma")),
            ("avx512f", cfg!(target_feature = "avx512f")),
        ] {
            if enabled {
                compile_features.push(name.into());
            }
        }
        if std::arch::is_x86_feature_detected!("sse2") {
            runtime_features.push("sse2".into());
            max_bits = max_bits.max(128);
        }
        if std::arch::is_x86_feature_detected!("sse4.2") {
            runtime_features.push("sse4.2".into());
            max_bits = max_bits.max(128);
        }
        if std::arch::is_x86_feature_detected!("avx") {
            runtime_features.push("avx".into());
            max_bits = max_bits.max(256);
        }
        if std::arch::is_x86_feature_detected!("avx2") {
            runtime_features.push("avx2".into());
            max_bits = max_bits.max(256);
        }
        if std::arch::is_x86_feature_detected!("fma") {
            runtime_features.push("fma".into());
            max_bits = max_bits.max(256);
        }
        if std::arch::is_x86_feature_detected!("avx512f") {
            runtime_features.push("avx512f".into());
            max_bits = max_bits.max(512);
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if cfg!(target_feature = "neon") {
            compile_features.push("neon".into());
        }
        runtime_features.push("neon".into());
        max_bits = 128;
    }

    SimdCapabilities {
        arch: std::env::consts::ARCH.into(),
        max_vector_bits: max_bits,
        compile_features,
        runtime_features,
    }
}

/// Scalar `u32` wrapping sum baseline (explicitly prevents unrequested SIMD unrolling in debug).
#[inline(never)]
pub fn sum_u32_scalar(data: &[u32]) -> u32 {
    let mut acc = 0u32;
    for &x in data {
        acc = black_box(acc.wrapping_add(x));
    }
    acc
}

/// Architecture-dispatched `u32` wrapping sum (`AVX2` -> `SSE2` -> NEON -> 8-lane unrolled fallback).
pub fn sum_u32_simd(data: &[u32]) -> u32 {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 availability verified above at runtime via `is_x86_feature_detected!`.
            return unsafe { sum_u32_avx2(data) };
        }
        if std::arch::is_x86_feature_detected!("sse2") {
            // SAFETY: SSE2 availability verified above at runtime via `is_x86_feature_detected!`.
            return unsafe { sum_u32_sse2(data) };
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: NEON is mandatory on AArch64 targets supported by std.
        return unsafe { sum_u32_neon(data) };
    }
    #[allow(unreachable_code)]
    sum_u32_portable_lanes(data)
}

fn sum_u32_portable_lanes(data: &[u32]) -> u32 {
    let mut lanes = [0u32; 8];
    let (chunks, rem) = data.as_chunks::<8>();
    for c in chunks {
        for (l, &v) in lanes.iter_mut().zip(c.iter()) {
            *l = l.wrapping_add(v);
        }
    }
    let mut total = lanes.into_iter().fold(0u32, u32::wrapping_add);
    for &x in rem {
        total = total.wrapping_add(x);
    }
    total
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn sum_u32_avx2(data: &[u32]) -> u32 {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let mut acc = _mm256_setzero_si256();
    let (chunks, rem) = data.as_chunks::<8>();
    for chunk in chunks {
        // SAFETY: chunk has length 8, aligned load not required for `_mm256_loadu_si256`.
        let vec = unsafe { _mm256_loadu_si256(chunk.as_ptr().cast()) };
        acc = _mm256_add_epi32(acc, vec);
    }
    let mut tmp = [0u32; 8];
    // SAFETY: `tmp` has space for 8 `u32` (32 bytes).
    unsafe { _mm256_storeu_si256(tmp.as_mut_ptr().cast(), acc) };
    let mut total = tmp.into_iter().fold(0u32, u32::wrapping_add);
    for &x in rem {
        total = total.wrapping_add(x);
    }
    total
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse2")]
unsafe fn sum_u32_sse2(data: &[u32]) -> u32 {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let mut acc = _mm_setzero_si128();
    let (chunks, rem) = data.as_chunks::<4>();
    for chunk in chunks {
        // SAFETY: chunk has length 4, unaligned load.
        let vec = unsafe { _mm_loadu_si128(chunk.as_ptr().cast()) };
        acc = _mm_add_epi32(acc, vec);
    }
    let mut tmp = [0u32; 4];
    // SAFETY: `tmp` has space for 4 `u32` (16 bytes).
    unsafe { _mm_storeu_si128(tmp.as_mut_ptr().cast(), acc) };
    let mut total = tmp.into_iter().fold(0u32, u32::wrapping_add);
    for &x in rem {
        total = total.wrapping_add(x);
    }
    total
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn sum_u32_neon(data: &[u32]) -> u32 {
    use std::arch::aarch64::*;
    let mut acc = vdupq_n_u32(0);
    let (chunks, rem) = data.as_chunks::<4>();
    for chunk in chunks {
        // SAFETY: chunk has length 4.
        let vec = unsafe { vld1q_u32(chunk.as_ptr()) };
        acc = vaddq_u32(acc, vec);
    }
    let mut tmp = [0u32; 4];
    // SAFETY: `tmp` has space for 4 `u32`.
    unsafe { vst1q_u32(tmp.as_mut_ptr(), acc) };
    let mut total = tmp.into_iter().fold(0u32, u32::wrapping_add);
    for &x in rem {
        total = total.wrapping_add(x);
    }
    total
}

/// Scalar `f32` dot product.
#[inline(never)]
pub fn dot_f32_scalar(a: &[f32], b: &[f32]) -> f32 {
    let mut acc = 0.0f32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        acc = black_box(acc + x * y);
    }
    acc
}

/// Dispatched `f32` dot product (`AVX2` -> `SSE2` -> NEON -> 4-lane portable).
pub fn dot_f32_simd(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    let (a, b) = (&a[..n], &b[..n]);
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 checked at runtime.
            return unsafe { dot_f32_avx2(a, b) };
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: NEON available on AArch64.
        return unsafe { dot_f32_neon(a, b) };
    }
    #[allow(unreachable_code)]
    {
        let mut acc = [0.0f32; 4];
        let mut i = 0;
        while i + 4 <= n {
            acc[0] += a[i] * b[i];
            acc[1] += a[i + 1] * b[i + 1];
            acc[2] += a[i + 2] * b[i + 2];
            acc[3] += a[i + 3] * b[i + 3];
            i += 4;
        }
        let mut sum = acc.into_iter().sum::<f32>();
        while i < n {
            sum += a[i] * b[i];
            i += 1;
        }
        sum
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn dot_f32_avx2(a: &[f32], b: &[f32]) -> f32 {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let mut acc = _mm256_setzero_ps();
    let mut i = 0;
    while i + 8 <= a.len() {
        // SAFETY: bounds checked by `i + 8 <= a.len()`.
        let va = unsafe { _mm256_loadu_ps(a.as_ptr().add(i)) };
        // SAFETY: bounds checked by `i + 8 <= b.len()` (`a.len() == b.len()`).
        let vb = unsafe { _mm256_loadu_ps(b.as_ptr().add(i)) };
        acc = _mm256_add_ps(acc, _mm256_mul_ps(va, vb));
        i += 8;
    }
    let mut tmp = [0.0f32; 8];
    // SAFETY: `tmp` holds 8 `f32`.
    unsafe { _mm256_storeu_ps(tmp.as_mut_ptr(), acc) };
    let mut sum = tmp.into_iter().sum::<f32>();
    while i < a.len() {
        sum += a[i] * b[i];
        i += 1;
    }
    sum
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn dot_f32_neon(a: &[f32], b: &[f32]) -> f32 {
    use std::arch::aarch64::*;
    let mut acc = vdupq_n_f32(0.0);
    let mut i = 0;
    while i + 4 <= a.len() {
        // SAFETY: bounds checked.
        let va = unsafe { vld1q_f32(a.as_ptr().add(i)) };
        // SAFETY: bounds checked.
        let vb = unsafe { vld1q_f32(b.as_ptr().add(i)) };
        acc = vmlaq_f32(acc, va, vb);
        i += 4;
    }
    let mut tmp = [0.0f32; 4];
    // SAFETY: `tmp` holds 4 `f32`.
    unsafe { vst1q_f32(tmp.as_mut_ptr(), acc) };
    let mut sum = tmp.into_iter().sum::<f32>();
    while i < a.len() {
        sum += a[i] * b[i];
        i += 1;
    }
    sum
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimdEfficiencyReport {
    pub capabilities: SimdCapabilities,
    pub len: usize,
    pub scalar_ns: u128,
    pub simd_ns: u128,
    pub speedup: f64,
    pub theoretical_u32_lanes: u16,
    pub lane_utilization_ratio: f64,
}

impl SimdEfficiencyReport {
    /// Benchmark scalar vs SIMD `sum_u32` over `len` elements and compute lane utilization.
    pub fn measure(len: usize) -> Self {
        let len = len.max(64);
        let data: Vec<u32> = (0..len as u32)
            .map(|i| i.wrapping_mul(2654435761))
            .collect();
        let cap = detect();
        let theoretical_u32_lanes = (cap.max_vector_bits / 32).max(1);

        let t0 = Instant::now();
        let s1 = sum_u32_scalar(black_box(&data));
        let scalar_ns = t0.elapsed().as_nanos().max(1);

        let t1 = Instant::now();
        let s2 = sum_u32_simd(black_box(&data));
        let simd_ns = t1.elapsed().as_nanos().max(1);

        assert_eq!(s1, s2);
        let speedup = scalar_ns as f64 / simd_ns as f64;
        Self {
            capabilities: cap,
            len,
            scalar_ns,
            simd_ns,
            speedup,
            theoretical_u32_lanes,
            lane_utilization_ratio: speedup / f64::from(theoretical_u32_lanes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simd_kernels_match_scalar_and_report_capabilities() {
        let cap = detect();
        assert!(!cap.arch.is_empty());
        assert!(cap.max_vector_bits >= 64);

        let data: Vec<u32> = (1..=1031u32).collect();
        assert_eq!(sum_u32_scalar(&data), sum_u32_simd(&data));

        let a: Vec<f32> = (0..137).map(|i| i as f32 * 0.25).collect();
        let b: Vec<f32> = (0..137).map(|i| (137 - i) as f32 * 0.5).collect();
        let diff = (dot_f32_scalar(&a, &b) - dot_f32_simd(&a, &b)).abs();
        assert!(diff < 1e-2, "diff={diff}");

        let report = SimdEfficiencyReport::measure(4096);
        assert_eq!(report.len, 4096);
        assert!(report.speedup > 0.0);
    }
}
