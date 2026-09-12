use crate::{error, Result};
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Run {
    pub schema: u32,
    pub id: String,
    pub status: Status,
    pub environment: BTreeMap<String, String>,
    pub provenance: BTreeMap<String, String>,
    pub cases: Vec<Case>,
    pub observations: Vec<Observation>,
    pub notes: Vec<String>,
}
impl Run {
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
            notes: vec![],
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.schema != SCHEMA {
            return Err(error(format!("unsupported schema {}", self.schema)));
        }
        if self.cases.is_empty() && self.status == Status::Complete {
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
        }
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
