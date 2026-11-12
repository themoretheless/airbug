//! GPU load and memory, per platform. Every provider degrades to `None`: a host without a
//! readable GPU is sampled like any other.
// One provider is used per platform; the others are still compiled so their parsers are tested
// everywhere.
#![allow(dead_code)]
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpuSample {
    /// Busy share, percent.
    pub util: f32,
    /// Bytes the GPU holds. On a unified-memory machine this is its share of system RAM.
    pub mem_used: u64,
    /// Bytes the GPU can hold; 0 when the platform does not report it. With unified memory
    /// the sampler fills in the size of system RAM, the pool the GPU allocates from.
    pub mem_total: u64,
    /// CPU and GPU allocate from one pool of RAM (Apple Silicon); there is no VRAM of its own.
    pub unified: bool,
    pub temp: Option<f32>,
}

pub trait GpuProvider {
    fn sample(&mut self) -> Option<GpuSample>;
}

pub struct NullProvider;

impl GpuProvider for NullProvider {
    fn sample(&mut self) -> Option<GpuSample> {
        None
    }
}

fn runs(program: &str, arg: &str) -> bool {
    Command::new(program)
        .arg(arg)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn leading_number(text: &str) -> Option<f64> {
    let digits: String = text
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    digits.parse().ok()
}

// ---- macOS: powermetrics (needs sudo) ----

pub struct PowermetricsProvider {
    available: bool,
}

impl PowermetricsProvider {
    pub fn new() -> Self {
        Self {
            available: runs("powermetrics", "--help"),
        }
    }
}

impl GpuProvider for PowermetricsProvider {
    fn sample(&mut self) -> Option<GpuSample> {
        if !self.available {
            return None;
        }
        let output = Command::new("powermetrics")
            .args(["--samplers", "gpu_power", "-i", "200", "-n", "1"])
            .output()
            .ok()?;
        if !output.status.success() {
            self.available = false;
            return None;
        }
        parse_powermetrics(
            &String::from_utf8_lossy(&output.stdout),
            cfg!(target_arch = "aarch64"),
        )
    }
}

/// `unified`: the GPU shares system RAM, so the allocated figure is what it uses and the
/// total is not the GPU's to report.
fn parse_powermetrics(text: &str, unified: bool) -> Option<GpuSample> {
    let mut util = None;
    let mut allocated = 0u64;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("GPU active residency:") {
            util = leading_number(rest).map(|v| v as f32);
        } else if let Some(rest) = line.strip_prefix("GPU HW allocated memory:")
            && let Some(mb) = leading_number(rest)
        {
            allocated = (mb * 1_048_576.0) as u64;
        }
    }
    util.map(|util| GpuSample {
        util,
        mem_used: allocated,
        mem_total: 0,
        unified,
        temp: None,
    })
}

// ---- Linux: nvidia-smi, else the DRM sysfs counters (amdgpu and others) ----

pub struct LinuxProvider {
    nvidia: bool,
    cards: Vec<PathBuf>,
}

impl LinuxProvider {
    pub fn new() -> Self {
        Self {
            nvidia: runs("nvidia-smi", "--list-gpus"),
            cards: drm_cards(Path::new("/sys/class/drm")),
        }
    }
}

/// `cardN/device` directories that report a busy percentage.
fn drm_cards(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut cards: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.strip_prefix("card")
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        })
        .map(|e| e.path().join("device"))
        .filter(|device| device.join("gpu_busy_percent").is_file())
        .collect();
    cards.sort();
    cards
}

impl GpuProvider for LinuxProvider {
    fn sample(&mut self) -> Option<GpuSample> {
        if self.nvidia {
            let output = Command::new("nvidia-smi")
                .args([
                    "--query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu",
                    "--format=csv,noheader,nounits",
                ])
                .output()
                .ok();
            match output.filter(|o| o.status.success()) {
                Some(o) => return parse_nvidia_smi(&String::from_utf8_lossy(&o.stdout)),
                None => self.nvidia = false,
            }
        }
        combine(self.cards.iter().filter_map(|card| read_drm_card(card)))
    }
}

/// One line per GPU: `utilization %, used MiB, total MiB, temperature °C`.
fn parse_nvidia_smi(text: &str) -> Option<GpuSample> {
    combine(text.lines().filter_map(|line| {
        let mut fields = line.split(',').map(|f| f.trim().parse::<f64>().ok());
        let util = fields.next()??;
        let used = fields.next().flatten().unwrap_or(0.0);
        let total = fields.next().flatten().unwrap_or(0.0);
        let temp = fields.next().flatten();
        Some(GpuSample {
            util: util as f32,
            mem_used: (used * 1_048_576.0) as u64,
            mem_total: (total * 1_048_576.0) as u64,
            unified: false,
            temp: temp.map(|t| t as f32),
        })
    }))
}

