use crate::runner;
use rbench::{model::write_new, *};
use std::{collections::BTreeMap, path::Path};
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Matrix {
    pub plan: runner::Plan,
    pub axes: BTreeMap<String, Vec<String>>,
}
pub fn run(m: Matrix, out: &Path) -> Result<()> {
    if m.axes.is_empty() || m.axes.len() > 8 {
        return Err(error("matrix needs 1..8 CLI argument axes"));
    }
    let mut combinations = vec![BTreeMap::new()];
    for (key, values) in m.axes {
        if !key.starts_with("--")
            || key.len() < 3
            || !key[2..]
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || values.is_empty()
            || values.iter().any(|v| v.is_empty() || v.len() > 256)
            || values
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != values.len()
        {
            return Err(error("invalid axis; use unique nonempty argument values"));
        }
        if combinations
            .len()
            .checked_mul(values.len())
            .is_none_or(|n| n > 256)
        {
            return Err(error("matrix limited to 256 combinations"));
        }
        let mut next = vec![];
        for c in combinations {
            for value in &values {
                let mut row = c.clone();
                row.insert(key.clone(), value.clone());
                next.push(row);
            }
        }
        combinations = next;
    }
    if out.exists() {
        return Err(error("matrix output exists"));
    }
    let mut plans = vec![];
    for (i, c) in combinations.iter().enumerate() {
        let mut p = m.plan.clone();
        for program in std::iter::once(&mut p.candidate)
            .chain(p.baseline.iter_mut())
            .chain(p.variants.values_mut())
        {
            for (key, value) in c {
                if program
                    .args
                    .iter()
                    .any(|a| a == key || a.starts_with(&format!("{key}=")))
                {
                    return Err(error(
                        "matrix axis conflicts with existing program argument",
                    ));
                }
                program.args.extend([key.clone(), value.clone()]);
            }
        }
        for (key, value) in c {
            p.contract.insert(format!("matrix.{key}"), value.clone());
        }
        runner::preflight(p.clone(), &out.join(i.to_string()))?;
        plans.push(p);
    }
    std::fs::create_dir_all(out)?;
    write_new(&out.join("matrix.json"), &combinations)?;
    for (i, plan) in plans.into_iter().enumerate() {
        runner::run(plan, &out.join(i.to_string()))?;
    }
    Ok(())
}
