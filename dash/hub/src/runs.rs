//! Bench run registry (UUID sessions) + filesystem progress protocol.
use crate::error::{HubError, Result};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    Registered,
    Running,
    Complete,
    Failed,
    Cancelled,
}

impl RunState {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Registered => "registered",
            Self::Running => "running",
            Self::Complete => "complete",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    fn parse(s: &str) -> Self {
        match s {
            "running" => Self::Running,
            "complete" => Self::Complete,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            _ => Self::Registered,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
    pub title: Option<String>,
    pub command: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateRunResponse {
    pub hub_id: String,
    pub run_id: String,
    pub out_dir: String,
    pub dash_url: String,
}

#[derive(Debug, Serialize)]
pub struct RunSummary {
    pub run_id: String,
    pub title: String,
    pub command: Option<String>,
    pub state: RunState,
    pub out_dir: String,
    pub created_at: String,
    pub updated_at: String,
    pub progress: Option<Value>,
    pub dash_path: String,
}

#[derive(Debug, Serialize)]
pub struct RunsList {
    pub hub_id: String,
    pub runs: Vec<RunSummary>,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct RunDetail {
    pub hub_id: String,
    pub run_id: String,
    pub title: String,
    pub command: Option<String>,
    pub state: RunState,
    pub out_dir: String,
    pub created_at: String,
    pub updated_at: String,
    pub live: Value,
    pub has_run_json: bool,
    pub has_report_html: bool,
    pub report_url: Option<String>,
    pub run_json_url: Option<String>,
    pub jaeger_url: String,
    pub dash_path: String,
}

pub struct RunStore {
    conn: Mutex<Connection>,
    root: PathBuf,
    hub_id: String,
    port: u16,
}

impl RunStore {
    pub fn open(root: PathBuf, db_path: &Path, hub_id: String, port: u16) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir_all(root.join(".airbug-bench/runs"))?;
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS runs (
               run_id TEXT PRIMARY KEY,
               title TEXT NOT NULL,
               command TEXT,
               out_dir TEXT NOT NULL,
               state TEXT NOT NULL,
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL
             );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
            root,
            hub_id,
            port,
        })
    }

    pub fn hub_id(&self) -> &str {
        &self.hub_id
    }

    pub fn create(&self, req: CreateRunRequest) -> Result<CreateRunResponse> {
        let run_id = Uuid::new_v4().to_string();
        let title = req
            .title
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| format!("bench {run_id}"));
        let out_dir = self.root.join(".airbug-bench/runs").join(&run_id);
        fs::create_dir_all(&out_dir)?;
        let now = iso_now();
        let out_s = out_dir.display().to_string();
        {
            let conn = self
                .conn
                .lock()
                .map_err(|_| HubError::msg("runs db lock"))?;
            conn.execute(
                "INSERT INTO runs(run_id, title, command, out_dir, state, created_at, updated_at)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    run_id,
                    title,
                    req.command,
                    out_s,
                    RunState::Registered.as_str(),
                    now,
                    now
                ],
            )?;
        }
        let dash_url = format!("http://127.0.0.1:{}/#/bench/{run_id}", self.port);
        Ok(CreateRunResponse {
            hub_id: self.hub_id.clone(),
            run_id,
            out_dir: out_s,
            dash_url,
        })
    }

    pub fn list(&self) -> Result<RunsList> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| HubError::msg("runs db lock"))?;
        let mut stmt = conn.prepare(
            "SELECT run_id, title, command, out_dir, state, created_at, updated_at
             FROM runs ORDER BY updated_at DESC LIMIT 100",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        drop(stmt);
        drop(conn);

        let mut runs = Vec::with_capacity(rows.len());
        for (run_id, title, command, out_dir, _state, created_at, updated_at) in rows {
            let out = PathBuf::from(&out_dir);
            let (state, progress) = observe_fs(&out);
            let _ = self.touch_state(&run_id, &state);
            runs.push(RunSummary {
                run_id: run_id.clone(),
                title,
                command,
                state,
                out_dir,
                created_at,
                updated_at,
                progress,
                dash_path: format!("#/bench/{run_id}"),
            });
        }
        runs.sort_by(|a, b| {
            let rank = |s: &RunState| match s {
                RunState::Running => 0,
                RunState::Registered => 1,
                _ => 2,
            };
            rank(&a.state)
                .cmp(&rank(&b.state))
                .then_with(|| b.updated_at.cmp(&a.updated_at))
        });
        let note = if runs.is_empty() {
            "No registered bench runs yet. POST /api/v1/bench/runs".into()
        } else {
            let running = runs.iter().filter(|r| r.state == RunState::Running).count();
            format!("{} run(s), {running} running", runs.len())
        };
        Ok(RunsList {
            hub_id: self.hub_id.clone(),
            runs,
            note,
        })
    }

    pub fn get(&self, run_id: &str) -> Result<RunDetail> {
        if !is_uuidish(run_id) {
            return Err(HubError::msg("invalid run_id"));
        }
        let conn = self
            .conn
            .lock()
            .map_err(|_| HubError::msg("runs db lock"))?;
        let row = conn
            .query_row(
                "SELECT title, command, out_dir, state, created_at, updated_at FROM runs WHERE run_id = ?1",
                params![run_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| HubError::msg("run not found"))?;
        drop(conn);

        let (title, command, out_dir, _state, created_at, updated_at) = row;
        let out = PathBuf::from(&out_dir);
        let (state, live) = observe_live(&out);
        let _ = self.touch_state(run_id, &state);
        let has_run_json = out.join("run.json").is_file();
        let has_report_html = out.join("report.html").is_file();
        Ok(RunDetail {
            hub_id: self.hub_id.clone(),
            run_id: run_id.to_string(),
            title,
            command,
            state,
            out_dir,
            created_at,
            updated_at,
            live,
            has_run_json,
            has_report_html,
            report_url: has_report_html.then(|| format!("/bench/runs/{run_id}/report.html")),
            run_json_url: has_run_json.then(|| format!("/bench/runs/{run_id}/run.json")),
            jaeger_url: format!(
                "http://127.0.0.1:16686/search?tags=%7B%22airbug.run_id%22%3A%22{run_id}%22%7D"
            ),
            dash_path: format!("#/bench/{run_id}"),
        })
    }

    fn touch_state(&self, run_id: &str, state: &RunState) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| HubError::msg("runs db lock"))?;
        conn.execute(
            "UPDATE runs SET state = ?1, updated_at = ?2 WHERE run_id = ?3",
            params![state.as_str(), iso_now(), run_id],
        )?;
        Ok(())
    }
}

