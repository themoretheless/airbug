//! What the machine was doing between measured processes: CPU frequency, temperature and
//! thermal-throttle events.
//!
//! Read from sysfs, so Linux only and no privileges; other platforms record nothing. Samples
//! are taken between processes, never while one is being measured. The series goes to
//! `host.json`; a run that was throttled, or whose frequency sagged, says so in its notes —
//! a slower candidate measured on a hotter machine is not a regression.
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Frequency drop between the first and the last sample that is worth a note.
const SAG_PERCENT: f64 = 10.0;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Sample {
    /// Measured processes completed so far.
    pub after_process: u32,
    /// Mean current frequency over the CPUs that report one, kHz.
    pub cpu_khz: Option<u64>,
    /// Hottest thermal zone, °C.
    pub max_temp_c: Option<f64>,
    /// Thermal-throttle events since boot, summed over cores.
    pub throttle_events: Option<u64>,
    /// Cumulative package energy counter from Intel RAPL sysfs (`energy_uj`), microjoules.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_uj: Option<u64>,
}

pub struct Probe {
    root: PathBuf,
    samples: Vec<Sample>,
}

fn number(path: &Path) -> Option<u64> {
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn children(dir: &Path, prefix: &str) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .strip_prefix(prefix)
                .is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()))
        })
        .map(|e| e.path())
        .collect()
}

impl Probe {
    pub fn new() -> Self {
        Self::at(Path::new("/sys"))
    }

    fn at(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            samples: Vec::new(),
        }
    }

    fn read(&self, after_process: u32) -> Sample {
        let cpus = children(&self.root.join("devices/system/cpu"), "cpu");
        let mut khz: Vec<u64> = cpus
            .iter()
            .filter_map(|cpu| number(&cpu.join("cpufreq/scaling_cur_freq")))
            .collect();
        let throttle: Vec<u64> = cpus
            .iter()
            .filter_map(|cpu| number(&cpu.join("thermal_throttle/core_throttle_count")))
            .collect();
        let temps = children(&self.root.join("class/thermal"), "thermal_zone")
            .iter()
            .filter_map(|zone| number(&zone.join("temp")))
            .map(|millidegrees| millidegrees as f64 / 1000.0)
            .fold(None, |hottest: Option<f64>, t| {
                Some(hottest.map_or(t, |h| h.max(t)))
            });
        let energy_uj =
            number(&self.root.join("class/powercap/intel-rapl:0/energy_uj")).or_else(|| {
                number(
                    &self
                        .root
                        .join("class/powercap/intel-rapl/intel-rapl:0/energy_uj"),
                )
            });

        if khz.is_empty()
            && self.root == Path::new("/sys")
            && let Some(os_khz) = fallback_cpu_khz()
        {
            khz.push(os_khz);
        }

        Sample {
            after_process,
            cpu_khz: (!khz.is_empty()).then(|| khz.iter().sum::<u64>() / khz.len() as u64),
            max_temp_c: temps,
            throttle_events: (!throttle.is_empty()).then(|| throttle.iter().sum()),
            energy_uj,
        }
    }

    pub fn snapshot(&self, after_process: u32) -> Sample {
        self.read(after_process)
    }

    pub fn msr_available() -> bool {
        #[cfg(target_os = "linux")]
        {
            Path::new("/dev/cpu/0/msr").exists()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    /// Take a sample; cheap, and silent where the machine reports nothing.
    pub fn observe(&mut self, after_process: u32) {
        let sample = self.read(after_process);
        if sample.cpu_khz.is_some()
            || sample.max_temp_c.is_some()
            || sample.throttle_events.is_some()
            || sample.energy_uj.is_some()
        {
            self.samples.push(sample);
        }
    }

    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }

    /// Notes for the run: throttling and frequency sag between the first and last sample.
    pub fn notes(&self) -> Vec<String> {
        let (Some(first), Some(last)) = (self.samples.first(), self.samples.last()) else {
            return Vec::new();
        };
        let mut notes = Vec::new();
        if let (Some(a), Some(b)) = (first.throttle_events, last.throttle_events)
            && b > a
        {
            notes.push(format!(
                "host: {} thermal-throttle event(s) during the run; timings are not comparable with an unthrottled run (host.json)",
                b - a
            ));
        }
        if let (Some(a), Some(b)) = (first.cpu_khz, last.cpu_khz)
            && a > 0
        {
            let drop = (a as f64 - b as f64) / a as f64 * 100.0;
            if drop >= SAG_PERCENT {
                notes.push(format!(
                    "host: CPU frequency fell {drop:.0}% during the run ({} → {} MHz); later processes ran on a slower machine (host.json)",
                    a / 1000,
                    b / 1000
                ));
            }
        }
        notes
    }
}

