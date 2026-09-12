//! Bounded regression search with full interval audit, so nonmonotonic histories are not misreported.
use crate::git_run;
use rbench::{
    analysis::{compare, Decision},
    model::write_new,
    *,
};
use std::{path::Path, process::Command};
pub struct Search<'a> {
    pub repo: &'a Path,
    pub good: &'a str,
    pub bad: &'a str,
    pub target: &'a str,
    pub manifest: &'a Path,
    pub out: &'a Path,
    pub repetitions: u32,
    pub max_commits: usize,
    pub offline: bool,
    pub args: Vec<String>,
}
pub fn run(r: Search<'_>) -> Result<()> {
    if !(2..=64).contains(&r.max_commits) {
        return Err(error("max-commits must be 2..64"));
    }
    let git = |args: &[&str]| -> Result<String> {
        let o = Command::new("git")
            .arg("-C")
            .arg(r.repo)
            .args(args)
            .output()?;
        if !o.status.success() {
            return Err(error("Git revision query failed"));
        }
        Ok(String::from_utf8(o.stdout)?.trim().into())
    };
    let good = git(&[
        "rev-parse",
        "--verify",
        "--end-of-options",
        &format!("{}^{{commit}}", r.good),
    ])?;
    let bad = git(&[
        "rev-parse",
        "--verify",
        "--end-of-options",
        &format!("{}^{{commit}}", r.bad),
    ])?;
    let ancestry = git(&["rev-list", "--first-parent", &bad])?;
    let mut revisions: Vec<_> = ancestry
        .lines()
        .take_while(|v| *v != good)
        .map(str::to_string)
        .collect();
    if !ancestry.lines().any(|v| v == good)
        || revisions.is_empty()
        || revisions.len() + 1 > r.max_commits
    {
        return Err(error("require distinct first-parent ancestor and bounded history; increase explicit max-commits if needed"));
    }
    revisions.reverse();
    if r.out.exists() {
        return Err(error("output exists"));
    }
    std::fs::create_dir_all(r.out)?;
    write_new(
        &r.out.join("search-plan.json"),
        &serde_json::json!({"reference":good,"revisions":revisions,"repetitions":r.repetitions,"alpha_family":0.05,"threshold_percent":5,"algorithm":"bounded full first-parent audit; no monotonic assumption; stop on inconclusive"}),
    )?;
    let mut rows = vec![];
    let mut seen_regression = false;
    let mut nonmonotonic = false;
    let mut first = None;
    for (i, revision) in revisions.iter().enumerate() {
        let output = r.out.join(format!("revision-{i}"));
        let measured = git_run::run_with_alpha(
            git_run::Request {
                repo: r.repo,
                base: &good,
                head: revision,
                target: r.target,
                manifest: r.manifest,
                output: &output,
                repetitions: r.repetitions,
                offline: r.offline,
                args: r.args.clone(),
            },
            0.05 / revisions.len() as f64,
        );
        if let Err(e) = measured {
            write_new(
                &r.out.join("search-result.json"),
                &serde_json::json!({"state":"failed","revision":revision,"error":e.to_string(),"completed":rows}),
            )?;
            return Err(e);
        }
        let run = Run::load(output.join("run"))?;
        let comparison = compare(&run, None, 5., 0.05 / revisions.len() as f64)?;
        let uncertain = comparison
            .iter()
            .any(|c| matches!(c.decision, Decision::Inconclusive | Decision::Unavailable));
        let regression = comparison
            .iter()
            .any(|c| c.decision == Decision::Regression);
        rows.push(serde_json::json!({"revision":revision,"comparison":comparison}));
        if uncertain {
            write_new(
                &r.out.join("search-result.json"),
                &serde_json::json!({"state":"inconclusive","revision":revision,"completed":rows,"culprit":null}),
            )?;
            return Err(error(
                "revision search inconclusive; no culprit claimed, no adaptive retries",
            ));
        }
        if regression {
            seen_regression = true;
            if first.is_none() {
                first = Some(revision.clone());
            }
        } else if seen_regression {
            nonmonotonic = true;
        }
    }
    let state = if nonmonotonic {
        "nonmonotonic"
    } else if first.is_some() {
        "confirmed_first_regression"
    } else {
        "no_confirmed_regression"
    };
    let result = serde_json::json!({"state":state,"first_observed_regression":first,"culprit":if nonmonotonic{None}else{first.clone()},"completed":rows,"scope":"first-parent commits only; all metrics against fixed good reference; family correction over all planned revisions; no claim about unmeasured branches"});
    write_new(&r.out.join("search-result.json"), &result)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
