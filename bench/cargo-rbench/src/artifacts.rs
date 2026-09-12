use rbench::{
    error,
    model::{hash_file, write_new},
    Result, Run,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};
#[derive(Serialize)]
pub struct History {
    pub path: PathBuf,
    pub id: String,
    pub status: String,
    pub modified_ns: u128,
    pub revision: Option<String>,
    pub error: Option<String>,
}
pub fn history(store: &Path) -> Result<Vec<History>> {
    fn walk(p: &Path, depth: usize, visited: &mut usize, rows: &mut Vec<History>) -> Result<()> {
        if depth > 8 {
            return Ok(());
        }
        *visited += 1;
        if *visited > 20000 {
            return Err(error(
                "history scan limit exceeded; choose a narrower --store",
            ));
        }
        if p.join("run.json").is_file() && !p.join("run.json").is_symlink() {
            let path = p.join("run.json");
            let modified_ns = fs::metadata(&path)?
                .modified()?
                .duration_since(UNIX_EPOCH)?
                .as_nanos();
            let row = match Run::load(&path) {
                Ok(r) => History {
                    path: fs::canonicalize(path)?,
                    id: r.id,
                    status: format!("{:?}", r.status),
                    modified_ns,
                    revision: r.provenance.get("user.git.candidate").cloned(),
                    error: None,
                },
                Err(e) => History {
                    path,
                    id: String::new(),
                    status: "Invalid".into(),
                    modified_ns,
                    revision: None,
                    error: Some(e.to_string()),
                },
            };
            rows.push(row);
            return Ok(());
        }
        for e in fs::read_dir(p)? {
            let e = e?;
            if e.file_type()?.is_dir()
                && !matches!(
                    e.file_name().to_str(),
                    Some("target" | ".git" | "baselines" | "notes" | "checkouts" | "quarantine")
                )
            {
                walk(&e.path(), depth + 1, visited, rows)?;
            }
        }
        Ok(())
    }
    let mut rows = vec![];
    if store.exists() {
        walk(store, 0, &mut 0, &mut rows)?;
    }
    rows.sort_by(|a, b| (a.modified_ns, &a.path).cmp(&(b.modified_ns, &b.path)));
    Ok(rows)
}
pub fn last(store: &Path) -> Result<PathBuf> {
    history(store)?
        .into_iter()
        .rev()
        .find(|r| r.status == "Complete")
        .map(|r| r.path)
        .ok_or_else(|| error("no complete run in --store"))
}
#[derive(Serialize, Deserialize)]
pub struct Note {
    pub run_sha256: String,
    pub text: String,
    pub created_ns: u128,
}
fn file(p: &Path) -> PathBuf {
    if p.is_dir() {
        p.join("run.json")
    } else {
        p.into()
    }
}
pub fn notes(store: &Path, run: &Path) -> Result<Vec<Note>> {
    let sha = hash_file(&file(run))?;
    let dir = store.join("notes").join(&sha);
    let mut notes = vec![];
    if dir.exists() {
        for e in fs::read_dir(dir)? {
            let e = e?;
            if e.path().extension().is_some_and(|x| x == "json") {
                let n: Note = serde_json::from_slice(&fs::read(e.path())?)?;
                if n.run_sha256 != sha {
                    return Err(error("note hash mismatch"));
                }
                notes.push(n);
            }
        }
    }
    {
        let imported = file(run).parent().unwrap().join("notes.json");
        if imported.is_file() {
            let saved: Vec<Note> = serde_json::from_slice(&fs::read(imported)?)?;
            for n in saved {
                if n.run_sha256 != sha {
                    return Err(error("imported note hash mismatch"));
                }
                if !notes
                    .iter()
                    .any(|old| old.created_ns == n.created_ns && old.text == n.text)
                {
                    notes.push(n);
                }
            }
        }
    }
    notes.sort_by_key(|n| n.created_ns);
    Ok(notes)
}
pub fn note(store: &Path, run: &Path, text: &str) -> Result<()> {
    Run::load(run)?;
    if text.trim().is_empty() || text.len() > 16384 {
        return Err(error("note must contain 1..16384 bytes"));
    }
    let sha = hash_file(&file(run))?;
    let created_ns = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_nanos();
    let dir = store.join("notes").join(&sha);
    fs::create_dir_all(&dir)?;
    write_new(
        &dir.join(format!("{created_ns}-{}.json", std::process::id())),
        &Note {
            run_sha256: sha,
            text: text.into(),
            created_ns,
        },
    )
}
fn csv_cell(s: &str) -> String {
    // Spreadsheet text cells beginning with formulas are prefixed with an apostrophe.
    let prefix = if s.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        "'"
    } else {
        ""
    };
    format!("\"{prefix}{}\"", s.replace('"', "\"\""))
}
pub fn export(run: &Run, format: &str) -> Result<String> {
    run.validate()?;
    let mut out = String::new();
    if format == "csv" {
        out.push_str("case,metric,variant,process,pair,sequence,value,operations,unit,scope,phase,statistic,availability\n");
    } else if format != "jsonl" {
        return Err(error("format must be csv or jsonl"));
    }
    for o in &run.observations {
        let m = run
            .cases
            .iter()
            .find(|c| c.id == o.case)
            .unwrap()
            .metrics
            .iter()
            .find(|m| m.id == o.metric)
            .unwrap();
        if format == "jsonl" {
            out.push_str(&serde_json::to_string(
                &serde_json::json!({"observation":o,"descriptor":m}),
            )?);
            out.push('\n');
        } else {
            out.push_str(
                &[
                    o.case.clone(),
                    o.metric.clone(),
                    o.variant.clone(),
                    o.process.to_string(),
                    o.pair.map(|v| v.to_string()).unwrap_or_default(),
                    o.sequence.to_string(),
                    o.value.clone().unwrap_or_default(),
                    o.operations.to_string(),
                    m.unit.clone(),
                    m.scope.clone(),
                    m.phase.clone(),
                    m.statistic.clone(),
                    serde_json::to_string(&o.availability)?,
                ]
                .iter()
                .map(|s| csv_cell(s))
                .collect::<Vec<_>>()
                .join(","),
            );
            out.push('\n');
        }
    }
    Ok(out)
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    name: String,
    sha256: String,
    data: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Bundle {
    schema: u32,
    files: Vec<Entry>,
}
fn hash(s: &str) -> String {
    rbench::model::hex(&Sha256::digest(s.as_bytes()))
}
pub fn bundle(store: &Path, run: &Path, out: &Path) -> Result<()> {
    let r = Run::load(run)?;
    let mut files = vec![];
    for (name, data) in [
        ("run.json", fs::read_to_string(file(run))?),
        ("report.html", rbench::report::html_run(&r)?),
        (
            "notes.json",
            serde_json::to_string_pretty(&notes(store, run)?)?,
        ),
    ] {
        files.push(Entry {
            name: name.into(),
            sha256: hash(&data),
            data,
        });
    }
    write_new(out, &Bundle { schema: 1, files })
}
pub fn unpack(input: &Path, out: &Path) -> Result<()> {
    if fs::metadata(input)?.len() > 128 * 1024 * 1024 {
        return Err(error("bundle exceeds 128 MiB"));
    }
    let bundle: Bundle = serde_json::from_slice(&fs::read(input)?)?;
    if bundle.schema != 1 || bundle.files.len() != 3 {
        return Err(error("unknown bundle schema/content"));
    }
    let mut names = std::collections::BTreeSet::new();
    for e in &bundle.files {
        if !matches!(e.name.as_str(), "run.json" | "report.html" | "notes.json")
            || !names.insert(&e.name)
            || hash(&e.data) != e.sha256
        {
            return Err(error("invalid bundle path/duplicate/checksum"));
        }
        if e.name == "run.json" {
            let run: Run = serde_json::from_str(&e.data)?;
            run.validate()?;
        }
    }
    let raw = &bundle
        .files
        .iter()
        .find(|e| e.name == "run.json")
        .unwrap()
        .data;
    let imported: Vec<Note> = serde_json::from_str(
        &bundle
            .files
            .iter()
            .find(|e| e.name == "notes.json")
            .unwrap()
            .data,
    )?;
    if imported.iter().any(|n| n.run_sha256 != hash(raw)) {
        return Err(error("bundle note/run hash mismatch"));
    }
    fs::create_dir(out)?;
    // Regenerate HTML from validated data; imported HTML is never treated as executable authority.
    let run: Run = serde_json::from_str(
        &bundle
            .files
            .iter()
            .find(|e| e.name == "run.json")
            .unwrap()
            .data,
    )?;
    for e in bundle.files {
        let data = if e.name == "report.html" {
            rbench::report::html_run(&run)?
        } else {
            e.data
        };
        std::io::Write::write_all(
            &mut fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(out.join(e.name))?,
            data.as_bytes(),
        )?;
    }
    Ok(())
}
#[derive(Serialize)]
pub struct ContextDiff {
    pub key: String,
    pub a: Option<String>,
    pub b: Option<String>,
}
/// Keys whose environment/provenance/contract values differ between two runs.
pub fn context_diff(a: &Run, b: &Run) -> Vec<ContextDiff> {
    let maps = |r: &Run| {
        let mut m = std::collections::BTreeMap::new();
        for (k, v) in &r.environment {
            m.insert(format!("environment.{k}"), v.clone());
        }
        for (k, v) in &r.provenance {
            m.insert(format!("provenance.{k}"), v.clone());
        }
        for c in &r.cases {
            m.insert(format!("case.{}", c.id), serde_json::to_string(c).unwrap());
        }
        m
    };
    let a = maps(a);
    let b = maps(b);
    let keys: std::collections::BTreeSet<_> = a.keys().chain(b.keys()).collect();
    let mut diffs = Vec::new();
    for k in keys {
        if a.get(k) != b.get(k) {
            diffs.push(ContextDiff {
                key: k.clone(),
                a: a.get(k).cloned(),
                b: b.get(k).cloned(),
            });
        }
    }
    diffs
}
pub fn context(a: &Run, b: &Run) -> String {
    let mut lines = vec!["# Run context differences".to_string()];
    for d in context_diff(a, b) {
        lines.push(format!(
            "\n- {}\n  A: {}\n  B: {}",
            rbench::report::escape(&d.key),
            rbench::report::escape(d.a.as_deref().unwrap_or("<missing>")),
            rbench::report::escape(d.b.as_deref().unwrap_or("<missing>"))
        ));
    }
    if lines.len() == 1 {
        lines.push("No differences.".into());
    }
    lines.join("\n")
}
#[derive(Serialize)]
pub struct TrendPoint {
    pub id: String,
    pub revision: Option<String>,
    pub median: Option<f64>,
    pub unit: String,
    pub context: String,
}
/// Per-run candidate median for one case/metric across the store's complete runs.
pub fn trend_points(store: &Path, case: &str, metric: &str) -> Result<Vec<TrendPoint>> {
    let mut points = Vec::new();
    for h in history(store)?
        .into_iter()
        .filter(|h| h.status == "Complete")
    {
        let r = Run::load(&h.path)?;
        let Some(c) = r.cases.iter().find(|c| c.id == case) else {
            continue;
        };
        let Some(m) = c.metrics.iter().find(|m| m.id == metric) else {
            continue;
        };
        let mut processes: std::collections::BTreeMap<u32, Vec<f64>> = Default::default();
        let mut unavailable = false;
        for o in r
            .observations
            .iter()
            .filter(|o| o.case == case && o.metric == metric && o.variant == "candidate")
        {
            match o.number()? {
                Some(v) => {
                    processes
                        .entry(o.process)
                        .or_default()
                        .push(if m.statistic == "batch_total" {
                            v / o.operations as f64
                        } else {
                            v
                        })
                }
                None => unavailable = true,
            }
        }
        let medians: Vec<_> = processes
            .values()
            .map(|v| rbench::analysis::median(v))
            .collect();
        let median = if unavailable || medians.is_empty() {
            None
        } else {
            Some(rbench::analysis::median(&medians))
        };
        let context = hash(&serde_json::to_string(&(&r.environment, c))?);
        points.push(TrendPoint {
            id: h.id,
            revision: h.revision,
            median,
            unit: m.unit.clone(),
            context,
        });
    }
    Ok(points)
}
pub fn trend(store: &Path, case: &str, metric: &str) -> Result<String> {
    let mut out=String::from("# History of process medians\n\n| Run | Revision | Candidate median | Unit | Context SHA256 |\n|---|---|---:|---|---|\n");
    for p in trend_points(store, case, metric)? {
        let value = match p.median {
            Some(v) => format!("{v:.4}"),
            None => "n/a".into(),
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            rbench::report::escape(&p.id),
            rbench::report::escape(p.revision.as_deref().unwrap_or("unrecorded")),
            value,
            rbench::report::escape(&p.unit),
            p.context
        ));
    }
    out.push_str("\nDescriptive history only. Different context hashes must not be interpreted as a code effect. Each point summarizes independent process medians; no significance claim.\n");
    Ok(out)
}

pub fn trend_html(markdown: &str) -> String {
    let mut groups: std::collections::BTreeMap<String, Vec<(f64, f64)>> = Default::default();
    let mut index = 0.;
    for line in markdown.lines().filter(|l| l.starts_with('|')).skip(2) {
        let cells: Vec<_> = line.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() == 5 {
            if let Ok(v) = cells[2].parse::<f64>() {
                groups.entry(cells[4].into()).or_default().push((index, v));
            }
            index += 1.;
        }
    }
    let mut charts=String::from("<section><h2>History by measurement context</h2><p>One chart per context hash; x = chronological run index, y = median. Missing data creates gaps between points.</p>");
    for (context, points) in groups {
        charts.push_str(&rbench::report::plot(&context, &points));
    }
    charts.push_str("</section>");
    rbench::report::html(markdown).replace("<!--CHARTS-->", &charts)
}
