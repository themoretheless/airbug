//! Recorder for application-owned event loops. It never waits, renders or
//! creates a window; completion boundaries are provided by the application.
use crate::*;
use std::collections::BTreeMap;
pub struct Recorder {
    run: Run,
    sequences: BTreeMap<(String, String), u64>,
}
impl Recorder {
    pub fn new() -> Self {
        Self {
            run: Run::new(),
            sequences: BTreeMap::new(),
        }
    }
    pub fn case(&mut self, case: Case) -> Result<()> {
        if self.run.cases.iter().any(|c| c.id == case.id) {
            return Err(error("duplicate scenario case"));
        }
        self.run.cases.push(case);
        Ok(())
    }
    /// Register a named synchronous phase; repeated registration must match its descriptor.
    pub fn phase(&mut self, case: &str, name: &str, scope: &str) -> Result<()> {
        let c = self
            .run
            .cases
            .iter_mut()
            .find(|c| c.id == case)
            .ok_or_else(|| error("unknown case"))?;
        let mut m = Metric::duration(name, scope, "individual phase");
        m.phase = name.into();
        if let Some(existing) = c.metrics.iter().find(|m| m.id == name) {
            if existing != &m {
                return Err(error("phase descriptor mismatch"));
            }
        } else {
            c.metrics.push(m);
        }
        Ok(())
    }
    /// Times only the closure; observation insertion and output Drop happen afterwards.
    pub fn measure<T>(&mut self, case: &str, phase: &str, f: impl FnOnce() -> T) -> Result<T> {
        if !self
            .run
            .cases
            .iter()
            .any(|c| c.id == case && c.metrics.iter().any(|m| m.id == phase))
        {
            return Err(error("register phase before measuring"));
        }
        let start = std::time::Instant::now();
        let output = f();
        let ns = start.elapsed().as_nanos();
        self.observe(case, phase, ns)?;
        Ok(output)
    }
    pub fn observe(&mut self, case: &str, metric: &str, value: u128) -> Result<()> {
        self.push(
            case,
            metric,
            Some(value.to_string()),
            Availability::Available,
        )
    }
    pub fn unavailable(
        &mut self,
        case: &str,
        metric: &str,
        availability: Availability,
    ) -> Result<()> {
        if availability == Availability::Available {
            return Err(error("use observe for available metrics"));
        }
        self.push(case, metric, None, availability)
    }
    fn push(
        &mut self,
        case: &str,
        metric: &str,
        value: Option<String>,
        availability: Availability,
    ) -> Result<()> {
        if !self
            .run
            .cases
            .iter()
            .any(|c| c.id == case && c.metrics.iter().any(|m| m.id == metric))
        {
            return Err(error("unknown scenario metric"));
        }
        let sequence = self
            .sequences
            .entry((case.into(), metric.into()))
            .or_default();
        self.run.observations.push(Observation {
            case: case.into(),
            metric: metric.into(),
            variant: "candidate".into(),
            process: 0,
            pair: None,
            sequence: *sequence,
            value,
            operations: 1,
            availability,
        });
        *sequence += 1;
        Ok(())
    }
    pub fn note(&mut self, note: impl Into<String>) {
        self.run.notes.push(note.into());
    }
    pub fn finish(mut self) -> Result<Run> {
        self.run.status = Status::Complete;
        self.run.validate()?;
        Ok(self.run)
    }
    pub fn fail(mut self, reason: impl Into<String>) -> Run {
        self.run.status = Status::Failed;
        self.run.notes.push(reason.into());
        self.run
    }
}
impl Default for Recorder {
    fn default() -> Self {
        Self::new()
    }
}
