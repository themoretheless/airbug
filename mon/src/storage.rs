use std::path::{Path, PathBuf};

use rusqlite::{Connection, params};

use crate::sampler::Sample;

const RAW_RETENTION_SECS: i64 = 24 * 3600;
const AGG_RETENTION_SECS: i64 = 30 * 24 * 3600;

pub fn default_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    Path::new(&home).join(".local/share/airbug-mon/airbug-mon.db")
}

pub struct Storage {
    conn: Connection,
}

#[derive(Clone, Debug)]
pub struct HistoryPoint {
    pub ts: i64,
    pub cpu_total: f32,
    pub mem_used: u64,
    pub mem_total: u64,
    pub swap_used: u64,
    pub gpu_util: Option<f32>,
    pub gpu_mem: Option<u64>,
    pub net_rx: f64,
    pub net_tx: f64,
    pub disk_r: f64,
    pub disk_w: f64,
}

impl Storage {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> rusqlite::Result<Self> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             CREATE TABLE IF NOT EXISTS samples(
                 ts INTEGER PRIMARY KEY,
                 cpu_total REAL,
                 cpu_per_core_json TEXT,
                 mem_used INTEGER, mem_total INTEGER,
                 swap_used INTEGER,
                 gpu_util REAL, gpu_mem INTEGER,
                 net_rx REAL, net_tx REAL,
                 disk_r REAL, disk_w REAL
             );
             CREATE TABLE IF NOT EXISTS proc_samples(
                 ts INTEGER, pid INTEGER, name TEXT,
                 cpu REAL, mem INTEGER, read_b REAL, write_b REAL
             );
             CREATE INDEX IF NOT EXISTS idx_proc_ts ON proc_samples(ts);
             CREATE TABLE IF NOT EXISTS samples_min(
                 ts INTEGER PRIMARY KEY,
                 cpu_total REAL,
                 mem_used INTEGER, mem_total INTEGER,
                 swap_used INTEGER,
                 gpu_util REAL, gpu_mem INTEGER,
                 net_rx REAL, net_tx REAL,
                 disk_r REAL, disk_w REAL
             );",
        )?;
        Ok(Self { conn })
    }

    pub fn insert_batch(&mut self, samples: &[Sample]) -> rusqlite::Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut s_stmt = tx.prepare(
                "INSERT OR REPLACE INTO samples
                 (ts, cpu_total, cpu_per_core_json, mem_used, mem_total, swap_used,
                  gpu_util, gpu_mem, net_rx, net_tx, disk_r, disk_w)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            )?;
            let mut p_stmt = tx.prepare(
                "INSERT INTO proc_samples (ts, pid, name, cpu, mem, read_b, write_b)
                 VALUES (?1,?2,?3,?4,?5,?6,?7)",
            )?;
            for s in samples {
                let cores = serde_json::to_string(&s.cpu_per_core).unwrap_or_default();
                let (gpu_util, gpu_mem) = match &s.gpu {
                    Some(g) => (Some(g.util), Some(g.mem_used as i64)),
                    None => (None, None),
                };
                s_stmt.execute(params![
                    s.ts,
                    s.cpu_total,
                    cores,
                    s.mem_used as i64,
                    s.mem_total as i64,
                    s.swap_used as i64,
                    gpu_util,
                    gpu_mem,
                    s.net_rx_rate,
                    s.net_tx_rate,
                    s.disk_read_rate,
                    s.disk_write_rate,
                ])?;
                for p in &s.top_procs {
                    p_stmt.execute(params![
                        s.ts,
                        p.pid,
                        p.name,
                        p.cpu,
                        p.mem as i64,
                        p.read_rate,
                        p.write_rate,
                    ])?;
                }
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn aggregate_minutes(&self) -> rusqlite::Result<usize> {
        let n = self.conn.execute(
            "INSERT OR REPLACE INTO samples_min
             SELECT ts - (ts % 60),
                    AVG(cpu_total),
                    CAST(AVG(mem_used) AS INTEGER), MAX(mem_total),
                    CAST(AVG(swap_used) AS INTEGER),
                    AVG(gpu_util), CAST(AVG(gpu_mem) AS INTEGER),
                    AVG(net_rx), AVG(net_tx), AVG(disk_r), AVG(disk_w)
             FROM samples
             GROUP BY ts - (ts % 60)",
            [],
        )?;
        Ok(n)
    }

    pub fn enforce_retention(&self, now: i64) -> rusqlite::Result<()> {
        self.conn.execute(
            "DELETE FROM samples WHERE ts < ?1",
            params![now - RAW_RETENTION_SECS],
        )?;
        self.conn.execute(
            "DELETE FROM proc_samples WHERE ts < ?1",
            params![now - RAW_RETENTION_SECS],
        )?;
        self.conn.execute(
            "DELETE FROM samples_min WHERE ts < ?1",
            params![now - AGG_RETENTION_SECS],
        )?;
        Ok(())
    }

    pub fn history(&self, from_ts: i64, aggregated: bool) -> rusqlite::Result<Vec<HistoryPoint>> {
        let table = if aggregated { "samples_min" } else { "samples" };
        let sql = format!(
            "SELECT ts, cpu_total, mem_used, mem_total, swap_used,
                    gpu_util, gpu_mem, net_rx, net_tx, disk_r, disk_w
             FROM {table} WHERE ts >= ?1 ORDER BY ts"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![from_ts], |r| {
            Ok(HistoryPoint {
                ts: r.get(0)?,
                cpu_total: r.get(1)?,
                mem_used: r.get::<_, i64>(2)? as u64,
                mem_total: r.get::<_, i64>(3)? as u64,
                swap_used: r.get::<_, i64>(4)? as u64,
                gpu_util: r.get(5)?,
                gpu_mem: r.get::<_, Option<i64>>(6)?.map(|v| v as u64),
                net_rx: r.get(7)?,
                net_tx: r.get(8)?,
                disk_r: r.get(9)?,
                disk_w: r.get(10)?,
            })
        })?;
        rows.collect()
    }

    pub fn count_samples(&self) -> rusqlite::Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM samples", [], |r| r.get(0))
    }

    pub fn export_csv(&self, from_ts: i64, aggregated: bool, path: &Path) -> std::io::Result<()> {
        let points = self
            .history(from_ts, aggregated)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let mut out = String::from(
            "ts,datetime,cpu_total,mem_used,mem_total,swap_used,gpu_util,gpu_mem,net_rx,net_tx,disk_r,disk_w\n",
        );
        for p in points {
            let dt = chrono::DateTime::from_timestamp(p.ts, 0)
                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default();
            out.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{}\n",
                p.ts,
                dt,
                p.cpu_total,
                p.mem_used,
                p.mem_total,
                p.swap_used,
                p.gpu_util.map(|v| v.to_string()).unwrap_or_default(),
                p.gpu_mem.map(|v| v.to_string()).unwrap_or_default(),
                p.net_rx,
                p.net_tx,
                p.disk_r,
                p.disk_w,
            ));
        }
        std::fs::write(path, out)
    }
}