/// Load-or-create persistent hub instance UUID.
pub fn load_or_create_hub_id(path: &Path) -> Result<String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if path.is_file() {
        let id = fs::read_to_string(path)?.trim().to_string();
        if is_uuidish(&id) {
            return Ok(id);
        }
    }
    let id = Uuid::new_v4().to_string();
    fs::write(path, format!("{id}\n"))?;
    Ok(id)
}

fn is_uuidish(s: &str) -> bool {
    Uuid::parse_str(s).is_ok()
}

fn observe_fs(out: &Path) -> (RunState, Option<Value>) {
    let (state, _live) = observe_live(out);
    let progress = if matches!(state, RunState::Running) {
        read_json_opt(&out.join("progress.json"))
    } else {
        None
    };
    (state, progress)
}

fn observe_live(out: &Path) -> (RunState, Value) {
    let final_path = out.join("status-final.json");
    if final_path.is_file() {
        let live =
            read_json_opt(&final_path).unwrap_or_else(|| serde_json::json!({"state":"complete"}));
        let state = live
            .get("state")
            .and_then(|v| v.as_str())
            .map(RunState::parse)
            .unwrap_or(RunState::Complete);
        let state = match state {
            RunState::Registered | RunState::Running => RunState::Complete,
            other => other,
        };
        return (state, live);
    }
    let progress_path = out.join("progress.json");
    if progress_path.is_file() {
        let live =
            read_json_opt(&progress_path).unwrap_or_else(|| serde_json::json!({"state":"running"}));
        return (RunState::Running, live);
    }
    if out.join("run.json").is_file() {
        return (RunState::Complete, serde_json::json!({"state":"complete"}));
    }
    if out.is_dir() {
        return (
            RunState::Registered,
            serde_json::json!({"state":"registered"}),
        );
    }
    (RunState::Registered, serde_json::json!({"state":"missing"}))
}

fn read_json_opt(path: &Path) -> Option<Value> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn iso_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}s unix")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn register_and_progress() {
        let dir = env::temp_dir().join(format!("airbug-runs-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let store = RunStore::open(
            dir.clone(),
            &dir.join("runs.sqlite"),
            "11111111-1111-1111-1111-111111111111".into(),
            8790,
        )
        .unwrap();
        let created = store
            .create(CreateRunRequest {
                title: Some("t".into()),
                command: None,
            })
            .unwrap();
        let list = store.list().unwrap();
        assert_eq!(list.runs.len(), 1);
        assert_eq!(list.runs[0].state, RunState::Registered);
        fs::write(
            Path::new(&created.out_dir).join("progress.json"),
            r#"{"state":"running","completed":1,"total":3,"variant":"a"}"#,
        )
        .unwrap();
        let detail = store.get(&created.run_id).unwrap();
        assert_eq!(detail.state, RunState::Running);
        assert_eq!(detail.live["completed"], 1);
        fs::write(
            Path::new(&created.out_dir).join("status-final.json"),
            r#"{"state":"complete"}"#,
        )
        .unwrap();
        let detail = store.get(&created.run_id).unwrap();
        assert_eq!(detail.state, RunState::Complete);
        let _ = fs::remove_dir_all(&dir);
    }
}
