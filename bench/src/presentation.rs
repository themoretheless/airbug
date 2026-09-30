//! Persisted report preferences, separate from measurement compatibility contracts.
use crate::{Result, Run, error, viz::charts::AxisScale};
use std::collections::BTreeMap;
const KEY: &str = "airbug.presentation.summary_scales.v1";

/// Preserve preferences only for cases present in this run. Does not change
/// workload contracts, observations, environment, or comparison decisions.
pub fn save_scales(run: &mut Run, scales: &BTreeMap<String, AxisScale>) -> Result<()> {
    let names: BTreeMap<_, _> = run
        .cases
        .iter()
        .filter_map(|case| {
            scales.get(&case.id).map(|scale| {
                (
                    &case.id,
                    match scale {
                        AxisScale::Linear => "linear",
                        AxisScale::Logarithmic => "logarithmic",
                    },
                )
            })
        })
        .collect();
    run.provenance
        .insert(KEY.into(), serde_json::to_string(&names)?);
    Ok(())
}
/// Legacy runs have no preferences. Invalid stored settings are explicit errors;
/// preferences for cases removed by filtering are ignored.
pub fn load_scales(run: &Run) -> Result<BTreeMap<String, AxisScale>> {
    let Some(value) = run.provenance.get(KEY) else {
        return Ok(BTreeMap::new());
    };
    let names: BTreeMap<String, String> = serde_json::from_str(value)?;
    let mut result = BTreeMap::new();
    for (case, value) in names {
        let scale = match value.as_str() {
            "linear" => AxisScale::Linear,
            "logarithmic" => AxisScale::Logarithmic,
            _ => {
                return Err(error(format!(
                    "invalid saved summary scale for {case:?}: {value:?}"
                )));
            }
        };
        if run.cases.iter().any(|c| c.id == case) {
            result.insert(case, scale);
        }
    }
    Ok(result)
}

const FAMILIES: &str = "airbug.presentation.summary_families.v1";
/// Assign explicit family identities without changing workload compatibility.
pub fn save_families(run: &mut Run, families: &BTreeMap<String, String>) -> Result<()> {
    let present: BTreeMap<_, _> = families
        .iter()
        .filter(|(id, _)| run.cases.iter().any(|c| c.id == **id))
        .collect();
    run.provenance
        .insert(FAMILIES.into(), serde_json::to_string(&present)?);
    Ok(())
}
pub fn load_families(run: &Run) -> Result<BTreeMap<String, String>> {
    let Some(value) = run.provenance.get(FAMILIES) else {
        return Ok(BTreeMap::new());
    };
    let mut families: BTreeMap<String, String> = serde_json::from_str(value)?;
    families.retain(|id, _| run.cases.iter().any(|c| &c.id == id));
    Ok(families)
}

