#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct GpuSample {
    pub util: f32,
    pub mem_used: u64,
    pub mem_total: u64,
    pub temp: Option<f32>,
}

pub trait GpuProvider {
    fn sample(&mut self) -> Option<GpuSample>;
}

#[allow(dead_code)]
pub struct NullProvider;

impl GpuProvider for NullProvider {
    fn sample(&mut self) -> Option<GpuSample> {
        None
    }
}

#[cfg(target_os = "macos")]
pub struct PowermetricsProvider {
    available: bool,
}

#[cfg(target_os = "macos")]
impl PowermetricsProvider {
    pub fn new() -> Self {
        let available = std::process::Command::new("powermetrics")
            .arg("--help")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        Self { available }
    }
}

#[cfg(target_os = "macos")]
impl GpuProvider for PowermetricsProvider {
    fn sample(&mut self) -> Option<GpuSample> {
        if !self.available {
            return None;
        }
        let output = std::process::Command::new("powermetrics")
            .args(["--samplers", "gpu_power", "-i", "1000", "-n", "1"])
            .output()
            .ok()?;
        if !output.status.success() {
            self.available = false;
            return None;
        }
        parse_powermetrics(&String::from_utf8_lossy(&output.stdout))
    }
}

#[cfg(target_os = "macos")]
fn parse_powermetrics(text: &str) -> Option<GpuSample> {
    let mut util = None;
    let mem_used = 0u64;
    let mut mem_total = 0u64;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("GPU active residency:") {
            let num: String = rest
                .trim_start()
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            util = num.parse::<f32>().ok();
        } else if let Some(rest) = line.strip_prefix("GPU HW allocated memory:") {
            let num: String = rest
                .trim_start()
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if let Ok(mb) = num.parse::<f64>() {
                mem_total = (mb * 1_048_576.0) as u64;
            }
        }
    }
    util.map(|u| GpuSample {
        util: u,
        mem_used,
        mem_total,
        temp: None,
    })
}

pub fn default_provider() -> Box<dyn GpuProvider + Send> {
    #[cfg(target_os = "macos")]
    {
        Box::new(PowermetricsProvider::new())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Box::new(NullProvider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn parses_powermetrics_output() {
        let text =
            "**** GPU usage ****\nGPU active residency: 12.34 %\nGPU HW allocated memory: 512 MB\n";
        let s = parse_powermetrics(text).unwrap();
        assert!((s.util - 12.34).abs() < 0.01);
        assert_eq!(s.mem_total, 512 * 1_048_576);
    }
}
