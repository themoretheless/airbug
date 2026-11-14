//! System & scheduling overhead probes (`#11–15`, `#17–20`):
//! yield latency, interrupt jitter, SMT contention, locality stride, thread affinity,
//! energy-per-op accounting, wake-from-idle penalty, and warmup decomposition.
use crate::{Result, error};
use serde::{Deserialize, Serialize};
use std::hint::black_box;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldOverheadReport {
    pub iterations: usize,
    pub total_ns: u128,
    pub ns_per_yield: f64,
}

/// `#11` Measure scheduler `thread::yield_now()` cost across `iterations`.
pub fn measure_yield_overhead(iterations: usize) -> YieldOverheadReport {
    let iterations = iterations.max(1);
    let t0 = Instant::now();
    for _ in 0..iterations {
        std::thread::yield_now();
    }
    let total_ns = t0.elapsed().as_nanos();
    YieldOverheadReport {
        iterations,
        total_ns,
        ns_per_yield: total_ns as f64 / iterations as f64,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitterReport {
    pub samples: usize,
    pub median_ns: f64,
    pub p99_ns: f64,
    pub max_ns: u64,
    pub spike_ratio: f64,
}

/// `#12` Tight-loop timer delta check exposing OS timer/interrupt preemption spikes.
pub fn measure_interrupt_jitter(samples: usize) -> JitterReport {
    let samples = samples.max(16);
    let mut deltas = Vec::with_capacity(samples);
    for _ in 0..samples {
        let start = Instant::now();
        let mut x = 0u64;
        for i in 0..256u64 {
            x = black_box(x.wrapping_add(i));
        }
        black_box(x);
        deltas.push(start.elapsed().as_nanos().min(u64::MAX as u128) as u64);
    }
    deltas.sort_unstable();
    let median_ns = deltas[deltas.len() / 2] as f64;
    let p99_idx = ((deltas.len() - 1) * 99) / 100;
    let p99_ns = deltas[p99_idx] as f64;
    let max_ns = *deltas.last().unwrap_or(&0);
    let spike_ratio = if median_ns > 0.0 {
        max_ns as f64 / median_ns
    } else {
        1.0
    };
    JitterReport {
        samples,
        median_ns,
        p99_ns,
        max_ns,
        spike_ratio,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtContentionReport {
    pub iterations: usize,
    pub single_ns: u128,
    pub concurrent_pair_max_ns: u128,
    pub slowdown_factor: f64,
}

fn integer_compute_kernel(iterations: usize) -> u64 {
    let mut acc = 0x1234_5678_9abc_def0u64;
    for i in 0..iterations as u64 {
        acc = acc.wrapping_mul(6364136223846793005).wrapping_add(i);
        acc ^= acc.rotate_left(17);
    }
    black_box(acc)
}

/// `#13` Measure throughput degradation from 1 thread to 2 contending threads (SMT/core sharing check).
pub fn measure_smt_contention(iterations: usize) -> SmtContentionReport {
    let iterations = iterations.max(1_000);
    let t0 = Instant::now();
    black_box(integer_compute_kernel(iterations));
    let single_ns = t0.elapsed().as_nanos().max(1);

    let barrier = Arc::new(Barrier::new(2));
    let b1 = Arc::clone(&barrier);
    let b2 = Arc::clone(&barrier);
    let h1 = std::thread::spawn(move || {
        b1.wait();
        let s = Instant::now();
        black_box(integer_compute_kernel(iterations));
        s.elapsed().as_nanos()
    });
    let h2 = std::thread::spawn(move || {
        b2.wait();
        let s = Instant::now();
        black_box(integer_compute_kernel(iterations));
        s.elapsed().as_nanos()
    });
    let d1 = h1.join().unwrap_or(single_ns);
    let d2 = h2.join().unwrap_or(single_ns);
    let concurrent_pair_max_ns = d1.max(d2).max(1);
    SmtContentionReport {
        iterations,
        single_ns,
        concurrent_pair_max_ns,
        slowdown_factor: concurrent_pair_max_ns as f64 / single_ns as f64,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalityReport {
    pub buffer_bytes: usize,
    pub stride_bytes: usize,
    pub steps: usize,
    pub ns_per_step: f64,
}

/// `#14` Strided memory probe measuring cache-line / memory locality effects.
pub fn measure_locality_stride(buffer_bytes: usize, stride_bytes: usize) -> LocalityReport {
    let words = (buffer_bytes / std::mem::size_of::<usize>()).max(64);
    let stride_words = (stride_bytes / std::mem::size_of::<usize>()).max(1);
    let buf: Vec<usize> = (0..words).collect();
    let steps = words.max(4096);
    let mut idx = 0usize;
    let mut sum = 0usize;
    let t0 = Instant::now();
    for _ in 0..steps {
        sum = sum.wrapping_add(unsafe { *buf.get_unchecked(idx) });
        idx = (idx + stride_words) % words;
    }
    black_box(sum);
    let ns = t0.elapsed().as_nanos();
    LocalityReport {
        buffer_bytes: words * std::mem::size_of::<usize>(),
        stride_bytes: stride_words * std::mem::size_of::<usize>(),
        steps,
        ns_per_step: ns as f64 / steps as f64,
    }
}

/// `#15` Pin the calling thread to `cpu` index where supported (`Ok(true)` if applied, `Ok(false)` on unsupported targets).
pub fn pin_current_thread(cpu: usize) -> Result<bool> {
    #[cfg(target_os = "linux")]
    {
        if cpu >= 1024 {
            return Err(error("cpu index exceeds cpu_set_t width (1024)"));
        }
        // SAFETY: `cpu_set_t` is zero-initialized and populated via `CPU_SET` within bounds.
        let rc = unsafe {
            let mut set: libc::cpu_set_t = std::mem::zeroed();
            libc::CPU_ZERO(&mut set);
            libc::CPU_SET(cpu, &mut set);
            libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &set)
        };
        if rc != 0 {
            return Err(error(format!(
                "sched_setaffinity failed: {}",
                std::io::Error::last_os_error()
            )));
        }
        return Ok(true);
    }
    #[cfg(target_os = "windows")]
    {
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetCurrentThread() -> *mut core::ffi::c_void;
            fn SetThreadAffinityMask(thread: *mut core::ffi::c_void, mask: usize) -> usize;
        }
        if cpu >= usize::BITS as usize {
            return Err(error(
                "cpu index exceeds pointer-width thread affinity mask",
            ));
        }
        let mask = 1usize << cpu;
        // SAFETY: `GetCurrentThread()` returns a valid pseudo-handle; `mask` has bit `cpu` set.
        let prev = unsafe { SetThreadAffinityMask(GetCurrentThread(), mask) };
        if prev == 0 {
            return Err(error(format!(
                "SetThreadAffinityMask failed: {}",
                std::io::Error::last_os_error()
            )));
        }
        Ok(true)
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = cpu;
        Ok(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnergyPerOp {
    pub operations: u64,
    pub energy_uj: u64,
    pub wall_ns: u128,
    pub uj_per_op: f64,
    pub ops_per_joule: f64,
    pub avg_watts: f64,
}

impl EnergyPerOp {
    /// `#17–18` Compute energy-per-op, ops-per-joule efficiency, and mean package power.
    pub fn from_counters(operations: u64, energy_uj: u64, wall_ns: u128) -> Option<Self> {
        if operations == 0 || wall_ns == 0 {
            return None;
        }
        let uj = energy_uj as f64;
        let joules = uj / 1_000_000.0;
        let secs = wall_ns as f64 / 1_000_000_000.0;
        Some(Self {
            operations,
            energy_uj,
            wall_ns,
            uj_per_op: uj / operations as f64,
            ops_per_joule: if joules > 0.0 {
                operations as f64 / joules
            } else {
                f64::INFINITY
            },
            avg_watts: joules / secs,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdleWakeReport {
    pub requested_sleep_ns: u128,
    pub trials: usize,
    pub median_overshoot_ns: f64,
    pub max_overshoot_ns: u128,
}

/// `#19` Measure timer wake-from-idle overshoot relative to requested sleep duration.
pub fn measure_idle_wake_penalty(sleep_dur: Duration, trials: usize) -> IdleWakeReport {
    let trials = trials.max(3);
    let requested = sleep_dur.as_nanos();
    let mut overshoots = Vec::with_capacity(trials);
    for _ in 0..trials {
        let t0 = Instant::now();
        std::thread::sleep(sleep_dur);
        overshoots.push(t0.elapsed().as_nanos().saturating_sub(requested));
    }
    overshoots.sort_unstable();
    IdleWakeReport {
        requested_sleep_ns: requested,
        trials,
        median_overshoot_ns: overshoots[overshoots.len() / 2] as f64,
        max_overshoot_ns: *overshoots.last().unwrap_or(&0),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WarmupDecomposition {
    pub steady_index: usize,
    pub transient_mean: f64,
    pub steady_mean: f64,
    pub cold_to_steady_ratio: f64,
}

/// `#20` Decompose an ordered series into initial transient warmup samples and steady-state suffix.
pub fn decompose_warmup(samples: &[f64]) -> Option<WarmupDecomposition> {
    if samples.len() < 4 || samples.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let half = samples.len() / 2;
    let mut tail = samples[half..].to_vec();
    tail.sort_by(f64::total_cmp);
    let tail_median = tail[tail.len() / 2];
    let mad = {
        let mut dev: Vec<f64> = tail.iter().map(|x| (x - tail_median).abs()).collect();
        dev.sort_by(f64::total_cmp);
        dev[dev.len() / 2].max(tail_median.abs() * 0.05).max(1e-9)
    };

    let threshold = tail_median + 3.0 * mad;
    let steady_index = samples
        .windows(2)
        .position(|w| w[0] <= threshold && w[1] <= threshold)
        .unwrap_or(0);

    let transient_mean = if steady_index == 0 {
        samples[0]
    } else {
        samples[..steady_index].iter().sum::<f64>() / steady_index as f64
    };
    let steady_mean =
        samples[steady_index..].iter().sum::<f64>() / (samples.len() - steady_index) as f64;
    let cold_to_steady_ratio = if steady_mean.abs() > 0.0 {
        transient_mean / steady_mean
    } else {
        1.0
    };
    Some(WarmupDecomposition {
        steady_index,
        transient_mean,
        steady_mean,
        cold_to_steady_ratio,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overhead_and_warmup_probes_produce_valid_metrics() {
        let y = measure_yield_overhead(8);
        assert_eq!(y.iterations, 8);
        let j = measure_interrupt_jitter(16);
        assert!(j.p99_ns >= j.median_ns);
        let smt = measure_smt_contention(5_000);
        assert!(smt.slowdown_factor > 0.0);
        let loc = measure_locality_stride(4096, 64);
        assert!(loc.ns_per_step >= 0.0);
        let pinned = pin_current_thread(0).unwrap();
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        assert!(pinned);
        let _ = pinned;

        let e = EnergyPerOp::from_counters(1_000_000, 500_000, 100_000_000).unwrap();
        assert!((e.uj_per_op - 0.5).abs() < 1e-9);
        assert!((e.avg_watts - 5.0).abs() < 1e-6);

        let w = measure_idle_wake_penalty(Duration::from_millis(1), 3);
        assert_eq!(w.trials, 3);

        let d = decompose_warmup(&[500.0, 300.0, 100.0, 102.0, 99.0, 101.0, 100.0]).unwrap();
        assert_eq!(d.steady_index, 2);
        assert!(d.cold_to_steady_ratio > 3.0);
    }
}