/// Pearson correlation; `None` without at least three points or without variation.
fn pearson(points: &[(f64, f64)]) -> Option<f64> {
    if points.len() < 3 {
        return None;
    }
    let n = points.len() as f64;
    let (mx, my) = (
        points.iter().map(|p| p.0).sum::<f64>() / n,
        points.iter().map(|p| p.1).sum::<f64>() / n,
    );
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (x, y) in points {
        sxy += (x - mx) * (y - my);
        sxx += (x - mx) * (x - mx);
        syy += (y - my) * (y - my);
    }
    (sxx > 0.0 && syy > 0.0).then(|| sxy / (sxx * syy).sqrt())
}

/// How each process's timings sat against the machine's state right after it: for
/// `diagnose`. Timings are made comparable across cases by dividing each value by the median
/// of its case, metric and variant, then averaged per process.
pub fn correlate(run: &airbug_bench::Run, samples: &[Sample]) -> serde_json::Value {
    use std::collections::BTreeMap;
    /// (case, metric, variant) → (process, value per operation).
    type Groups<'a> = BTreeMap<(&'a str, &'a str, &'a str), Vec<(u32, f64)>>;
    let mut groups: Groups = BTreeMap::new();
    for o in &run.observations {
        if o.availability != airbug_bench::Availability::Available {
            continue;
        }
        if let Some(value) = o.value.as_deref().and_then(|v| v.parse::<f64>().ok()) {
            groups
                .entry((o.case.as_str(), o.metric.as_str(), o.variant.as_str()))
                .or_default()
                .push((o.process, value / o.operations.max(1) as f64));
        }
    }
    let mut relative: BTreeMap<u32, Vec<f64>> = BTreeMap::new();
    for values in groups.values() {
        let mut sorted: Vec<f64> = values.iter().map(|v| v.1).collect();
        sorted.sort_by(f64::total_cmp);
        let median = sorted[sorted.len() / 2];
        if median > 0.0 {
            for (process, value) in values {
                relative.entry(*process).or_default().push(value / median);
            }
        }
    }
    let timing = |process: u32| {
        relative
            .get(&process)
            .map(|v| v.iter().sum::<f64>() / v.len() as f64)
    };
    // The sample taken after process `k` describes the machine that process just ran on.
    let against = |state: fn(&Sample) -> Option<f64>| {
        let points: Vec<(f64, f64)> = samples
            .iter()
            .filter(|s| s.after_process > 0)
            .filter_map(|s| Some((state(s)?, timing(s.after_process - 1)?)))
            .collect();
        pearson(&points)
    };
    let range = |values: Vec<f64>| {
        let (Some(first), Some(last)) = (values.first(), values.last()) else {
            return serde_json::Value::Null;
        };
        serde_json::json!({
            "first": first, "last": last,
            "min": values.iter().copied().fold(f64::INFINITY, f64::min),
            "max": values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        })
    };
    let throttle: Vec<u64> = samples.iter().filter_map(|s| s.throttle_events).collect();
    let energy: Vec<u64> = samples.iter().filter_map(|s| s.energy_uj).collect();
    serde_json::json!({
        "samples": samples.len(),
        "cpu_khz": range(samples.iter().filter_map(|s| s.cpu_khz.map(|v| v as f64)).collect()),
        "max_temp_c": range(samples.iter().filter_map(|s| s.max_temp_c).collect()),
        "energy_uj": range(energy.iter().map(|v| *v as f64).collect()),
        "energy_uj_during_run": energy.first().zip(energy.last()).map(|(a, b)| b.saturating_sub(*a)),
        "throttle_events_during_run": throttle.first().zip(throttle.last()).map(|(a, b)| b.saturating_sub(*a)),
        "msr_accessible": Probe::msr_available(),
        "timing_vs_cpu_khz_r": against(|s| s.cpu_khz.map(|v| v as f64)),
        "timing_vs_max_temp_c_r": against(|s| s.max_temp_c),
        "timing_vs_energy_uj_r": against(|s| s.energy_uj.map(|v| v as f64)),
        "interpretation": "Pearson r of each process's relative timing against the machine state sampled right after it. Strongly negative for frequency or positive for temperature means the machine, not the code, moved the timings. Descriptive only.",
    })
}

