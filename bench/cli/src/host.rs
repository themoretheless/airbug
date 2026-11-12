//! What the machine was doing between measured processes: CPU frequency, temperature and
//! thermal-throttle events.
//!
//! Read from sysfs, so Linux only and no privileges; other platforms record nothing. Samples
//! are taken between processes, never while one is being measured. The series goes to
//! `host.json`; a run that was throttled, or whose frequency sagged, says so in its notes —
//! a slower candidate measured on a hotter machine is not a regression.
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Frequency drop between the first and the last sample that is worth a note.
const SAG_PERCENT: f64 = 10.0;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Sample {
    /// Measured processes completed so far.
    pub after_process: u32,
    /// Mean current frequency over the CPUs that report one, kHz.
    pub cpu_khz: Option<u64>,
    /// Hottest thermal zone, °C.
    pub max_temp_c: Option<f64>,
    /// Thermal-throttle events since boot, summed over cores.
    pub throttle_events: Option<u64>,
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
        let khz: Vec<u64> = cpus
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
        Sample {
            after_process,
            cpu_khz: (!khz.is_empty()).then(|| khz.iter().sum::<u64>() / khz.len() as u64),
            max_temp_c: temps,
            throttle_events: (!throttle.is_empty()).then(|| throttle.iter().sum()),
        }
    }

    /// Take a sample; cheap, and silent where the machine reports nothing.
    pub fn observe(&mut self, after_process: u32) {
        let sample = self.read(after_process);
        if sample.cpu_khz.is_some()
            || sample.max_temp_c.is_some()
            || sample.throttle_events.is_some()
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
                throttle_events: Some(4)
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
}