pub fn run_db_writer(rx: crossbeam_channel::Receiver<Vec<Sample>>, mut storage: Storage) {
    let mut last_agg = std::time::Instant::now();
    while let Ok(batch) = rx.recv() {
        let _ = storage.insert_batch(&batch);
        if last_agg.elapsed() >= std::time::Duration::from_secs(60) {
            let _ = storage.aggregate_minutes();
            let _ = storage.enforce_retention(chrono::Utc::now().timestamp());
            last_agg = std::time::Instant::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampler::Sample;

    fn make_sample(ts: i64, cpu: f32, mem: u64) -> Sample {
        Sample {
            ts,
            cpu_total: cpu,
            cpu_per_core: vec![cpu; 4],
            mem_used: mem,
            mem_total: 1024,
            swap_used: 0,
            swap_total: 0,
            net_rx_rate: 10.0,
            net_tx_rate: 20.0,
            disk_read_rate: 30.0,
            disk_write_rate: 40.0,
            gpu: None,
            top_procs: vec![],
        }
    }

    #[test]
    fn aggregates_to_minute_points() {
        let mut st = Storage::open_in_memory().unwrap();
        let base = 1_700_000_000 - (1_700_000_000 % 60);
        let samples: Vec<Sample> = (0..120)
            .map(|i| make_sample(base + i, 10.0 + i as f32, 100 + i as u64))
            .collect();
        st.insert_batch(&samples).unwrap();
        let n = st.aggregate_minutes().unwrap();
        assert_eq!(n, 2);
        let agg = st.history(base, true).unwrap();
        assert_eq!(agg.len(), 2);
        let expected_cpu: f32 = (0..60).map(|i| 10.0 + i as f32).sum::<f32>() / 60.0;
        assert!((agg[0].cpu_total - expected_cpu).abs() < 0.01);
        assert_eq!(agg[0].mem_total, 1024);
        assert!((agg[0].net_rx - 10.0).abs() < 1e-6);
        assert_eq!(agg[0].ts, base - (base % 60));
    }

    #[test]
    fn retention_prunes_old_rows() {
        let mut st = Storage::open_in_memory().unwrap();
        let now = chrono::Utc::now().timestamp();
        let samples = vec![
            make_sample(now - 25 * 3600, 50.0, 500),
            make_sample(now - 60, 60.0, 600),
        ];
        st.insert_batch(&samples).unwrap();
        st.aggregate_minutes().unwrap();
        st.conn
            .execute(
                "INSERT OR REPLACE INTO samples_min (ts, cpu_total, mem_used, mem_total, swap_used, gpu_util, gpu_mem, net_rx, net_tx, disk_r, disk_w)
                 VALUES (?1, 1.0, 1, 1, 0, NULL, NULL, 0, 0, 0, 0)",
                params![now - 31 * 24 * 3600],
            )
            .unwrap();
        st.enforce_retention(now).unwrap();
        assert_eq!(st.count_samples().unwrap(), 1);
        let agg_count: i64 = st
            .conn
            .query_row("SELECT COUNT(*) FROM samples_min", [], |r| r.get(0))
            .unwrap();
        assert_eq!(agg_count, 2);
    }

    #[test]
    fn history_reads_raw_and_aggregated() {
        let mut st = Storage::open_in_memory().unwrap();
        let base = 1_700_000_000 - (1_700_000_000 % 60);
        st.insert_batch(&[make_sample(base, 42.0, 777)]).unwrap();
        st.aggregate_minutes().unwrap();
        let raw = st.history(base - 1, false).unwrap();
        assert_eq!(raw.len(), 1);
        assert_eq!(raw[0].mem_used, 777);
        let agg = st.history(base - 1, true).unwrap();
        assert_eq!(agg.len(), 1);
    }
}