#[cfg(target_os = "windows")]
fn fallback_cpu_khz() -> Option<u64> {
    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct ProcessorPowerInfo {
        number: u32,
        max_mhz: u32,
        current_mhz: u32,
        mhz_limit: u32,
        max_idle_state: u32,
        current_idle_state: u32,
    }
    #[link(name = "powrprof")]
    unsafe extern "system" {
        fn CallNtPowerInformation(
            information_level: i32,
            input_buffer: *const core::ffi::c_void,
            input_buffer_length: u32,
            output_buffer: *mut core::ffi::c_void,
            output_buffer_length: u32,
        ) -> i32;
    }
    let cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .max(1);
    let mut infos = vec![ProcessorPowerInfo::default(); cpus];
    let byte_len = (infos.len() * std::mem::size_of::<ProcessorPowerInfo>()) as u32;
    // SAFETY: ProcessorInformation = 11; output buffer holds `cpus` structs matching Win32 ABI.
    let status = unsafe {
        CallNtPowerInformation(11, std::ptr::null(), 0, infos.as_mut_ptr().cast(), byte_len)
    };
    if status != 0 {
        return None;
    }
    let mhz: Vec<u64> = infos
        .iter()
        .filter(|i| i.current_mhz > 0)
        .map(|i| u64::from(i.current_mhz))
        .collect();
    (!mhz.is_empty()).then(|| (mhz.iter().sum::<u64>() / mhz.len() as u64) * 1000)
}

