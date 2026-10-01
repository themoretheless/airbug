use crate::{Result, error};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub const SCHEMA: u32 = 1;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Lower,
    Higher,
    Neutral,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Complete,
    Failed,
    Cancelled,
    Incomplete,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "state", content = "reason", rename_all = "snake_case")]
pub enum Availability {
    Available,
    Unsupported(String),
    Invalid(String),
    NotApplicable(String),
    Incomplete(String),
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Metric {
    pub id: String,
    pub unit: String,
    pub scope: String,
    pub phase: String,
    pub statistic: String,
    pub direction: Direction,
}
impl Metric {
    pub fn duration(id: &str, scope: &str, statistic: &str) -> Self {
        Self {
            id: id.into(),
            unit: "ns".into(),
            scope: scope.into(),
            phase: "measurement".into(),
            statistic: statistic.into(),
            direction: Direction::Lower,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Case {
    pub id: String,
    pub contract: BTreeMap<String, String>,
    pub metrics: Vec<Metric>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Observation {
    /// Actual logical work totals for this batch, encoded as decimal integers.
    /// Empty for legacy observations and cases with fixed per-operation counters.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub work_totals: BTreeMap<String, String>,
    /// Actual input work by worker slot. Empty when attribution is unavailable.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub worker_work_totals: BTreeMap<u64, BTreeMap<String, String>>,
    pub case: String,
    pub metric: String,
    pub variant: String,
    pub process: u32,
    pub pair: Option<u32>,
    pub sequence: u64,
    /// Decimal strings preserve integer precision when read by JavaScript.
    pub value: Option<String>,
    pub operations: u64,
    pub availability: Availability,
}
impl Observation {
    pub(crate) fn validate_work_totals(&self, case: &Case) -> Result<()> {
        if !self.worker_work_totals.is_empty() {
            let workers: u64 = case
                .contract
                .get("threads")
                .ok_or_else(|| error("worker work without threads"))?
                .parse()?;
            if self.metric != "wall"
                || self.availability != Availability::Available
                || self.worker_work_totals.len() as u64 != workers
                || self.worker_work_totals.keys().any(|w| *w >= workers)
            {
                return Err(error("invalid worker work identities"));
            }
            let mut sums = BTreeMap::<String, u128>::new();
            for totals in self.worker_work_totals.values() {
                if !totals.keys().eq(self.work_totals.keys()) {
                    return Err(error("incomplete worker work units"));
                }
                for (unit, value) in totals {
                    let sum = sums.entry(unit.clone()).or_default();
                    *sum = sum
                        .checked_add(value.parse::<u128>()?)
                        .ok_or_else(|| error("worker work total overflow"))?;
                }
            }
            for (unit, sum) in sums {
                if sum != self.work_totals[&unit].parse::<u128>()? {
                    return Err(error("worker work disagrees with aggregate"));
                }
            }
        }
        for (unit, value) in &self.work_totals {
            if unit.is_empty()
                || self.metric != "wall"
                || !case.contract.contains_key(&format!("work.input.{unit}"))
            {
                return Err(error("undeclared input work total"));
            }
            value.parse::<u128>()?;
        }
        if self.metric == "wall" && self.availability == Availability::Available {
            for (key, value) in &case.contract {
                if let Some(unit) = key.strip_prefix("work.input.") {
                    if unit.is_empty()
                        || value != "batch_total"
                        || !self.work_totals.contains_key(unit)
                    {
                        return Err(error("missing or invalid input work total"));
                    }
                }
            }
        }
        Ok(())
    }
    pub fn number(&self) -> Result<Option<f64>> {
        match (&self.availability, &self.value) {
            (Availability::Available, Some(v)) => {
                let n: f64 = v.parse()?;
                if !n.is_finite() || n < 0.0 {
                    return Err(error("metric must be finite and nonnegative"));
                }
                Ok(Some(n))
            }
            (Availability::Available, None) => Err(error("available metric has no value")),
            (_, Some(_)) => Err(error("unavailable metric must not have a value")),
            (_, None) => Ok(None),
        }
    }
}
/// One worker's measured wave, linked to its enclosing sample. Worker indices
/// identify slots within the recorded process. Thread reuse depends on the case
/// executor contract; legacy runs may have created fresh threads for each wave.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerAllocation {
    pub case: String,
    pub variant: String,
    pub process: u32,
    pub sequence: u64,
    pub wave: u64,
    pub worker: u64,
    pub operations: u64,
    pub wall_ns: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adjusted_wall_ns: Option<String>,
    pub metrics: BTreeMap<String, String>,
}
/// One worker wave linked to a collected aggregate sample, without allocation data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerTiming {
    pub case: String,
    pub variant: String,
    pub process: u32,
    pub sequence: u64,
    pub wave: u64,
    pub worker: u64,
    pub operations: u64,
    pub wall_ns: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adjusted_wall_ns: Option<String>,
}
/// All waves for one worker slot within a collected sample.
/// A slot can run on different OS threads across waves. These records are not
/// independent process repetitions; timer boundaries are unchanged.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerSlotSample {
    pub case: String,
    pub variant: String,
    pub process: u32,
    pub sequence: u64,
    pub worker: u64,
    pub waves: u64,
    pub operations: u64,
    pub wall_ns: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adjusted_wall_ns: Option<String>,
    /// Exact logical work known for this slot; missing units are unknown, not zero.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub work_totals: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Run {
    pub schema: u32,
    pub id: String,
    pub status: Status,
    pub environment: BTreeMap<String, String>,
    pub provenance: BTreeMap<String, String>,
    pub cases: Vec<Case>,
    pub observations: Vec<Observation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub worker_allocations: Vec<WorkerAllocation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub worker_timings: Vec<WorkerTiming>,
    pub notes: Vec<String>,
}
impl Run {
    /// Combine timing waves by sample and worker slot, summing exact
    /// durations and operations before normalization. Validates the run first.
    /// Ordinary timings take priority over allocation records for each process/case/variant.
    /// Adjusted time is available only when every component has adjusted time.
    pub fn worker_slot_samples(&self) -> crate::Result<Vec<WorkerSlotSample>> {
        self.validate()?;
        crate::worker_timing::slot_samples(self)
    }
    pub fn new() -> Self {
        let mut environment = BTreeMap::new();
        environment.insert("os".into(), std::env::consts::OS.into());
        environment.insert("arch".into(), std::env::consts::ARCH.into());
        Self {
            schema: SCHEMA,
            id: format!(
                "{}-{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos(),
                std::process::id()
            ),
            status: Status::Incomplete,
            environment,
            provenance: BTreeMap::new(),
            cases: vec![],
            observations: vec![],
            worker_allocations: vec![],
            worker_timings: vec![],
            notes: vec![],
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.schema != SCHEMA {
            return Err(error(format!("unsupported schema {}", self.schema)));
        }
        if self.cases.is_empty() && self.status == Status::Complete {
            let disabled = self
                .provenance
                .get("sampling.disabled_cases")
                .map(|value| serde_json::from_str::<Vec<String>>(value))
                .transpose()?;
            if self.observations.is_empty()
                && self.worker_timings.is_empty()
                && self.worker_allocations.is_empty()
                && disabled
                    .is_some_and(|cases| !cases.is_empty() && cases.iter().all(|id| !id.is_empty()))
            {
                return Ok(());
            }
            return Err(error("run has no cases"));
        }
        let mut cases = BTreeMap::new();
        for c in &self.cases {
            if c.id.is_empty() || cases.insert(&c.id, c).is_some() {
                return Err(error("empty/duplicate case ID"));
            }
            let mut ids = BTreeSet::new();
            if c.metrics.is_empty() {
                return Err(error("case has no metrics"));
            }
            for m in &c.metrics {
                if m.id.is_empty()
                    || m.unit.is_empty()
                    || m.scope.is_empty()
                    || m.phase.is_empty()
                    || m.statistic.is_empty()
                    || !ids.insert(&m.id)
                {
                    return Err(error("invalid/duplicate metric descriptor"));
                }
            }
        }
        let mut unique = BTreeSet::new();
        for o in &self.observations {
            let c = cases
                .get(&o.case)
                .ok_or_else(|| error("observation refers to unknown case"))?;
            if !c.metrics.iter().any(|m| m.id == o.metric)
                || o.variant.is_empty()
                || o.operations == 0
            {
                return Err(error("invalid observation identity or operations"));
            }
            if !unique.insert((&o.case, &o.metric, &o.variant, o.process, o.sequence)) {
                return Err(error("duplicate observation"));
            }
            o.number()?;
            o.validate_work_totals(c)?;
        }
        crate::worker_timing::validate(self)?;
        let mut worker_ids = BTreeSet::new();
        for w in &self.worker_allocations {
            let case = cases
                .get(&w.case)
                .ok_or_else(|| error("worker allocation refers to unknown case"))?;
            let workers: u64 = case
                .contract
                .get("threads")
                .ok_or_else(|| error("worker allocation without thread contract"))?
                .parse()?;
            if w.worker >= workers
                || w.operations == 0
                || !worker_ids
                    .insert((&w.case, &w.variant, w.process, w.sequence, w.wave, w.worker))
            {
                return Err(error("invalid or duplicate worker allocation identity"));
            }
            w.wall_ns.parse::<u128>()?;
            if !self.observations.iter().any(|o| {
                o.case == w.case
                    && o.variant == w.variant
                    && o.process == w.process
                    && o.sequence == w.sequence
                    && o.metric == "wall"
                    && o.availability == Availability::Available
            }) {
                return Err(error("worker allocation has no enclosing timing sample"));
            }
            if let Some(adjusted) = &w.adjusted_wall_ns {
                if adjusted.parse::<u128>()? > w.wall_ns.parse::<u128>()? {
                    return Err(error("adjusted worker duration exceeds raw duration"));
                }
            }
            if w.metrics.len() != crate::alloc::ThreadStats::METRICS.len() {
                return Err(error("incomplete worker allocation metrics"));
            }
            for (id, _, _) in crate::alloc::ThreadStats::METRICS {
                w.metrics
                    .get(id)
                    .ok_or_else(|| error("missing worker allocation metric"))?
                    .parse::<u128>()?;
            }
        }
        self.validate_worker_aggregates()?;
        if self.status == Status::Complete {
            let variants: BTreeSet<_> = self.observations.iter().map(|o| &o.variant).collect();
            for c in &self.cases {
                for variant in &variants {
                    let processes: BTreeSet<_> = self
                        .observations
                        .iter()
                        .filter(|o| o.case == c.id && &o.variant == *variant)
                        .map(|o| o.process)
                        .collect();
                    if processes.is_empty() {
                        return Err(error("complete run missing case variant"));
                    }
                    for process in processes {
                        for m in &c.metrics {
                            if !self.observations.iter().any(|o| {
                                o.case == c.id
                                    && o.metric == m.id
                                    && &o.variant == *variant
                                    && o.process == process
                            }) {
                                return Err(error("complete run missing process metric"));
                            }
                        }
                    }
                }
                if variants.is_empty() {
                    return Err(error("complete run has no observations"));
                }
            }
        }
        Ok(())
    }
    fn validate_worker_aggregates(&self) -> Result<()> {
        fn add(a: u128, b: u128) -> Result<u128> {
            a.checked_add(b)
                .ok_or_else(|| error("worker aggregate overflow"))
        }
        for wall in self.observations.iter().filter(|o| o.metric == "wall") {
            let case = self.cases.iter().find(|c| c.id == wall.case).unwrap();
            let mut waves: BTreeMap<u64, Vec<&WorkerAllocation>> = BTreeMap::new();
            for worker in self.worker_allocations.iter().filter(|w| {
                w.case == wall.case
                    && w.variant == wall.variant
                    && w.process == wall.process
                    && w.sequence == wall.sequence
            }) {
                waves.entry(worker.wave).or_default().push(worker);
            }
            let required = case
                .contract
                .get("alloc.worker_records")
                .is_some_and(|v| v == "wave-v1");
            if waves.is_empty() && !required {
                continue;
            }
            if waves.is_empty() {
                return Err(error("missing worker allocation records"));
            }
            let worker_count: u64 = case
                .contract
                .get("threads")
                .ok_or_else(|| error("missing worker count"))?
                .parse()?;
            let mut operations = 0u128;
            let adjusted_observation = self.observations.iter().find(|o| {
                o.metric == "wall.adjusted"
                    && o.case == wall.case
                    && o.variant == wall.variant
                    && o.process == wall.process
                    && o.sequence == wall.sequence
            });
            let mut adjusted_elapsed = 0u128;
            let mut elapsed = 0u128;
            let mut aggregate: BTreeMap<&str, u128> = BTreeMap::new();
            for (expected, (&index, workers)) in waves.iter().enumerate() {
                if index != expected as u64
                    || workers.len() as u64 != worker_count
                    || workers
                        .iter()
                        .any(|w| w.operations != workers[0].operations)
                {
                    return Err(error("incomplete or inconsistent worker wave"));
                }
                let mut wave_elapsed = 0;
                let mut wave_adjusted = 0;
                for worker in workers {
                    operations = add(operations, worker.operations as u128)?;
                    wave_elapsed = wave_elapsed.max(worker.wall_ns.parse::<u128>()?);
                    if adjusted_observation.is_some() {
                        let adjusted = worker
                            .adjusted_wall_ns
                            .as_deref()
                            .ok_or_else(|| error("missing adjusted worker interval"))?
                            .parse::<u128>()?;
                        wave_adjusted = wave_adjusted.max(adjusted);
                    }
                }
                elapsed = add(elapsed, wave_elapsed)?;
                adjusted_elapsed = add(adjusted_elapsed, wave_adjusted)?;
                for (id, _, statistic) in crate::alloc::ThreadStats::METRICS {
                    let mut value = 0;
                    for worker in workers {
                        value = add(value, worker.metrics[id].parse()?)?;
                    }
                    let total = aggregate.entry(id).or_default();
                    *total = if statistic == "sample_peak" {
                        (*total).max(value)
                    } else {
                        add(*total, value)?
                    };
                }
            }
            if let Some(adjusted) = adjusted_observation {
                if adjusted.operations != wall.operations
                    || adjusted
                        .value
                        .as_deref()
                        .ok_or_else(|| error("missing adjusted aggregate"))?
                        .parse::<u128>()?
                        != adjusted_elapsed
                {
                    return Err(error("adjusted worker durations disagree with aggregate"));
                }
            }
            // Net growth/release are the positive/negative halves of one signed
            // balance; opposing worker balances cancel before comparison.
            let growth = aggregate["alloc.net_growth_bytes"];
            let release = aggregate["alloc.net_release_bytes"];
            aggregate.insert("alloc.net_growth_bytes", growth.saturating_sub(release));
            aggregate.insert("alloc.net_release_bytes", release.saturating_sub(growth));
            if operations != wall.operations as u128
                || wall
                    .value
                    .as_deref()
                    .ok_or_else(|| error("missing worker aggregate time"))?
                    .parse::<u128>()?
                    != elapsed
            {
                return Err(error(
                    "worker operations or duration disagree with aggregate",
                ));
            }
            for observation in self.observations.iter().filter(|o| {
                o.case == wall.case
                    && o.variant == wall.variant
                    && o.process == wall.process
                    && o.sequence == wall.sequence
                    && o.metric.starts_with("alloc.")
            }) {
                let expected = aggregate
                    .get(observation.metric.as_str())
                    .ok_or_else(|| error("unknown worker aggregate metric"))?;
                let actual = observation
                    .value
                    .as_deref()
                    .ok_or_else(|| error("missing worker aggregate metric value"))?
                    .parse::<u128>()?;
                if actual != *expected {
                    return Err(error("worker allocation metrics disagree with aggregate"));
                }
            }
        }
        Ok(())
    }
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let p = path.as_ref();
        let p = if p.is_dir() {
            p.join("run.json")
        } else {
            p.to_owned()
        };
        let r: Self = serde_json::from_reader(std::fs::File::open(p)?)?;
        r.validate()?;
        Ok(r)
    }
    /// A run directory is never overwritten; incomplete writes remain visibly incomplete.
    pub fn save_new(&self, path: impl AsRef<Path>) -> Result<()> {
        self.validate()?;
        fs::create_dir(path.as_ref())?;
        write_new(&path.as_ref().join("run.json"), self)
    }
}
impl Default for Run {
    fn default() -> Self {
        Self::new()
    }
}
pub fn write_new(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut f = OpenOptions::new().write(true).create_new(true).open(path)?;
    serde_json::to_writer_pretty(&mut f, value)?;
    f.write_all(b"\n")?;
    f.sync_all()?;
    Ok(())
}
/// Lowercase hex encoding of a byte slice.
///
/// sha2 0.11 returns a `hybrid_array::Array` digest that no longer implements
/// `LowerHex`, so hashing sites format the raw bytes through this helper.
pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}
pub fn hash_file(path: &Path) -> Result<String> {
    let mut f = fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buf = [0; 65536];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    Ok(hex(&hash.finalize()))
}