fn read_drm_card(device: &Path) -> Option<GpuSample> {
    let number = |name: &str| -> Option<u64> {
        std::fs::read_to_string(device.join(name))
            .ok()?
            .trim()
            .parse()
            .ok()
    };
    let temp = std::fs::read_dir(device.join("hwmon"))
        .ok()
        .and_then(|mut dirs| dirs.find_map(|d| d.ok()))
        .and_then(|d| std::fs::read_to_string(d.path().join("temp1_input")).ok())
        .and_then(|t| t.trim().parse::<f32>().ok())
        .map(|millidegrees| millidegrees / 1000.0);
    Some(GpuSample {
        util: number("gpu_busy_percent")? as f32,
        mem_used: number("mem_info_vram_used").unwrap_or(0),
        mem_total: number("mem_info_vram_total").unwrap_or(0),
        unified: false,
        temp,
    })
}

/// Several GPUs as one: the busiest load and temperature, memory summed.
fn combine(samples: impl Iterator<Item = GpuSample>) -> Option<GpuSample> {
    samples.reduce(|a, b| GpuSample {
        util: a.util.max(b.util),
        mem_used: a.mem_used + b.mem_used,
        mem_total: a.mem_total + b.mem_total,
        unified: a.unified && b.unified,
        temp: match (a.temp, b.temp) {
            (Some(x), Some(y)) => Some(x.max(y)),
            (x, y) => x.or(y),
        },
    })
}

pub fn default_provider() -> Box<dyn GpuProvider + Send> {
    #[cfg(target_os = "macos")]
    {
        Box::new(PowermetricsProvider::new())
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(LinuxProvider::new())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Box::new(NullProvider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const POWERMETRICS: &str =
        "**** GPU usage ****\nGPU active residency: 12.34 %\nGPU HW allocated memory: 512 MB\n";

    #[test]
    fn unified_memory_reports_what_the_gpu_holds_not_a_total() {
        let s = parse_powermetrics(POWERMETRICS, true).unwrap();
        assert!((s.util - 12.34).abs() < 0.01);
        assert_eq!(s.mem_used, 512 * 1_048_576);
        assert_eq!(
            s.mem_total, 0,
            "the pool is system RAM; the sampler fills it in"
        );
        assert!(s.unified);
        assert!(!parse_powermetrics(POWERMETRICS, false).unwrap().unified);
        assert!(parse_powermetrics("no gpu lines", true).is_none());
    }

    #[test]
    fn nvidia_smi_rows_are_combined() {
        let s = parse_nvidia_smi("12, 345, 8192, 55\n80, 1000, 8192, 71\n").unwrap();
        assert_eq!(s.util, 80.0);
        assert_eq!(s.mem_used, 1345 * 1_048_576);
        assert_eq!(s.mem_total, 16384 * 1_048_576);
        assert_eq!(s.temp, Some(71.0));
        assert!(!s.unified);
        assert!(parse_nvidia_smi("").is_none());
        // `[N/A]` fields are skipped, the row is kept.
        let partial = parse_nvidia_smi("7, [N/A], [N/A], [N/A]\n").unwrap();
        assert_eq!(
            (partial.util, partial.mem_total, partial.temp),
            (7.0, 0, None)
        );
    }

    #[test]
    fn drm_sysfs_card_is_read_and_others_are_skipped() {
        let root = std::env::temp_dir().join(format!("airbug-mon-drm-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let device = root.join("card0/device");
        std::fs::create_dir_all(device.join("hwmon/hwmon3")).unwrap();
        std::fs::create_dir_all(root.join("card1/device")).unwrap();
        std::fs::create_dir_all(root.join("card0-DP-1")).unwrap();
        std::fs::write(device.join("gpu_busy_percent"), "37\n").unwrap();
        std::fs::write(device.join("mem_info_vram_used"), "1048576\n").unwrap();
        std::fs::write(device.join("mem_info_vram_total"), "8388608\n").unwrap();
        std::fs::write(device.join("hwmon/hwmon3/temp1_input"), "61500\n").unwrap();

        let cards = drm_cards(&root);
        assert_eq!(
            cards,
            vec![device.clone()],
            "only cards with a busy counter"
        );
        let s = read_drm_card(&cards[0]).unwrap();
        assert_eq!(
            (s.util, s.mem_used, s.mem_total),
            (37.0, 1_048_576, 8_388_608)
        );
        assert_eq!(s.temp, Some(61.5));
        let _ = std::fs::remove_dir_all(&root);
    }
}