#[cfg(target_os = "macos")]
fn fallback_cpu_khz() -> Option<u64> {
    if let Ok(out) = std::process::Command::new("sysctl")
        .args(["-n", "hw.cpufrequency"])
        .output()
        && out.status.success()
        && let Ok(text) = String::from_utf8(out.stdout)
        && let Ok(hz) = text.trim().parse::<u64>()
        && hz > 0
    {
        return Some(hz / 1000);
    }
    // On Apple Silicon `hw.cpufrequency` is absent; measure real-time core frequency over a
    // short calibration spin using XNU's `thread_selfcounts` cycle counter against Instant.
    unsafe extern "C" {
        fn thread_selfcounts(kind: libc::c_int, buf: *mut u64, nbytes: libc::size_t)
        -> libc::c_int;
    }
    let mut c0 = [0u64; 2];
    let mut c1 = [0u64; 2];
    let t0 = std::time::Instant::now();
    // SAFETY: `c0` is a 16-byte buffer for `[instructions, cycles]`.
    if unsafe { thread_selfcounts(1, c0.as_mut_ptr(), std::mem::size_of::<[u64; 2]>()) } != 0 {
        return None;
    }
    let mut acc = 1u64;
    for i in 0..50_000u64 {
        acc = std::hint::black_box(acc.wrapping_mul(6364136223846793005).wrapping_add(i));
    }
    std::hint::black_box(acc);
    // SAFETY: `c1` is a 16-byte buffer for `[instructions, cycles]`.
    if unsafe { thread_selfcounts(1, c1.as_mut_ptr(), std::mem::size_of::<[u64; 2]>()) } != 0 {
        return None;
    }
    let elapsed_ns = t0.elapsed().as_nanos() as u64;
    let cycles = c1[1].checked_sub(c0[1])?;
    if elapsed_ns == 0 || cycles == 0 {
        return None;
    }
    // cycles / ns = GHz = 1_000_000 kHz
    let khz = u128::from(cycles)
        .checked_mul(1_000_000)?
        .checked_div(u128::from(elapsed_ns))?;
    u64::try_from(khz).ok().filter(|&v| v > 0)
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn fallback_cpu_khz() -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, rel: &str, text: &str) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    #[test]
    fn throttling_and_frequency_sag_become_notes() {
        let dir = tempfile::tempdir().unwrap();
        let sys = dir.path();
        for (cpu, khz) in [(0, 3_000_000), (1, 3_400_000)] {
            write(
                sys,
                &format!("devices/system/cpu/cpu{cpu}/cpufreq/scaling_cur_freq"),
                &khz.to_string(),
            );
            write(
                sys,
                &format!("devices/system/cpu/cpu{cpu}/thermal_throttle/core_throttle_count"),
                "2\n",
            );
        }
        // Not a CPU: must not be counted.
        write(sys, "devices/system/cpu/cpufreq/scaling_cur_freq", "1");
        write(sys, "class/thermal/thermal_zone0/temp", "45000\n");
        write(sys, "class/thermal/thermal_zone1/temp", "61500\n");

        let mut probe = Probe::at(sys);
        probe.observe(0);
        assert_eq!(
            probe.samples()[0],
            Sample {
                after_process: 0,
                cpu_khz: Some(3_200_000),
                max_temp_c: Some(61.5),
                throttle_events: Some(4),
                energy_uj: None,
            }
        );
        assert!(probe.notes().is_empty(), "one sample is not a trend");

        for cpu in 0..2 {
            write(
                sys,
                &format!("devices/system/cpu/cpu{cpu}/cpufreq/scaling_cur_freq"),
                "2400000",
            );
            write(
                sys,
                &format!("devices/system/cpu/cpu{cpu}/thermal_throttle/core_throttle_count"),
                "5",
            );
        }
        probe.observe(6);
        let notes = probe.notes();
        assert_eq!(notes.len(), 2, "{notes:?}");
        assert!(
            notes[0].contains("6 thermal-throttle event(s)"),
            "{notes:?}"
        );
        assert!(
            notes[1].contains("fell 25%") && notes[1].contains("3200 → 2400 MHz"),
            "{notes:?}"
        );
    }

    #[test]
    fn a_machine_that_reports_nothing_records_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let mut probe = Probe::at(dir.path());
        probe.observe(0);
        assert!(probe.samples().is_empty() && probe.notes().is_empty());
    }

    #[test]
    fn rapl_energy_uj_is_read_and_correlated_from_both_sysfs_layouts() {
        use airbug_bench::{Availability, Observation, Run};
        let dir = tempfile::tempdir().unwrap();
        let sys = dir.path();
        write(sys, "class/powercap/intel-rapl:0/energy_uj", "1000000\n");
        let mut probe = Probe::at(sys);
        probe.observe(0);
        assert_eq!(probe.samples()[0].energy_uj, Some(1_000_000));

        // Nested layout (`class/powercap/intel-rapl/intel-rapl:0/energy_uj`) when direct symlink is absent.
        let dir_nested = tempfile::tempdir().unwrap();
        write(
            dir_nested.path(),
            "class/powercap/intel-rapl/intel-rapl:0/energy_uj",
            "2500000\n",
        );
        let mut probe_nested = Probe::at(dir_nested.path());
        probe_nested.observe(0);
        assert_eq!(probe_nested.samples()[0].energy_uj, Some(2_500_000));

        let mut run = Run::new();
        for (i, uj) in [1_500_000u64, 2_200_000, 3_100_000, 4_200_000]
            .into_iter()
            .enumerate()
        {
            write(
                sys,
                "class/powercap/intel-rapl:0/energy_uj",
                &uj.to_string(),
            );
            probe.observe(i as u32 + 1);
            run.observations.push(Observation {
                case: "work".into(),
                metric: "wall".into(),
                variant: "candidate".into(),
                process: i as u32,
                pair: None,
                sequence: i as u64 + 1,
                value: Some((1000.0 + i as f64 * 150.0).to_string()),
                operations: 1,
                worker_work_totals: Default::default(),
                work_totals: Default::default(),
                availability: Availability::Available,
            });
        }
        let report = correlate(&run, probe.samples());
        assert_eq!(report["energy_uj_during_run"], 3_200_000);
        assert_eq!(report["energy_uj"]["first"], 1_000_000.0);
        assert_eq!(report["energy_uj"]["last"], 4_200_000.0);
        assert!(
            report["timing_vs_energy_uj_r"].as_f64().unwrap() > 0.98,
            "{report}"
        );
    }

    #[test]
    fn timings_that_follow_the_frequency_correlate_with_it() {
        use airbug_bench::{Availability, Observation, Run};
        let mut run = Run::new();
        let khz = [3_200_000u64, 3_000_000, 2_800_000, 2_600_000];
        let mut samples = vec![Sample {
            after_process: 0,
            cpu_khz: Some(3_400_000),
            max_temp_c: Some(40.0),
            throttle_events: Some(1),
            energy_uj: None,
        }];
        for (process, khz) in khz.iter().enumerate() {
            run.observations.push(Observation {
                case: "work".into(),
                metric: "wall".into(),
                variant: "candidate".into(),
                process: process as u32,
                pair: None,
                sequence: process as u64 + 1,
                // Slower clock, longer time.
                value: Some((3.2e9 / *khz as f64 * 1000.0).to_string()),
                operations: 1,
                worker_work_totals: Default::default(),
                work_totals: Default::default(),
                availability: Availability::Available,
            });
            samples.push(Sample {
                after_process: process as u32 + 1,
                cpu_khz: Some(*khz),
                max_temp_c: Some(50.0 + process as f64 * 5.0),
                throttle_events: Some(1 + process as u64),
                energy_uj: None,
            });
        }
        let report = correlate(&run, &samples);
        assert!(
            report["timing_vs_cpu_khz_r"].as_f64().unwrap() < -0.98,
            "{report}"
        );
        assert!(
            report["timing_vs_max_temp_c_r"].as_f64().unwrap() > 0.98,
            "{report}"
        );
        assert_eq!(report["throttle_events_during_run"], 3);
        assert_eq!(report["cpu_khz"]["min"], 2_600_000.0);
        assert!(correlate(&run, &samples[..2])["timing_vs_cpu_khz_r"].is_null());
    }
}