/// Consensus across process results. A disagreement keeps measurements usable
/// while removing that case's automatic preference; explicit CLI settings win.
#[derive(Default)]
pub struct Consensus {
    scales: BTreeMap<String, Option<AxisScale>>,
    families: BTreeMap<String, Option<String>>,
    family_conflicts: std::collections::BTreeSet<String>,
}
impl Consensus {
    pub fn observe(&mut self, run: &Run) -> Result<()> {
        let scales = load_scales(run)?;
        let families = load_families(run)?;
        for case in &run.cases {
            let family = families.get(&case.id).cloned();
            if let Some(previous) = self.families.get(&case.id) {
                if previous != &family {
                    self.family_conflicts.insert(case.id.clone());
                }
            } else {
                self.families.insert(case.id.clone(), family);
            }
            let value = scales.get(&case.id).copied().unwrap_or_default();
            self.scales
                .entry(case.id.clone())
                .and_modify(|old| {
                    if *old != Some(value) {
                        *old = None;
                    }
                })
                .or_insert(Some(value));
        }
        Ok(())
    }
    pub fn save(&self, run: &mut Run) -> Result<()> {
        let scales = self
            .scales
            .iter()
            .filter_map(|(case, scale)| scale.map(|s| (case.clone(), s)))
            .collect();
        save_scales(run, &scales)?;
        let families = self
            .families
            .iter()
            .filter(|(id, _)| !self.family_conflicts.contains(*id))
            .filter_map(|(id, family)| family.as_ref().map(|name| (id.clone(), name.clone())))
            .collect();
        save_families(run, &families)?;
        for case in &self.family_conflicts {
            let note = format!(
                "Summary family differs across workers for {case:?}; keeping this case as separate points."
            );
            if !run.notes.contains(&note) {
                run.notes.push(note);
            }
        }

        for (case, scale) in &self.scales {
            if scale.is_none() {
                let note = format!(
                    "Summary scale differs across workers for {case:?}; using default scale unless explicitly overridden."
                );
                if !run.notes.contains(&note) {
                    run.notes.push(note);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn family_consensus_preserves_agreement_and_refuses_conflicting_lines() {
        let mut run = Run::new();
        run.cases.push(crate::Case {
            id: "case".into(),
            contract: Default::default(),
            metrics: vec![],
        });
        save_families(&mut run, &BTreeMap::from([("case".into(), "sort".into())])).unwrap();
        for alternative in [Some("other"), None] {
            let mut consensus = Consensus::default();
            consensus.observe(&run).unwrap();
            consensus.observe(&run).unwrap();
            let mut output = run.clone();
            consensus.save(&mut output).unwrap();
            assert_eq!(load_families(&output).unwrap()["case"], "sort");
            let mut conflicting = run.clone();
            conflicting.provenance.clear();
            if let Some(name) = alternative {
                save_families(
                    &mut conflicting,
                    &BTreeMap::from([("case".into(), name.into())]),
                )
                .unwrap();
            }
            consensus.observe(&conflicting).unwrap();
            consensus.observe(&run).unwrap();
            consensus.save(&mut output).unwrap();
            consensus.save(&mut output).unwrap();
            assert!(load_families(&output).unwrap().is_empty());
            assert_eq!(
                output
                    .notes
                    .iter()
                    .filter(|n| n.contains("Summary family differs"))
                    .count(),
                1
            );
        }
    }
    #[test]
    fn consensus_preserves_agreement_and_keeps_conflicts_sticky() {
        let mut run = Run::new();
        run.cases.push(crate::Case {
            id: "case".into(),
            contract: Default::default(),
            metrics: vec![],
        });
        save_scales(
            &mut run,
            &BTreeMap::from([("case".into(), AxisScale::Logarithmic)]),
        )
        .unwrap();
        let mut consensus = Consensus::default();
        consensus.observe(&run).unwrap();
        consensus.observe(&run).unwrap();
        let mut output = run.clone();
        consensus.save(&mut output).unwrap();
        assert_eq!(
            load_scales(&output).unwrap()["case"],
            AxisScale::Logarithmic
        );
        let mut legacy = run.clone();
        legacy.provenance.clear();
        consensus.observe(&legacy).unwrap();
        consensus.observe(&run).unwrap();
        consensus.save(&mut output).unwrap();
        consensus.save(&mut output).unwrap();
        assert!(load_scales(&output).unwrap().is_empty());
        assert_eq!(
            output
                .notes
                .iter()
                .filter(|s| s.contains("differs across workers"))
                .count(),
            1
        );
    }
    #[test]
    fn scales_roundtrip_without_changing_measurement_compatibility() {
        let mut recorder = crate::Recorder::new();
        recorder
            .case(crate::Case {
                id: "case".into(),
                contract: Default::default(),
                metrics: vec![crate::Metric::duration("wall", "operation", "batch_total")],
            })
            .unwrap();
        recorder.observe("case", "wall", 10).unwrap();
        let original = recorder.finish().unwrap();
        assert!(load_scales(&original).unwrap().is_empty());
        let mut changed = original.clone();
        save_scales(
            &mut changed,
            &BTreeMap::from([("case".into(), AxisScale::Logarithmic)]),
        )
        .unwrap();
        let restored: Run =
            serde_json::from_str(&serde_json::to_string(&changed).unwrap()).unwrap();
        assert_eq!(
            load_scales(&restored).unwrap()["case"],
            AxisScale::Logarithmic
        );
        assert_eq!(
            serde_json::to_value(
                crate::analysis::compare(&original, Some(&original), 5., 0.05).unwrap()
            )
            .unwrap(),
            serde_json::to_value(
                crate::analysis::compare(&original, Some(&restored), 5., 0.05).unwrap()
            )
            .unwrap()
        );
        changed
            .provenance
            .insert(KEY.into(), "{\"case\":\"invalid\"}".into());
        assert!(load_scales(&changed).is_err());
    }
}
