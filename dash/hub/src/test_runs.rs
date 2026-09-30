//! Read the shared test/benchmark launch history and bounded per-test diagnostics.
use serde_json::{Value, json};
use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

fn store(root: &Path) -> PathBuf {
    root.join("target/airbug-report/runs")
}
fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() < 100 && id.bytes().all(|b| b.is_ascii_digit() || b == b'-')
}
fn safe_file(root: &Path, relative: &Path) -> io::Result<PathBuf> {
    let root = root.canonicalize()?;
    let path = root.join(relative).canonicalize()?;
    if !path.starts_with(root) || !path.is_file() {
        return Err(io::ErrorKind::NotFound.into());
    }
    Ok(path)
}
fn read_bounded(path: &Path, max: u64) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(max + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max {
        return Err(io::Error::other("report exceeds size limit"));
    }
    Ok(bytes)
}

pub fn detail(root: &Path, id: &str) -> io::Result<Value> {
    if !valid_id(id) {
        return Err(io::ErrorKind::NotFound.into());
    }
    let path = safe_file(&store(root), &PathBuf::from(id).join("run.json"))?;
    let value: Value = serde_json::from_slice(&read_bounded(&path, 64 * 1024 * 1024)?)?;
    if value["id"] != id || value["version"] != 1 || !value["tests"].is_array() {
        return Err(io::Error::other("unsupported report schema"));
    }
    Ok(value)
}

pub fn list(root: &Path) -> io::Result<Value> {
    let directory = store(root);
    if !directory.exists() {
        return Ok(json!({"runs": [], "errors": []}));
    }
    let mut ids = fs::read_dir(directory)?
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().to_str().map(str::to_owned))
        .filter(|id| valid_id(id))
        .collect::<Vec<_>>();
    ids.sort();
    ids.reverse();
    let mut runs = Vec::new();
    let mut errors = Vec::new();
    for id in ids.into_iter().take(100) {
        match detail(root, &id) {
            Ok(mut value) => {
                let mut counts = serde_json::Map::new();
                for case in value["tests"].as_array().into_iter().flatten() {
                    let status = case["status"].as_str().unwrap_or("unknown").to_owned();
                    let count = counts.entry(status).or_insert(json!(0));
                    *count = json!(count.as_u64().unwrap_or(0) + 1);
                }
                value["counts"] = Value::Object(counts);
                value.as_object_mut().unwrap().remove("tests");
                runs.push(value);
            }
            Err(error) => errors.push(format!("{id}: {error}")),
        }
    }
    Ok(json!({"runs": runs, "errors": errors}))
}

pub fn case(root: &Path, id: &str, index: usize) -> io::Result<Value> {
    let run = detail(root, id)?;
    if run["tests"].get(index).is_none() {
        return Err(io::ErrorKind::NotFound.into());
    }
    let relative = PathBuf::from(id).join(index.to_string());
    let mut events = Vec::new();
    let mut warnings = Vec::new();
    let events_path = relative.join("events.jsonl");
    if store(root).join(&events_path).exists() {
        let path = safe_file(&store(root), &events_path)?;
        let data = read_bounded(&path, 8 * 1024 * 1024)?;
        for line in data.split(|b| *b == b'\n').filter(|line| !line.is_empty()) {
            match serde_json::from_slice::<Value>(line) {
                Ok(value) => events.push(value),
                Err(_) => warnings.push(
                    "Incomplete diagnostics event; refresh after the test finishes".to_owned(),
                ),
            }
        }
    }
    if let Ok(path) = safe_file(&store(root), &relative.join("report-error.txt")) {
        warnings.push(String::from_utf8_lossy(&read_bounded(&path, 64 * 1024)?).into_owned());
    }
    Ok(json!({"case": run["tests"][index], "events": events, "warnings": warnings}))
}

pub fn attachment(root: &Path, id: &str, index: usize, file: &str) -> io::Result<Vec<u8>> {
    if !file.starts_with("attachment-") || !file.ends_with(".bin") || file.contains(['/', '\\']) {
        return Err(io::ErrorKind::NotFound.into());
    }
    let data = case(root, id, index)?;
    if !data["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["type"] == "attachment" && e["file"] == file)
    {
        return Err(io::ErrorKind::NotFound.into());
    }
    let path = safe_file(
        &store(root),
        &PathBuf::from(id).join(index.to_string()).join(file),
    )?;
    read_bounded(&path, 1024 * 1024)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_paths_outside_history() {
        assert!(!valid_id("../1"));
        assert!(!valid_id("1/2"));
        assert!(valid_id("123-42"));
        assert!(detail(Path::new("."), "../Cargo.toml").is_err());
    }
}
