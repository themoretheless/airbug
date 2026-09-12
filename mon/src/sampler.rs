use std::collections::HashMap;

use crossbeam_channel::Sender;
use sysinfo::{Disks, Networks, ProcessesToUpdate, System};

use crate::gpu::{GpuProvider, GpuSample};

#[derive(Clone, Debug)]
pub struct ProcSample {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub mem: u64,
    pub read_rate: f64,
    pub write_rate: f64,
}

#[derive(Clone, Debug)]
pub struct Sample {
    pub ts: i64,
    pub cpu_total: f32,
    pub cpu_per_core: Vec<f32>,
    pub mem_used: u64,
    pub mem_total: u64,
    pub swap_used: u64,
    pub swap_total: u64,
    pub net_rx_rate: f64,
    pub net_tx_rate: f64,
    pub disk_read_rate: f64,
    pub disk_write_rate: f64,
    pub gpu: Option<GpuSample>,
    pub top_procs: Vec<ProcSample>,
}

pub struct Sampler {
    system: System,
    networks: Networks,
    disks: Disks,
    gpu: Box<dyn GpuProvider + Send>,
    prev_net: Option<(u64, u64)>,
    prev_disk: Option<(u64, u64)>,
    prev_proc_disk: HashMap<u32, (u64, u64)>,
}

impl Sampler {
    pub fn new(gpu: Box<dyn GpuProvider + Send>) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        Self {
            system,
            networks,
            disks,
            gpu,
            prev_net: None,
            prev_disk: None,
            prev_proc_disk: HashMap::new(),
        }
    }

    pub fn sample(&mut self) -> Sample {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.system
            .refresh_processes(ProcessesToUpdate::All, true);
        self.networks.refresh(true);
        self.disks.refresh(true);

        let cpus = self.system.cpus();
        let cpu_total = self.system.global_cpu_usage();
        let cpu_per_core: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();

        let mem_used = self.system.used_memory();
        let mem_total = self.system.total_memory();
        let swap_used = self.system.used_swap();
        let swap_total = self.system.total_swap();

        let rx: u64 = self.networks.iter().map(|(_, d)| d.total_received()).sum();
        let tx: u64 = self
            .networks
            .iter()
            .map(|(_, d)| d.total_transmitted())
            .sum();
        let (net_rx_rate, net_tx_rate) = match self.prev_net.replace((rx, tx)) {
            Some((prx, ptx)) => ((rx.saturating_sub(prx)) as f64, (tx.saturating_sub(ptx)) as f64),
            None => (0.0, 0.0),
        };

        let dr: u64 = self
            .disks
            .iter()
            .map(|d| d.usage().total_read_bytes)
            .sum();
        let dw: u64 = self
            .disks
            .iter()
            .map(|d| d.usage().total_written_bytes)
            .sum();
        let (disk_read_rate, disk_write_rate) = match self.prev_disk.replace((dr, dw)) {
            Some((pr, pw)) => ((dr.saturating_sub(pr)) as f64, (dw.saturating_sub(pw)) as f64),
            None => (0.0, 0.0),
        };

        let mut procs: Vec<ProcSample> = self
            .system
            .processes()
            .iter()
            .map(|(pid, p)| {
                let pid_u32 = pid.as_u32();
                let usage = p.disk_usage();
                let (read_rate, write_rate) = match self.prev_proc_disk.get(&pid_u32) {
                    Some(&(pr, pw)) => (
                        usage.total_read_bytes.saturating_sub(pr) as f64,
                        usage.total_written_bytes.saturating_sub(pw) as f64,
                    ),
                    None => (0.0, 0.0),
                };
                self.prev_proc_disk.insert(
                    pid_u32,
                    (usage.total_read_bytes, usage.total_written_bytes),
                );
                ProcSample {
                    pid: pid_u32,
                    name: p.name().to_string_lossy().into_owned(),
                    cpu: p.cpu_usage(),
                    mem: p.memory(),
                    read_rate,
                    write_rate,
                }
            })
            .collect();
        self.prev_proc_disk
            .retain(|pid, _| self.system.process(sysinfo::Pid::from_u32(*pid)).is_some());
        procs.sort_by(|a, b| {
            let score = |p: &ProcSample| p.cpu as f64 + (p.mem as f64 / 1_048_576.0) * 0.001;
            score(b)
                .partial_cmp(&score(a))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let top_procs: Vec<ProcSample> = procs.into_iter().take(10).collect();

        let gpu = self.gpu.sample();

        Sample {
            ts: chrono::Utc::now().timestamp(),
            cpu_total,
            cpu_per_core,
            mem_used,
            mem_total,
            swap_used,
            swap_total,
            net_rx_rate,
            net_tx_rate,
            disk_read_rate,
            disk_write_rate,
            gpu,
            top_procs,
        }
    }
}

pub fn run_sampler(
    tx: Sender<Sample>,
    db_tx: Sender<Vec<Sample>>,
    gpu: Box<dyn GpuProvider + Send>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    let mut sampler = Sampler::new(gpu);
    let mut batch: Vec<Sample> = Vec::new();
    loop {
        if stop.load(std::sync::atomic::Ordering::Relaxed) && tx.is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
        if stop.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
        let sample = sampler.sample();
        batch.push(sample.clone());
        if batch.len() >= 5 {
            let _ = db_tx.send(std::mem::take(&mut batch));
        }
        let _ = tx.send(sample);
    }
    if !batch.is_empty() {
        let _ = db_tx.send(batch);
    }
}
